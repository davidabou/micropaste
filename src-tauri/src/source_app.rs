use std::path::Path;

/// Prefix of the `path` of Microsoft Store (packaged) apps, followed by their AppUserModelID.
/// The shell resolves it to the app's display name and icon, like the Start menu does.
pub const PACKAGED_APP_PREFIX: &str = r"shell:AppsFolder\";

#[derive(Debug, Clone)]
pub struct SourceApp {
    /// Display name, e.g. "Google Chrome"
    pub name: String,
    /// Executable path, or `shell:AppsFolder\<AppUserModelID>` for Microsoft Store apps
    pub path: String,
}

/// Finds the app that just wrote to the clipboard.
#[cfg(windows)]
pub fn detect() -> Option<SourceApp> {
    use windows::Win32::System::DataExchange::GetClipboardOwner;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    // Copied in micropaste's own window, e.g. part of a text selected on a tile
    if is_own_window() {
        let exe = std::env::current_exe().ok()?.to_string_lossy().into_owned();
        return Some(SourceApp { name: "Micropaste".to_owned(), path: exe });
    }

    // The clipboard owner is the window that wrote the data. Some apps write without
    // an owner, and Store apps write through a Windows service: in both cases fall
    // back to the foreground window, where Ctrl+C was pressed.
    let owner = unsafe { GetClipboardOwner() }.ok().and_then(win::process_of_window);
    let via_system_host = owner.as_ref().is_some_and(win::Process::is_system_host);
    let owner = owner.filter(|_| !via_system_host);

    let foreground = || win::process_of_window(unsafe { GetForegroundWindow() });
    let process = owner.or_else(|| {
        let foreground = foreground()?;
        // A classic app in the foreground doesn't write through the Windows service:
        // the copy came from a background Store app, usually a Win+Shift+S screenshot
        if via_system_host && foreground.app_user_model_id.is_none() {
            if let Some(snipping_tool) = win::running_snipping_tool() {
                return Some(snipping_tool);
            }
        }
        Some(foreground)
    })?;
    describe(process)
}

/// The app in the foreground, e.g. the one content is dragged from: the click that starts a drag
/// brings it there. `None` if it is micropaste itself.
#[cfg(windows)]
pub fn foreground() -> Option<SourceApp> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let window = unsafe { GetForegroundWindow() };
    if win::window_pid(window) == Some(std::process::id()) {
        return None;
    }
    describe(win::process_of_window(window)?)
}

#[cfg(not(windows))]
pub fn foreground() -> Option<SourceApp> {
    None
}

/// Display name and path of an app from its process: the shell's for a Store app.
#[cfg(windows)]
fn describe(process: win::Process) -> Option<SourceApp> {
    if let Some(aumid) = process.app_user_model_id {
        let path = format!("{PACKAGED_APP_PREFIX}{aumid}");
        if let Some(name) = win::shell_display_name(&path) {
            return Some(SourceApp { name, path });
        }
    }

    let name = win::file_description(&process.exe).unwrap_or_else(|| file_stem(&process.exe));
    Some(SourceApp { name, path: process.exe })
}

#[cfg(not(windows))]
pub fn detect() -> Option<SourceApp> {
    None
}

/// Whether the clipboard was written by the micropaste process itself, i.e. a clip put back
/// by clicking its tile. Copies made in its window come from its WebView2 processes instead.
#[cfg(windows)]
pub fn is_written_by_own_process() -> bool {
    clipboard_owner_pid() == Some(std::process::id())
}

#[cfg(not(windows))]
pub fn is_written_by_own_process() -> bool {
    false
}

#[cfg(windows)]
fn clipboard_owner_pid() -> Option<u32> {
    use windows::Win32::System::DataExchange::GetClipboardOwner;
    let owner = unsafe { GetClipboardOwner() }.ok().filter(|hwnd| !hwnd.is_invalid());
    owner.and_then(win::window_pid)
}

/// Whether the copy was made in micropaste's window, handled by its WebView2 processes.
#[cfg(windows)]
fn is_own_window() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let own_pid = std::process::id();
    let owner_pid = clipboard_owner_pid();
    let foreground_is_own = win::window_pid(unsafe { GetForegroundWindow() }) == Some(own_pid);
    let owner_is_webview = match owner_pid {
        None => true,
        Some(pid) => win::open_process(pid)
            .is_some_and(|process| file_name_is(&process.exe, "msedgewebview2.exe")),
    };
    foreground_is_own && owner_is_webview
}

fn file_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned())
}

fn file_name_is(path: &str, name: &str) -> bool {
    Path::new(path)
        .file_name()
        .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
}

/// Initializes COM on the current thread, required by the shell APIs.
#[cfg(windows)]
pub fn init_com() {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
    // Fails harmlessly when COM is already initialized on this thread
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::mem::size_of;

    use windows::core::{HSTRING, PWSTR};
    use windows::Win32::Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, HWND};
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };
    use windows::Win32::Storage::Packaging::Appx::GetApplicationUserModelId;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName, SIGDN_NORMALDISPLAY};
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, GetWindowThreadProcessId};

    use super::{file_name_is, init_com};

    pub struct Process {
        pub exe: String,
        /// Set for Microsoft Store (packaged) apps
        pub app_user_model_id: Option<String>,
    }

    impl Process {
        /// Windows services that write to the clipboard on behalf of other apps.
        pub fn is_system_host(&self) -> bool {
            file_name_is(&self.exe, "svchost.exe")
        }
    }

    pub fn process_of_window(hwnd: HWND) -> Option<Process> {
        if hwnd.is_invalid() {
            return None;
        }
        let pid = window_pid(hwnd)?;
        let process = open_process(pid)?;

        // Store apps run inside a frame window owned by ApplicationFrameHost:
        // the real app is the child window that belongs to another process
        if file_name_is(&process.exe, "ApplicationFrameHost.exe") {
            return hosted_app_pid(hwnd, pid).and_then(open_process).or(Some(process));
        }
        Some(process)
    }

    pub fn window_pid(hwnd: HWND) -> Option<u32> {
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        (pid != 0).then_some(pid)
    }

    fn hosted_app_pid(frame: HWND, frame_pid: u32) -> Option<u32> {
        let mut child = None;
        loop {
            let next = unsafe { FindWindowExW(Some(frame), child, None, None) }.ok()?;
            match window_pid(next) {
                Some(pid) if pid != frame_pid => return Some(pid),
                _ => child = Some(next),
            }
        }
    }

    /// The Snipping Tool process if it is running (it handles Win+Shift+S screenshots).
    pub fn running_snipping_tool() -> Option<Process> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut found = None;
        let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
        while has_entry {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            let exe = String::from_utf16_lossy(&entry.szExeFile[..len]);
            if ["ScreenClippingHost.exe", "SnippingTool.exe"]
                .iter()
                .any(|name| exe.eq_ignore_ascii_case(name))
            {
                found = Some(entry.th32ProcessID);
                break;
            }
            has_entry = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
        }

        unsafe {
            let _ = CloseHandle(snapshot);
        }
        open_process(found?)
    }

    pub fn open_process(pid: u32) -> Option<Process> {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
        let exe = exe_path(handle);
        let app_user_model_id = app_user_model_id(handle);
        unsafe {
            let _ = CloseHandle(handle);
        }
        Some(Process { exe: exe?, app_user_model_id })
    }

    fn exe_path(process: HANDLE) -> Option<String> {
        let mut buffer = [0u16; 1024];
        let mut len = buffer.len() as u32;
        unsafe {
            QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut len)
        }
        .ok()?;
        Some(String::from_utf16_lossy(&buffer[..len as usize]))
    }

    fn app_user_model_id(process: HANDLE) -> Option<String> {
        let mut buffer = [0u16; 512];
        let mut len = buffer.len() as u32;
        let result =
            unsafe { GetApplicationUserModelId(process, &mut len, Some(PWSTR(buffer.as_mut_ptr()))) };
        if result != ERROR_SUCCESS || len == 0 {
            return None; // Not a packaged app
        }
        // `len` includes the trailing NUL
        Some(String::from_utf16_lossy(&buffer[..len as usize - 1]))
    }

    /// Localized name the shell shows for a path, e.g. "Calculatrice".
    pub fn shell_display_name(path: &str) -> Option<String> {
        init_com();
        let item: IShellItem =
            unsafe { SHCreateItemFromParsingName(&HSTRING::from(path), None) }.ok()?;
        let name = unsafe { item.GetDisplayName(SIGDN_NORMALDISPLAY) }.ok()?;
        let text = unsafe { name.to_string() }.ok();
        unsafe { CoTaskMemFree(Some(name.0 as *const c_void)) };
        text.filter(|text| !text.is_empty())
    }

    /// The "File description" from the executable's version info, e.g. "Google Chrome".
    pub fn file_description(path: &str) -> Option<String> {
        let path = HSTRING::from(path);
        let size = unsafe { GetFileVersionInfoSizeW(&path, None) };
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        unsafe { GetFileVersionInfoW(&path, None, size, data.as_mut_ptr().cast()) }.ok()?;

        // Strings are stored per language: use the first declared language/codepage.
        // The translation length is in bytes, string lengths are in characters.
        let (translation, bytes) = query_value(&data, r"\VarFileInfo\Translation")?;
        if bytes < 4 {
            return None;
        }
        let (lang, codepage) = unsafe { (*translation, *translation.add(1)) };
        let key = format!(r"\StringFileInfo\{lang:04x}{codepage:04x}\FileDescription");

        let (chars, len) = query_value(&data, &key)?;
        let chars = unsafe { std::slice::from_raw_parts(chars, len as usize) };
        // Some executables declare a wrong length: the string really ends at the first NUL
        let end = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
        let description = String::from_utf16_lossy(&chars[..end]);
        let description = description.trim();
        (!description.is_empty()).then(|| description.to_owned())
    }

    fn query_value(data: &[u8], key: &str) -> Option<(*const u16, u32)> {
        let mut value: *mut c_void = std::ptr::null_mut();
        let mut len = 0u32;
        let found =
            unsafe { VerQueryValueW(data.as_ptr().cast(), &HSTRING::from(key), &mut value, &mut len) };
        if !found.as_bool() || value.is_null() || len == 0 {
            return None;
        }
        Some((value as *const u16, len))
    }
}
