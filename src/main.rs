use gpui::*;
use gpui_component::{button::*, input::*, *};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

/// Asset source that looks in the project's /assets folder
// In the struct definition, only the type is allowed
pub struct LocalAssets {
    pub base: PathBuf,
}

impl LocalAssets {
    // Create a constructor to handle the path logic
    pub fn new() -> Self {
        Self {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        }
    }
}

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

#[derive(Clone)] 
pub enum IconName {
    Close,
    Minimize,
}

impl IconNamed for IconName {
    fn path(self) -> SharedString {
        match self {
            IconName::Close => "icons/close.svg".into(),
            IconName::Minimize => "icons/minus.svg".into(),
        }
    }
}

pub struct Main {
    // State handle for the text input
    input_state: Entity<InputState>,
}

impl Main {
    // Constructor called during window creation
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            // Set the ghost text displayed when the input is empty
            state.set_placeholder("Enter your name", window, cx);
            state
        });

        Self { input_state }
    }
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .bg(rgb(0x1e1e1e))
            .child(
                // Custom Title Bar
                div()
                    .h_8()
                    .w_full()
                    .flex()
                    .justify_end()
                    .items_center()
                    .gap_4()
                    .p_2()
                    // ... inside the Title Bar div ...
                    .child(
                        // Minimize Button
                        div()
                            .id("minimize-button")
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_4()
                            .rounded_full()
                            .hover(|s| s.bg(rgb(0x3b82f6)))
                            // In GPUI 0.2.2, the 3rd parameter 'cx' provides the window/app access
                            .on_click(|_, window, _| {
                                                    window.minimize_window();
                            })
                            .child(
                                // Ripple effect div
                                div()
                                    .id("ripple-effect")
                                    .absolute()
                                    .inset_0()
                                    .rounded_full()
                                    // English comment: Use border and active state to show the press
                                    .active(|s| s
                                        .border_2()
                                        .border_color(rgb(0x93c5fd))
                                        // If .transform is missing, we use negative margins to "grow" the circle
                                        .margins(-px(4.0)) 
                                    )
                            )
                            .child(Icon::new(IconName::Minimize))
                    )
                    .child(
                        // Close Button
                        div()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.quit();
                            })
                            .child(Icon::new(IconName::Close))
                    ) // End of Close Button
            ) // End of Title Bar
            .child(
                // Main Content Area
                div()
                    .flex_grow()
                    .v_flex()
                    .items_center()
                    .justify_start()
                    .p_4()
                    .child(Input::new(&self.input_state).w_full())
                    .child(
                        div()
                            .mt_4()
                            .child(
                                Button::new("ok")
                                    .primary()
                                    .label("Let's Go!")
                                    .on_click(|_, _, _| println!("Clicked!")),
                            )
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
            // Determine screen size for relative window scaling
            let screen_bounds = cx.update(|cx| {
                cx.displays()
                    .first()
                    .map(|d| d.bounds())
                    .unwrap_or(Bounds::default())
            })?;

            let width = screen_bounds.size.width * 0.6;
            let height = screen_bounds.size.height * 0.3;
            let x = screen_bounds.origin.x + (screen_bounds.size.width - width) / 2.0;
            let y = screen_bounds.origin.y + (screen_bounds.size.height - height) / 2.0;

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(point(x, y), size(width, height)))),
                is_resizable: false, // Prevent users from changing window size
                // This removes the native OS title bar and buttons
                titlebar: None,
                ..Default::default()
            };

            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| Main::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}