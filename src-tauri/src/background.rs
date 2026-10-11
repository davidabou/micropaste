//! Micropaste runs in the background to record every copy: an icon in the notification area,
//! a window that hides when closed instead of quitting, and a launch at Windows startup.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

/// Passed by the launch at Windows startup: the app starts with its window hidden.
pub const HIDDEN_ARG: &str = "--hidden";

/// Brings the window back, restored and focused.
pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Sets up the notification area icon, the hiding window and the launch at startup.
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

    // Started by Windows at login: stays in the background. Otherwise, shown right away
    if !std::env::args().any(|arg| arg == HIDDEN_ARG) {
        let _ = window.show();
    }

    enable_launch_at_startup_once(app);
    create_tray(app)
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
    let open = MenuItem::with_id(app, "open", "Open Micropaste", true, None::<&str>)?;
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
