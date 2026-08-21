//! Hands paths over to the desktop environment: open an entry with its default
//! application, or show it selected in the system file manager.

use crate::core::log_bus;
use std::path::Path;

/// Opens `path` with the application the desktop associates with it.
pub fn open_path(path: &Path) {
    match open::that_detached(path) {
        Ok(()) => log_bus::log(format!("[open] {}", path.display())),
        Err(error) => log_bus::log(format!("[open:error] {} : {error}", path.display())),
    }
}

/// Reveals `path` in the system file manager, selected when possible.
pub fn reveal_path(path: &Path) {
    match reveal(path) {
        Ok(()) => log_bus::log(format!("[reveal] {}", path.display())),
        Err(error) => log_bus::log(format!("[reveal:error] {} : {error}", path.display())),
    }
}

#[cfg(windows)]
fn reveal(path: &Path) -> std::io::Result<()> {
    use std::process::Command;

    Command::new("explorer.exe").arg("/select,").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "macos")]
fn reveal(path: &Path) -> std::io::Result<()> {
    use std::process::Command;

    Command::new("open").arg("-R").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn reveal(path: &Path) -> std::io::Result<()> {
    use std::process::Command;

    // The freedesktop file manager interface selects the entry; when no file
    // manager answers on the bus, opening the parent folder is close enough.
    let uri = format!("file://{}", path.display());
    let shown = Command::new("dbus-send")
        .args([
            "--session",
            "--dest=org.freedesktop.FileManager1",
            "--type=method_call",
            "/org/freedesktop/FileManager1",
            "org.freedesktop.FileManager1.ShowItems",
            &format!("array:string:{uri}"),
            "string:",
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if shown {
        return Ok(());
    }

    let parent = path.parent().unwrap_or(path);
    open::that_detached(parent)
}
