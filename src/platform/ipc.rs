//! Single instance control channel, used to toggle a running instance from
//! `file-organizer --toggle` without relying on any OS-level global shortcut.
//!
//! `Ctrl+Space` is grabbed automatically where the platform allows it (see
//! `hotkey.rs`), but Wayland compositors without an XWayland-reachable window,
//! or without a `GlobalShortcuts` portal backend, never deliver the shortcut
//! to the application. Binding `file-organizer --toggle` to a key in the
//! compositor's own settings works everywhere, since it only needs a process
//! to start and a local socket to write to.

use crate::core::log_bus;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Owns the server thread that listens for `--toggle` requests.
pub struct IpcServer {
    running: Arc<AtomicBool>,
}

impl IpcServer {
    /// Starts listening in the background; `on_toggle` runs once per request.
    pub fn start(on_toggle: impl Fn() + Send + Sync + 'static) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        imp::start_server(Arc::clone(&running), Arc::new(on_toggle));
        Self { running }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        imp::wake_server();
    }
}

/// Asks an already running instance to toggle its window. Used by the
/// `--toggle` command line flag.
pub fn send_toggle() -> Result<(), String> {
    imp::send_toggle()
}

fn address_error(context: &str, error: io::Error) -> String {
    format!("{context}: {error}")
}

#[cfg(target_os = "linux")]
mod imp {
    use super::{address_error, log_bus};
    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    const REQUEST: &[u8] = b"toggle";

    fn socket_path() -> PathBuf {
        let base = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("file-organizer.sock")
    }

    pub fn start_server(running: Arc<AtomicBool>, on_toggle: Arc<dyn Fn() + Send + Sync>) {
        let path = socket_path();
        // A leftover socket from a previous crash is not a running server:
        // binding fails with AddrInUse unless it is removed first.
        let _ = std::fs::remove_file(&path);

        let listener = match UnixListener::bind(&path) {
            Ok(listener) => listener,
            Err(error) => {
                log_bus::log(format!("[ipc:error] {}", address_error("unable to bind the control socket", error)));
                return;
            }
        };
        log_bus::log(format!("[ipc] listening on {}", path.display()));

        std::thread::Builder::new()
            .name("ipc-server".to_string())
            .spawn(move || {
                for connection in listener.incoming() {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    let Ok(mut stream) = connection else { continue };

                    let mut buffer = [0u8; REQUEST.len()];
                    if stream.read_exact(&mut buffer).is_ok() && buffer == REQUEST {
                        log_bus::log("[ipc] toggle request received");
                        on_toggle();
                    }
                }
                let _ = std::fs::remove_file(&path);
            })
            .expect("ipc server thread can be spawned");
    }

    /// Unblocks the server's `accept()` loop so it can observe `running` went
    /// false; connecting is enough, the request does not need to be valid.
    pub fn wake_server() {
        let _ = UnixStream::connect(socket_path());
    }

    pub fn send_toggle() -> Result<(), String> {
        let mut stream =
            UnixStream::connect(socket_path()).map_err(|error| address_error("no running instance found", error))?;
        stream.write_all(REQUEST).map_err(|error| address_error("unable to send the toggle request", error))
    }
}

#[cfg(windows)]
mod imp {
    use super::{address_error, log_bus};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, ReadFile, WriteFile, FILE_SHARE_NONE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
        PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
    };

    const REQUEST: &[u8] = b"toggle";
    const PIPE_NAME: &str = r"\\.\pipe\file-organizer";

    fn wide(text: &str) -> Vec<u16> {
        OsString::from(text).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn start_server(running: Arc<AtomicBool>, on_toggle: Arc<dyn Fn() + Send + Sync>) {
        log_bus::log(format!("[ipc] listening on {PIPE_NAME}"));

        std::thread::Builder::new()
            .name("ipc-server".to_string())
            .spawn(move || {
                while running.load(Ordering::Relaxed) {
                    let name = wide(PIPE_NAME);
                    let pipe = unsafe {
                        CreateNamedPipeW(
                            name.as_ptr(),
                            PIPE_ACCESS_DUPLEX,
                            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                            PIPE_UNLIMITED_INSTANCES,
                            64,
                            64,
                            0,
                            std::ptr::null_mut(),
                        )
                    };
                    if pipe == INVALID_HANDLE_VALUE {
                        log_bus::log("[ipc:error] unable to create the control pipe");
                        break;
                    }

                    let connected = unsafe { ConnectNamedPipe(pipe, std::ptr::null_mut()) } != 0;
                    if connected && running.load(Ordering::Relaxed) {
                        let mut buffer = [0u8; REQUEST.len()];
                        let mut read = 0u32;
                        let ok =
                            unsafe { ReadFile(pipe, buffer.as_mut_ptr().cast(), buffer.len() as u32, &mut read, std::ptr::null_mut()) };
                        if ok != 0 && read as usize == REQUEST.len() && buffer == REQUEST {
                            log_bus::log("[ipc] toggle request received");
                            on_toggle();
                        }
                    }

                    unsafe {
                        DisconnectNamedPipe(pipe);
                        CloseHandle(pipe);
                    }
                }
            })
            .expect("ipc server thread can be spawned");
    }

    /// Unblocks the server's `ConnectNamedPipe` wait so it can observe
    /// `running` went false; connecting is enough, no data needs to be sent.
    pub fn wake_server() {
        let name = wide(PIPE_NAME);
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_WRITE,
                FILE_SHARE_NONE,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(handle) };
        }
    }

    pub fn send_toggle() -> Result<(), String> {
        let name = wide(PIPE_NAME);
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_NONE,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(address_error("no running instance found", std::io::Error::last_os_error()));
        }

        let mut written = 0u32;
        let ok = unsafe { WriteFile(handle, REQUEST.as_ptr().cast(), REQUEST.len() as u32, &mut written, std::ptr::null_mut()) };
        unsafe { CloseHandle(handle) };

        if ok == 0 {
            return Err(address_error("unable to send the toggle request", std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod imp {
    use super::log_bus;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    pub fn start_server(_running: Arc<AtomicBool>, _on_toggle: Arc<dyn Fn() + Send + Sync>) {
        log_bus::log("[ipc] the --toggle control socket is only available on Linux and Windows");
    }

    pub fn wake_server() {}

    pub fn send_toggle() -> Result<(), String> {
        Err("--toggle: unsupported platform".to_string())
    }
}
