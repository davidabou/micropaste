use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub struct Updates {
    checking: tauri::async_runtime::Mutex<()>,
    ready: Mutex<Option<(Update, Vec<u8>)>>,
}

pub fn start(app: AppHandle) {
    app.manage(Updates::default());
    thread::spawn(move || {
        loop {
            if let Err(e) = tauri::async_runtime::block_on(fetch(&app)) {
                eprintln!("[updater] check failed: {e}");
            }
            thread::sleep(CHECK_INTERVAL);
        }
    });
}

pub fn ready_version(app: &AppHandle) -> Option<String> {
    let updates = app.state::<Updates>();
    let ready = updates.ready.lock().unwrap();
    ready.as_ref().map(|(update, _)| update.version.clone())
}

/// Downloads a newer version in the background, emits `update-ready` once it can be installed
pub async fn fetch(app: &AppHandle) -> Result<Option<String>, String> {
    let updates = app.state::<Updates>();
    let _checking = updates.checking.lock().await;
    if let Some(version) = ready_version(app) {
        return Ok(Some(version));
    }

    let updater = app.updater().map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let bytes = update.download(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    let version = update.version.clone();
    *updates.ready.lock().unwrap() = Some((update, bytes));
    let _ = app.emit("update-ready", &version);
    Ok(Some(version))
}

/// Runs the installer, which quits the app and opens it again once updated
pub fn install(app: &AppHandle) -> Result<(), String> {
    let updates = app.state::<Updates>();
    let ready = updates.ready.lock().unwrap();
    let (update, bytes) = ready.as_ref().ok_or("No update is ready")?;
    update.install(bytes).map_err(|e| e.to_string())
}
