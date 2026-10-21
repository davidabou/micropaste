use std::path::PathBuf;

use tauri::{AppHandle, Manager, Window};

use crate::db::{self, Db};

#[derive(Clone)]
enum Content {
    Text(String),
    Link(String),
    Files(Vec<PathBuf>),
}

#[derive(Clone)]
struct Shared {
    title: String,
    content: Content,
}

/// Opens the Windows share panel for a clip.
pub fn share(app: &AppHandle, window: Window, id: i64) -> Result<(), String> {
    let clip = db::get(&app.state::<Db>().conn(), id).map_err(|e| e.to_string())?;
    let shared = match clip.kind.as_str() {
        "image" => Shared {
            title: "Image".into(),
            content: Content::Files(vec![crate::drag_out::image_file(app, &clip.content)?]),
        },
        "files" => {
            let paths: Vec<PathBuf> = clip.content.lines().map(PathBuf::from).filter(|path| path.exists()).collect();
            if paths.is_empty() {
                return Err(crate::copy::files_not_found(&clip.content));
            }
            Shared { title: files_title(&paths), content: Content::Files(paths) }
        }
        _ => match link(&clip.content) {
            Some(url) => Shared { title: url.clone(), content: Content::Link(url) },
            None => Shared { title: text_title(&clip.content), content: Content::Text(clip.content) },
        },
    };
    win::show(app, window, shared)
}

fn link(text: &str) -> Option<String> {
    let text = text.trim();
    if text.contains(char::is_whitespace) {
        return None;
    }
    let url = url::Url::parse(text).ok()?;
    matches!(url.scheme(), "http" | "https").then(|| text.to_owned())
}

const TITLE_LENGTH: usize = 60;

fn text_title(text: &str) -> String {
    let line = text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("Text");
    if line.chars().count() <= TITLE_LENGTH {
        return line.to_owned();
    }
    let short: String = line.chars().take(TITLE_LENGTH - 1).collect();
    format!("{}…", short.trim_end())
}

fn files_title(paths: &[PathBuf]) -> String {
    match paths {
        [path] => path.file_name().map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy()).into_owned(),
        _ => format!("{} files", paths.len()),
    }
}

#[cfg(windows)]
mod win {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::channel;
    use std::sync::{Mutex, PoisonError};

    use tauri::{AppHandle, Window};
    use windows::ApplicationModel::DataTransfer::{DataPackage, DataRequestedEventArgs, DataTransferManager};
    use windows::Foundation::{TypedEventHandler, Uri};
    use windows::Storage::{IStorageItem, StorageFile, StorageFolder};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::IDataTransferManagerInterop;
    use windows::core::{HSTRING, Interface, Ref};
    use windows_collections::IIterable;

    use super::{Content, Shared};

    /// What the panel asks for once open: the handler is registered once per window.
    static SHARED: Mutex<Option<Shared>> = Mutex::new(None);
    static LISTENING: AtomicBool = AtomicBool::new(false);

    pub fn show(app: &AppHandle, window: Window, shared: Shared) -> Result<(), String> {
        *SHARED.lock().unwrap_or_else(PoisonError::into_inner) = Some(shared);
        let (tx, rx) = channel();
        app.run_on_main_thread(move || {
            // The panel and its events belong to the window thread
            let _ = tx.send(open_panel(&window).map_err(|e| e.message()));
        })
        .map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())?
    }

    fn open_panel(window: &Window) -> windows::core::Result<()> {
        let hwnd = HWND(window.hwnd().map_err(|_| windows::core::Error::empty())?.0);
        let interop = windows::core::factory::<DataTransferManager, IDataTransferManagerInterop>()?;
        if !LISTENING.swap(true, Ordering::SeqCst) {
            let manager: DataTransferManager = unsafe { interop.GetForWindow(hwnd) }?;
            if let Err(error) = manager.DataRequested(&TypedEventHandler::new(on_data_requested)) {
                LISTENING.store(false, Ordering::SeqCst);
                return Err(error);
            }
        }
        unsafe { interop.ShowShareUIForWindow(hwnd) }
    }

    fn on_data_requested(_: Ref<DataTransferManager>, args: Ref<DataRequestedEventArgs>) -> windows::core::Result<()> {
        let request = args.ok()?.Request()?;
        let shared = SHARED.lock().unwrap_or_else(PoisonError::into_inner).clone();
        let Some(Shared { title, content }) = shared else {
            return request.FailWithDisplayText(&HSTRING::from("Nothing to share"));
        };
        let package = request.Data()?;
        package.Properties()?.SetTitle(&HSTRING::from(title))?;
        match content {
            Content::Text(text) => package.SetText(&HSTRING::from(text))?,
            Content::Link(url) => {
                package.SetWebLink(&Uri::CreateUri(&HSTRING::from(&url))?)?;
                package.SetText(&HSTRING::from(url))?;
            }
            Content::Files(paths) => {
                // Getting storage items waits on async calls: not on the window thread
                let deferral = request.GetDeferral()?;
                std::thread::spawn(move || {
                    if set_files(&package, &paths).is_err() {
                        let _ = request.FailWithDisplayText(&HSTRING::from("Couldn’t share the files"));
                    }
                    let _ = deferral.Complete();
                });
            }
        }
        Ok(())
    }

    fn set_files(package: &DataPackage, paths: &[PathBuf]) -> windows::core::Result<()> {
        use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let items = paths.iter().map(|path| storage_item(path).map(Some)).collect::<windows::core::Result<Vec<_>>>()?;
        package.SetStorageItemsReadOnly(&IIterable::<IStorageItem>::from(items))
    }

    fn storage_item(path: &Path) -> windows::core::Result<IStorageItem> {
        let name = HSTRING::from(path.as_os_str());
        if path.is_dir() {
            StorageFolder::GetFolderFromPathAsync(&name)?.join()?.cast()
        } else {
            StorageFile::GetFileFromPathAsync(&name)?.join()?.cast()
        }
    }
}

#[cfg(not(windows))]
mod win {
    use tauri::{AppHandle, Window};

    use super::Shared;

    pub fn show(_app: &AppHandle, _window: Window, _shared: Shared) -> Result<(), String> {
        Err("Sharing is only available on Windows".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_web_addresses_are_links() {
        assert_eq!(link(" https://example.com/a ").as_deref(), Some("https://example.com/a"));
        assert_eq!(link("ftp://example.com"), None);
        assert_eq!(link("see https://example.com"), None);
    }

    #[test]
    fn text_title_is_the_first_line_shortened() {
        assert_eq!(text_title("\n  Hello  \nworld"), "Hello");
        assert_eq!(text_title("   "), "Text");
        let title = text_title(&"a".repeat(100));
        assert_eq!(title.chars().count(), TITLE_LENGTH);
        assert!(title.ends_with('…'));
    }
}
