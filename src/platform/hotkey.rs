//! Global `Ctrl+Space` hotkey that toggles the search window.
//!
//! Windows binds a hotkey to the thread that registered it, so registration,
//! the message pump and every later (un)registration all happen on one
//! dedicated thread. Other threads drive it by posting messages to it.

use crate::core::log_bus;
use crate::platform::focus_monitor::{FocusMonitor, FocusVerdict};
use std::sync::Arc;

/// Human readable name of the shortcut, used in log lines.
pub const HOTKEY_NAME: &str = "Ctrl+Space";

/// Invoked on the hotkey thread every time the shortcut fires.
pub type ToggleCallback = Arc<dyn Fn() + Send + Sync + 'static>;

/// Reports whether the shortcut could be claimed from the system.
pub type RegistrationCallback = Arc<dyn Fn(Result<&str, String>) + Send + Sync + 'static>;

/// Owns the hotkey thread and the focus monitor that mutes it.
pub struct HotkeyService {
    on_toggle: ToggleCallback,
    on_registration: Option<RegistrationCallback>,
    handle: Option<imp::Handle>,
    focus_monitor: Option<FocusMonitor>,
}

impl HotkeyService {
    pub fn new(on_toggle: ToggleCallback) -> Self {
        Self { on_toggle, on_registration: None, handle: None, focus_monitor: None }
    }

    /// Registers a callback notified of registration successes and failures.
    pub fn on_registration(mut self, callback: RegistrationCallback) -> Self {
        self.on_registration = Some(callback);
        self
    }

    /// Claims the shortcut and starts muting it while editors or games are focused.
    pub fn start(&mut self) {
        if self.handle.is_some() {
            log_bus::log("[hotkey] service already started");
            return;
        }

        log_bus::log("[hotkey] starting hotkey service...");
        let handle = imp::start(Arc::clone(&self.on_toggle), self.on_registration.clone());

        let monitor_handle = handle.clone();
        let monitor = FocusMonitor::new(Arc::new(move |verdict| {
            monitor_handle.set_active(verdict == FocusVerdict::KeepHotkey);
        }));
        monitor.start();

        self.handle = Some(handle);
        self.focus_monitor = Some(monitor);
    }

    /// Releases the shortcut and stops both threads.
    pub fn stop(&mut self) {
        if let Some(monitor) = self.focus_monitor.take() {
            monitor.stop();
        }
        if let Some(handle) = self.handle.take() {
            log_bus::log("[hotkey] stopping hotkey service...");
            handle.stop();
        }
    }
}

impl Drop for HotkeyService {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
mod imp {
    use super::{RegistrationCallback, ToggleCallback, HOTKEY_NAME};
    use crate::core::log_bus;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_HOTKEY_ALREADY_REGISTERED, ERROR_HOTKEY_NOT_REGISTERED};
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_CONTROL};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, PostThreadMessageW, TranslateMessage, MSG, WM_APP, WM_HOTKEY, WM_QUIT,
    };

    const HOTKEY_ID: i32 = 0xBEEF;
    const VK_SPACE: u32 = 0x20;

    /// Asks the hotkey thread to claim (`wparam` = 1) or release the shortcut.
    const WM_SET_ACTIVE: u32 = WM_APP + 1;

    /// Remote control for the hotkey thread, safe to share across threads.
    #[derive(Clone)]
    pub struct Handle {
        thread_id: Arc<AtomicU32>,
    }

    impl Handle {
        /// Claims or releases the shortcut, depending on `active`.
        pub fn set_active(&self, active: bool) {
            self.post(WM_SET_ACTIVE, usize::from(active));
        }

        /// Ends the message pump, which unregisters the shortcut on its way out.
        pub fn stop(&self) {
            self.post(WM_QUIT, 0);
        }

        fn post(&self, message: u32, wparam: usize) {
            let thread_id = self.thread_id.load(Ordering::SeqCst);
            if thread_id == 0 {
                return;
            }
            unsafe { PostThreadMessageW(thread_id, message, wparam, 0) };
        }
    }

    pub fn start(on_toggle: ToggleCallback, on_registration: Option<RegistrationCallback>) -> Handle {
        let thread_id = Arc::new(AtomicU32::new(0));
        let handle = Handle { thread_id: Arc::clone(&thread_id) };

        std::thread::Builder::new()
            .name("hotkey-loop".to_string())
            .spawn(move || {
                thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
                log_bus::log("[hotkey] hotkey thread started");

                let mut registered = register(on_registration.as_ref());

                log_bus::log("[hotkey] starting message loop...");
                let mut message: MSG = unsafe { std::mem::zeroed() };

                loop {
                    let result = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
                    if result == 0 {
                        log_bus::log("[hotkey] WM_QUIT received, stopping loop");
                        break;
                    }
                    if result == -1 {
                        log_bus::log("[hotkey:error] GetMessage failed");
                        break;
                    }

                    match message.message {
                        WM_HOTKEY if message.wParam as i32 == HOTKEY_ID => {
                            log_bus::log("[hotkey] hotkey detected");
                            on_toggle();
                        }
                        WM_SET_ACTIVE => {
                            let wanted = message.wParam != 0;
                            if wanted && !registered {
                                registered = register(on_registration.as_ref());
                            } else if !wanted && registered {
                                unregister();
                                registered = false;
                            }
                        }
                        _ => unsafe {
                            TranslateMessage(&message);
                            DispatchMessageW(&message);
                        },
                    }
                }

                if registered {
                    unregister();
                }
                log_bus::log("[hotkey] message loop finished");
            })
            .expect("hotkey thread can be spawned");

        handle
    }

    /// Claims `Ctrl+Space` for the calling thread.
    fn register(on_registration: Option<&RegistrationCallback>) -> bool {
        log_bus::log("[hotkey] registering hotkey...");

        let ok = unsafe { RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID, MOD_CONTROL, VK_SPACE) } != 0;
        if ok {
            log_bus::log(format!("[hotkey] {HOTKEY_NAME} registered successfully"));
            if let Some(callback) = on_registration {
                callback(Ok(HOTKEY_NAME));
            }
            return true;
        }

        let code = unsafe { GetLastError() };
        // Another process already owns the shortcut; it will still reach us.
        if code == ERROR_HOTKEY_ALREADY_REGISTERED {
            log_bus::log(format!("[hotkey] {HOTKEY_NAME} already registered elsewhere, treated as claimed"));
            if let Some(callback) = on_registration {
                callback(Ok(HOTKEY_NAME));
            }
            return true;
        }

        let reason = error_message(code);
        log_bus::log(format!("[hotkey:error] unable to register {HOTKEY_NAME}: {reason}"));
        if let Some(callback) = on_registration {
            callback(Err(format!("{HOTKEY_NAME}: {reason}")));
        }
        false
    }

    /// Gives `Ctrl+Space` back to the system.
    fn unregister() {
        let ok = unsafe { UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID) } != 0;
        if ok {
            log_bus::log(format!("[hotkey] {HOTKEY_NAME} released"));
            return;
        }

        let code = unsafe { GetLastError() };
        if code == ERROR_HOTKEY_NOT_REGISTERED {
            log_bus::log(format!("[hotkey] {HOTKEY_NAME} was already released"));
        } else {
            log_bus::log(format!("[hotkey:warning] release failed: {}", error_message(code)));
        }
    }

    fn error_message(code: u32) -> String {
        match code {
            1409 => "hotkey already registered (ERROR_HOTKEY_ALREADY_REGISTERED)".to_string(),
            1419 => "hotkey not registered (ERROR_HOTKEY_NOT_REGISTERED)".to_string(),
            5 => "access denied (ERROR_ACCESS_DENIED)".to_string(),
            8 => "not enough memory (ERROR_NOT_ENOUGH_MEMORY)".to_string(),
            87 => "invalid parameter (ERROR_INVALID_PARAMETER)".to_string(),
            1408 => "invalid window handle (ERROR_INVALID_WINDOW_HANDLE)".to_string(),
            other => format!("unknown error code: {other}"),
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::{RegistrationCallback, ToggleCallback, HOTKEY_NAME};
    use crate::core::log_bus;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{ConnectionExt, GrabMode, Keycode, ModMask};
    use x11rb::protocol::Event;

    /// X11 keysym for the space bar.
    const XK_SPACE: u32 = 0x0020;

    /// How long the event loop sleeps when the X queue is empty.
    const IDLE_POLL: Duration = Duration::from_millis(50);

    /// Lock modifier combinations to grab alongside Control, so the shortcut
    /// still fires with Caps Lock, Num Lock or Scroll Lock engaged.
    const LOCK_COMBINATIONS: [u16; 8] = [
        0,
        1 << 1,           // Lock (Caps Lock)
        1 << 4,           // Mod2 (Num Lock)
        1 << 7,           // Mod5 (Scroll Lock)
        (1 << 1) | (1 << 4),
        (1 << 1) | (1 << 7),
        (1 << 4) | (1 << 7),
        (1 << 1) | (1 << 4) | (1 << 7),
    ];

    /// Remote control for the hotkey thread, safe to share across threads.
    #[derive(Clone)]
    pub struct Handle {
        active: Arc<AtomicBool>,
        running: Arc<AtomicBool>,
    }

    impl Handle {
        /// Mutes or unmutes the shortcut.
        ///
        /// The X grab itself stays in place: releasing and retaking it on every
        /// focus change would race with the application that just took the
        /// focus, and could drop the grab for good.
        pub fn set_active(&self, active: bool) {
            self.active.store(active, Ordering::Relaxed);
        }

        pub fn stop(&self) {
            self.running.store(false, Ordering::Relaxed);
        }
    }

    pub fn start(on_toggle: ToggleCallback, on_registration: Option<RegistrationCallback>) -> Handle {
        let handle = Handle {
            active: Arc::new(AtomicBool::new(true)),
            running: Arc::new(AtomicBool::new(true)),
        };

        // Wayland deliberately keeps global grabs away from applications; only
        // a compositor binding can provide the shortcut there.
        if std::env::var_os("DISPLAY").is_none() {
            let reason = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                "Wayland has no global shortcut API, bind the shortcut in your compositor instead"
            } else {
                "no X display available"
            };
            log_bus::log(format!("[hotkey:error] {HOTKEY_NAME}: {reason}"));
            if let Some(callback) = on_registration {
                callback(Err(format!("{HOTKEY_NAME}: {reason}")));
            }
            return handle;
        }

        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            // XWayland forwards only the key presses of X11 clients, so the
            // grab is real but silent while a native Wayland window is focused.
            log_bus::log(format!(
                "[hotkey:warning] running under Wayland: {HOTKEY_NAME} reaches XWayland windows only, \
                 bind it in your compositor for full coverage"
            ));
        }

        let thread_handle = handle.clone();
        std::thread::Builder::new()
            .name("hotkey-loop".to_string())
            .spawn(move || {
                if let Err(reason) = run(&on_toggle, &thread_handle) {
                    log_bus::log(format!("[hotkey:error] unable to grab {HOTKEY_NAME}: {reason}"));
                    if let Some(callback) = on_registration {
                        callback(Err(format!("{HOTKEY_NAME}: {reason}")));
                    }
                    return;
                }
                if let Some(callback) = on_registration {
                    callback(Ok(HOTKEY_NAME));
                }
            })
            .expect("hotkey thread can be spawned");

        handle
    }

    /// Grabs the shortcut on the root window and dispatches key presses until
    /// the handle is stopped. Errors only describe the grab itself.
    fn run(on_toggle: &ToggleCallback, handle: &Handle) -> Result<(), String> {
        let (connection, screen_index) = x11rb::connect(None).map_err(|error| error.to_string())?;
        let root = connection.setup().roots[screen_index].root;
        let keycode = keycode_for(&connection, XK_SPACE)?;

        for lock in LOCK_COMBINATIONS {
            connection
                .grab_key(
                    true,
                    root,
                    ModMask::CONTROL | ModMask::from(lock),
                    keycode,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .map_err(|error| error.to_string())?;
        }
        connection.flush().map_err(|error| error.to_string())?;
        log_bus::log(format!("[hotkey] {HOTKEY_NAME} grabbed on the root window"));

        while handle.running.load(Ordering::Relaxed) {
            match connection.poll_for_event() {
                Ok(Some(Event::KeyPress(event))) if event.detail == keycode => {
                    if handle.active.load(Ordering::Relaxed) {
                        log_bus::log("[hotkey] hotkey detected");
                        on_toggle();
                    }
                }
                Ok(Some(_)) => {}
                Ok(None) => std::thread::sleep(IDLE_POLL),
                Err(error) => {
                    log_bus::log(format!("[hotkey:error] X connection lost: {error}"));
                    return Ok(());
                }
            }
        }

        for lock in LOCK_COMBINATIONS {
            let _ = connection.ungrab_key(keycode, root, ModMask::CONTROL | ModMask::from(lock));
        }
        let _ = connection.flush();
        log_bus::log(format!("[hotkey] {HOTKEY_NAME} released"));

        Ok(())
    }

    /// Resolves a keysym to the keycode the current layout assigns it.
    fn keycode_for(connection: &impl Connection, keysym: u32) -> Result<Keycode, String> {
        let setup = connection.setup();
        let first = setup.min_keycode;
        let count = setup.max_keycode - first + 1;

        let mapping = connection
            .get_keyboard_mapping(first, count)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;

        let per_keycode = usize::from(mapping.keysyms_per_keycode).max(1);
        mapping
            .keysyms
            .chunks(per_keycode)
            .position(|symbols| symbols.contains(&keysym))
            .map(|index| first + index as Keycode)
            .ok_or_else(|| "the current keyboard layout has no space key".to_string())
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod imp {
    use super::{RegistrationCallback, ToggleCallback, HOTKEY_NAME};
    use crate::core::log_bus;

    /// Outside Windows and X11 there is no global shortcut backend, so the
    /// service reports itself unavailable.
    #[derive(Clone)]
    pub struct Handle;

    impl Handle {
        pub fn set_active(&self, _active: bool) {}
        pub fn stop(&self) {}
    }

    pub fn start(_on_toggle: ToggleCallback, on_registration: Option<RegistrationCallback>) -> Handle {
        log_bus::log(format!("[hotkey] the global {HOTKEY_NAME} shortcut is only available on Windows"));
        if let Some(callback) = on_registration {
            callback(Err(format!("{HOTKEY_NAME}: unsupported platform")));
        }
        Handle
    }
}
