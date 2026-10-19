use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use image::GrayImage;
use image::imageops::{self, FilterType};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::db::{self, Db};
use crate::images;

pub const IMAGE_TEXT_EVENT: &str = "image-text-recognized";

pub struct Ocr(Sender<i64>);

#[derive(Clone, Serialize)]
struct Recognized {
    id: i64,
    text: String,
}

/// One worker for every image, newest first: recognition is slow and the clips are only touched briefly.
pub fn start(app: AppHandle) {
    let (sender, receiver) = mpsc::channel();
    let pending = db::images_without_text(&app.state::<Db>().0.lock().unwrap()).unwrap_or_default();
    for id in pending {
        let _ = sender.send(id);
    }
    app.manage(Ocr(sender));
    thread::spawn(move || run(&app, receiver));
}

pub fn queue(app: &AppHandle, id: i64) {
    if let Some(ocr) = app.try_state::<Ocr>() {
        let _ = ocr.0.send(id);
    }
}

fn run(app: &AppHandle, receiver: Receiver<i64>) {
    // Nothing is saved without an OCR language, so the images are tried again once one is installed
    let Some(engine) = Engine::new() else {
        return eprintln!("[ocr] no OCR language installed");
    };
    let Ok(dir) = images::dir(app) else { return };
    for id in receiver {
        let clip = {
            let db = app.state::<Db>();
            let conn = db.0.lock().unwrap();
            match db::get(&conn, id) {
                Ok(clip) if db::needs_image_text(&conn, id).unwrap_or(false) => clip,
                _ => continue,
            }
        };
        let image = if clip.kind == "image" {
            images::load_rgba(&dir, &clip.content)
        } else if Path::new(&clip.content).exists() {
            images::decode_file(Path::new(&clip.content))
        } else {
            // Left for the next launch, the file may come back
            continue;
        };
        // Saved even when empty, so a broken image isn't tried at every launch
        let text = image
            .map_err(|e| e.to_string())
            .and_then(|(width, height, rgba)| engine.recognize(width, height, &rgba))
            .unwrap_or_else(|e| {
                eprintln!("[ocr] cannot read image {id}: {e}");
                String::new()
            });
        let saved = db::set_image_text(&app.state::<Db>().0.lock().unwrap(), id, &text);
        match saved {
            Ok(()) => {
                let _ = app.emit(IMAGE_TEXT_EVENT, Recognized { id, text });
            }
            Err(e) => eprintln!("[ocr] cannot save text of image {id}: {e}"),
        }
    }
}

/// Over white: text on a transparent background would otherwise be black on black.
fn grayscale(width: u32, height: u32, rgba: &[u8]) -> GrayImage {
    let gray = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| {
            let [r, g, b, a] = pixel.map(u32::from);
            let luma = (299 * r + 587 * g + 114 * b) / 1000;
            ((luma * a + 255 * (255 - a)) / 255) as u8
        })
        .collect();
    GrayImage::from_raw(width, height, gray).unwrap_or_else(|| GrayImage::new(width, height))
}

fn fit(width: u32, height: u32, max: u32) -> (u32, u32) {
    let longest = width.max(height);
    if longest <= max {
        return (width, height);
    }
    let scale = |side: u32| ((u64::from(side) * u64::from(max) / u64::from(longest)) as u32).max(1);
    (scale(width), scale(height))
}

fn prepare(width: u32, height: u32, rgba: &[u8], max: u32) -> GrayImage {
    let gray = grayscale(width, height, rgba);
    let (fit_width, fit_height) = fit(width, height, max);
    if (fit_width, fit_height) == (width, height) {
        return gray;
    }
    imageops::resize(&gray, fit_width, fit_height, FilterType::Triangle)
}

#[cfg(windows)]
struct Engine(windows::Media::Ocr::OcrEngine);

#[cfg(windows)]
impl Engine {
    fn new() -> Option<Self> {
        use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};

        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages().ok().map(Self)
    }

    fn recognize(&self, width: u32, height: u32, rgba: &[u8]) -> Result<String, String> {
        use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
        use windows::Media::Ocr::OcrEngine;
        use windows::Storage::Streams::DataWriter;

        let recognize = || -> windows::core::Result<String> {
            let gray = prepare(width, height, rgba, OcrEngine::MaxImageDimension()?);
            let writer = DataWriter::new()?;
            writer.WriteBytes(gray.as_raw())?;
            let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
                &writer.DetachBuffer()?,
                BitmapPixelFormat::Gray8,
                gray.width() as i32,
                gray.height() as i32,
            )?;
            let result = self.0.RecognizeAsync(&bitmap)?.join()?;
            // `OcrResult::Text` puts every line on a single one
            let lines: Vec<String> =
                result.Lines()?.into_iter().filter_map(|line| line.Text().ok()).map(|text| text.to_string()).collect();
            Ok(lines.join("\n").trim().to_owned())
        };
        recognize().map_err(|e| e.to_string())
    }
}

#[cfg(not(windows))]
struct Engine;

#[cfg(not(windows))]
impl Engine {
    fn new() -> Option<Self> {
        None
    }

    fn recognize(&self, _width: u32, _height: u32, _rgba: &[u8]) -> Result<String, String> {
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_pixels_turn_white() {
        let rgba = [0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 128];
        assert_eq!(grayscale(4, 1, &rgba).into_raw(), [255, 0, 255, 127]);
    }

    #[test]
    fn large_images_are_scaled_down_keeping_their_ratio() {
        assert_eq!(fit(800, 600, 10_000), (800, 600));
        assert_eq!(fit(20_000, 5_000, 10_000), (10_000, 2_500));
        assert_eq!(fit(100, 40_000, 10_000), (25, 10_000));
        assert_eq!(fit(1, 40_000, 10_000), (1, 10_000));
        assert_eq!(prepare(4, 2, &[0; 32], 2).dimensions(), (2, 1));
    }
}
