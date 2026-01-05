use gpui::*;
use gpui_component::{button::*, input::*, *};

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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_4()
            .p_4()
            .size_full() // Fill the allocated 60%x30% window space
            .items_center()
            .justify_start()
            .child(
                // Render the input component linked to our state
                Input::new(&self.input_state)
                    .w_full()
            )
            .child(
                Button::new("ok")
                    .primary()
                    .label("Let's Go!")
                    .on_click(|_, _, _| println!("Clicked!")),
            )
    }
}

fn main() {
    let app = Application::new();
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