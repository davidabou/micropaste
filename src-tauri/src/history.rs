use std::thread;
use std::time::Duration;

use rusqlite::Connection;
use tauri::{AppHandle, Emitter, Manager};

use crate::clipboard::now_ms;
use crate::db::{self, Db};
use crate::images;

/// Clips not used (copied or recopied) for this long leave the history, unless they are in a collection.
const RETENTION_MS: i64 = 30 * 24 * 60 * 60 * 1000;
const PURGE_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Emitted to the frontend with the ids of the clips removed from the history by the cleanup.
pub const CLIPS_REMOVED_EVENT: &str = "clips-removed";

/// Deletes a clip, and its images from the disk once nothing else uses them.
pub fn remove(app: &AppHandle, conn: &Connection, id: i64) -> Result<(), String> {
    let files = db::delete(conn, id).map_err(|e| e.to_string())?;
    let images_dir = images::dir(app).map_err(|e| e.to_string())?;
    for file in files {
        if !db::is_image_file_used(conn, &file).unwrap_or(true) {
            let _ = images::remove_file(&images_dir, &file);
        }
    }
    Ok(())
}

/// Keeps the history from growing forever: removes old clips now, then every hour.
/// The first cleanup runs before the window loads the history, so it never shows expired clips.
pub fn start_cleanup(app: AppHandle) {
    purge_or_log(&app);
    thread::spawn(move || loop {
        thread::sleep(PURGE_INTERVAL);
        purge_or_log(&app);
    });
}

fn purge_or_log(app: &AppHandle) {
    if let Err(e) = purge(app) {
        eprintln!("[history] cleanup failed: {e}");
    }
}

fn purge(app: &AppHandle) -> Result<(), String> {
    let removed = {
        let db = app.state::<Db>();
        let conn = db.0.lock().unwrap();
        let expired = db::expired(&conn, now_ms() - RETENTION_MS).map_err(|e| e.to_string())?;
        let mut removed = Vec::with_capacity(expired.len());
        for id in expired {
            match remove(app, &conn, id) {
                Ok(()) => removed.push(id),
                Err(e) => eprintln!("[history] cannot remove clip {id}: {e}"),
            }
        }
        removed
    };
    if !removed.is_empty() {
        let _ = app.emit(CLIPS_REMOVED_EVENT, removed);
    }
    Ok(())
}
