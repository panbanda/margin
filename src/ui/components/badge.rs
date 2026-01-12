//! Badge component.
//!
//! Displays status indicators, counts, or labels.

use gpui::{
    div, px, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled,
};

use crate::ui::theme::ThemeColors;

/// Badge variant styles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Default neutral badge.
    #[default]
    Default,
    /// Primary accent badge.
    Primary,
    /// Success/positive badge.
    Success,
    /// Warning badge.
    Warning,
    /// Error/danger badge.
    Error,
    /// Muted/subtle badge.
    Muted,
}

/// Badge size options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeSize {
    /// Small badge (16px height).
    Small,
    /// Medium badge (20px height).
    #[default]
    Medium,
    /// Large badge (24px height).
    Large,
}

impl BadgeSize {
    fn height(self) -> f32 {
        match self {
            BadgeSize::Small => 16.0,
            BadgeSize::Medium => 20.0,
            BadgeSize::Large => 24.0,
        }
    }

    fn font_size(self) -> f32 {
        match self {
            BadgeSize::Small => 10.0,
            BadgeSize::Medium => 11.0,
            BadgeSize::Large => 12.0,
        }
    }

    fn padding_x(self) -> f32 {
        match self {
            BadgeSize::Small => 4.0,
            BadgeSize::Medium => 6.0,
            BadgeSize::Large => 8.0,
        }
    }
}

/// A badge component for status indicators or counts.
pub struct Badge {
    id: ElementId,
    label: SharedString,
    variant: BadgeVariant,
    size: BadgeSize,
    pill: bool,
}

impl Badge {
    /// Create a new badge with the given label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: BadgeVariant::Default,
            size: BadgeSize::Medium,
            pill: false,
        }
    }

    /// Set the badge variant.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the badge size.
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    /// Make the badge pill-shaped (fully rounded).
    pub fn pill(mut self) -> Self {
        self.pill = true;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();

        let (bg, text_color) = match self.variant {
            BadgeVariant::Default => (colors.surface_elevated, colors.text_secondary),
            BadgeVariant::Primary => (colors.accent, colors.text_primary),
            BadgeVariant::Success => (colors.success, colors.text_primary),
            BadgeVariant::Warning => (colors.warning, colors.background),
            BadgeVariant::Error => (colors.error, colors.text_primary),
            BadgeVariant::Muted => (colors.border, colors.text_muted),
        };

        let height = self.size.height();
        let font_size = self.size.font_size();
        let padding_x = self.size.padding_x();
        let radius = if self.pill { height / 2.0 } else { 4.0 };

        div()
            .id(self.id)
            .h(px(height))
            .px(px(padding_x))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius))
            .bg(bg)
            .text_color(text_color)
            .text_size(px(font_size))
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(self.label)
    }
}

/// A count badge (circular, typically for unread counts).
pub struct CountBadge {
    id: ElementId,
    count: u32,
    max_count: u32,
    variant: BadgeVariant,
    size: BadgeSize,
}

impl CountBadge {
    /// Create a new count badge.
    pub fn new(id: impl Into<ElementId>, count: u32) -> Self {
        Self {
            id: id.into(),
            count,
            max_count: 99,
            variant: BadgeVariant::Primary,
            size: BadgeSize::Small,
        }
    }

    /// Set the maximum displayable count (shows "99+" if exceeded).
    pub fn max_count(mut self, max: u32) -> Self {
        self.max_count = max;
        self
    }

    /// Set the badge variant.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the badge size.
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for CountBadge {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();

        let (bg, text_color) = match self.variant {
            BadgeVariant::Default => (colors.surface_elevated, colors.text_secondary),
            BadgeVariant::Primary => (colors.accent, colors.text_primary),
            BadgeVariant::Success => (colors.success, colors.text_primary),
            BadgeVariant::Warning => (colors.warning, colors.background),
            BadgeVariant::Error => (colors.error, colors.text_primary),
            BadgeVariant::Muted => (colors.border, colors.text_muted),
        };

        let height = self.size.height();
        let font_size = self.size.font_size();

        let label = if self.count > self.max_count {
            format!("{}+", self.max_count)
        } else {
            self.count.to_string()
        };

        // Minimum width is the height to ensure circular shape for single digits
        let min_width = height;
        let padding_x = if label.len() > 1 {
            self.size.padding_x()
        } else {
            0.0
        };

        div()
            .id(self.id)
            .h(px(height))
            .min_w(px(min_width))
            .px(px(padding_x))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(bg)
            .text_color(text_color)
            .text_size(px(font_size))
            .font_weight(gpui::FontWeight::BOLD)
            .child(label)
    }
}

/// A dot indicator (no text, just a colored dot).
pub struct DotIndicator {
    id: ElementId,
    variant: BadgeVariant,
    size: f32,
    pulse: bool,
}

impl DotIndicator {
    /// Create a new dot indicator.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: BadgeVariant::Primary,
            size: 8.0,
            pulse: false,
        }
    }

    /// Set the dot variant (color).
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the dot size in pixels.
    pub fn size_px(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Enable pulsing animation indicator.
    pub fn pulse(mut self) -> Self {
        self.pulse = true;
        self
    }
}

impl RenderOnce for DotIndicator {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();

        let bg = match self.variant {
            BadgeVariant::Default => colors.text_muted,
            BadgeVariant::Primary => colors.accent,
            BadgeVariant::Success => colors.success,
            BadgeVariant::Warning => colors.warning,
            BadgeVariant::Error => colors.error,
            BadgeVariant::Muted => colors.border,
        };

        div().id(self.id).size(px(self.size)).rounded_full().bg(bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_builder() {
        let badge = Badge::new("test", "Label")
            .variant(BadgeVariant::Success)
            .size(BadgeSize::Large)
            .pill();

        assert_eq!(badge.variant, BadgeVariant::Success);
        assert_eq!(badge.size, BadgeSize::Large);
        assert!(badge.pill);
    }

    #[test]
    fn badge_sizes() {
        assert_eq!(BadgeSize::Small.height(), 16.0);
        assert_eq!(BadgeSize::Medium.height(), 20.0);
        assert_eq!(BadgeSize::Large.height(), 24.0);
    }

    #[test]
    fn count_badge_builder() {
        let badge = CountBadge::new("count", 42)
            .max_count(50)
            .variant(BadgeVariant::Error)
            .size(BadgeSize::Medium);

        assert_eq!(badge.count, 42);
        assert_eq!(badge.max_count, 50);
        assert_eq!(badge.variant, BadgeVariant::Error);
    }

    #[test]
    fn dot_indicator_builder() {
        let dot = DotIndicator::new("dot")
            .variant(BadgeVariant::Success)
            .size_px(12.0)
            .pulse();

        assert_eq!(dot.variant, BadgeVariant::Success);
        assert_eq!(dot.size, 12.0);
        assert!(dot.pulse);
    }
}
