//! Starting the application with the user's session.
//!
//! Both backends target the current user only, so enabling autostart never
//! needs elevated rights and never affects anyone else on the machine.

use crate::core::log_bus;
use std::io;

/// Name the entry is registered under, on both platforms.
pub const ENTRY_NAME: &str = "file-organizer";

/// Whether the application is set to start with the session.
pub fn is_enabled() -> bool {
    imp::is_enabled()
}

/// Registers the application to start with the session.
pub fn enable() -> io::Result<()> {
    imp::set_enabled(true)
}

/// Unregisters the application from the session startup.
pub fn disable() -> io::Result<()> {
    imp::set_enabled(false)
}

/// Flips the setting and reports the state it ended up in.
pub fn toggle() -> bool {
    let wanted = !is_enabled();

    match imp::set_enabled(wanted) {
        Ok(()) => {
            log_bus::log(format!("[autostart] {}", if wanted { "enabled" } else { "disabled" }));
            wanted
        }
        Err(error) => {
            log_bus::log(format!("[autostart:error] {error}"));
            !wanted
        }
    }
}

/// Path of the executable to launch, resolved once at call time so a moved or
/// reinstalled binary registers its new location.
fn executable() -> io::Result<std::path::PathBuf> {
    std::env::current_exe()
}

#[cfg(windows)]
mod imp {
    use super::{executable, ENTRY_NAME};
    use std::io;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
    };

    /// The per user key Windows reads at logon.
    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    pub fn is_enabled() -> bool {
        let Ok(key) = open_run_key(KEY_READ) else { return false };

        let mut size = 0u32;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                wide(ENTRY_NAME).as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            )
        };

        status == ERROR_SUCCESS
    }

    pub fn set_enabled(enabled: bool) -> io::Result<()> {
        let key = open_run_key(KEY_WRITE)?;
        let name = wide(ENTRY_NAME);

        let status = if enabled {
            // Quoting keeps a path with spaces from being split into arguments.
            let command = wide(&format!("\"{}\"", executable()?.display()));
            let bytes = std::mem::size_of_val(command.as_slice()) as u32;

            unsafe { RegSetValueExW(key.0, name.as_ptr(), 0, REG_SZ, command.as_ptr().cast(), bytes) }
        } else {
            match unsafe { RegDeleteValueW(key.0, name.as_ptr()) } {
                // Already absent is the state we wanted.
                ERROR_FILE_NOT_FOUND => ERROR_SUCCESS,
                other => other,
            }
        };

        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(status as i32))
        }
    }

    /// Owns an open registry key and closes it on the way out.
    struct OwnedKey(HKEY);

    impl Drop for OwnedKey {
        fn drop(&mut self) {
            unsafe { RegCloseKey(self.0) };
        }
    }

    fn open_run_key(access: u32) -> io::Result<OwnedKey> {
        let mut key: HKEY = std::ptr::null_mut();
        let status =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), 0, access, &mut key) };

        if status == ERROR_SUCCESS {
            Ok(OwnedKey(key))
        } else {
            Err(io::Error::from_raw_os_error(status as i32))
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::{executable, ENTRY_NAME};
    use std::io;
    use std::path::PathBuf;

    pub fn is_enabled() -> bool {
        entry_path().map(|path| path.is_file()).unwrap_or(false)
    }

    pub fn set_enabled(enabled: bool) -> io::Result<()> {
        let path = entry_path()
            .ok_or_else(|| io::Error::other("no XDG config directory to write the autostart entry to"))?;

        if !enabled {
            return match std::fs::remove_file(&path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                result => result,
            };
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=File Organizer\n\
             Comment=Keep the home folder tidy and search it instantly\n\
             Exec={}\n\
             Icon={ENTRY_NAME}\n\
             Terminal=false\n\
             Categories=Utility;FileTools;\n\
             X-GNOME-Autostart-enabled=true\n",
            executable()?.display()
        );

        std::fs::write(&path, entry)
    }

    /// The XDG autostart directory is read by every major desktop.
    fn entry_path() -> Option<PathBuf> {
        dirs::config_dir().map(|config| config.join("autostart").join(format!("{ENTRY_NAME}.desktop")))
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod imp {
    use std::io;

    /// Session autostart is wired per desktop; only Windows and the XDG
    /// desktops are supported here.
    pub fn is_enabled() -> bool {
        false
    }

    pub fn set_enabled(_enabled: bool) -> io::Result<()> {
        Err(io::Error::other("autostart is only supported on Windows and Linux"))
    }
}
