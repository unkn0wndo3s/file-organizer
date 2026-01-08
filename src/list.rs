use gpui::*;
use gpui_component::{Icon, StyledExt};
use crate::icons::*;
use std::fs;
use std::path::{Path, PathBuf};
use notify::{Event, EventKind, event::{ModifyKind, RenameMode}};

#[derive(Clone, Debug)]
struct FileItem {
    name: String,
    item_type: String,
    path: String,
}

pub struct List {
    searched_string: String,
    items: Vec<FileItem>,
}

impl List {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let items = Self::load_files();
        Self { 
            searched_string: String::new(),
            items,
        }
    }

    fn load_files() -> Vec<FileItem> {
        let mut items = vec![];
        let main_folders = vec!["Documents", "Pictures", "Videos", "Music", "Downloads"];
        let base_path = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:/".to_string());

        for folder in main_folders {
            let full_path = format!("{}\\{}", base_path, folder);
            if let Ok(entries) = fs::read_dir(full_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        List::create_item_from_path(path).map(|item| items.push(item));
                    }
                }
            }
        }
        items
    }

    // Helper to standardise item creation
    fn create_item_from_path(path: PathBuf) -> Option<FileItem> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        let item_type = if path.is_dir() { "Folder" } else { "File" };
        let path_str = path.to_string_lossy().into_owned();

        Some(FileItem {
            name,
            item_type: item_type.to_string(),
            path: path_str,
        })
    }

    pub fn set_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.searched_string = query;
        cx.notify(); 
    }

    pub fn handle_fs_event(&mut self, event: Event, cx: &mut Context<Self>) {
        // notify can sometimes send events for the parent folder itself (e.g. "Documents" modified)
        // We only care about files/folders INSIDE the watched folders.
        
        match event.kind {
            EventKind::Create(_) => {
                for path in event.paths {
                    if path.exists() {
                        self.add_file(path);
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    self.remove_file(&path);
                }
            }
            EventKind::Modify(modify_kind) => {
                 match modify_kind {
                     ModifyKind::Name(mode) => {
                         match mode {
                             RenameMode::From => {
                                 for path in &event.paths {
                                     self.remove_file(path);
                                 }
                             }
                             RenameMode::To => {
                                 for path in event.paths {
                                     if path.exists() {
                                         self.add_file(path);
                                     }
                                 }
                             }
                             _ => {
                                 // Simple rename: usually [from, to]
                                 if event.paths.len() == 2 {
                                     let from = &event.paths[0];
                                     let to = &event.paths[1];
                                     self.rename_file(from, to);
                                 }
                             }
                         }
                     }
                     // Trigger update for content modification if needed
                     // ModifyKind::Data(_) => { ... } 
                     _ => {}
                 }
            }
            _ => {}
        }
        cx.notify();
    }

    fn add_file(&mut self, path: PathBuf) {
        let path_str = path.to_string_lossy().into_owned();
        
        // Prevent duplicates
        if self.items.iter().any(|i| i.path == path_str) {
            return;
        }

        if let Some(item) = List::create_item_from_path(path) {
            self.items.push(item);
            // Optional: Sort items here if you want new files to appear in order
        }
    }

    fn remove_file(&mut self, path: &Path) {
        let path_str = path.to_string_lossy();
        self.items.retain(|i| i.path != path_str);
    }

    fn rename_file(&mut self, from: &Path, to: &Path) {
        let from_str = from.to_string_lossy();
        if let Some(item) = self.items.iter_mut().find(|i| i.path == from_str) {
            item.path = to.to_string_lossy().into_owned();
            item.name = to.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            item.item_type = if to.is_dir() { "Folder".to_string() } else { "File".to_string() };
        }
    }
}

impl Render for List {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .w_full()
            .gap_2()
            .items_start()
            .children(
                self.items
                    .iter()
                    .filter(|item| {
                        self.searched_string.is_empty() || 
                        item.name.to_lowercase().contains(&self.searched_string.to_lowercase())
                    })
                    .map(|item| {
                        let icon = if item.item_type == "Folder" {
                            Icon::new(IconName::Folder)
                        } else {
                            Icon::new(IconName::File)
                        };

                        div()
                            .w_full()
                            .p_2()
                            .bg(rgb(0x2d2d2d))
                            .hover(|s| s.bg(rgb(0x3d3d3d)))
                            .rounded_md()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .child(icon.size_4().text_color(rgb(0x999999)))
                                    .child(
                                        div()
                                            .text_color(rgb(0xffffff))
                                            .child(item.name.clone())
                                    )
                            )
                    })
            )
    }
}