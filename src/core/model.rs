use crate::core::rules::Category;
use std::path::PathBuf;
use std::time::SystemTime;

/// A regular file discovered by a recursive scan.
#[derive(Clone, Debug)]
pub struct FileRecord {
    pub path: PathBuf,
    pub name: String,
    pub extension_lower: String,
    pub size_bytes: u64,
    pub last_modified: SystemTime,
}

/// A top level entry, either a file or a directory.
#[derive(Clone, Debug)]
pub struct FsEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub extension_lower: String,
    pub size_bytes: u64,
    pub last_modified: SystemTime,
}

/// Where the planner suggests moving a file, before anything is touched on disk.
#[derive(Clone, Debug)]
pub struct FilePlan {
    pub record: FileRecord,
    pub category: Category,
    pub proposed_destination_dir: PathBuf,
    pub proposed_destination_path: PathBuf,
}
