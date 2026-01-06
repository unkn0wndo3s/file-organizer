// list.rs
use gpui::*;
use gpui_component::{Icon, StyledExt};
use crate::icons::*;

pub struct List {
    searched_string: String,
}

struct FileItem {
    name: &'static str,
    item_type: &'static str,
    path: &'static str,
}

impl List {
    pub fn new() -> Self {
        Self { 
            searched_string: String::new(),
        }
    }

    pub fn set_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.searched_string = query;
        cx.notify(); // Re-renders the list
    }
}

impl Render for List {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 2. Initialize the list of structs
        let items = vec![
            FileItem { name: "Document1.txt", item_type: "File", path: "C:/Users/potevinl/Documents/Document1.txt" },
            FileItem { name: "Image1.png", item_type: "File", path: "C:/Users/potevinl/Images/Image1.png" },
            FileItem { name: "Presentation.pptx", item_type: "File", path: "C:/Users/potevinl/Presentations/Presentation.pptx" },
            FileItem { name: "Notes.docx", item_type: "File", path: "C:/Users/potevinl/Notes/Notes.docx" },
            FileItem { name: "Photo.jpg", item_type: "File", path: "C:/Users/potevinl/Photos/Photo.jpg" },
            FileItem { name: "Archive.zip", item_type: "File", path: "C:/Users/potevinl/Archives/Archive.zip" },
            FileItem { name: "Music.mp3", item_type: "File", path: "C:/Users/potevinl/Music/Music.mp3" },
            FileItem { name: "Videos", item_type: "Folder", path: "C:/Users/potevinl/Videos" },
            FileItem { name: "Projects", item_type: "Folder", path: "C:/Users/potevinl/Projects" },
        ];

        div()
            .v_flex()
            .w_full()
            .gap_2()
            .items_start()
            .children(
                items
                    .into_iter()
                    .filter(|item| {
                        // 3. Filter based on the 'name' field
                        self.searched_string.is_empty() || 
                        item.name.to_lowercase().contains(&self.searched_string.to_lowercase())
                    })
                    .map(|item| {
                        // 4. Render the UI for each item
                        let icon = if item.item_type == "Folder" {
                            Icon::new(IconName::Minimize)
                        } else {
                            Icon::new(IconName::Close)
                        };
                        div()
                            .w_full()
                            .p_2()
                            .bg(rgb(0x2d2d2d))
                            .hover(|s| s.bg(rgb(0x3d3d3d)))
                            .rounded_md()
                            .child(
                                // Use a horizontal flex to align Icon and Text
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3() // Space between icon and text
                                    .child(icon.size_4().text_color(rgb(0x999999)))
                                    .child(
                                        div()
                                            .text_color(rgb(0xffffff))
                                            .child(item.name.to_string())
                                    )
                            )
                    })
            )
    }
}