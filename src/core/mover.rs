use crate::core::fs_util::{extension_lower, unique_target};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Home-relative folder collecting directories moved out of Downloads.
const FOLDERS_BUCKET: &str = "Folders";

struct Buckets {
    documents: HashSet<&'static str>,
    images: HashSet<&'static str>,
    music: HashSet<&'static str>,
    videos: HashSet<&'static str>,
    executables: HashSet<&'static str>,
    archives: HashSet<&'static str>,
}

fn buckets() -> &'static Buckets {
    static BUCKETS: OnceLock<Buckets> = OnceLock::new();
    BUCKETS.get_or_init(|| Buckets {
        documents: HashSet::from(["txt", "pdf", "doc", "docx", "rtf", "odt", "xls", "xlsx", "csv", "ppt", "pptx", "md", "json", "xml", "yaml", "yml"]),
        images: HashSet::from(["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "heic", "svg", "ico"]),
        music: HashSet::from(["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"]),
        videos: HashSet::from(["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"]),
        executables: HashSet::from(["exe", "msi", "iso", "jar", "bat", "cmd", "sh"]),
        archives: HashSet::from(["zip", "rar", "7z", "tar", "gz", "bz2", "xz"]),
    })
}

/// Home-relative bucket an extension belongs to, or `None` when unknown.
fn bucket_for(extension: &str) -> Option<&'static str> {
    if extension.is_empty() {
        return None;
    }

    let buckets = buckets();
    if buckets.documents.contains(extension) {
        Some("Documents")
    } else if buckets.images.contains(extension) {
        Some("Images")
    } else if buckets.music.contains(extension) {
        Some("Musics")
    } else if buckets.videos.contains(extension) {
        Some("Videos")
    } else if buckets.executables.contains(extension) {
        Some("Executables")
    } else if buckets.archives.contains(extension) {
        Some("Archives")
    } else {
        None
    }
}

/// Moves a file or a directory into its bucket and reports what happened.
pub fn move_path(home: &Path, path: &Path) -> String {
    if path.is_dir() {
        move_directory(home, path)
    } else if path.is_file() {
        move_file(home, path)
    } else {
        format!("[move] ignored (neither file nor folder): {}", path.display())
    }
}

/// Moves a single file into the bucket matching its extension.
pub fn move_file(home: &Path, file: &Path) -> String {
    if !file.is_file() {
        return format!("[move:file] ignored (not a file): {}", file.display());
    }

    let Some(name) = file_name_of(file) else {
        return format!("[move:file] ignored (no name): {}", file.display());
    };

    let Some(bucket) = bucket_for(&extension_lower(&name)) else {
        return format!("[move] ignored (unknown ext): {}", file.display());
    };

    match relocate(&home.join(bucket), file, &name) {
        Ok(target) => format!("[move:file] {}  ->  {}", file.display(), target.display()),
        Err(error) => format!("[move:file] fail: {} : {error}", file.display()),
    }
}

/// Moves a directory into the home-level `Folders` bucket.
pub fn move_directory(home: &Path, dir: &Path) -> String {
    if !dir.is_dir() {
        return format!("[move:dir] ignored (not a folder): {}", dir.display());
    }

    let Some(name) = file_name_of(dir) else {
        return format!("[move:dir] ignored (no name): {}", dir.display());
    };

    match relocate(&home.join(FOLDERS_BUCKET), dir, &name) {
        Ok(target) => format!("[move:dir] {}  ->  {}", dir.display(), target.display()),
        Err(error) => format!("[move:dir] fail: {} : {error}", dir.display()),
    }
}

/// Creates the destination directory and renames `source` into a free slot.
///
/// Falls back to a copy followed by a delete when the source and destination
/// live on different volumes, where a rename cannot succeed.
fn relocate(target_dir: &Path, source: &Path, name: &str) -> io::Result<PathBuf> {
    fs::create_dir_all(target_dir)?;
    let target = unique_target(target_dir, name)?;

    match fs::rename(source, &target) {
        Ok(()) => Ok(target),
        Err(_) if source.is_file() => {
            fs::copy(source, &target)?;
            fs::remove_file(source)?;
            Ok(target)
        }
        Err(error) => Err(error),
    }
}

fn file_name_of(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_resolve_to_their_home_bucket() {
        assert_eq!(bucket_for("docx"), Some("Documents"));
        assert_eq!(bucket_for("flac"), Some("Musics"));
        assert_eq!(bucket_for("7z"), Some("Archives"));
        assert_eq!(bucket_for("exe"), Some("Executables"));
    }

    #[test]
    fn unknown_extensions_have_no_bucket() {
        assert_eq!(bucket_for("qwerty"), None);
        assert_eq!(bucket_for(""), None);
    }

    #[test]
    fn moving_a_file_creates_the_bucket_and_reports_the_target() {
        let home = std::env::temp_dir().join("file-organizer-move-file");
        let _ = fs::remove_dir_all(&home);
        let source_dir = home.join("Downloads");
        fs::create_dir_all(&source_dir).unwrap();

        let source = source_dir.join("song.mp3");
        fs::write(&source, b"audio").unwrap();

        let report = move_path(&home, &source);

        assert!(report.starts_with("[move:file]"), "{report}");
        assert!(!source.exists());
        assert!(home.join("Musics").join("song.mp3").exists());

        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn a_file_with_an_unknown_extension_stays_put() {
        let home = std::env::temp_dir().join("file-organizer-move-unknown");
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(&home).unwrap();

        let source = home.join("mystery.qwerty");
        fs::write(&source, b"data").unwrap();

        assert!(move_path(&home, &source).contains("unknown ext"));
        assert!(source.exists());

        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn moving_a_directory_lands_it_in_the_folders_bucket() {
        let home = std::env::temp_dir().join("file-organizer-move-dir");
        let _ = fs::remove_dir_all(&home);
        let source = home.join("Downloads").join("project");
        fs::create_dir_all(&source).unwrap();

        let report = move_path(&home, &source);

        assert!(report.starts_with("[move:dir]"), "{report}");
        assert!(!source.exists());
        assert!(home.join(FOLDERS_BUCKET).join("project").is_dir());

        fs::remove_dir_all(&home).unwrap();
    }
}
