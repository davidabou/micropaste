use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, Window};
use tauri_plugin_dialog::DialogExt;

use crate::db::{self, Db};
use crate::images;

#[derive(Serialize)]
pub struct Saved {
    saved: usize,
    total: usize,
}

/// The dialog runs here, not in the page: the page never gets to write files.
/// `None` when the dialog is cancelled.
pub fn save_as(app: &AppHandle, window: &Window, id: i64) -> Result<Option<Saved>, String> {
    let clip = db::get(&app.state::<Db>().conn(), id).map_err(|e| e.to_string())?;
    let dialog = app.dialog().file().set_parent(window);
    match clip.kind.as_str() {
        "image" => {
            let source = images::dir(app).map_err(|e| e.to_string())?.join(format!("{}.png", clip.content));
            let name = match (clip.width, clip.height) {
                (Some(width), Some(height)) => format!("Image ({width} × {height}).png"),
                _ => "Image.png".to_owned(),
            };
            let Some(target) = dialog.set_file_name(name).add_filter("PNG image", &["png"]).blocking_save_file() else {
                return Ok(None);
            };
            copy(&source, &target.into_path().map_err(|e| e.to_string())?)?;
            Ok(Some(Saved { saved: 1, total: 1 }))
        }
        "files" => {
            let paths: Vec<&Path> = clip.content.lines().map(Path::new).collect();
            if let [path] = paths[..]
                && path.is_dir()
            {
                return Err("Folders can’t be saved".into());
            }
            let existing: Vec<&Path> = paths.iter().copied().filter(|path| path.is_file()).collect();
            if existing.is_empty() {
                return Err(crate::copy::files_not_found(&clip.content));
            }
            if let [path] = paths[..] {
                let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
                let Some(target) = dialog.set_file_name(name).blocking_save_file() else { return Ok(None) };
                copy(path, &target.into_path().map_err(|e| e.to_string())?)?;
                return Ok(Some(Saved { saved: 1, total: 1 }));
            }

            let Some(folder) = dialog.blocking_pick_folder() else { return Ok(None) };
            let folder = folder.into_path().map_err(|e| e.to_string())?;
            for path in &existing {
                let name = path.file_name().ok_or("Not a file")?;
                copy(path, &free_path(&folder, Path::new(name)))?;
            }
            Ok(Some(Saved { saved: existing.len(), total: paths.len() }))
        }
        _ => Err("Only images and files can be saved".into()),
    }
}

fn copy(source: &Path, target: &Path) -> Result<(), String> {
    // Copying a file onto itself would empty it
    if fs::canonicalize(source).ok() == fs::canonicalize(target).ok() && target.exists() {
        return Ok(());
    }
    fs::copy(source, target).map(drop).map_err(|e| e.to_string())
}

/// Like the Explorer: `name (2).ext` when the folder already has `name.ext`.
fn free_path(folder: &Path, name: &Path) -> PathBuf {
    let stem = name.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
    let extension = name.extension().map(|extension| format!(".{}", extension.to_string_lossy())).unwrap_or_default();
    (1..)
        .map(|n| if n == 1 { folder.join(name) } else { folder.join(format!("{stem} ({n}){extension}")) })
        .find(|path| !path.exists())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_files_never_replace_existing_ones() {
        let folder = std::env::temp_dir().join(format!("micropaste-save-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        assert_eq!(free_path(&folder, Path::new("a.txt")), folder.join("a.txt"));

        fs::write(folder.join("a.txt"), "").unwrap();
        fs::write(folder.join("a (2).txt"), "").unwrap();
        fs::write(folder.join("README"), "").unwrap();
        assert_eq!(free_path(&folder, Path::new("a.txt")), folder.join("a (3).txt"));
        assert_eq!(free_path(&folder, Path::new("README")), folder.join("README (2)"));

        let source = folder.join("README");
        fs::write(&source, "keep").unwrap();
        copy(&source, &source).unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "keep");
        let _ = fs::remove_dir_all(&folder);
    }
}
