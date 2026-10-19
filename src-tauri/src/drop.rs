use arboard::ImageData;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::clipboard::{now_ms, total_size};
use crate::db::{self, Clip, Db, NewClip};
use crate::images;

pub const CLIP_DROPPED_EVENT: &str = "clip-dropped";

#[cfg(windows)]
const FILES_MESSAGE: &str = "micropaste-drop-files:";

#[derive(Clone, Serialize)]
struct Dropped {
    collection: i64,
    clip: Clip,
}

fn save(app: &AppHandle, collection: i64, clip: NewClip) -> Result<Clip, String> {
    let source = crate::source_app::foreground();
    let db = app.state::<Db>();
    let conn = db.0.lock().unwrap();
    let saved = db::save_dropped(&conn, clip, source.as_ref(), now_ms()).map_err(|e| e.to_string())?;
    db::add_to_collection(&conn, collection, saved.id, now_ms()).map_err(|e| e.to_string())?;
    Ok(saved)
}

pub fn text(app: &AppHandle, collection: i64, text: &str) -> Result<Clip, String> {
    if text.trim().is_empty() {
        return Err("Nothing to add".into());
    }
    save(app, collection, NewClip::Text(text))
}

pub fn image(app: &AppHandle, collection: i64, bytes: &[u8]) -> Result<Clip, String> {
    let rgba = image::load_from_memory(bytes).map_err(|e| e.to_string())?.to_rgba8();
    let (width, height) = rgba.dimensions();
    let image = ImageData { width: width as usize, height: height as usize, bytes: rgba.into_raw().into() };
    let hash = images::hash(&image);
    let dir = images::dir(app).map_err(|e| e.to_string())?;
    let byte_size = images::save(&dir, &hash, &image).map_err(|e| e.to_string())?;
    let clip = save(app, collection, NewClip::Image { hash: &hash, width, height, byte_size: byte_size as i64 })?;
    crate::ocr::queue(app, clip.id);
    Ok(clip)
}

fn files(app: &AppHandle, collection: i64, paths: &[String]) -> Result<Clip, String> {
    let paths = paths.join("\n");
    let clip = save(app, collection, NewClip::Files { paths: &paths, byte_size: total_size(&paths) })?;
    crate::ocr::queue(app, clip.id);
    Ok(clip)
}

/// Listens for the dropped files the page hands to WebView2: unlike the page, WebView2 knows their
/// paths. The page posts them with `chrome.webview.postMessageWithAdditionalObjects`.
#[cfg(windows)]
pub fn listen(window: &tauri::WebviewWindow) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2File, ICoreWebView2WebMessageReceivedEventArgs2};
    use webview2_com::WebMessageReceivedEventHandler;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::core::{Interface, PWSTR};

    fn take(value: PWSTR) -> String {
        let text = unsafe { value.to_string() }.unwrap_or_default();
        unsafe { CoTaskMemFree(Some(value.0 as *const _)) };
        text
    }

    let app = window.app_handle().clone();
    let _ = window.with_webview(move |webview| {
        let Ok(core) = (unsafe { webview.controller().CoreWebView2() }) else { return };
        let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else { return Ok(()) };
            let mut message = PWSTR::null();
            if unsafe { args.TryGetWebMessageAsString(&mut message) }.is_err() {
                return Ok(());
            }
            let Some(collection) = take(message).strip_prefix(FILES_MESSAGE).and_then(|id| id.parse::<i64>().ok())
            else {
                return Ok(());
            };

            let objects = args.cast::<ICoreWebView2WebMessageReceivedEventArgs2>()?;
            let objects = unsafe { objects.AdditionalObjects() }?;
            let mut count = 0;
            unsafe { objects.Count(&mut count) }?;
            let mut paths = Vec::new();
            for index in 0..count {
                let Ok(file) =
                    unsafe { objects.GetValueAtIndex(index) }.and_then(|item| item.cast::<ICoreWebView2File>())
                else {
                    continue;
                };
                let mut path = PWSTR::null();
                if unsafe { file.Path(&mut path) }.is_ok() {
                    let path = take(path);
                    if !path.is_empty() {
                        paths.push(path);
                    }
                }
            }
            if paths.is_empty() {
                return Ok(());
            }

            match files(&app, collection, &paths) {
                Ok(clip) => {
                    let _ = app.emit(CLIP_DROPPED_EVENT, Dropped { collection, clip });
                }
                Err(e) => eprintln!("[drop] cannot save files: {e}"),
            }
            Ok(())
        }));
        let mut token = 0;
        let _ = unsafe { core.add_WebMessageReceived(&handler, &mut token) };
    });
}

#[cfg(not(windows))]
pub fn listen(_window: &tauri::WebviewWindow) {}
