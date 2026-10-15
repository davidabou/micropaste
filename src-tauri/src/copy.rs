use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use arboard::{Clipboard, ImageData};
use tauri::{AppHandle, Emitter, Manager};

use crate::clipboard::{CLIP_ADDED_EVENT, now_ms, total_size};
use crate::db::{self, Db, NewClip};
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

#[derive(serde::Serialize)]
pub struct PartialCopy {
    copied: usize,
    total: usize,
}

pub fn copy(app: &AppHandle, id: i64) -> Result<Option<PartialCopy>, String> {
    let db = app.state::<Db>();
    let clip = db::get(&db.0.lock().unwrap(), id).map_err(|e| e.to_string())?;

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let partial = match clip.kind.as_str() {
        "image" => {
            let dir = images::dir(app).map_err(|e| e.to_string())?;
            let (width, height, rgba) = images::load_rgba(&dir, &clip.content).map_err(|e| e.to_string())?;
            let image = ImageData { width: width as usize, height: height as usize, bytes: rgba.into() };
            clipboard.set_image(image).map_err(|e| e.to_string())?;
            None
        }
        "files" => {
            let total = clip.content.lines().count();
            let paths: Vec<&Path> = clip.content.lines().map(Path::new).filter(|path| path.exists()).collect();
            if paths.is_empty() {
                return Err(crate::copy::files_not_found(&clip.content));
            }
            clipboard.set().file_list(&paths).map_err(|e| e.to_string())?;
            (paths.len() < total).then_some(PartialCopy { copied: paths.len(), total })
        }
        _ => {
            clipboard.set_text(&clip.content).map_err(|e| e.to_string())?;
            None
        }
    };
    COPIED_SEQUENCE.store(clipboard_sequence(), Ordering::SeqCst);

    let clip = db::touch(&db.0.lock().unwrap(), id, now_ms()).map_err(|e| e.to_string())?;
    let _ = app.emit(CLIP_ADDED_EVENT, clip);
    Ok(partial)
}

/// One file of a files clip. Only paths of that clip are accepted, not arbitrary files.
pub fn clip_file(app: &AppHandle, id: i64, path: &str) -> Result<PathBuf, String> {
    let clip = db::get(&app.state::<Db>().0.lock().unwrap(), id).map_err(|e| e.to_string())?;
    if clip.kind != "files" || !clip.content.lines().any(|line| line == path) {
        return Err("Not a copied file".into());
    }
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("File not found".into());
    }
    Ok(path)
}

/// Copies a single file of a group: like a real copy, it goes to the top of the history.
pub fn copy_file(app: &AppHandle, id: i64, path: &str) -> Result<(), String> {
    let file = clip_file(app, id, path)?;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set().file_list(&[file]).map_err(|e| e.to_string())?;
    COPIED_SEQUENCE.store(clipboard_sequence(), Ordering::SeqCst);

    let db = app.state::<Db>();
    let source = crate::source_app::micropaste();
    let new_clip = NewClip::Files { paths: path, byte_size: total_size(path) };
    let clip = db::save(&db.0.lock().unwrap(), new_clip, source.as_ref(), now_ms()).map_err(|e| e.to_string())?;
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
