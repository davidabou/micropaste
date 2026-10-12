use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use arboard::{Clipboard, ImageData};
use tauri::{AppHandle, Emitter, Manager};

use crate::clipboard::{CLIP_ADDED_EVENT, now_ms};
use crate::db::{self, Db};
use crate::images;

static COPIED_SEQUENCE: AtomicU32 = AtomicU32::new(0);

pub fn is_own_copy() -> bool {
    // The sequence number is only stored once writing is done, the watcher can be faster:
    // the clipboard owner, a window of this process, tells it right away
    if crate::source_app::is_written_by_own_process() {
        return true;
    }
    let copied = COPIED_SEQUENCE.load(Ordering::SeqCst);
    copied != 0 && clipboard_sequence() == copied
}

pub fn files_not_found(paths: &str) -> String {
    if paths.lines().count() > 1 { "Files not found" } else { "File not found" }.into()
}

pub fn copy(app: &AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Db>();
    let clip = db::get(&db.0.lock().unwrap(), id).map_err(|e| e.to_string())?;

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    match clip.kind.as_str() {
        "image" => {
            let dir = images::dir(app).map_err(|e| e.to_string())?;
            let (width, height, rgba) = images::load_rgba(&dir, &clip.content).map_err(|e| e.to_string())?;
            let image = ImageData { width: width as usize, height: height as usize, bytes: rgba.into() };
            clipboard.set_image(image).map_err(|e| e.to_string())?;
        }
        "files" => {
            let paths: Vec<&Path> = clip.content.lines().map(Path::new).filter(|path| path.exists()).collect();
            if paths.is_empty() {
                return Err(crate::copy::files_not_found(&clip.content));
            }
            clipboard.set().file_list(&paths).map_err(|e| e.to_string())?;
        }
        _ => clipboard.set_text(&clip.content).map_err(|e| e.to_string())?,
    }
    COPIED_SEQUENCE.store(clipboard_sequence(), Ordering::SeqCst);

    let clip = db::touch(&db.0.lock().unwrap(), id, now_ms()).map_err(|e| e.to_string())?;
    let _ = app.emit(CLIP_ADDED_EVENT, clip);
    Ok(())
}

#[cfg(windows)]
fn clipboard_sequence() -> u32 {
    clipboard_win::seq_num().map_or(0, |seq| seq.get())
}

#[cfg(not(windows))]
fn clipboard_sequence() -> u32 {
    0
}
