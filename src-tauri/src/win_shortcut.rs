//! Shortcuts with Win. Explorer owns most of them (`RegisterHotKey` fails on Win+V) and releasing
//! Win would open Start, so a low-level keyboard hook takes them before Explorer does. While the
//! settings record a shortcut, it also takes any Win combination, which the page never receives.
//! Set once at launch and kept: it lets every other key through right away.

use std::sync::atomic::{AtomicU32, Ordering};

pub const RECORDED_EVENT: &str = "shortcut-recorded";

const CTRL: u32 = 1;
const ALT: u32 = 2;
const SHIFT: u32 = 4;
const WIN: u32 = 8;

/// `modifiers << 16 | virtual key` of the shortcut, 0 when it has no Win.
static TARGET: AtomicU32 = AtomicU32::new(0);

pub fn uses_win(shortcut: &str) -> bool {
    shortcut.split('+').any(|part| modifier(part) == Some(WIN))
}

pub use hook::start;

pub fn set_target(shortcut: Option<&str>) -> Result<(), String> {
    let target = match shortcut {
        Some(shortcut) => parse(shortcut).ok_or("This shortcut isn’t supported")?,
        None => 0,
    };
    TARGET.store(target, Ordering::Relaxed);
    Ok(())
}

fn modifier(part: &str) -> Option<u32> {
    match part.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some(CTRL),
        "alt" | "option" => Some(ALT),
        "shift" => Some(SHIFT),
        "super" | "win" | "meta" | "cmd" | "command" => Some(WIN),
        _ => None,
    }
}

fn parse(shortcut: &str) -> Option<u32> {
    let mut parts: Vec<&str> = shortcut.split('+').collect();
    let key = key_code(parts.pop()?)?;
    let mods = parts.into_iter().try_fold(0, |mods, part| modifier(part).map(|bit| mods | bit))?;
    Some(mods << 16 | u32::from(key))
}

/// Same format as the settings page: `Ctrl+Alt+Shift+Super+V`.
fn shortcut_name(mods: u32, key: u16) -> Option<String> {
    let names = [(CTRL, "Ctrl"), (ALT, "Alt"), (SHIFT, "Shift"), (WIN, "Super")];
    let mut parts: Vec<String> =
        names.iter().filter(|(bit, _)| mods & bit != 0).map(|(_, name)| (*name).to_string()).collect();
    parts.push(key_name(key)?);
    Some(parts.join("+"))
}

/// Punctuation is left out: its virtual keys depend on the keyboard layout.
const NAMED_KEYS: [(u16, &str); 15] = [
    (0x08, "Backspace"),
    (0x09, "Tab"),
    (0x0D, "Enter"),
    (0x20, "Space"),
    (0x21, "PageUp"),
    (0x22, "PageDown"),
    (0x23, "End"),
    (0x24, "Home"),
    (0x25, "ArrowLeft"),
    (0x26, "ArrowUp"),
    (0x27, "ArrowRight"),
    (0x28, "ArrowDown"),
    (0x2D, "Insert"),
    (0x2E, "Delete"),
    (0x1B, "Escape"),
];

fn key_name(key: u16) -> Option<String> {
    match key {
        0x30..=0x39 | 0x41..=0x5A => Some(char::from(key as u8).to_string()),
        0x70..=0x87 => Some(format!("F{}", key - 0x6F)),
        _ => NAMED_KEYS.iter().find(|(code, _)| *code == key).map(|(_, name)| (*name).to_string()),
    }
}

fn key_code(name: &str) -> Option<u16> {
    let name = name.strip_prefix("Key").or_else(|| name.strip_prefix("Digit")).unwrap_or(name);
    if let [byte] = name.as_bytes() {
        let byte = byte.to_ascii_uppercase();
        return (byte.is_ascii_uppercase() || byte.is_ascii_digit()).then_some(u16::from(byte));
    }
    if let Some(number) = name.strip_prefix(['F', 'f']).and_then(|n| n.parse::<u16>().ok()) {
        return (1..=24).contains(&number).then_some(0x6F + number);
    }
    NAMED_KEYS.iter().find(|(_, key)| key.eq_ignore_ascii_case(name)).map(|(code, _)| *code)
}

#[cfg(not(windows))]
mod hook {
    pub fn start(_app: &tauri::AppHandle) {}
}

#[cfg(windows)]
mod hook {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering};
    use std::time::{Duration, Instant};

    use tauri::{AppHandle, Emitter};
    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
        VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, KBDLLHOOKSTRUCT, MSG, PM_NOREMOVE, PM_REMOVE, PeekMessageW, PostThreadMessageW,
        SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_APP, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    use super::{ALT, CTRL, SHIFT, TARGET, WIN, key_name, shortcut_name};
    use crate::{background, settings};

    /// Unassigned virtual key, sent while Win is down so that releasing it doesn't open Start.
    const MASK_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);
    const TAKEN_MSG: u32 = WM_APP + 1;
    /// Marks the presses that check the hook gets keys.
    const WARM_UP_TAG: usize = 0x4D50_4B59;
    const WARM_UP_WAIT: Duration = Duration::from_millis(50);
    const WARM_UP_TRIES: u32 = 100;

    static APP: OnceLock<AppHandle> = OnceLock::new();
    static THREAD_ID: AtomicU32 = AtomicU32::new(0);
    /// The key whose press was taken: its repeats and its release are taken too.
    static TAKEN_KEY: AtomicU16 = AtomicU16::new(0);
    static RECEIVING: AtomicBool = AtomicBool::new(false);

    pub fn start(app: &tauri::AppHandle) {
        if APP.set(app.clone()).is_ok() {
            std::thread::spawn(run);
        }
    }

    /// Low-level hooks run on the thread that set them, which must pump messages.
    fn run() {
        let mut msg = MSG::default();
        // Creates the message queue, so that messages can be posted to it
        let _ = unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE) };
        THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
        let module = unsafe { GetModuleHandleW(None) }.ok().map(|module| HINSTANCE(module.0));
        let hook = match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(on_key), module, 0) } {
            Ok(hook) => hook,
            Err(e) => return eprintln!("[shortcut] cannot set the keyboard hook: {e}"),
        };
        warm_up();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            handle(&msg);
        }
        let _ = unsafe { UnhookWindowsHookEx(hook) };
    }

    /// Windows skips a new hook for a while, the window shown at launch then got the user's first
    /// keys (Win+V included): presses of its own go through that, until the hook gets one.
    fn warm_up() {
        let mut msg = MSG::default();
        for _ in 0..WARM_UP_TRIES {
            send_mask(WARM_UP_TAG);
            let deadline = Instant::now() + WARM_UP_WAIT;
            while Instant::now() < deadline {
                while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
                    handle(&msg);
                }
                if RECEIVING.load(Ordering::Relaxed) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        eprintln!("[shortcut] the keyboard hook gets no keys");
    }

    fn handle(msg: &MSG) {
        if msg.message == TAKEN_MSG {
            send_mask(0);
            on_taken(msg.wParam.0 as u32);
        }
    }

    /// Only decides: past a short timeout, Windows lets the key through (and then drops the hook).
    /// The rest runs on the hook's thread once the key is taken.
    unsafe extern "system" fn on_key(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
            let down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
            if info.dwExtraInfo == WARM_UP_TAG {
                RECEIVING.store(true, Ordering::Relaxed);
            }
            if take(info.vkCode as u16, down) {
                return LRESULT(1);
            }
        }
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    fn take(key: u16, down: bool) -> bool {
        if is_modifier(key) {
            return false;
        }
        if !down {
            return TAKEN_KEY.compare_exchange(key, 0, Ordering::Relaxed, Ordering::Relaxed).is_ok();
        }
        if TAKEN_KEY.load(Ordering::Relaxed) == key {
            return true;
        }
        let mods = modifiers();
        if mods & WIN == 0 {
            return false;
        }
        let combo = mods << 16 | u32::from(key);
        let wanted = if settings::RECORDING_SHORTCUT.load(Ordering::Relaxed) {
            key_name(key).is_some()
        } else {
            combo == TARGET.load(Ordering::Relaxed)
        };
        if wanted {
            TAKEN_KEY.store(key, Ordering::Relaxed);
            let thread = THREAD_ID.load(Ordering::Relaxed);
            let _ = unsafe { PostThreadMessageW(thread, TAKEN_MSG, WPARAM(combo as usize), LPARAM(0)) };
        }
        wanted
    }

    fn on_taken(combo: u32) {
        let Some(app) = APP.get() else { return };
        if settings::RECORDING_SHORTCUT.load(Ordering::Relaxed) {
            if let Some(shortcut) = shortcut_name(combo >> 16, combo as u16) {
                let _ = app.emit(super::RECORDED_EVENT, shortcut);
            }
        } else {
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || background::toggle_window(&handle));
        }
    }

    fn is_modifier(key: u16) -> bool {
        matches!(key, 0x10..=0x12 | 0xA0..=0xA5 | 0x5B | 0x5C) || key == MASK_KEY.0
    }

    fn modifiers() -> u32 {
        let mut mods = 0;
        if is_down(VK_CONTROL) {
            mods |= CTRL;
        }
        if is_down(VK_MENU) {
            mods |= ALT;
        }
        if is_down(VK_SHIFT) {
            mods |= SHIFT;
        }
        if is_win_down() {
            mods |= WIN;
        }
        mods
    }

    fn is_win_down() -> bool {
        is_down(VK_LWIN) || is_down(VK_RWIN)
    }

    fn is_down(key: VIRTUAL_KEY) -> bool {
        let state = unsafe { GetAsyncKeyState(i32::from(key.0)) };
        state < 0
    }

    fn send_mask(tag: usize) {
        let inputs = [mask_input(false, tag), mask_input(true, tag)];
        unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    }

    fn mask_input(up: bool, tag: usize) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: MASK_KEY,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                    dwExtraInfo: tag,
                    ..Default::default()
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shortcuts_with_win() {
        assert_eq!(parse("Super+V"), Some(WIN << 16 | 0x56));
        assert_eq!(parse("Ctrl+Shift+Super+F5"), Some((CTRL | SHIFT | WIN) << 16 | 0x74));
        assert_eq!(parse("Super+Space"), Some(WIN << 16 | 0x20));
        assert_eq!(parse("Super+Comma"), None);
        assert_eq!(parse("Hyper+V"), None);
    }

    #[test]
    fn names_what_it_parses() {
        for shortcut in ["Super+V", "Alt+Super+7", "Ctrl+Alt+Shift+Super+ArrowUp", "Super+F12"] {
            let combo = parse(shortcut).unwrap();
            assert_eq!(shortcut_name(combo >> 16, combo as u16).as_deref(), Some(shortcut));
        }
    }

    #[test]
    fn detects_win() {
        assert!(uses_win("Super+V"));
        assert!(uses_win("Ctrl+Win+V"));
        assert!(!uses_win("Alt+Shift+V"));
    }
}
