use std::sync::atomic::Ordering;

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::plugin::TauriPlugin;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent, Wry};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::settings;

pub const HIDDEN_ARG: &str = "--hidden";

pub const LAUNCH_AT_STARTUP_EVENT: &str = "launch-at-startup-changed";

struct TrayItems {
    open: MenuItem<Wry>,
    startup: CheckMenuItem<Wry>,
}

pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    let in_front = window.is_visible().unwrap_or(false)
        && !window.is_minimized().unwrap_or(false)
        && window.is_focused().unwrap_or(false);
    if in_front {
        let _ = window.hide();
    } else {
        show_window(app);
    }
}

pub fn shortcut_plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == ShortcutState::Pressed && !settings::RECORDING_SHORTCUT.load(Ordering::Relaxed) {
                toggle_window(app);
            }
        })
        .build()
}

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let window = app.get_webview_window("main").expect("main window");

    let hidden = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = hidden.hide();
        }
    });

    move_to_bottom_right(&window);

    if !std::env::args().any(|arg| arg == HIDDEN_ARG) {
        let _ = window.show();
    }

    let shortcut = settings::shortcut(app.handle());
    if let Err(e) = app.global_shortcut().register(shortcut.as_str()) {
        eprintln!("[shortcut] cannot register {shortcut}: {e}");
    }

    enable_launch_at_startup_once(app);
    create_tray(app, &shortcut)
}

const SCREEN_MARGIN: f64 = 12.0;

fn move_to_bottom_right(window: &WebviewWindow) {
    let (Ok(Some(monitor)), Ok(size)) = (window.primary_monitor(), window.outer_size()) else { return };
    let area = monitor.work_area();
    let margin = (SCREEN_MARGIN * monitor.scale_factor()).round() as i32;
    let x = area.position.x + area.size.width as i32 - size.width as i32 - margin;
    let y = area.position.y + area.size.height as i32 - size.height as i32 - margin;
    let position = PhysicalPosition::new(x.max(area.position.x), y.max(area.position.y));
    let _ = window.set_position(position);
}

/// Installed builds only: a dev build would register its temporary executable.
fn enable_launch_at_startup_once(app: &App) {
    if cfg!(debug_assertions) {
        return;
    }
    let Ok(dir) = app.path().app_data_dir() else { return };
    let marker = dir.join("launch-at-startup-set");
    if marker.exists() {
        return;
    }
    if app.autolaunch().enable().is_ok() {
        let _ = std::fs::write(marker, "");
    }
}

pub fn show_shortcut_in_tray(app: &AppHandle, shortcut: &str) {
    let _ = app.state::<TrayItems>().open.set_accelerator(Some(shortcut));
}

pub fn show_launch_at_startup_in_tray(app: &AppHandle, enabled: bool) {
    let _ = app.state::<TrayItems>().startup.set_checked(enabled);
}

fn create_tray(app: &App, shortcut: &str) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Micropaste", true, Some(shortcut))?;
    let startup_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let startup = CheckMenuItem::with_id(app, "startup", "Launch at startup", true, startup_enabled, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Micropaste", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &startup, &quit])?;
    app.manage(TrayItems { open, startup });

    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Micropaste")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "startup" => {
                let enabled = app.autolaunch().is_enabled().unwrap_or(false);
                if let Ok(enabled) = settings::set_launch_at_startup(app, !enabled) {
                    let _ = app.emit(LAUNCH_AT_STARTUP_EVENT, enabled);
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
