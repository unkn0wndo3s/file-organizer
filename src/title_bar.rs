// title_bar.rs
use gpui::*;
use gpui_component::Icon;
use crate::icons::*;

pub struct TitleBar {
    hover_opacity: f32,
}

impl TitleBar {
    pub fn new() -> Self {
        Self { hover_opacity: 0.0 }
    }

    pub fn update_hover(&mut self, target: f32, cx: &mut Context<Self>) {
        if (self.hover_opacity - target).abs() < 0.01 {
            self.hover_opacity = target;
            return;
        }
        self.hover_opacity += (target - self.hover_opacity) * 0.2;
        cx.notify();
    }
}

impl Render for TitleBar {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_8()
            .w_full()
            .flex()
            .bg(rgb(0x2d2d2d)) // Slight background to distinguish title bar
            .justify_between() // Space between drag area and buttons
            .items_center()
            .child(
                div().px_4().text_sm().child("My App") // Optional title text
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    // Minimize Button
                    .child(
                        div()
                            .id("minimize-button")
                            .size_6()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .hover(|s| s.bg(rgba(0xffffff10)))
                            .on_click(|_, window, _| {
                                window.minimize_window();
                            })
                            .child(Icon::new(IconName::Minimize))
                    )
                    // Close Button
                    .child(
                        div()
                            .id("close-button")
                            .size_6()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_md()
                            .hover(|s| s.bg(rgba(0xef4444aa))) // Red hover for close
                            .on_click(|_, _, cx| {
                                cx.quit();
                            })
                            .child(Icon::new(IconName::Close))
                    )
            )
    }
}