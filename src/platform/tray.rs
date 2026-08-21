//! System tray icon with a context menu, replacing the AWT `SystemTray` the
//! Java version relied on.
//!
//! The icon belongs to a hidden helper window, and Win32 windows are bound to
//! the thread that created them, so the icon, its menu and its message pump all
//! live on one dedicated thread.

// Most of this surface is only reachable on Windows.
#![allow(dead_code)]

use crate::core::log_bus;
use std::sync::Arc;

/// Tooltip and menu title of the tray icon.
pub const TRAY_TOOLTIP: &str = "File Organizer";

/// What the user picked in the tray.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommand {
    /// Show the search window.
    Open,
    /// Hide the search window.
    Hide,
    /// Show or hide the search window.
    Toggle,
    /// Show or hide the log console.
    ToggleConsole,
    /// Quit the application.
    Quit,
}

/// Invoked on the tray thread for every command.
pub type TrayCallback = Arc<dyn Fn(TrayCommand) + Send + Sync + 'static>;

/// Owns the tray icon for as long as it is kept alive.
pub struct Tray {
    handle: imp::Handle,
}

impl Tray {
    /// Installs the tray icon and starts its message pump.
    pub fn install(callback: TrayCallback) -> Self {
        log_bus::log("[tray] installing tray icon...");
        Self { handle: imp::install(callback) }
    }

    /// Removes the tray icon and stops its message pump.
    pub fn remove(&self) {
        self.handle.remove();
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        self.remove();
    }
}

#[cfg(windows)]
mod imp {
    use super::{TrayCallback, TrayCommand, TRAY_TOOLTIP};
    use crate::core::log_bus;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
        DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, LoadImageW, PostQuitMessage,
        PostThreadMessageW, RegisterClassW, SetForegroundWindow, TrackPopupMenu, TranslateMessage,
        IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MF_SEPARATOR, MF_STRING, MSG,
        TPM_BOTTOMALIGN, TPM_RIGHTALIGN, WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP,
        WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPED,
    };

    /// Message the shell posts to the helper window for every tray interaction.
    const WM_TRAY_ICON: u32 = WM_APP + 1;

    /// Message other threads post to ask the tray thread to shut down.
    const WM_TRAY_REMOVE: u32 = WM_APP + 2;

    const TRAY_ICON_ID: u32 = 1;

    const MENU_OPEN: usize = 1;
    const MENU_HIDE: usize = 2;
    const MENU_CONSOLE: usize = 3;
    const MENU_QUIT: usize = 4;

    thread_local! {
        /// The tray callback, reachable from the window procedure, which Win32
        /// calls without any way to pass state along.
        static CALLBACK: RefCell<Option<TrayCallback>> = const { RefCell::new(None) };
    }

    /// Remote control for the tray thread, safe to share across threads.
    #[derive(Clone)]
    pub struct Handle {
        thread_id: Arc<AtomicU32>,
    }

    impl Handle {
        pub fn remove(&self) {
            let thread_id = self.thread_id.load(Ordering::SeqCst);
            if thread_id == 0 {
                return;
            }
            unsafe { PostThreadMessageW(thread_id, WM_TRAY_REMOVE, 0, 0) };
        }
    }

    pub fn install(callback: TrayCallback) -> Handle {
        let thread_id = Arc::new(AtomicU32::new(0));
        let handle = Handle { thread_id: Arc::clone(&thread_id) };

        std::thread::Builder::new()
            .name("tray-icon".to_string())
            .spawn(move || {
                thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
                CALLBACK.with(|slot| *slot.borrow_mut() = Some(callback));

                let Some(window) = create_helper_window() else {
                    log_bus::log("[tray:error] unable to create the tray helper window");
                    return;
                };

                let mut icon_data = notify_icon_data(window);
                if unsafe { Shell_NotifyIconW(NIM_ADD, &icon_data) } == 0 {
                    log_bus::log("[tray:error] unable to add the tray icon");
                    unsafe { DestroyWindow(window) };
                    return;
                }
                log_bus::log("[tray] tray icon installed");

                pump_messages();

                unsafe {
                    Shell_NotifyIconW(NIM_DELETE, &mut icon_data);
                    DestroyWindow(window);
                }
                log_bus::log("[tray] tray icon removed");
            })
            .expect("tray thread can be spawned");

        handle
    }

    /// Runs the tray message loop until the icon is asked to go away.
    fn pump_messages() {
        let mut message: MSG = unsafe { std::mem::zeroed() };

        loop {
            let result = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
            if result == 0 || result == -1 {
                break;
            }

            // Thread messages carry no window, so the window procedure never
            // sees them and they are handled right here.
            if message.hwnd.is_null() && message.message == WM_TRAY_REMOVE {
                break;
            }

            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    /// Creates the hidden window that receives the tray notifications.
    fn create_helper_window() -> Option<HWND> {
        let class_name = wide("FileOrganizerTrayWindow");
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };

        let mut class: WNDCLASSW = unsafe { std::mem::zeroed() };
        class.lpfnWndProc = Some(window_proc);
        class.hInstance = instance;
        class.lpszClassName = class_name.as_ptr();

        // A class registered by an earlier install is fine to reuse.
        unsafe { RegisterClassW(&class) };

        let window = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                wide(TRAY_TOOLTIP).as_ptr(),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            )
        };

        (!window.is_null()).then_some(window)
    }

    fn notify_icon_data(window: HWND) -> NOTIFYICONDATAW {
        let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = window;
        data.uID = TRAY_ICON_ID;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = WM_TRAY_ICON;
        data.hIcon = load_icon();

        for (slot, unit) in data.szTip.iter_mut().zip(wide(TRAY_TOOLTIP)) {
            *slot = unit;
        }

        data
    }

    /// Loads the bundled icon, falling back to the generic application icon.
    fn load_icon() -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
        let bundled = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("fileorganizer.ico")))
            .filter(|icon| icon.exists());

        if let Some(path) = bundled {
            let icon = unsafe {
                LoadImageW(
                    std::ptr::null_mut(),
                    wide(&path.to_string_lossy()).as_ptr(),
                    IMAGE_ICON,
                    0,
                    0,
                    LR_LOADFROMFILE | LR_DEFAULTSIZE,
                )
            };
            if !icon.is_null() {
                return icon;
            }
        }

        unsafe { LoadIconW(std::ptr::null_mut(), IDI_APPLICATION) }
    }

    /// Pops the context menu open at the cursor.
    fn show_menu(window: HWND) {
        let menu = unsafe { CreatePopupMenu() };
        if menu.is_null() {
            return;
        }

        unsafe {
            AppendMenuW(menu, MF_STRING, MENU_OPEN, wide("Open (Ctrl+Space)").as_ptr());
            AppendMenuW(menu, MF_STRING, MENU_HIDE, wide("Hide").as_ptr());
            AppendMenuW(menu, MF_STRING, MENU_CONSOLE, wide("Console").as_ptr());
            AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
            AppendMenuW(menu, MF_STRING, MENU_QUIT, wide("Quit").as_ptr());

            let mut cursor = POINT { x: 0, y: 0 };
            GetCursorPos(&mut cursor);

            // Without this the menu refuses to close when it loses the focus.
            SetForegroundWindow(window);

            TrackPopupMenu(
                menu,
                TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
                cursor.x,
                cursor.y,
                0,
                window,
                std::ptr::null(),
            );
            DestroyMenu(menu);
        }
    }

    fn dispatch(command: TrayCommand) {
        CALLBACK.with(|slot| {
            if let Some(callback) = slot.borrow().as_ref() {
                callback(command);
            }
        });
    }

    unsafe extern "system" fn window_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_TRAY_ICON => {
                match lparam as u32 {
                    WM_LBUTTONUP => dispatch(TrayCommand::Toggle),
                    WM_RBUTTONUP => show_menu(window),
                    _ => {}
                }
                0
            }
            WM_COMMAND => {
                match wparam & 0xFFFF {
                    MENU_OPEN => dispatch(TrayCommand::Open),
                    MENU_HIDE => dispatch(TrayCommand::Hide),
                    MENU_CONSOLE => dispatch(TrayCommand::ToggleConsole),
                    MENU_QUIT => dispatch(TrayCommand::Quit),
                    _ => {}
                }
                0
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                0
            }
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        }
    }

    /// Encodes a string as the NUL terminated UTF-16 the Win32 API expects.
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
mod imp {
    use super::TrayCallback;
    use crate::core::log_bus;

    /// The tray is built on the Win32 shell notification area, so elsewhere the
    /// application simply runs without an icon.
    #[derive(Clone)]
    pub struct Handle;

    impl Handle {
        pub fn remove(&self) {}
    }

    pub fn install(_callback: TrayCallback) -> Handle {
        log_bus::log("[tray] the tray icon is only available on Windows");
        Handle
    }
}
