//! Searchable index of the managed folders.
//!
//! It replaces both the JavaFX result list and the Java `SearchIndex`: every
//! entry carries its own path, so activating one opens exactly that entry
//! instead of resolving a name back to a file.

use crate::core::fs_util::extension_lower;
use crate::core::folders;
use crate::core::model::FsEntry;
use crate::core::scanner::FileScanner;
use crate::core::{home_dir, log_bus};
use crate::platform::shell;
use crate::ui::icons::IconName;
use gpui::*;
use gpui_component::{Icon, StyledExt};
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};

/// Extensions shown with a dedicated icon.
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "heic", "svg", "ico"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"];
const MUSIC_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso"];

#[derive(Clone, Debug)]
struct FileItem {
    name: String,
    /// Managed folder the entry was found in, shown to tell duplicates apart.
    location: String,
    path: PathBuf,
    is_directory: bool,
}

impl FileItem {
    fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        let location = path
            .parent()
            .and_then(Path::file_name)
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();

        Some(Self { is_directory: path.is_dir(), name, location, path })
    }

    fn from_entry(entry: FsEntry) -> Self {
        let location = entry
            .path
            .parent()
            .and_then(Path::file_name)
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();

        Self { name: entry.name, location, path: entry.path, is_directory: entry.is_directory }
    }

    fn icon(&self) -> IconName {
        if self.is_directory {
            return IconName::Folder;
        }

        let extension = extension_lower(&self.name);
        let extension = extension.as_str();

        if IMAGE_EXTENSIONS.contains(&extension) {
            IconName::Image
        } else if VIDEO_EXTENSIONS.contains(&extension) {
            IconName::Video
        } else if MUSIC_EXTENSIONS.contains(&extension) {
            IconName::Music
        } else if ARCHIVE_EXTENSIONS.contains(&extension) {
            IconName::Archive
        } else {
            IconName::File
        }
    }
}

pub struct List {
    searched_string: String,
    items: Vec<FileItem>,
}

impl List {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { searched_string: String::new(), items: Self::load_items() }
    }

    /// The managed folders that are indexed and watched.
    pub fn indexed_roots() -> Vec<PathBuf> {
        folders::managed_paths(&home_dir())
    }

    /// Rebuilds the index from disk, keeping the current search query.
    pub fn reload(&mut self, reason: &str, cx: &mut Context<Self>) {
        log_bus::log(format!("[reload] start ({reason})"));
        self.items = Self::load_items();
        log_bus::log(format!("[reload] done, entries={}", self.items.len()));
        cx.notify();
    }

    fn load_items() -> Vec<FileItem> {
        let mut items: Vec<FileItem> =
            FileScanner::new().scan_top_level(&Self::indexed_roots()).into_iter().map(FileItem::from_entry).collect();

        items.sort_by_key(|item| item.name.to_lowercase());
        items
    }

    pub fn set_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.searched_string = query;
        cx.notify();
    }

    /// Applies a file system notification to the index.
    ///
    /// Watched folders also report events about themselves, so only entries
    /// that sit inside a managed folder are taken into account.
    pub fn handle_fs_event(&mut self, event: Event, cx: &mut Context<Self>) {
        match event.kind {
            EventKind::Create(_) => {
                for path in event.paths {
                    self.add_item(path);
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    self.remove_item(path);
                }
            }
            EventKind::Modify(ModifyKind::Name(mode)) => match mode {
                RenameMode::From => {
                    for path in &event.paths {
                        self.remove_item(path);
                    }
                }
                RenameMode::To => {
                    for path in event.paths {
                        self.add_item(path);
                    }
                }
                // A completed rename is reported as [from, to].
                _ if event.paths.len() == 2 => {
                    self.remove_item(&event.paths[0]);
                    self.add_item(event.paths[1].clone());
                }
                _ => {}
            },
            _ => {}
        }

        cx.notify();
    }

    fn add_item(&mut self, path: PathBuf) {
        if !path.exists() || self.items.iter().any(|item| item.path == path) {
            return;
        }

        let Some(item) = FileItem::from_path(path) else { return };
        let position = self
            .items
            .binary_search_by(|existing| existing.name.to_lowercase().cmp(&item.name.to_lowercase()))
            .unwrap_or_else(|position| position);
        self.items.insert(position, item);
    }

    fn remove_item(&mut self, path: &Path) {
        self.items.retain(|item| item.path != path);
    }

    fn matches_search(&self, item: &FileItem) -> bool {
        if self.searched_string.is_empty() {
            return true;
        }
        item.name.to_lowercase().contains(&self.searched_string.to_lowercase())
    }
}

impl Render for List {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().v_flex().w_full().gap_2().items_start().children(
            self.items.iter().filter(|item| self.matches_search(item)).enumerate().map(|(index, item)| {
                let open_target = item.path.clone();
                let reveal_target = item.path.clone();

                div()
                    .id(("list-item", index))
                    .w_full()
                    .p_2()
                    .bg(rgb(0x2d2d2d))
                    .hover(|style| style.bg(rgb(0x3d3d3d)))
                    .rounded_md()
                    .cursor_pointer()
                    .on_click(move |_, _, _| shell::open_path(&open_target))
                    .on_mouse_down(MouseButton::Right, move |_, _, _| shell::reveal_path(&reveal_target))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(Icon::new(item.icon()).size_4().text_color(rgb(0x999999)))
                            .child(div().text_color(rgb(0xffffff)).child(item.name.clone()))
                            .child(div().text_xs().text_color(rgb(0x777777)).child(item.location.clone())),
                    )
            }),
        )
    }
}
