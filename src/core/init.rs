use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Folders the application indexes and pins, relative to the user's home.
pub const MANAGED_FOLDERS: [&str; 9] = [
    "Desktop",
    "Downloads",
    "Documents",
    "Pictures",
    "Videos",
    "Music",
    "Folders",
    "Executables",
    "Archives",
];

/// Creates every managed folder under `home` and returns their paths.
pub fn ensure_base_and_folders(home: &Path) -> io::Result<Vec<PathBuf>> {
    let mut ensured = Vec::with_capacity(MANAGED_FOLDERS.len());

    for folder in MANAGED_FOLDERS {
        let path = home.join(folder);
        fs::create_dir_all(&path)?;
        ensured.push(path);
    }

    Ok(ensured)
}
