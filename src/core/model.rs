use std::path::PathBuf;

/// A visible top level entry of a managed folder.
#[derive(Clone, Debug)]
pub struct FsEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
}
