//! Snap Layouts (the window layouts Windows 11 shows over the maximize button) for the custom title bar.
//! Windows only offers them over its own maximize button, which the window doesn't have: the title bar
//! opens them with their shortcut, Win+Z, as the native button would after a short hover. And closes
//! them, like the native button, once the pointer has left both the button and the layouts.

use serde::Deserialize;

/// Maximize button of the title bar, in physical screen pixels.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ButtonRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(windows)]
pub fn show(button: ButtonRect) {
    use std::thread::sleep;
    use std::time::Duration;
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_LWIN, VK_MENU, VIRTUAL_KEY};
    use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, GetWindowRect, IsWindowVisible};

    const VK_Z: VIRTUAL_KEY = VIRTUAL_KEY(0x5A);
    /// Time the pointer may spend outside, e.g. moving from the button to the layouts
    const GRACE: Duration = Duration::from_millis(300);
    const POLL: Duration = Duration::from_millis(50);

    let window = unsafe { GetForegroundWindow() };
    press(&[(VK_LWIN, false), (VK_Z, false), (VK_Z, true), (VK_LWIN, true)]);

    // The shortcut also numbers the layouts for the keyboard: a press of Alt hides those numbers
    sleep(Duration::from_millis(50));
    press(&[(VK_MENU, false), (VK_MENU, true)]);

    // The layouts take the foreground: that is how they are found
    sleep(Duration::from_millis(100));
    let layouts = unsafe { GetForegroundWindow() };
    if layouts.is_invalid() || layouts == window {
        return;
    }

    let inside = |point: POINT, rect: RECT| {
        point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
    };
    // A few pixels around the button, so the pointer doesn't leave it on its very edge
    let button = RECT {
        left: button.x as i32 - 4,
        top: button.y as i32 - 4,
        right: (button.x + button.width) as i32 + 4,
        bottom: (button.y + button.height) as i32 + 4,
    };

    let mut outside = Duration::ZERO;
    loop {
        sleep(POLL);
        // Closed meanwhile: a layout was picked, or a click elsewhere
        if unsafe { GetForegroundWindow() } != layouts || !unsafe { IsWindowVisible(layouts) }.as_bool() {
            return;
        }
        let mut point = POINT::default();
        let mut rect = RECT::default();
        if unsafe { GetCursorPos(&mut point) }.is_err() || unsafe { GetWindowRect(layouts, &mut rect) }.is_err() {
            return;
        }
        if inside(point, button) || inside(point, rect) {
            outside = Duration::ZERO;
            continue;
        }
        outside += POLL;
        if outside >= GRACE {
            press(&[(VK_ESCAPE, false), (VK_ESCAPE, true)]);
            return;
        }
    }
}

/// Sends key presses (`true` for a release) to the focused window.
#[cfg(windows)]
fn press(keys: &[(windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY, bool)]) {
    use std::mem::size_of;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    };

    let inputs: Vec<INPUT> = keys
        .iter()
        .map(|&(vk, up)| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                    ..Default::default()
                },
            },
        })
        .collect();
    unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
}

#[cfg(not(windows))]
pub fn show(_button: ButtonRect) {}
