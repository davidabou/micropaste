use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use arboard::ImageData;
use tauri::{AppHandle, Manager};

pub fn dir(app: &AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().app_data_dir()?.join("images"))
}

pub fn hash(image: &ImageData) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(image.width as u64).to_le_bytes());
    hasher.update(&(image.height as u64).to_le_bytes());
    hasher.update(&image.bytes);
    hasher.finalize().to_hex().to_string()
}

pub fn save(dir: &Path, hash: &str, image: &ImageData) -> io::Result<u64> {
    let file = format!("{hash}.png");
    if let Some(size) = file_size(dir, hash) {
        return Ok(size);
    }
    let png = encode_png(image.width as u32, image.height as u32, &image.bytes)
        .ok_or_else(|| io::Error::other("cannot encode image as PNG"))?;
    save_file(dir, &file, &png)?;
    Ok(png.len() as u64)
}

pub fn file_size(dir: &Path, hash: &str) -> Option<u64> {
    fs::metadata(dir.join(format!("{hash}.png"))).ok().map(|meta| meta.len())
}

pub fn remove_file(dir: &Path, file: &str) -> io::Result<()> {
    match fs::remove_file(dir.join(file)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

pub fn save_file(dir: &Path, file: &str, bytes: &[u8]) -> io::Result<()> {
    let path = dir.join(file);
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(dir)?;
    // Write to a temporary file first so a crash never leaves a half-written image
    let tmp = dir.join(format!("{file}.tmp"));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, &path)
}

pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgba).ok()?;
    writer.finish().ok()?;
    Some(png)
}

pub fn load_rgba(dir: &Path, hash: &str) -> io::Result<(u32, u32, Vec<u8>)> {
    let file = fs::read(dir.join(format!("{hash}.png")))?;
    // Large screenshots exceed the decoder's default memory limit
    let limits = png::Limits { bytes: 1 << 30 };
    let mut decoder = png::Decoder::new_with_limits(io::Cursor::new(file), limits);
    decoder.set_transformations(png::Transformations::ALPHA);
    let mut reader = decoder.read_info().map_err(io::Error::other)?;

    let size = reader.output_buffer_size().ok_or_else(|| io::Error::other("image too large"))?;
    let mut rgba = vec![0; size];
    let info = reader.next_frame(&mut rgba).map_err(io::Error::other)?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(io::Error::other("unexpected image format"));
    }
    rgba.truncate(info.buffer_size());
    Ok((info.width, info.height, rgba))
}
