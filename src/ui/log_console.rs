//! Log console, the gpui counterpart of the Java `LogWindow`.
//!
//! It is a collapsible panel of the main window rather than a separate stage,
//! and it is toggled from the title bar or from the tray menu.

use gpui::*;
use gpui_component::StyledExt;

/// Number of lines kept in the console. The Java text area grew without
/// bound; here the oldest lines are dropped instead.
const MAX_LINES: usize = 1000;

pub struct LogConsole {
    lines: Vec<String>,
    visible: bool,
}

impl LogConsole {
    pub fn new() -> Self {
        Self { lines: Vec::new(), visible: false }
    }

    /// Appends a line, dropping the oldest ones once the buffer is full.
    pub fn append(&mut self, line: String, cx: &mut Context<Self>) {
        self.lines.push(line);
        if self.lines.len() > MAX_LINES {
            let overflow = self.lines.len() - MAX_LINES;
            self.lines.drain(..overflow);
        }
        if self.visible {
            cx.notify();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        if self.visible != visible {
            self.visible = visible;
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.set_visible(!self.visible, cx);
    }
}

impl Render for LogConsole {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();

        // Mounted by `Main` only in place of the search bar and the result
        // list, so it always has the whole content area to itself.
        div()
            .size_full()
            .v_flex()
            .bg(rgb(0x141414))
            .rounded_md()
            .border_1()
            .border_color(rgb(0x2d2d2d))
            .child(
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child("Console"),
            )
            .child(
                div().w_full().h_0().flex_grow().px_3().pb_2().child(
                    // Only the visible lines are built each frame: rendering
                    // all of them, up to `MAX_LINES`, made every redraw the
                    // panel triggers - including one on every mouse move
                    // while it has hover state to update - noticeably janky.
                    uniform_list("log-console-lines", self.lines.len(), move |visible, _window, cx| {
                        this.read(cx)
                            .lines
                            .get(visible)
                            .unwrap_or_default()
                            .iter()
                            .map(|line| {
                                div().text_xs().font_family("monospace").text_color(rgb(0xbbbbbb)).child(line.clone())
                            })
                            .collect()
                    })
                    .size_full(),
                ),
            )
    }
}
