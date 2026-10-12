use std::io;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arboard::{Clipboard, ImageData};
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::{self, Db, NewClip};
use crate::images;
use crate::source_app;

pub const CLIP_ADDED_EVENT: &str = "clip-added";

pub fn start_watcher(app: AppHandle) {
    thread::spawn(move || {
        let clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(e) => return eprintln!("[clipboard] cannot open clipboard: {e}"),
        };
        // The listener window must be created on the thread that runs it
        let result = Master::new(Watcher { app, clipboard }).and_then(|mut master| master.run());
        if let Err(e) = result {
            eprintln!("[clipboard] watcher stopped: {e}");
        }
    });
}

struct Watcher {
    app: AppHandle,
    clipboard: Clipboard,
}

impl Watcher {
    /// Files first (copied in the Explorer), unless a picture comes with them: some browsers
    /// add the temporary file of a copied image. Then text: apps like Word or Excel also put
    /// an image of the copied text.
    fn read_content(&mut self) -> Option<Content> {
        if !has_bitmap() {
            let paths = read(|| self.clipboard.get().file_list()).unwrap_or_default();
            if !paths.is_empty() {
                return Some(Content::Files(
                    paths.iter().map(|path| path.to_string_lossy()).collect::<Vec<_>>().join("\n"),
                ));
            }
        }
        if let Some(text) = read(|| self.clipboard.get_text()) {
            return (!text.trim().is_empty()).then_some(Content::Text(text));
        }
        let image = read(|| self.clipboard.get_image())?;
        let hash = images::hash(&image);
        Some(Content::Image { image, hash })
    }

    fn save(&self, content: &Content) -> Result<db::Clip, String> {
        let clip = match content {
            Content::Text(text) => NewClip::Text(text),
            Content::Image { image, hash } => {
                let dir = images::dir(&self.app).map_err(|e| e.to_string())?;
                let byte_size = images::save(&dir, hash, image).map_err(|e| e.to_string())?;
                NewClip::Image {
                    hash,
                    width: image.width as u32,
                    height: image.height as u32,
                    byte_size: byte_size as i64,
                }
            }
            Content::Files(paths) => NewClip::Files { paths, byte_size: total_size(paths) },
        };
        let source = source_app::detect();
        let db = self.app.state::<Db>();
        let saved = db::save(&db.0.lock().unwrap(), clip, source.as_ref(), now_ms());
        saved.map_err(|e| e.to_string())
    }
}

enum Content {
    Text(String),
    Image { image: ImageData<'static>, hash: String },
    Files(String),
}

impl ClipboardHandler for Watcher {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        if crate::copy::is_own_copy() || is_excluded_from_history() {
            return CallbackResult::Next;
        }
        let Some(content) = self.read_content() else {
            return CallbackResult::Next;
        };

        match self.save(&content) {
            Ok(clip) => {
                let _ = self.app.emit(CLIP_ADDED_EVENT, clip);
            }
            Err(e) => eprintln!("[clipboard] cannot save clip: {e}"),
        }
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        eprintln!("[clipboard] watcher error: {error}");
        CallbackResult::Next
    }
}

fn read<T>(mut read: impl FnMut() -> Result<T, arboard::Error>) -> Option<T> {
    for _ in 0..5 {
        match read() {
            Ok(value) => return Some(value),
            Err(arboard::Error::ClipboardOccupied) => thread::sleep(Duration::from_millis(20)),
            Err(_) => return None,
        }
    }
    None
}

pub fn total_size(paths: &str) -> Option<i64> {
    let sizes: Vec<u64> = paths
        .lines()
        .filter_map(|path| std::fs::metadata(path).ok())
        .filter(|meta| meta.is_file())
        .map(|meta| meta.len())
        .collect();
    (!sizes.is_empty()).then(|| sizes.iter().sum::<u64>() as i64)
}

#[cfg(windows)]
fn has_bitmap() -> bool {
    clipboard_win::is_format_avail(clipboard_win::formats::CF_DIB)
        || clipboard_win::is_format_avail(clipboard_win::formats::CF_BITMAP)
}

#[cfg(not(windows))]
fn has_bitmap() -> bool {
    false
}

/// Password managers mark secrets with these formats so clipboard tools don't record them.
#[cfg(windows)]
fn is_excluded_from_history() -> bool {
    ["ExcludeClipboardContentFromMonitorProcessing", "Clipboard Viewer Ignore"]
        .into_iter()
        .filter_map(clipboard_win::register_format)
        .any(|format| clipboard_win::is_format_avail(format.get()))
}

#[cfg(not(windows))]
fn is_excluded_from_history() -> bool {
    false
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or_default()
}
