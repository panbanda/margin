//! Tooltip component.
//!
//! Provides hover tooltips for additional context on UI elements.

use gpui::{
    div, px, AnyElement, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled,
};

use crate::ui::theme::ThemeColors;

/// Tooltip position relative to the trigger element.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipPosition {
    /// Tooltip appears above the trigger.
    #[default]
    Top,
    /// Tooltip appears below the trigger.
    Bottom,
    /// Tooltip appears to the left.
    Left,
    /// Tooltip appears to the right.
    Right,
}

/// A tooltip component wrapper.
///
/// Wraps a child element and displays a tooltip on hover.
pub struct Tooltip {
    id: ElementId,
    #[allow(dead_code)]
    content: SharedString,
    position: TooltipPosition,
    delay_ms: u32,
    max_width: f32,
    child: Option<AnyElement>,
}

impl Tooltip {
    /// Create a new tooltip with the given content.
    pub fn new(id: impl Into<ElementId>, content: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            position: TooltipPosition::Top,
            delay_ms: 300,
            max_width: 200.0,
            child: None,
        }
    }

    /// Set the tooltip position.
    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the delay before showing the tooltip (in milliseconds).
    pub fn delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Set the maximum width of the tooltip.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Set the child element that triggers the tooltip.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        // For now, we render just the child. Full tooltip implementation
        // would require gpui's tooltip system or custom hover state management.
        // This provides the structure for future implementation.
        let container = div().id(self.id);

        if let Some(child) = self.child {
            container.child(child)
        } else {
            container
        }
    }
}

/// A standalone tooltip box (for custom positioning).
pub struct TooltipBox {
    id: ElementId,
    content: SharedString,
    max_width: f32,
    arrow: bool,
    arrow_position: TooltipPosition,
}

impl TooltipBox {
    /// Create a new tooltip box.
    pub fn new(id: impl Into<ElementId>, content: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            max_width: 200.0,
            arrow: true,
            arrow_position: TooltipPosition::Bottom,
        }
    }

    /// Set the maximum width.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Show or hide the arrow.
    pub fn arrow(mut self, show: bool) -> Self {
        self.arrow = show;
        self
    }

    /// Set the arrow position (where the arrow points).
    pub fn arrow_position(mut self, position: TooltipPosition) -> Self {
        self.arrow_position = position;
        self
    }
}

impl RenderOnce for TooltipBox {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();

        div()
            .id(self.id)
            .max_w(px(self.max_width))
            .px(px(8.0))
            .py(px(6.0))
            .rounded(px(4.0))
            .bg(colors.surface_elevated)
            .border_1()
            .border_color(colors.border)
            .text_color(colors.text_primary)
            .text_size(px(12.0))
            .shadow_md()
            .child(self.content)
    }
}

/// A help tooltip with an info icon trigger.
pub struct HelpTooltip {
    id: ElementId,
    #[allow(dead_code)]
    content: SharedString,
}

impl HelpTooltip {
    /// Create a new help tooltip.
    pub fn new(id: impl Into<ElementId>, content: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
        }
    }
}

impl RenderOnce for HelpTooltip {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();

        // Render an info icon that would show tooltip on hover
        div()
            .id(self.id)
            .size(px(16.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .text_color(colors.text_muted)
            .text_size(px(12.0))
            .cursor_pointer()
            .hover(move |style| style.text_color(colors.text_secondary))
            .child("\u{2139}") // Info symbol
    }
}

/// A keyboard shortcut hint.
pub struct KeyboardHint {
    id: ElementId,
    keys: Vec<SharedString>,
}

impl KeyboardHint {
    /// Create a new keyboard hint.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            keys: Vec::new(),
        }
    }

    /// Add a key to the hint.
    pub fn key(mut self, key: impl Into<SharedString>) -> Self {
        self.keys.push(key.into());
        self
    }

    /// Set multiple keys at once.
    pub fn keys(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.keys = keys.into_iter().map(|k| k.into()).collect();
        self
    }
}

impl RenderOnce for KeyboardHint {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();

        let mut container = div().id(self.id).flex().items_center().gap(px(2.0));

        for (i, key) in self.keys.iter().enumerate() {
            if i > 0 {
                container = container.child(
                    div()
                        .text_color(colors.text_muted)
                        .text_size(px(10.0))
                        .child("+"),
                );
            }

            container = container.child(
                div()
                    .px(px(4.0))
                    .py(px(2.0))
                    .rounded(px(3.0))
                    .bg(colors.surface_elevated)
                    .border_1()
                    .border_color(colors.border)
                    .text_color(colors.text_secondary)
                    .text_size(px(11.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(key.clone()),
            );
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_builder() {
        let tooltip = Tooltip::new("test", "Help text")
            .position(TooltipPosition::Bottom)
            .delay(500)
            .max_width(300.0);

        assert_eq!(tooltip.position, TooltipPosition::Bottom);
        assert_eq!(tooltip.delay_ms, 500);
        assert_eq!(tooltip.max_width, 300.0);
    }

    #[test]
    fn tooltip_box_builder() {
        let tooltip = TooltipBox::new("box", "Content")
            .max_width(250.0)
            .arrow(false)
            .arrow_position(TooltipPosition::Left);

        assert_eq!(tooltip.max_width, 250.0);
        assert!(!tooltip.arrow);
        assert_eq!(tooltip.arrow_position, TooltipPosition::Left);
    }

    #[test]
    fn keyboard_hint_builder() {
        let hint = KeyboardHint::new("hint").key("Ctrl").key("S");

        assert_eq!(hint.keys.len(), 2);
    }

    #[test]
    fn keyboard_hint_keys() {
        let hint = KeyboardHint::new("hint").keys(["Cmd", "Shift", "P"]);

        assert_eq!(hint.keys.len(), 3);
    }
}
