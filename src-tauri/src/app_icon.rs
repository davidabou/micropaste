/// Size of the extracted icons, in pixels. 32px stays sharp when shown at 16px on HiDPI screens.
pub const ICON_SIZE: u32 = 32;

/// Extracts the icon of a source app (see `SourceApp::path`) as PNG bytes.
#[cfg(windows)]
pub fn extract_png(path: &str) -> Option<Vec<u8>> {
    let (width, height, rgba) = if path.starts_with(crate::source_app::PACKAGED_APP_PREFIX) {
        // Store apps keep their icon in their package, not in their executable
        win::shell_image_rgba(path, ICON_SIZE, win::SIIGBF_ICONONLY)?
    } else {
        win::extract_rgba(path, ICON_SIZE)?
    };
    crate::images::encode_png(width, height, &rgba)
}

#[cfg(not(windows))]
pub fn extract_png(_path: &str) -> Option<Vec<u8>> {
    None
}

/// Size of file thumbnails on tiles, in pixels: sharp enough to fill a tile on HiDPI screens.
pub const THUMBNAIL_SIZE: u32 = 384;
/// Size of file icons on tiles, shown at 64px.
pub const FILE_ICON_SIZE: u32 = 96;

/// Preview of a file as the Explorer shows it, as PNG bytes: its thumbnail (pictures,
/// videos, PDFs…) fitting in `thumbnail_size`, with `true`, or else its icon of `icon_size`,
/// with `false`. `None` if the file is gone.
#[cfg(windows)]
pub fn file_preview_png(path: &str, thumbnail_size: u32, icon_size: u32) -> Option<(Vec<u8>, bool)> {
    if let Some((width, height, rgba)) = win::shell_image_rgba(path, thumbnail_size, win::SIIGBF_THUMBNAILONLY) {
        return Some((crate::images::encode_png(width, height, &rgba)?, true));
    }
    let (width, height, rgba) = win::shell_image_rgba(path, icon_size, win::SIIGBF_ICONONLY)?;
    Some((crate::images::encode_png(width, height, &rgba)?, false))
}

#[cfg(not(windows))]
pub fn file_preview_png(_path: &str, _thumbnail_size: u32, _icon_size: u32) -> Option<(Vec<u8>, bool)> {
    None
}

#[cfg(windows)]
mod win {
    use std::mem::size_of;

    use windows::core::HSTRING;
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC,
    };
    use windows::Win32::Foundation::SIZE;
    pub use windows::Win32::UI::Shell::{SIIGBF_ICONONLY, SIIGBF_THUMBNAILONLY};
    use windows::Win32::UI::Shell::{
        IShellItemImageFactory, SHCreateItemFromParsingName, SHDefExtractIconW, SIIGBF,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

    /// Returns `(width, height, rgba pixels)` of the executable's main icon.
    pub fn extract_rgba(exe_path: &str, size: u32) -> Option<(u32, u32, Vec<u8>)> {
        let mut icon = HICON::default();
        let result =
            unsafe { SHDefExtractIconW(&HSTRING::from(exe_path), 0, 0, Some(&mut icon), None, size) };
        if result.is_err() || icon.is_invalid() {
            return None;
        }

        let pixels = icon_to_rgba(icon);
        unsafe {
            let _ = DestroyIcon(icon);
        }
        pixels
    }

    /// Image the shell shows for a path, e.g. a Store app in the Start menu or a file in the Explorer.
    /// `flags` picks the icon or the thumbnail, scaled down to fit in `size`.
    pub fn shell_image_rgba(path: &str, size: u32, flags: SIIGBF) -> Option<(u32, u32, Vec<u8>)> {
        crate::source_app::init_com();
        let factory: IShellItemImageFactory =
            unsafe { SHCreateItemFromParsingName(&HSTRING::from(path), None) }.ok()?;
        let size = SIZE { cx: size as i32, cy: size as i32 };
        let bitmap = unsafe { factory.GetImage(size, flags) }.ok()?;

        let dc = unsafe { GetDC(None) };
        let pixels = bitmap_size(bitmap).and_then(|(width, height)| {
            let mut pixels = read_bgra(dc, bitmap, width, height)?;
            // Thumbnails of opaque pictures come without alpha channel
            let opaque = pixels.chunks_exact(4).all(|px| px[3] == 0);
            for px in pixels.chunks_exact_mut(4) {
                if opaque {
                    px[3] = 255;
                }
                // The shell returns premultiplied alpha
                let alpha = px[3] as u32;
                if alpha > 0 && alpha < 255 {
                    for channel in &mut px[..3] {
                        *channel = (*channel as u32 * 255 / alpha).min(255) as u8;
                    }
                }
                px.swap(0, 2); // BGRA -> RGBA
            }
            Some((width, height, pixels))
        });

        unsafe {
            ReleaseDC(None, dc);
            let _ = DeleteObject(bitmap.into());
        }
        pixels
    }

    fn icon_to_rgba(icon: HICON) -> Option<(u32, u32, Vec<u8>)> {
        let mut info = ICONINFO::default();
        unsafe { GetIconInfo(icon, &mut info) }.ok()?;

        let dc = unsafe { GetDC(None) };
        let pixels = if info.hbmColor.is_invalid() {
            None // Monochrome icon, not worth supporting
        } else {
            read_rgba(dc, info.hbmColor, info.hbmMask)
        };

        unsafe {
            ReleaseDC(None, dc);
            let _ = DeleteObject(info.hbmColor.into());
            let _ = DeleteObject(info.hbmMask.into());
        }
        pixels
    }

    fn read_rgba(dc: HDC, color: HBITMAP, mask: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
        let (width, height) = bitmap_size(color)?;
        let mut pixels = read_bgra(dc, color, width, height)?;

        // Old icons have no alpha channel: take transparency from the mask instead
        if pixels.chunks_exact(4).all(|px| px[3] == 0) {
            let mask = read_bgra(dc, mask, width, height)?;
            for (px, mask_px) in pixels.chunks_exact_mut(4).zip(mask.chunks_exact(4)) {
                px[3] = if mask_px[0] == 0 { 255 } else { 0 };
            }
        }

        // BGRA -> RGBA
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        Some((width, height, pixels))
    }

    fn bitmap_size(bitmap: HBITMAP) -> Option<(u32, u32)> {
        let mut info = BITMAP::default();
        let read = unsafe {
            GetObjectW(bitmap.into(), size_of::<BITMAP>() as i32, Some((&mut info as *mut BITMAP).cast()))
        };
        (read != 0).then_some((info.bmWidth as u32, info.bmHeight.unsigned_abs()))
    }

    fn read_bgra(dc: HDC, bitmap: HBITMAP, width: u32, height: u32) -> Option<Vec<u8>> {
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // Negative height: rows top to bottom
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let lines = unsafe {
            GetDIBits(dc, bitmap, 0, height, Some(pixels.as_mut_ptr().cast()), &mut info, DIB_RGB_COLORS)
        };
        (lines != 0).then_some(pixels)
    }
}
