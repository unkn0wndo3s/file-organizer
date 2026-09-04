use anyhow::Result;
use gpui::{AssetSource, SharedString};
use gpui_component::IconNamed;
use std::borrow::Cow;

/// The SVG icons, embedded so a packaged build carries them with it instead of
/// looking for an assets folder next to the executable.
const ASSETS: &[(&str, &[u8])] = &[
    ("icons/file-archive.svg", include_bytes!("../../assets/icons/file-archive.svg")),
    ("icons/file-image.svg", include_bytes!("../../assets/icons/file-image.svg")),
    ("icons/file-music.svg", include_bytes!("../../assets/icons/file-music.svg")),
    ("icons/file-video.svg", include_bytes!("../../assets/icons/file-video.svg")),
    ("icons/file.svg", include_bytes!("../../assets/icons/file.svg")),
    ("icons/folder.svg", include_bytes!("../../assets/icons/folder.svg")),
    ("icons/search.svg", include_bytes!("../../assets/icons/search.svg")),
    ("icons/settings.svg", include_bytes!("../../assets/icons/settings.svg")),
];

/// Serves the embedded assets to gpui.
pub struct LocalAssets;

impl LocalAssets {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalAssets {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetSource for LocalAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(ASSETS
            .iter()
            .find(|(asset_path, _)| *asset_path == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let prefix = path.trim_end_matches('/');

        Ok(ASSETS
            .iter()
            .filter_map(|(asset_path, _)| {
                asset_path
                    .strip_prefix(prefix)
                    .map(|relative| SharedString::from(relative.trim_start_matches('/').to_string()))
            })
            .collect())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IconName {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_name_resolves_to_an_embedded_asset() {
        let icons = [
            IconName::File,
            IconName::Folder,
            IconName::Image,
            IconName::Video,
            IconName::Music,
            IconName::Archive,
            IconName::Search,
            IconName::Settings,
        ];

        let assets = LocalAssets::new();
        for icon in icons {
            let path = icon.path();
            let loaded = assets.load(&path).unwrap();
            assert!(loaded.is_some(), "{path} is not embedded");
        }
    }

    #[test]
    fn listing_a_folder_returns_its_file_names() {
        let listed = LocalAssets::new().list("icons").unwrap();
        assert!(listed.contains(&SharedString::from("folder.svg")));
        assert_eq!(listed.len(), ASSETS.len());
    }
}
