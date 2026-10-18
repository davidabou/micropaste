mod app_icon;
mod background;
mod clipboard;
mod copy;
mod db;
mod drag_out;
mod drop;
mod history;
mod images;
mod link_preview;
mod settings;
mod source_app;
mod window_icon;

use std::sync::Mutex;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use tauri::Manager;
use window_vibrancy::apply_mica;

use db::{Clip, Collection, Db};
use link_preview::LinkPreview;

#[tauri::command]
fn list_clips(
    db: tauri::State<Db>,
    limit: Option<u32>,
    collection: Option<i64>,
    favorites: Option<bool>,
) -> Result<Vec<Clip>, String> {
    db::list(&db.0.lock().unwrap(), limit.unwrap_or(500), collection, favorites.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn search_clips(
    db: tauri::State<Db>,
    query: String,
    limit: Option<u32>,
    collection: Option<i64>,
    favorites: Option<bool>,
) -> Result<Vec<Clip>, String> {
    db::search(&db.0.lock().unwrap(), &query, limit.unwrap_or(500), collection, favorites.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_favorite(db: tauri::State<Db>, id: i64, favorite: bool) -> Result<Clip, String> {
    db::set_favorite(&db.0.lock().unwrap(), id, favorite).map_err(|e| e.to_string())
}

#[tauri::command]
fn count_clips(db: tauri::State<Db>) -> Result<i64, String> {
    db::count(&db.0.lock().unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_collections(db: tauri::State<Db>) -> Result<Vec<Collection>, String> {
    db::collections(&db.0.lock().unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_collection(db: tauri::State<Db>, name: String) -> Result<Collection, String> {
    let name = collection_name(&name)?;
    db::create_collection(&db.0.lock().unwrap(), name, clipboard::now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_collection(db: tauri::State<Db>, id: i64, name: String) -> Result<(), String> {
    let name = collection_name(&name)?;
    db::rename_collection(&db.0.lock().unwrap(), id, name).map_err(|e| e.to_string())
}

#[tauri::command]
fn reorder_collections(db: tauri::State<Db>, ids: Vec<i64>) -> Result<(), String> {
    db::reorder_collections(&mut db.0.lock().unwrap(), &ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_collection(db: tauri::State<Db>, id: i64) -> Result<(), String> {
    db::delete_collection(&db.0.lock().unwrap(), id).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_to_collection(db: tauri::State<Db>, collection: i64, clip: i64) -> Result<(), String> {
    db::add_to_collection(&db.0.lock().unwrap(), collection, clip, clipboard::now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
fn reorder_collection(db: tauri::State<Db>, collection: i64, clips: Vec<i64>) -> Result<(), String> {
    db::reorder_collection(&mut db.0.lock().unwrap(), collection, &clips).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_from_collection(db: tauri::State<Db>, collection: i64, clip: i64) -> Result<(), String> {
    db::remove_from_collection(&db.0.lock().unwrap(), collection, clip).map_err(|e| e.to_string())
}

#[tauri::command]
fn clip_collections(db: tauri::State<Db>, clip: i64) -> Result<Vec<i64>, String> {
    db::clip_collections(&db.0.lock().unwrap(), clip).map_err(|e| e.to_string())
}

fn collection_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("A collection needs a name".into());
    }
    Ok(name)
}

#[tauri::command]
async fn get_app_icon(db: tauri::State<'_, Db>, path: String) -> Result<Option<String>, String> {
    {
        let conn = db.0.lock().unwrap();
        // Only serve icons of apps that clips were copied from, not arbitrary files
        if !db::is_source_app(&conn, &path).map_err(|e| e.to_string())? {
            return Ok(None);
        }
        if let Some(png) = db::app_icon(&conn, &path).map_err(|e| e.to_string())? {
            return Ok(Some(png_data_url(&png)));
        }
    }

    let Some(png) = app_icon::extract_png(&path) else {
        return Ok(None);
    };
    db::save_app_icon(&db.0.lock().unwrap(), &path, &png).map_err(|e| e.to_string())?;
    Ok(Some(png_data_url(&png)))
}

#[tauri::command]
async fn copy_clip(app: tauri::AppHandle, id: i64) -> Result<Option<copy::PartialCopy>, String> {
    copy::copy(&app, id)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FilePreview {
    image: Option<String>,
    thumbnail: bool,
    type_name: Option<String>,
}

#[tauri::command]
async fn get_file_preview(db: tauri::State<'_, Db>, path: String) -> Result<FilePreview, String> {
    // Only preview files that were copied, not arbitrary paths
    if !db::is_copied_file(&db.0.lock().unwrap(), &path).map_err(|e| e.to_string())? {
        return Err("Not a copied file".into());
    }
    let (preview, type_name) = tauri::async_runtime::spawn_blocking(move || {
        let preview = app_icon::file_preview_png(&path, app_icon::THUMBNAIL_SIZE, app_icon::FILE_ICON_SIZE);
        (preview, app_icon::file_type_name(&path))
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(match preview {
        Some((png, thumbnail)) => FilePreview { image: Some(png_data_url(&png)), thumbnail, type_name },
        None => FilePreview { image: None, thumbnail: false, type_name },
    })
}

#[tauri::command]
fn reveal_file(app: tauri::AppHandle, id: i64, path: String) -> Result<(), String> {
    let file = copy::clip_file(&app, id, &path)?;
    tauri_plugin_opener::reveal_item_in_dir(file).map_err(|e| e.to_string())
}

#[tauri::command]
async fn missing_files(db: tauri::State<'_, Db>, id: i64) -> Result<Vec<String>, String> {
    let clip = db::get(&db.0.lock().unwrap(), id).map_err(|e| e.to_string())?;
    if clip.kind != "files" {
        return Ok(Vec::new());
    }
    Ok(clip.content.lines().filter(|path| !std::path::Path::new(path).exists()).map(String::from).collect())
}

#[tauri::command]
async fn copy_file(app: tauri::AppHandle, id: i64, path: String) -> Result<(), String> {
    copy::copy_file(&app, id, &path)
}

#[tauri::command]
async fn start_drag(app: tauri::AppHandle, window: tauri::Window, id: i64) -> Result<(), String> {
    drag_out::start(&app, window, id)
}

#[tauri::command]
async fn start_file_drag(app: tauri::AppHandle, window: tauri::Window, id: i64, path: String) -> Result<(), String> {
    drag_out::start_file(&app, window, id, &path)
}

#[tauri::command]
fn drop_text(app: tauri::AppHandle, collection: i64, text: String) -> Result<Clip, String> {
    drop::text(&app, collection, &text)
}

#[tauri::command]
async fn drop_image(app: tauri::AppHandle, collection: i64, bytes: Vec<u8>) -> Result<Clip, String> {
    drop::image(&app, collection, &bytes)
}

#[tauri::command]
fn delete_clip(app: tauri::AppHandle, db: tauri::State<Db>, id: i64) -> Result<(), String> {
    history::remove(&app, &db.0.lock().unwrap(), id)
}

#[tauri::command]
async fn get_link_preview(
    app: tauri::AppHandle,
    db: tauri::State<'_, Db>,
    url: String,
) -> Result<Option<LinkPreview>, String> {
    let Some(parsed) = link_preview::parse_url(&url) else {
        return Ok(None);
    };
    {
        let conn = db.0.lock().unwrap();
        // Only load links that were copied, not arbitrary URLs
        if !db::is_text_clip(&conn, &url).map_err(|e| e.to_string())? {
            return Ok(None);
        }
        if let Some(mut preview) = db::link_preview(&conn, &url).map_err(|e| e.to_string())? {
            preview.stale = clipboard::now_ms() - preview.fetched_at > link_preview::MAX_AGE_MS;
            return Ok(Some(preview));
        }
    }
    load_link_preview(&app, &db, &url, &parsed, None).await
}

#[tauri::command]
async fn refresh_link_preview(
    app: tauri::AppHandle,
    db: tauri::State<'_, Db>,
    url: String,
) -> Result<Option<LinkPreview>, String> {
    let Some(parsed) = link_preview::parse_url(&url) else {
        return Ok(None);
    };
    let previous = {
        let conn = db.0.lock().unwrap();
        if !db::is_text_clip(&conn, &url).map_err(|e| e.to_string())? {
            return Ok(None);
        }
        db::link_preview(&conn, &url).map_err(|e| e.to_string())?
    };
    load_link_preview(&app, &db, &url, &parsed, previous).await
}

async fn load_link_preview(
    app: &tauri::AppHandle,
    db: &Db,
    url: &str,
    parsed: &url::Url,
    previous: Option<LinkPreview>,
) -> Result<Option<LinkPreview>, String> {
    let images_dir = images::dir(app).map_err(|e| e.to_string())?;
    let Ok(mut preview) = link_preview::fetch(parsed, &images_dir).await else {
        return Ok(previous);
    };
    // Cached under the copied text, which may differ slightly from the normalized URL
    preview.url = url.to_owned();

    let old_files: Vec<String> =
        previous.iter().flat_map(|previous| [previous.image.clone(), previous.icon.clone()]).flatten().collect();
    if let Some(previous) = previous
        && preview.title.is_none()
        && preview.image.is_none()
    {
        let icon = preview.icon.take().or_else(|| previous.icon.clone());
        preview = LinkPreview { stale: false, icon, ..previous };
    }

    let conn = db.0.lock().unwrap();
    db::save_link_preview(&conn, &preview, clipboard::now_ms()).map_err(|e| e.to_string())?;

    for old_file in old_files {
        if !db::is_image_file_used(&conn, &old_file).unwrap_or(true) {
            let _ = images::remove_file(&images_dir, &old_file);
        }
    }
    Ok(Some(preview))
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> settings::Settings {
    settings::get(&app)
}

#[tauri::command]
fn set_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    settings::set_shortcut(&app, &shortcut)
}

#[tauri::command]
fn set_recording_shortcut(recording: bool) {
    settings::set_recording_shortcut(recording);
}

#[tauri::command]
fn set_retention_days(app: tauri::AppHandle, days: Option<u32>) -> Result<(), String> {
    settings::set_retention_days(&app, days)?;
    std::thread::spawn(move || history::purge_or_log(&app));
    Ok(())
}

#[tauri::command]
fn count_expiring_clips(db: tauri::State<Db>, days: u32) -> Result<usize, String> {
    history::expiring_count(&db.0.lock().unwrap(), days).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_launch_at_startup(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    settings::set_launch_at_startup(&app, enabled)
}

#[tauri::command]
fn set_layout(app: tauri::AppHandle, layout: String) -> Result<(), String> {
    settings::set_layout(&app, &layout)
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    background::hide_window(&app);
}

fn backfill_image_sizes(conn: &rusqlite::Connection, images_dir: &std::path::Path) {
    let Ok(images) = db::images_without_size(conn) else { return };
    for (id, hash) in images {
        if let Some(size) = images::file_size(images_dir, &hash) {
            let _ = db::set_byte_size(conn, id, size as i64);
        }
    }
}

fn png_data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", BASE64.encode(png))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            window_icon::apply(&window);
            drop::listen(&window);

            if let Err(e) = apply_mica(&window, None) {
                eprintln!("[vibrancy] mica failed: {e}");
            }

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::open(&data_dir.join("micropaste.db"))?;
            backfill_image_sizes(&conn, &images::dir(app.handle())?);
            if let Ok(exe) = std::env::current_exe() {
                let _ = db::forget_app_icon(&conn, &exe.to_string_lossy());
            }
            app.manage(Db(Mutex::new(conn)));

            clipboard::start_watcher(app.handle().clone());
            history::start_cleanup(app.handle().clone());
            background::setup(app)?;

            Ok(())
        })
        // First: a second launch only brings the running app's window back
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| background::show_window(app)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![background::HIDDEN_ARG]),
        ))
        .plugin(background::shortcut_plugin())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_clips,
            search_clips,
            count_clips,
            set_favorite,
            list_collections,
            create_collection,
            rename_collection,
            reorder_collections,
            delete_collection,
            add_to_collection,
            remove_from_collection,
            reorder_collection,
            clip_collections,
            get_app_icon,
            copy_clip,
            copy_file,
            delete_clip,
            get_file_preview,
            reveal_file,
            missing_files,
            start_drag,
            start_file_drag,
            drop_text,
            drop_image,
            get_link_preview,
            refresh_link_preview,
            get_settings,
            set_shortcut,
            set_recording_shortcut,
            set_retention_days,
            count_expiring_clips,
            set_launch_at_startup,
            set_layout,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
