use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use tauri::{AppHandle, Manager, Window};

use crate::db::{self, Db};
use crate::{app_icon, images};

const DRAG_THUMBNAIL_SIZE: u32 = 128;
const DRAG_ICON_SIZE: u32 = 64;

pub fn start(app: &AppHandle, window: Window, id: i64) -> Result<(), String> {
    let clip = db::get(&app.state::<Db>().conn(), id).map_err(|e| e.to_string())?;
    let paths: Vec<PathBuf> = match clip.kind.as_str() {
        "image" => vec![image_file(app, &clip.content)?],
        "files" => clip.content.lines().map(PathBuf::from).filter(|path| path.exists()).collect(),
        _ => return Err("Only images and files are dragged natively".into()),
    };
    if paths.is_empty() {
        return Err(crate::copy::files_not_found(&clip.content));
    }
    drag(app, window, paths)
}

pub fn start_file(app: &AppHandle, window: Window, id: i64, path: &str) -> Result<(), String> {
    let file = crate::copy::clip_file(app, id, path)?;
    drag(app, window, vec![file])
}

fn drag(app: &AppHandle, window: Window, paths: Vec<PathBuf>) -> Result<(), String> {
    let preview = app_icon::file_preview_png(&paths[0].to_string_lossy(), DRAG_THUMBNAIL_SIZE, DRAG_ICON_SIZE)
        .map(|(png, _)| png)
        .unwrap_or_default();

    let (tx, rx) = channel();
    app.run_on_main_thread(move || {
        // OLE drag and drop must run on the window thread
        let result = drag::start_drag(
            &window,
            drag::DragItem::Files(paths),
            drag::Image::Raw(preview),
            |_, _| {},
            drag::Options::default(),
        );
        let _ = tx.send(result.map_err(|e| e.to_string()));
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

pub fn image_file(app: &AppHandle, hash: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("micropaste");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("Micropaste {}.png", &hash[..8.min(hash.len())]));
    if !path.exists() {
        let saved = images::dir(app).map_err(|e| e.to_string())?.join(format!("{hash}.png"));
        fs::copy(saved, &path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}
