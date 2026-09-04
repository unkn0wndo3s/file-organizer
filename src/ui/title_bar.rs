use crate::ui::icons::IconName;
use gpui::*;
use gpui_component::Icon;

/// Emitted when the user clicks a title bar button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleBarEvent {
    ToggleConsole,
}

impl EventEmitter<TitleBarEvent> for TitleBar {}

pub struct TitleBar;

impl TitleBar {
    pub fn new() -> Self {
        Self
    }
}

impl Render for TitleBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let console_emitter = cx.entity().downgrade();

        div()
            .h_8()
            .w_full()
            .flex()
            .justify_between()
            .items_center()
            .child(div().px_4().text_sm().text_color(rgb(0xffffff)).child("File Organizer"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .child(
                        title_bar_button("console-button", IconName::Settings)
                            .hover(|style| style.bg(rgba(0xffffff10)))
                            .on_click(move |_, _, cx| {
                                console_emitter
                                    .update(cx, |_, cx| cx.emit(TitleBarEvent::ToggleConsole))
                                    .ok();
                            }),
                    ),
            )
    }
}

fn title_bar_button(id: &'static str, icon: IconName) -> Stateful<Div> {
    div()
        .id(id)
        .size_6()
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .child(Icon::new(icon).text_color(rgb(0xffffff)))
}
