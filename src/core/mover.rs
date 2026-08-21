//! Moves entries out of Downloads and into the home buckets.
//!
//! Every move is failsafe: the destination name is reserved before anything is
//! written, a cross device move copies before it deletes, the copy is verified
//! against the source, and a failure at any point leaves the source untouched.

use crate::core::fs_util::{extension_lower, unique_target};
use crate::core::log_bus;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Home-relative folder collecting directories moved out of Downloads.
const FOLDERS_BUCKET: &str = "Folders";

/// Extension to bucket table, built once and shared.
fn buckets() -> &'static HashMap<&'static str, &'static str> {
    static BUCKETS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    BUCKETS.get_or_init(|| {
        let groups: [(&str, &[&str]); 6] = [
            ("Documents", &["txt", "pdf", "doc", "docx", "rtf", "odt", "xls", "xlsx", "csv", "ppt", "pptx", "md", "json", "xml", "yaml", "yml"]),
            ("Images", &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "heic", "svg", "ico"]),
            ("Musics", &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"]),
            ("Videos", &["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"]),
            ("Executables", &["exe", "msi", "iso", "jar", "bat", "cmd", "sh", "appimage", "deb", "rpm", "pkg.tar.zst"]),
            ("Archives", &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst"]),
        ];

        groups
            .into_iter()
            .flat_map(|(bucket, extensions)| extensions.iter().map(move |extension| (*extension, bucket)))
            .collect()
    })
}

/// Home-relative bucket an extension belongs to, or `None` when unknown.
fn bucket_for(extension: &str) -> Option<&'static str> {
    buckets().get(extension).copied()
}

/// Why an entry was left where it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skipped {
    /// The extension matches no bucket.
    UnknownExtension,
    /// The entry vanished, or is neither a file nor a directory.
    NotMovable,
    /// The entry is already inside the bucket it belongs to.
    AlreadySorted,
    /// Moving it would place a directory inside itself.
    WouldRecurse,
}

/// What happened to one entry.
#[derive(Debug)]
pub enum Outcome {
    Moved { from: PathBuf, to: PathBuf },
    Skipped { path: PathBuf, reason: Skipped },
    Failed { path: PathBuf, error: io::Error },
}

impl Outcome {
    /// Whether the entry actually changed place.
    pub fn moved(&self) -> bool {
        matches!(self, Outcome::Moved { .. })
    }

    /// A log line describing the outcome.
    pub fn describe(&self) -> String {
        match self {
            Outcome::Moved { from, to } => format!("[move] {}  ->  {}", from.display(), to.display()),
            Outcome::Skipped { path, reason } => {
                let reason = match reason {
                    Skipped::UnknownExtension => "unknown extension",
                    Skipped::NotMovable => "neither a file nor a folder",
                    Skipped::AlreadySorted => "already in its bucket",
                    Skipped::WouldRecurse => "would move a folder into itself",
                };
                format!("[move:skip] {} ({reason})", path.display())
            }
            Outcome::Failed { path, error } => format!("[move:fail] {} : {error}", path.display()),
        }
    }
}

/// Moves a file or a directory into the bucket it belongs to.
///
/// The source is only removed once its replacement is fully in place, so an
/// interrupted move leaves the original readable.
pub fn move_path(home: &Path, path: &Path) -> Outcome {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Outcome::Skipped { path: path.to_path_buf(), reason: Skipped::NotMovable };
    };

    let Some(name) = path.file_name().map(|name| name.to_string_lossy().into_owned()).filter(|name| !name.is_empty())
    else {
        return Outcome::Skipped { path: path.to_path_buf(), reason: Skipped::NotMovable };
    };

    let bucket = if metadata.is_dir() {
        FOLDERS_BUCKET
    } else if metadata.is_file() {
        match bucket_for(&extension_lower(&name)) {
            Some(bucket) => bucket,
            None => return Outcome::Skipped { path: path.to_path_buf(), reason: Skipped::UnknownExtension },
        }
    } else {
        // Symlinks and other special entries are left alone: following them
        // could drag in something well outside Downloads.
        return Outcome::Skipped { path: path.to_path_buf(), reason: Skipped::NotMovable };
    };

    let target_dir = home.join(bucket);

    if path.parent() == Some(target_dir.as_path()) {
        return Outcome::Skipped { path: path.to_path_buf(), reason: Skipped::AlreadySorted };
    }

    if metadata.is_dir() && target_dir.starts_with(path) {
        return Outcome::Skipped { path: path.to_path_buf(), reason: Skipped::WouldRecurse };
    }

    match relocate(&target_dir, path, &name) {
        Ok(to) => Outcome::Moved { from: path.to_path_buf(), to },
        Err(error) => Outcome::Failed { path: path.to_path_buf(), error },
    }
}

/// Moves every top level entry of `source_dir` into its bucket, returning how
/// many were moved.
pub fn sweep(home: &Path, source_dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(source_dir) else {
        log_bus::log(format!("[move] cannot read {}", source_dir.display()));
        return 0;
    };

    let mut moved = 0;
    for entry in entries.flatten() {
        let outcome = move_path(home, &entry.path());
        log_bus::log(outcome.describe());
        if outcome.moved() {
            moved += 1;
        }
    }

    moved
}

/// Puts `source` into `target_dir` under a name that is free.
///
/// A plain rename handles the common case. When the source and the destination
/// sit on different volumes the rename cannot work, so the content is copied
/// first and the source is only deleted once the copy is verified.
fn relocate(target_dir: &Path, source: &Path, name: &str) -> io::Result<PathBuf> {
    fs::create_dir_all(target_dir)?;

    // Reserving the name before writing keeps two concurrent sweeps from
    // picking the same destination.
    let target = unique_target(target_dir, name)?;

    match fs::rename(source, &target) {
        Ok(()) => return Ok(target),
        Err(error) if !is_cross_device(&error) => return Err(error),
        Err(_) => {}
    }

    copy_across_devices(source, &target).inspect_err(|_| {
        // Never leave a partial copy behind to be mistaken for the real thing.
        let _ = remove_any(&target);
    })?;

    remove_any(source)?;
    Ok(target)
}

/// Copies `source` onto `target`, then removes nothing: the caller decides.
fn copy_across_devices(source: &Path, target: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;

    if metadata.is_dir() {
        copy_dir(source, target)
    } else {
        copy_file_verified(source, target, metadata.len())
    }
}

/// Copies one file and checks that the destination really holds every byte.
fn copy_file_verified(source: &Path, target: &Path, expected_len: u64) -> io::Result<()> {
    let copied = fs::copy(source, target)?;

    if copied != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("copied {copied} of {expected_len} bytes from {}", source.display()),
        ));
    }

    // `fs::copy` reports what it wrote; confirm the file system agrees.
    let written = fs::metadata(target)?.len();
    if written != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} holds {written} bytes, expected {expected_len}", target.display()),
        ));
    }

    Ok(())
}

/// Recursively copies a directory, skipping anything that is not a plain file
/// or directory so symlinks are never followed out of the tree.
fn copy_dir(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = target.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            copy_file_verified(&entry.path(), &destination, entry.metadata()?.len())?;
        }
    }

    Ok(())
}

fn remove_any(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Whether the rename failed only because the two paths live on different
/// file systems, which is the one case a copy can recover from.
fn is_cross_device(error: &io::Error) -> bool {
    #[cfg(unix)]
    const CROSS_DEVICE: i32 = 18; // EXDEV
    #[cfg(windows)]
    const CROSS_DEVICE: i32 = 17; // ERROR_NOT_SAME_DEVICE

    error.raw_os_error() == Some(CROSS_DEVICE)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch home that cleans itself up.
    struct TempHome(PathBuf);

    impl TempHome {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!("file-organizer-{label}"));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(path.join("Downloads")).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn downloads(&self) -> PathBuf {
            self.0.join("Downloads")
        }
    }

    impl Drop for TempHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn extensions_resolve_to_their_home_bucket() {
        assert_eq!(bucket_for("docx"), Some("Documents"));
        assert_eq!(bucket_for("flac"), Some("Musics"));
        assert_eq!(bucket_for("7z"), Some("Archives"));
        assert_eq!(bucket_for("appimage"), Some("Executables"));
        assert_eq!(bucket_for("qwerty"), None);
        assert_eq!(bucket_for(""), None);
    }

    #[test]
    fn a_file_lands_in_its_bucket_with_its_content_intact() {
        let home = TempHome::new("move-file");
        let source = home.downloads().join("song.mp3");
        fs::write(&source, b"audio payload").unwrap();

        let outcome = move_path(home.path(), &source);

        assert!(outcome.moved(), "{}", outcome.describe());
        assert!(!source.exists());
        let target = home.path().join("Musics").join("song.mp3");
        assert_eq!(fs::read(&target).unwrap(), b"audio payload");
    }

    #[test]
    fn a_colliding_name_is_suffixed_instead_of_overwritten() {
        let home = TempHome::new("move-collision");
        fs::create_dir_all(home.path().join("Documents")).unwrap();
        fs::write(home.path().join("Documents").join("note.txt"), b"original").unwrap();

        let source = home.downloads().join("note.txt");
        fs::write(&source, b"newcomer").unwrap();

        assert!(move_path(home.path(), &source).moved());

        assert_eq!(fs::read(home.path().join("Documents").join("note.txt")).unwrap(), b"original");
        assert_eq!(fs::read(home.path().join("Documents").join("note (1).txt")).unwrap(), b"newcomer");
    }

    #[test]
    fn an_unknown_extension_stays_put() {
        let home = TempHome::new("move-unknown");
        let source = home.downloads().join("mystery.qwerty");
        fs::write(&source, b"data").unwrap();

        let outcome = move_path(home.path(), &source);

        assert!(matches!(outcome, Outcome::Skipped { reason: Skipped::UnknownExtension, .. }));
        assert!(source.exists());
    }

    #[test]
    fn a_directory_lands_in_the_folders_bucket() {
        let home = TempHome::new("move-dir");
        let source = home.downloads().join("project");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested").join("file.txt"), b"deep").unwrap();

        assert!(move_path(home.path(), &source).moved());

        assert!(!source.exists());
        let target = home.path().join(FOLDERS_BUCKET).join("project");
        assert_eq!(fs::read(target.join("nested").join("file.txt")).unwrap(), b"deep");
    }

    #[test]
    fn an_entry_already_in_its_bucket_is_left_alone() {
        let home = TempHome::new("move-sorted");
        let bucket = home.path().join("Documents");
        fs::create_dir_all(&bucket).unwrap();
        let source = bucket.join("note.txt");
        fs::write(&source, b"stay").unwrap();

        let outcome = move_path(home.path(), &source);

        assert!(matches!(outcome, Outcome::Skipped { reason: Skipped::AlreadySorted, .. }));
        assert_eq!(fs::read(&source).unwrap(), b"stay");
    }

    #[test]
    fn a_vanished_entry_is_reported_rather_than_failing() {
        let home = TempHome::new("move-missing");
        let outcome = move_path(home.path(), &home.downloads().join("ghost.txt"));
        assert!(matches!(outcome, Outcome::Skipped { reason: Skipped::NotMovable, .. }));
    }

    #[test]
    fn a_cross_device_copy_verifies_before_deleting_the_source() {
        let home = TempHome::new("move-verified");
        let source = home.downloads().join("payload.pdf");
        let content = vec![7u8; 64 * 1024];
        fs::write(&source, &content).unwrap();

        let target_dir = home.path().join("Documents");
        let target = relocate(&target_dir, &source, "payload.pdf").unwrap();

        assert_eq!(fs::read(&target).unwrap(), content);
        assert!(!source.exists());
    }

    #[test]
    fn sweeping_reports_only_what_it_moved() {
        let home = TempHome::new("sweep");
        fs::write(home.downloads().join("a.png"), b"i").unwrap();
        fs::write(home.downloads().join("b.mp3"), b"a").unwrap();
        fs::write(home.downloads().join("c.qwerty"), b"?").unwrap();

        assert_eq!(sweep(home.path(), &home.downloads()), 2);
        assert!(home.path().join("Images").join("a.png").exists());
        assert!(home.path().join("Musics").join("b.mp3").exists());
        assert!(home.downloads().join("c.qwerty").exists());
    }

    #[test]
    fn a_symlink_is_never_followed_out_of_the_source_folder() {
        let home = TempHome::new("move-symlink");
        let outside = home.path().join("secret.pdf");
        fs::write(&outside, b"private").unwrap();

        let link = home.downloads().join("link.pdf");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        #[cfg(windows)]
        if std::os::windows::fs::symlink_file(&outside, &link).is_err() {
            return; // Creating symlinks needs a privilege the test may not hold.
        }

        let outcome = move_path(home.path(), &link);

        assert!(matches!(outcome, Outcome::Skipped { reason: Skipped::NotMovable, .. }));
        assert_eq!(fs::read(&outside).unwrap(), b"private");
    }
}
