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
use gpui_component::Icon;
use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Height of one row, which `uniform_list` needs to be uniform to work.
const ROW_HEIGHT: Pixels = px(36.0);

/// Extensions shown with a dedicated icon.
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "heic", "svg", "ico"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"];
const MUSIC_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus"];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso"];

/// One indexed entry. Opaque to callers, which only ever move whole batches
/// of them from the scanner to the view.
#[derive(Clone, Debug)]
pub struct FileItem {
    /// Shared, so handing the name to the renderer never copies the text.
    name: SharedString,
    /// Managed folder the entry was found in, shown to tell duplicates apart.
    location: SharedString,
    /// Lowercased once here rather than on every keystroke of every frame.
    name_lower: String,
    path: Arc<Path>,
    icon: IconName,
}

impl FileItem {
    fn new(path: PathBuf, name: String, is_directory: bool) -> Self {
        let location = path
            .parent()
            .and_then(Path::file_name)
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();

        Self {
            name_lower: name.to_lowercase(),
            icon: icon_for(&name, is_directory),
            name: name.into(),
            location: location.into(),
            path: path.into(),
        }
    }

    fn from_entry(entry: FsEntry) -> Self {
        Self::new(entry.path, entry.name, entry.is_directory)
    }

    fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        let is_directory = path.is_dir();
        Some(Self::new(path, name, is_directory))
    }
}

fn icon_for(name: &str, is_directory: bool) -> IconName {
    if is_directory {
        return IconName::Folder;
    }

    let extension = extension_lower(name);
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

pub struct List {
    /// Already lowercased, so matching never allocates.
    query: String,
    items: Vec<FileItem>,
    /// Indices of `items` matching `query`, recomputed only when one of them
    /// changes rather than on every frame.
    matches: Vec<usize>,
}

impl List {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let mut list = Self { query: String::new(), items: Vec::new(), matches: Vec::new() };
        list.replace_items(Self::load_items());
        list
    }

    /// The managed folders that are indexed and watched.
    pub fn indexed_roots() -> Vec<PathBuf> {
        folders::managed_paths(&home_dir())
    }

    /// Reads every managed folder. Safe to call off the foreground thread.
    pub fn load_items() -> Vec<FileItem> {
        let mut items: Vec<FileItem> =
            FileScanner::new().scan_top_level(&Self::indexed_roots()).into_iter().map(FileItem::from_entry).collect();

        items.sort_by(|left, right| left.name_lower.cmp(&right.name_lower));
        items
    }

    /// Swaps in a freshly scanned index, keeping the current search query.
    pub fn apply_items(&mut self, items: Vec<FileItem>, reason: &str, cx: &mut Context<Self>) {
        self.replace_items(items);
        log_bus::log(format!("[reload] done ({reason}), entries={}", self.items.len()));
        cx.notify();
    }

    fn replace_items(&mut self, items: Vec<FileItem>) {
        self.items = items;
        self.rebuild_matches();
    }

    pub fn set_search(&mut self, query: String, cx: &mut Context<Self>) {
        let query = query.to_lowercase();
        if query == self.query {
            return;
        }

        self.query = query;
        self.rebuild_matches();
        cx.notify();
    }

    fn rebuild_matches(&mut self) {
        self.matches.clear();

        if self.query.is_empty() {
            self.matches.extend(0..self.items.len());
            return;
        }

        self.matches.extend(
            self.items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.name_lower.contains(&self.query))
                .map(|(index, _)| index),
        );
    }

    /// Applies a file system notification to the index.
    ///
    /// Watched folders also report events about themselves, so only entries
    /// that sit inside a managed folder are taken into account.
    pub fn handle_fs_event(&mut self, event: Event) -> bool {
        let mut changed = false;

        match event.kind {
            EventKind::Create(_) => {
                for path in event.paths {
                    changed |= self.add_item(path);
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    changed |= self.remove_item(path);
                }
            }
            EventKind::Modify(ModifyKind::Name(mode)) => match mode {
                RenameMode::From => {
                    for path in &event.paths {
                        changed |= self.remove_item(path);
                    }
                }
                RenameMode::To => {
                    for path in event.paths {
                        changed |= self.add_item(path);
                    }
                }
                // A completed rename is reported as [from, to].
                _ if event.paths.len() == 2 => {
                    changed |= self.remove_item(&event.paths[0]);
                    changed |= self.add_item(event.paths[1].clone());
                }
                _ => {}
            },
            _ => {}
        }

        // The match list is rebuilt once for the whole event rather than once
        // per path, so a burst of changes stays linear.
        if changed {
            self.rebuild_matches();
        }
        changed
    }

    fn add_item(&mut self, path: PathBuf) -> bool {
        if !path.exists() || self.items.iter().any(|item| *item.path == *path) {
            return false;
        }

        let Some(item) = FileItem::from_path(path) else { return false };
        let position = self
            .items
            .binary_search_by(|existing| existing.name_lower.cmp(&item.name_lower))
            .unwrap_or_else(|position| position);

        self.items.insert(position, item);
        true
    }

    fn remove_item(&mut self, path: &Path) -> bool {
        let before = self.items.len();
        self.items.retain(|item| *item.path != *path);
        self.items.len() != before
    }
}

impl Render for List {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();

        // Only the visible slice is built each frame, so the index can hold
        // tens of thousands of entries without the frame cost following it.
        uniform_list("file-list", self.matches.len(), move |visible, _window, cx| {
            this.read(cx)
                .matches
                .get(visible.clone())
                .unwrap_or_default()
                .iter()
                .zip(visible)
                .filter_map(|(item_index, row)| this.read(cx).items.get(*item_index).map(|item| (item, row)))
                .map(|(item, row)| row_for(item, row))
                .collect()
        })
        .size_full()
    }
}

fn row_for(item: &FileItem, row: usize) -> Stateful<Div> {
    let open_target = Arc::clone(&item.path);
    let reveal_target = Arc::clone(&item.path);

    div()
        .id(("list-item", row))
        .w_full()
        .h(ROW_HEIGHT)
        .px_2()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .bg(rgb(0x2d2d2d))
        .hover(|style| style.bg(rgb(0x3d3d3d)))
        .rounded_md()
        .cursor_pointer()
        .on_click(move |_, _, _| shell::open_path(&open_target))
        .on_mouse_down(MouseButton::Right, move |_, _, _| shell::reveal_path(&reveal_target))
        .child(Icon::new(item.icon).size_4().text_color(rgb(0x999999)))
        .child(div().text_color(rgb(0xffffff)).child(item.name.clone()))
        .child(div().text_xs().text_color(rgb(0x777777)).child(item.location.clone()))
}
