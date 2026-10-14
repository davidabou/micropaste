use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::background;
use crate::db::{self, Db};

const SHORTCUT_KEY: &str = "shortcut";
const RETENTION_KEY: &str = "retention_days";
const LAYOUT_KEY: &str = "layout";
const LAYOUTS: [&str; 2] = ["grid", "list"];
const DEFAULT_SHORTCUT: &str = "Alt+Shift+V";
const DEFAULT_RETENTION_DAYS: u32 = 30;
const KEEP_FOREVER: &str = "forever";

/// While the settings record a new shortcut, pressing the current one must not hide the window.
pub static RECORDING_SHORTCUT: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    shortcut: String,
    retention_days: Option<u32>,
    launch_at_startup: bool,
    /// `None` until chosen once.
    layout: Option<String>,
}

pub fn get(app: &AppHandle) -> Settings {
    Settings {
        shortcut: shortcut(app),
        retention_days: retention_days(app),
        launch_at_startup: app.autolaunch().is_enabled().unwrap_or(false),
        layout: read(app, LAYOUT_KEY).filter(|layout| LAYOUTS.contains(&layout.as_str())),
    }
}

fn read(app: &AppHandle, key: &str) -> Option<String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    db::setting(&conn, key).ok().flatten()
}

fn write(app: &AppHandle, key: &str, value: &str) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    db::set_setting(&conn, key, value).map_err(|e| e.to_string())
}

pub fn shortcut(app: &AppHandle) -> String {
    read(app, SHORTCUT_KEY).unwrap_or_else(|| DEFAULT_SHORTCUT.to_string())
}

/// `None` keeps the history forever.
pub fn retention_days(app: &AppHandle) -> Option<u32> {
    match read(app, RETENTION_KEY) {
        None => Some(DEFAULT_RETENTION_DAYS),
        Some(value) => value.parse().ok(),
    }
}

pub fn set_shortcut(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    let new = Shortcut::from_str(shortcut).map_err(|_| "This shortcut isn’t supported".to_string())?;
    if new.mods.is_empty() {
        return Err("Add Ctrl, Alt, Shift or Win".into());
    }
    let current = self::shortcut(app);
    let global = app.global_shortcut();
    if Shortcut::from_str(&current).is_ok_and(|current| current == new) {
        return Ok(());
    }
    let _ = global.unregister(current.as_str());
    if global.register(new).is_err() {
        let _ = global.register(current.as_str());
        return Err("Another app already uses this shortcut".into());
    }
    write(app, SHORTCUT_KEY, shortcut)?;
    background::show_shortcut_in_tray(app, shortcut);
    Ok(())
}

pub fn set_retention_days(app: &AppHandle, days: Option<u32>) -> Result<(), String> {
    let value = days.map_or_else(|| KEEP_FOREVER.to_string(), |days| days.to_string());
    write(app, RETENTION_KEY, &value)
}

pub fn set_layout(app: &AppHandle, layout: &str) -> Result<(), String> {
    if !LAYOUTS.contains(&layout) {
        return Err("Unknown layout".into());
    }
    write(app, LAYOUT_KEY, layout)
}

pub fn set_launch_at_startup(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    let launcher = app.autolaunch();
    if enabled { launcher.enable() } else { launcher.disable() }.map_err(|e| e.to_string())?;
    let enabled = launcher.is_enabled().unwrap_or(enabled);
    background::show_launch_at_startup_in_tray(app, enabled);
    Ok(enabled)
}

pub fn set_recording_shortcut(recording: bool) {
    RECORDING_SHORTCUT.store(recording, Ordering::Relaxed);
}
