//! Sharp window icon. Tauri gives the window a single image of its icon, the largest one, which
//! Windows then scales down for the taskbar and the title: blurry. The executable holds the icon
//! at every size: the window gets the ones matching the screen scale, as other apps do.

/// Resource id of the icon tauri-build puts in the executable.
#[cfg(windows)]
const ICON_RESOURCE: u16 = 32512;

#[cfg(windows)]
pub fn apply(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
    use windows::Win32::UI::WindowsAndMessaging::{
        ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW, SM_CXICON, SM_CXSMICON, SendMessageW, WM_SETICON,
    };
    use windows::core::PCWSTR;

    let Ok(handle) = window.hwnd() else { return };
    let hwnd = HWND(handle.0);
    let Ok(module) = (unsafe { GetModuleHandleW(PCWSTR::null()) }) else { return };
    let dpi = unsafe { GetDpiForWindow(hwnd) };

    for (kind, metric) in [(ICON_BIG, SM_CXICON), (ICON_SMALL, SM_CXSMICON)] {
        let size = unsafe { GetSystemMetricsForDpi(metric, dpi) };
        // An integer resource id passed as a string pointer, as Windows expects it
        let name = PCWSTR(ICON_RESOURCE as usize as *const u16);
        let Ok(icon) =
            (unsafe { LoadImageW(Some(HINSTANCE(module.0)), name, IMAGE_ICON, size, size, LR_DEFAULTCOLOR) })
        else {
            continue;
        };
        unsafe { SendMessageW(hwnd, WM_SETICON, Some(WPARAM(kind as usize)), Some(LPARAM(icon.0 as isize))) };
    }
}

#[cfg(not(windows))]
pub fn apply(_window: &tauri::WebviewWindow) {}
