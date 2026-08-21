#![allow(dead_code)]

use std::io;
use std::path::{Path, PathBuf};

/// Returns the lowercase extension of a file name, without its leading dot.
///
/// Dotfiles such as `.gitignore` and names ending with a dot have no extension.
pub fn extension_lower(name: &str) -> String {
    match name.rfind('.') {
        Some(index) if index > 0 && index + 1 < name.len() => name[index + 1..].to_lowercase(),
        _ => String::new(),
    }
}

/// Whether the entry is hidden according to the platform's convention.
pub fn is_hidden(path: &Path) -> bool {
    if file_name_of(path).starts_with('.') {
        return true;
    }
    has_windows_attributes(path, WindowsAttributes::Hidden)
}

/// Whether the entry is hidden or, on Windows, flagged as a system entry.
pub fn is_hidden_or_system(path: &Path) -> bool {
    if file_name_of(path).starts_with('.') {
        return true;
    }
    has_windows_attributes(path, WindowsAttributes::HiddenOrSystem)
}

/// Builds a destination path inside `dir` that does not collide with an
/// existing entry, appending ` (1)`, ` (2)`, ... before the extension.
pub fn unique_target(dir: &Path, file_name: &str) -> io::Result<PathBuf> {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return Ok(candidate);
    }

    let (base, dot_extension) = match file_name.rfind('.') {
        Some(index) if index > 0 => (&file_name[..index], &file_name[index..]),
        _ => (file_name, ""),
    };

    for counter in 1..=u32::MAX {
        let alternative = dir.join(format!("{base} ({counter}){dot_extension}"));
        if !alternative.exists() {
            return Ok(alternative);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("no free name left for {file_name} in {}", dir.display()),
    ))
}

fn file_name_of(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

enum WindowsAttributes {
    Hidden,
    HiddenOrSystem,
}

#[cfg(windows)]
fn has_windows_attributes(path: &Path, wanted: WindowsAttributes) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;

    let mask = match wanted {
        WindowsAttributes::Hidden => FILE_ATTRIBUTE_HIDDEN,
        WindowsAttributes::HiddenOrSystem => FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM,
    };

    std::fs::symlink_metadata(path)
        .map(|metadata| metadata.file_attributes() & mask != 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn has_windows_attributes(_path: &Path, _wanted: WindowsAttributes) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_is_lowercased_without_the_dot() {
        assert_eq!(extension_lower("Report.PDF"), "pdf");
        assert_eq!(extension_lower("archive.tar.gz"), "gz");
    }

    #[test]
    fn names_without_a_usable_extension_yield_nothing() {
        assert_eq!(extension_lower("README"), "");
        assert_eq!(extension_lower(".gitignore"), "");
        assert_eq!(extension_lower("trailing."), "");
    }

    #[test]
    fn unique_target_suffixes_around_the_extension() {
        let dir = std::env::temp_dir().join("file-organizer-unique-target");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert_eq!(unique_target(&dir, "note.txt").unwrap(), dir.join("note.txt"));

        std::fs::write(dir.join("note.txt"), b"").unwrap();
        assert_eq!(unique_target(&dir, "note.txt").unwrap(), dir.join("note (1).txt"));

        std::fs::write(dir.join("note (1).txt"), b"").unwrap();
        assert_eq!(unique_target(&dir, "note.txt").unwrap(), dir.join("note (2).txt"));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dotfiles_are_hidden_on_every_platform() {
        assert!(is_hidden(Path::new("/home/user/.bashrc")));
        assert!(!is_hidden(Path::new("/home/user/notes.txt")));
    }
}
