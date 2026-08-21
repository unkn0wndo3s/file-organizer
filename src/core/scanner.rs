use crate::core::fs_util::is_hidden;
use crate::core::model::FsEntry;
use std::fs;
use std::path::Path;

/// Lists the visible direct children of the managed folders.
pub struct FileScanner;

impl FileScanner {
    pub fn new() -> Self {
        Self
    }

    /// Collects the visible direct children of `roots`, files and directories alike.
    pub fn scan_top_level(&self, roots: &[impl AsRef<Path>]) -> Vec<FsEntry> {
        let mut found = Vec::new();
        self.scan_top_level_stream(roots, |entry| found.push(entry));
        found
    }

    /// Visits the visible direct children of `roots`, files and directories alike.
    pub fn scan_top_level_stream(&self, roots: &[impl AsRef<Path>], mut on_entry: impl FnMut(FsEntry)) {
        for root in roots {
            let Ok(entries) = fs::read_dir(root.as_ref()) else { continue };

            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let path = entry.path();

                if is_hidden(&name, &path) {
                    continue;
                }

                // `file_type` is served straight from the directory listing on
                // most platforms, unlike a full metadata lookup.
                let Ok(file_type) = entry.file_type() else { continue };

                on_entry(FsEntry { path, name, is_directory: file_type.is_dir() });
            }
        }
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}
