mod icons;
mod title_bar;
use gpui::*;
use gpui_component::{button::*, input::*, *};
use title_bar::TitleBar;
use icons::LocalAssets;

pub struct Main {
    title_bar: Entity<TitleBar>,
    // State handle for the text input
    input_state: Entity<InputState>,
}

impl Main {
    pub fn new(title_bar: Entity<TitleBar>,window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_placeholder("Enter your name", window, cx);
            state
        });

        Self { 
            title_bar,
            input_state,
        }
    }

}

impl Render for Main {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get the handle for this view to update it from closures
        let _view = cx.entity().clone();

        div()
            .size_full()
            .v_flex()
            .bg(rgb(0x1e1e1e))
            .child(
                //Title Bar
                self.title_bar.clone()
            )
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
                let title_bar = cx.new(|_cx| TitleBar::new());
                let view = cx.new(|cx| Main::new(title_bar, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}