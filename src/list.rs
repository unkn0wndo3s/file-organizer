use gpui::*;
use gpui_component::{Icon, StyledExt};
use crate::icons::*;
use std::fs;
use std::path::PathBuf;

// Changed fields to String to own the data (avoids memory leaks from Box::leak)
#[derive(Clone)]
struct FileItem {
    name: String,
    item_type: String,
    path: String,
}

pub struct List {
    searched_string: String,
    // Store the items in the struct so we don't read from disk every frame
    items: Vec<FileItem>, 
}

impl List {
    pub fn new() -> Self {
        // Load files immediately upon creation
        let items = Self::load_files();
        
        Self { 
            searched_string: String::new(),
            items,
        }
    }

    // Helper function to load files safely
    fn load_files() -> Vec<FileItem> {
        let mut items = vec![];
        let main_folders = vec!["Documents", "Pictures", "Videos", "Music", "Downloads"];

        // Get the user profile dynamically (Works on any Windows user)
        let base_path = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:/".to_string());

        for folder in main_folders {
            let full_path = format!("{}\\{}", base_path, folder);

            // Use `if let Ok` instead of `unwrap`. If a folder is missing, we skip it safely.
            if let Ok(entries) = fs::read_dir(full_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        // Safe conversion of filename
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        let item_type = if path.is_dir() { "Folder" } else { "File" };

                        items.push(FileItem {
                            name,
                            item_type: item_type.to_string(),
                            path: path.to_string_lossy().into_owned(),
                        });
                    }
                }
            }
        }
        items
    }

    pub fn set_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.searched_string = query;
        cx.notify(); 
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
                self.items // 6. Iterate over the pre-loaded items
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