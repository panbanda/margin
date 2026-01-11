//! Main application window

use gpui::{
    div, px, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window,
};

use crate::ui::theme::Theme;

/// Main window view containing the primary application layout
pub struct MainWindow {
    theme: Theme,
}

impl MainWindow {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            theme: Theme::dark(),
        }
    }

    fn render_title_bar(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;

        div()
            .id("title-bar")
            .h(px(40.0))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px(px(16.0))
            .bg(colors.surface)
            .border_b_1()
            .border_color(colors.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(SharedString::from("margin")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(SharedString::from("Search...")),
            )
    }

    fn render_sidebar(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;

        div()
            .id("sidebar")
            .w(px(220.0))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.surface)
            .border_r_1()
            .border_color(colors.border)
            .child(
                div()
                    .p(px(12.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(self.render_sidebar_item("Inbox", true))
                    .child(self.render_sidebar_item("Starred", false))
                    .child(self.render_sidebar_item("Sent", false))
                    .child(self.render_sidebar_item("Drafts", false))
                    .child(self.render_sidebar_item("Archive", false))
                    .child(self.render_sidebar_item("Trash", false)),
            )
    }

    fn render_sidebar_item(&self, label: &str, selected: bool) -> impl IntoElement {
        let colors = &self.theme.colors;
        let bg = if selected {
            colors.surface_elevated
        } else {
            colors.surface
        };
        let hover_bg = colors.surface_elevated;

        div()
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(6.0))
            .bg(bg)
            .text_color(colors.text_primary)
            .cursor_pointer()
            .hover(move |style| style.bg(hover_bg))
            .child(SharedString::from(label.to_string()))
    }

    fn render_message_list(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = &self.theme.colors;

        div()
            .id("message-list")
            .w(px(350.0))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .border_r_1()
            .border_color(colors.border)
            .child(
                div()
                    .p(px(12.0))
                    .border_b_1()
                    .border_color(colors.border)
                    .child(SharedString::from("Inbox")),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_y_hidden()
                    .child(self.render_message_item(
                        "Welcome to margin",
                        "The margin team",
                        "Get started with your new email client...",
                        true,
                    ))
                    .child(self.render_message_item(
                        "Meeting tomorrow",
                        "Alice",
                        "Don't forget about our meeting...",
                        false,
                    ))
                    .child(self.render_message_item(
                        "Project update",
                        "Bob",
                        "Here's the latest update on...",
                        false,
                    )),
            )
    }

    fn render_message_item(
        &self,
        subject: &str,
        sender: &str,
        preview: &str,
        unread: bool,
    ) -> impl IntoElement {
        let colors = &self.theme.colors;
        let text_color = if unread {
            colors.text_primary
        } else {
            colors.text_secondary
        };
        let hover_bg = colors.surface;
        let font_weight = if unread {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        };

        div()
            .px(px(12.0))
            .py(px(10.0))
            .border_b_1()
            .border_color(colors.border)
            .cursor_pointer()
            .hover(move |style| style.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_color(text_color)
                            .font_weight(font_weight)
                            .child(SharedString::from(subject.to_string())),
                    )
                    .child(
                        div()
                            .text_color(colors.text_muted)
                            .text_sm()
                            .child(SharedString::from("12:30")),
                    ),
            )
            .child(
                div()
                    .text_color(colors.text_secondary)
                    .text_sm()
                    .child(SharedString::from(format!("{} - {}", sender, preview))),
            )
    }

    fn render_reading_pane(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = &self.theme.colors;

        div()
            .id("reading-pane")
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .child(
                div().flex_1().flex().items_center().justify_center().child(
                    div()
                        .text_color(colors.text_muted)
                        .child(SharedString::from("Select a message to read")),
                ),
            )
    }

    fn render_status_bar(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;

        div()
            .id("status-bar")
            .h(px(24.0))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .bg(colors.surface)
            .border_t_1()
            .border_color(colors.border)
            .child(
                div()
                    .text_color(colors.text_muted)
                    .text_xs()
                    .child(SharedString::from("Ready")),
            )
            .child(
                div()
                    .text_color(colors.text_muted)
                    .text_xs()
                    .child(SharedString::from("3 unread")),
            )
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &self.theme.colors;

        div()
            .id("main-window")
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .text_color(colors.text_primary)
            .child(self.render_title_bar(window, cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .child(self.render_sidebar(window, cx))
                    .child(self.render_message_list(window, cx))
                    .child(self.render_reading_pane(window, cx)),
            )
            .child(self.render_status_bar(window, cx))
    }
}
