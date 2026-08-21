// The recursive walk only feeds the planning API, which the entry point does
// not use yet; the top level walk backs the searchable index.
#![allow(dead_code)]

use crate::core::fs_util::{extension_lower, is_hidden, is_hidden_or_system};
use crate::core::model::{FileRecord, FsEntry};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Walks the file system and turns entries into records the planner understands.
pub struct FileScanner;

impl FileScanner {
    pub fn new() -> Self {
        Self
    }

    /// Recursively collects every visible regular file under `roots`.
    pub fn scan(&self, roots: &[impl AsRef<Path>]) -> Vec<FileRecord> {
        let mut found = Vec::new();
        self.scan_stream(roots, |record| found.push(record));
        found
    }

    /// Recursively visits every visible regular file under `roots`.
    pub fn scan_stream(&self, roots: &[impl AsRef<Path>], mut on_file: impl FnMut(FileRecord)) {
        for root in roots {
            let root = root.as_ref();
            if !root.is_dir() {
                continue;
            }
            walk(root, &mut on_file);
        }
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
            let root = root.as_ref();
            if !root.is_dir() {
                continue;
            }

            let entries = match fs::read_dir(root) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if is_hidden(&path) {
                    continue;
                }

                let Ok(metadata) = entry.metadata() else { continue };
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_directory = metadata.is_dir();

                on_entry(FsEntry {
                    extension_lower: if is_directory { String::new() } else { extension_lower(&name) },
                    size_bytes: if is_directory { 0 } else { metadata.len() },
                    last_modified: last_modified_of(&metadata),
                    path,
                    name,
                    is_directory,
                });
            }
        }
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

fn walk(dir: &Path, on_file: &mut impl FnMut(FileRecord)) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else { continue };

        if metadata.is_dir() {
            if !is_hidden_or_system(&path) {
                walk(&path, on_file);
            }
            continue;
        }

        if !metadata.is_file() || is_hidden_or_system(&path) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        on_file(FileRecord {
            extension_lower: extension_lower(&name),
            size_bytes: metadata.len(),
            last_modified: last_modified_of(&metadata),
            path,
            name,
        });
    }
}

fn last_modified_of(metadata: &fs::Metadata) -> SystemTime {
    metadata.modified().unwrap_or(UNIX_EPOCH)
}
