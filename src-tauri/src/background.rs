//! Micropaste runs in the background to record every copy: an icon in the notification area,
//! a window that hides when closed instead of quitting, a launch at Windows startup, and a global
//! shortcut to bring the window up from anywhere.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::plugin::TauriPlugin;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, PhysicalPosition, WebviewWindow, WindowEvent, Wry};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Passed by the launch at Windows startup: the app starts with its window hidden.
pub const HIDDEN_ARG: &str = "--hidden";

/// Brings the window up from any app, or hides it when it is already in front.
pub const SHORTCUT: &str = "Alt+Shift+V";

/// Brings the window back, restored and focused.
pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hides the window when it is the one in front, brings it back otherwise.
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

/// Handles the global shortcut. It is registered in `setup`, where a failure (the shortcut already
/// taken by another app) is only logged instead of stopping the app.
pub fn shortcut_plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                toggle_window(app);
            }
        })
        .build()
}

/// Sets up the notification area icon, the hiding window, the shortcut and the launch at startup.
pub fn setup(app: &mut App) -> tauri::Result<()> {
    let window = app.get_webview_window("main").expect("main window");

    // Closing the window hides it: recording goes on until "Quit"
    let hidden = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = hidden.hide();
        }
    });

    move_to_bottom_right(&window);

    // Started by Windows at login: stays in the background. Otherwise, shown right away
    if !std::env::args().any(|arg| arg == HIDDEN_ARG) {
        let _ = window.show();
    }

    if let Err(e) = app.global_shortcut().register(SHORTCUT) {
        eprintln!("[shortcut] cannot register {SHORTCUT}: {e}");
    }

    enable_launch_at_startup_once(app);
    create_tray(app)
}

/// Gap between the window and the screen edges, in logical pixels, like the Windows flyouts.
const SCREEN_MARGIN: f64 = 12.0;

/// Places the window in the bottom right corner of the main screen, above the taskbar.
fn move_to_bottom_right(window: &WebviewWindow) {
    let (Ok(Some(monitor)), Ok(size)) = (window.primary_monitor(), window.outer_size()) else { return };
    let area = monitor.work_area();
    let margin = (SCREEN_MARGIN * monitor.scale_factor()).round() as i32;
    let x = area.position.x + area.size.width as i32 - size.width as i32 - margin;
    let y = area.position.y + area.size.height as i32 - size.height as i32 - margin;
    // A window larger than the screen keeps its top left corner visible
    let position = PhysicalPosition::new(x.max(area.position.x), y.max(area.position.y));
    let _ = window.set_position(position);
}

/// Turns the launch at startup on at the very first run, in installed builds only (a dev build
/// would register its temporary executable). It can be turned off from the tray menu afterwards.
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

fn create_tray(app: &App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Micropaste", true, Some(SHORTCUT))?;
    let startup_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let startup = CheckMenuItem::with_id(app, "startup", "Launch at startup", true, startup_enabled, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Micropaste", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &startup, &quit])?;

    // The 32px icon: sharp at the small sizes of the notification area
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Micropaste")
        .menu(&menu)
        // Left click opens the window, right click shows the menu
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_window(tray.app_handle());
            }
        })
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "startup" => {
                let launcher = app.autolaunch();
                let enabled = launcher.is_enabled().unwrap_or(false);
                let _ = if enabled { launcher.disable() } else { launcher.enable() };
                let _ = startup.set_checked(launcher.is_enabled().unwrap_or(false));
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
