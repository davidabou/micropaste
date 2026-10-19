use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Animation,
    Video,
    Audio,
}

/// What WebView2 plays natively.
pub fn kind(path: &Path) -> Option<MediaKind> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "gif" => Some(MediaKind::Animation),
        "mp4" | "m4v" | "webm" | "mov" => Some(MediaKind::Video),
        "mp3" | "wav" | "ogg" | "oga" | "opus" | "flac" | "m4a" | "aac" => Some(MediaKind::Audio),
        _ => None,
    }
}

/// Lets the page read this one file of the clip through the asset protocol, only if it can play it.
pub fn allow(app: &AppHandle, id: i64, path: &str) -> Result<Option<MediaKind>, String> {
    let file = crate::copy::clip_file(app, id, path)?;
    let Some(kind) = kind(&file) else {
        return Ok(None);
    };
    app.asset_protocol_scope().allow_file(&file).map_err(|e| e.to_string())?;
    Ok(Some(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_what_webview2_supports() {
        assert_eq!(kind(Path::new("C:\\a\\cat.GIF")), Some(MediaKind::Animation));
        assert_eq!(kind(Path::new("C:\\a\\clip.mp4")), Some(MediaKind::Video));
        assert_eq!(kind(Path::new("C:\\a\\song.flac")), Some(MediaKind::Audio));
        assert_eq!(kind(Path::new("C:\\a\\photo.png")), None);
        assert_eq!(kind(Path::new("C:\\a\\notes.txt")), None);
        assert_eq!(kind(Path::new("C:\\a\\Music")), None);
    }
}
