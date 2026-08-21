//! The folders the application owns, and which file types land in each.
//!
//! This is the single source of truth: the mover picks destinations from it,
//! the initializer creates them, and the index watches exactly the same set.
//! Keeping one list is what stops a file from being filed somewhere the search
//! never looks.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Where the sweep takes entries from.
pub const DOWNLOADS: &str = "Downloads";
/// Indexed alongside the buckets, but never a destination.
pub const DESKTOP: &str = "Desktop";

/// Destination for directories, which have no extension to classify.
pub const FOLDERS: &str = "Folders";

pub const DOCUMENTS: &str = "Documents";
/// The XDG user directory, and the Windows known folder, for images.
pub const PICTURES: &str = "Pictures";
/// The XDG user directory, and the Windows known folder, for audio.
pub const MUSIC: &str = "Music";
pub const VIDEOS: &str = "Videos";
pub const ARCHIVES: &str = "Archives";
pub const EXECUTABLES: &str = "Executables";

/// Every folder created under the home directory, indexed and watched.
pub const MANAGED: [&str; 9] =
    [DESKTOP, DOWNLOADS, DOCUMENTS, PICTURES, MUSIC, VIDEOS, ARCHIVES, EXECUTABLES, FOLDERS];

/// Extension to bucket table, built once and shared.
fn by_extension() -> &'static HashMap<&'static str, &'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let groups: [(&str, &[&str]); 6] = [
            (DOCUMENTS, &["txt", "pdf", "doc", "docx", "rtf", "odt", "xls", "xlsx", "csv", "ppt", "pptx", "md", "json", "xml", "yaml", "yml"]),
            (PICTURES, &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "heic", "svg", "ico"]),
            (MUSIC, &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"]),
            (VIDEOS, &["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"]),
            (EXECUTABLES, &["exe", "msi", "iso", "jar", "bat", "cmd", "sh", "appimage", "deb", "rpm"]),
            (ARCHIVES, &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst"]),
        ];

        groups
            .into_iter()
            .flat_map(|(bucket, extensions)| extensions.iter().map(move |extension| (*extension, bucket)))
            .collect()
    })
}

/// The bucket an already lowercased extension belongs to, if any.
pub fn for_extension(extension: &str) -> Option<&'static str> {
    by_extension().get(extension).copied()
}

/// The managed folders, resolved under `home`.
pub fn managed_paths(home: &Path) -> Vec<PathBuf> {
    MANAGED.iter().map(|folder| home.join(folder)).collect()
}

/// Creates every managed folder under `home` and returns their paths.
pub fn ensure_all(home: &Path) -> io::Result<Vec<PathBuf>> {
    let paths = managed_paths(home);

    for path in &paths {
        fs::create_dir_all(path)?;
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The invariant the two lists used to break: a file could be filed into
    /// a folder the index never looked at.
    #[test]
    fn every_destination_is_a_managed_folder() {
        let mut destinations: Vec<&str> = by_extension().values().copied().collect();
        destinations.push(FOLDERS);
        destinations.sort_unstable();
        destinations.dedup();

        for destination in destinations {
            assert!(MANAGED.contains(&destination), "{destination} is a destination but is not indexed");
        }
    }

    #[test]
    fn managed_folders_are_distinct() {
        let mut sorted = MANAGED;
        sorted.sort_unstable();
        let unique = sorted.len();
        let mut deduped = sorted.to_vec();
        deduped.dedup();
        assert_eq!(deduped.len(), unique);
    }

    #[test]
    fn extensions_resolve_to_their_bucket() {
        assert_eq!(for_extension("docx"), Some(DOCUMENTS));
        assert_eq!(for_extension("png"), Some(PICTURES));
        assert_eq!(for_extension("flac"), Some(MUSIC));
        assert_eq!(for_extension("appimage"), Some(EXECUTABLES));
        assert_eq!(for_extension("qwerty"), None);
        assert_eq!(for_extension(""), None);
    }
}
