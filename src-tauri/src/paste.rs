use tauri::AppHandle;

use crate::copy::PartialCopy;
use crate::source_app::SourceApp;

#[cfg(windows)]
pub fn target() -> Option<SourceApp> {
    crate::source_app::of_window(win::target_window()?)
}

#[cfg(not(windows))]
pub fn target() -> Option<SourceApp> {
    None
}

/// Copies the clip, then pastes it into the app used before Micropaste.
#[cfg(windows)]
pub fn paste(app: &AppHandle, id: i64) -> Result<Option<PartialCopy>, String> {
    let target = win::target_window().ok_or("No app to paste into")?;
    let partial = crate::copy::copy(app, id)?;
    crate::background::hide_window(app);
    win::paste_into(target)?;
    Ok(partial)
}

#[cfg(not(windows))]
pub fn paste(_app: &AppHandle, _id: i64) -> Result<Option<PartialCopy>, String> {
    Err("No app to paste into".into())
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
        VK_CONTROL, VK_V,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GW_HWNDNEXT, GWL_EXSTYLE, GetClassNameW, GetForegroundWindow, GetTopWindow, GetWindow, GetWindowLongPtrW,
        GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };

    /// The desktop and the taskbar have a title but aren't apps
    const SHELL_CLASSES: [&str; 4] = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

    /// The first app window under Micropaste in the z-order: the app used last, like Alt+Tab shows it.
    pub fn target_window() -> Option<HWND> {
        let own_pid = std::process::id();
        let foreground = unsafe { GetForegroundWindow() };
        let mut window = if window_pid(foreground) == Some(own_pid) {
            unsafe { GetWindow(foreground, GW_HWNDNEXT) }
        } else {
            unsafe { GetTopWindow(None) }
        }
        .ok();
        while let Some(hwnd) = window {
            if window_pid(hwnd) != Some(own_pid) && is_app_window(hwnd) {
                return Some(hwnd);
            }
            window = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok();
        }
        None
    }

    pub fn paste_into(target: HWND) -> Result<(), String> {
        // Allowed: Micropaste is the foreground app while its menu is in use
        let _ = unsafe { SetForegroundWindow(target) };
        let deadline = Instant::now() + Duration::from_millis(500);
        while unsafe { GetForegroundWindow() } != target {
            if Instant::now() > deadline {
                return Err("Copied, but couldn’t switch to the app".into());
            }
            sleep(Duration::from_millis(10));
        }
        // Some apps only take keys once they're done handling their activation
        sleep(Duration::from_millis(50));

        let inputs = [key(VK_CONTROL, false), key(VK_V, false), key(VK_V, true), key(VK_CONTROL, true)];
        let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            return Err("Copied, but couldn’t paste".into());
        }
        Ok(())
    }

    fn key(key: VIRTUAL_KEY, up: bool) -> INPUT {
        let flags = if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) };
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: key, dwFlags: flags, ..Default::default() } },
        }
    }

    fn is_app_window(hwnd: HWND) -> bool {
        let shown = unsafe { IsWindowVisible(hwnd) }.as_bool() && !unsafe { IsIconic(hwnd) }.as_bool();
        if !shown || is_cloaked(hwnd) || unsafe { GetWindowTextLengthW(hwnd) } == 0 {
            return false;
        }
        let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
        if ex_style & (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) != 0 {
            return false;
        }
        !SHELL_CLASSES.contains(&class_name(hwnd).as_str())
    }

    /// Suspended Store apps and windows on other virtual desktops are visible but cloaked.
    fn is_cloaked(hwnd: HWND) -> bool {
        let mut cloaked = 0u32;
        let result = unsafe {
            DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut u32 as *mut c_void, size_of::<u32>() as u32)
        };
        result.is_ok() && cloaked != 0
    }

    fn class_name(hwnd: HWND) -> String {
        let mut buffer = [0u16; 64];
        let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
        String::from_utf16_lossy(&buffer[..len.max(0) as usize])
    }

    fn window_pid(hwnd: HWND) -> Option<u32> {
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        (pid != 0).then_some(pid)
    }
}
