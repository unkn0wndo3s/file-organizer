mod icons;
mod title_bar;
mod list;

use gpui::*;
use gpui::{AsyncApp, Styled, WeakEntity, Entity, AsyncWindowContext};
use gpui_component::{input::*, *};
use title_bar::TitleBar;
use icons::LocalAssets;
use list::List;
use crossbeam_channel::{unbounded, Sender, Receiver};
use notify::{Watcher, RecursiveMode, Event};
use std::path::PathBuf;
use std::time::Duration;

pub struct Main {
    title_bar: Entity<TitleBar>,
    list: Entity<List>,
    input_state: Entity<InputState>,
    // Keep watcher alive as long as the app is running
    watcher: Option<notify::RecommendedWatcher>,
}

impl Main {
    pub fn new(title_bar: Entity<TitleBar>, list: Entity<List>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_placeholder("Search for file or folder", window, cx);
            state
        });

        let list_handle = list.clone();
        
        cx.subscribe(&input_state, move |_this, input_handle, _event: &gpui_component::input::InputEvent, cx| {
            let query = input_handle.read(cx).value().to_string();
            list_handle.update(cx, |list_entity, cx| {
                list_entity.set_search(query, cx);
            });
        }).detach();

        // Watcher Setup
        let (tx, rx): (Sender<Event>, Receiver<Event>) = unbounded();
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res { let _ = tx.send(event); }
        }).ok();
        
        if let Some(w) = watcher.as_mut() {
            let base_path = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:/".to_string());
            let main_folders = vec!["Desktop", "Documents", "Images", "Videos", "Music", "Downloads", "Folders", "Executables", "Archives"];
            for folder in main_folders {
                let full_path = PathBuf::from(&base_path).join(folder);
                if full_path.exists() {
                    let _ = w.watch(&full_path, RecursiveMode::NonRecursive);
                }
            }
        }
        
       let list_weak = list.downgrade();
        
        // Grab executor outside to avoid type inference issues
        let executor = cx.background_executor().clone();

        cx.spawn(move |_weak_main, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    executor.timer(Duration::from_millis(200)).await;
                
                    let mut events = vec![];
                    while let Ok(event) = rx.try_recv() {
                        events.push(event);
                    }
                
                    if !events.is_empty() {
                        // Use cx.clone() to pass a handle by value without consuming the original
                        let _ = list_weak.update(&mut cx.clone(), |list, cx| {
                            for event in events {
                                list.handle_fs_event(event, cx);
                            }
                        });
                    }
                }
            }
        }).detach();



        Self { 
            title_bar,
            input_state,
            list,
            watcher,
        }
    }
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _view = cx.entity().clone();

        div()
            .size_full()
            .v_flex()
            .bg(rgb(0x1e1e1e))
            .child(self.title_bar.clone())
            .child(
                div()
                    .flex_grow()
                    .v_flex()
                    .p_4()
                    .gap_4()
                    .child(Input::new(&self.input_state).w_full())
                    .child(
                        div()
                            .w_full()
                            .h_0()
                            .flex_grow()
                            .id("list-scroll-view")
                            .overflow_y_scroll() 
                            .child(self.list.clone())
                    )
            )
    }
}

fn main() {
    let assets = LocalAssets::new();
    let app = Application::new().with_assets(assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            let screen_bounds = cx.update(|cx| {
                cx.displays()
                    .first()
                    .map(|d| d.bounds())
                    .unwrap_or(Bounds::default())
            })?;

            let width = screen_bounds.size.width * 0.6;
            let height = screen_bounds.size.height * 0.6; // Increased height slightly
            let x = screen_bounds.origin.x + (screen_bounds.size.width - width) / 2.0;
            let y = screen_bounds.origin.y + (screen_bounds.size.height - height) / 2.0;

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(point(x, y), size(width, height)))),
                is_resizable: true, 
                titlebar: None,
                ..Default::default()
            };

            cx.open_window(options, |window, cx| {
                let title_bar = cx.new(|_cx| TitleBar::new());
                let list = cx.new(|cx| List::new(cx));
                let view = cx.new(|cx| Main::new(title_bar, list, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}