use anyhow::Result;
use gpui::*;
use gpui_component::*; // Needed for IconNamed and SharedString
use std::fs;
use std::path::PathBuf;

/// Asset source that looks in the project's /assets folder
pub struct LocalAssets {
    pub base: PathBuf,
}

impl LocalAssets {
    /// Create a constructor to handle the path logic
    pub fn new() -> Self {
        Self {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        }
    }
}

// Fixed: Removed 'let assets = LocalAssets::new();' from here. 
// This should be called inside your main() function instead.

impl AssetSource for LocalAssets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let Ok(entries) = fs::read_dir(self.base.join(path)) else {
            return Ok(vec![]);
        };

        let paths = entries
            .filter_map(|entry| {
                entry.ok()?.file_name().into_string().ok().map(SharedString::from)
            })
            .collect();
        Ok(paths)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IconName {
    Close,
    Minimize,
    File,
    Folder,
    Image,
    Video,
    Music,
    Archive,
    Search,
    Settings,
}

impl IconNamed for IconName {
    fn path(self) -> SharedString {
        match self {
            IconName::Close => "icons/close.svg".into(),
            IconName::Minimize => "icons/minus.svg".into(),
            IconName::File => "icons/file.svg".into(),
            IconName::Folder => "icons/folder.svg".into(),
            IconName::Image => "icons/file-image.svg".into(),
            IconName::Video => "icons/file-video.svg".into(),
            IconName::Music => "icons/file-music.svg".into(),
            IconName::Archive => "icons/file-archive.svg".into(),
            IconName::Search => "icons/search.svg".into(),
            IconName::Settings => "icons/settings.svg".into(),
        }
    }
}