pub mod fs_util;
pub mod init;
pub mod log_bus;
pub mod model;
pub mod mover;
pub mod planner;
pub mod rules;
pub mod scanner;
pub mod service;

use std::path::PathBuf;

/// The user's home directory, falling back to the current directory when the
/// platform cannot report one.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}
