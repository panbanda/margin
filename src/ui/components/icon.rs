//! Icon component.
//!
//! Provides a consistent way to render icons throughout the application.

use gpui::{
    div, px, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled,
};

use crate::ui::theme::ThemeColors;

/// Icon size options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IconSize {
    /// Extra small (12px).
    XSmall,
    /// Small (14px).
    Small,
    /// Medium (16px).
    #[default]
    Medium,
    /// Large (20px).
    Large,
    /// Extra large (24px).
    XLarge,
}

impl IconSize {
    /// Get the pixel size.
    pub fn px(self) -> f32 {
        match self {
            IconSize::XSmall => 12.0,
            IconSize::Small => 14.0,
            IconSize::Medium => 16.0,
            IconSize::Large => 20.0,
            IconSize::XLarge => 24.0,
        }
    }
}

/// Common icon names for consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconName {
    // Navigation
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronDown,

    // Actions
    Close,
    Check,
    Plus,
    Minus,
    Edit,
    Delete,
    Search,
    Filter,
    Sort,
    Refresh,

    // Email
    Mail,
    MailOpen,
    Reply,
    ReplyAll,
    Forward,
    Archive,
    Trash,
    Star,
    StarFilled,
    Attachment,
    Send,

    // Status
    Info,
    Warning,
    Error,
    Success,

    // Misc
    Settings,
    User,
    Users,
    Calendar,
    Clock,
    Link,
    Copy,
    Download,
    Upload,
    Folder,
    Tag,
    Label,
}

impl IconName {
    /// Get the symbol/character for this icon.
    /// In a real app, this would use an icon font or SVG.
    pub fn symbol(self) -> &'static str {
        match self {
            // Navigation
            IconName::ArrowLeft => "\u{2190}",
            IconName::ArrowRight => "\u{2192}",
            IconName::ArrowUp => "\u{2191}",
            IconName::ArrowDown => "\u{2193}",
            IconName::ChevronLeft => "\u{2039}",
            IconName::ChevronRight => "\u{203A}",
            IconName::ChevronUp => "\u{2303}",
            IconName::ChevronDown => "\u{2304}",

            // Actions
            IconName::Close => "\u{2715}",
            IconName::Check => "\u{2713}",
            IconName::Plus => "+",
            IconName::Minus => "\u{2212}",
            IconName::Edit => "\u{270E}",
            IconName::Delete => "\u{2717}",
            IconName::Search => "\u{26B2}",
            IconName::Filter => "\u{25BC}",
            IconName::Sort => "\u{21C5}",
            IconName::Refresh => "\u{21BB}",

            // Email
            IconName::Mail => "\u{2709}",
            IconName::MailOpen => "\u{2709}",
            IconName::Reply => "\u{21A9}",
            IconName::ReplyAll => "\u{21A9}",
            IconName::Forward => "\u{21AA}",
            IconName::Archive => "\u{2636}",
            IconName::Trash => "\u{2717}",
            IconName::Star => "\u{2606}",
            IconName::StarFilled => "\u{2605}",
            IconName::Attachment => "\u{1F4CE}",
            IconName::Send => "\u{27A4}",

            // Status
            IconName::Info => "\u{2139}",
            IconName::Warning => "\u{26A0}",
            IconName::Error => "\u{2716}",
            IconName::Success => "\u{2714}",

            // Misc
            IconName::Settings => "\u{2699}",
            IconName::User => "\u{263A}",
            IconName::Users => "\u{263A}",
            IconName::Calendar => "\u{1F4C5}",
            IconName::Clock => "\u{23F0}",
            IconName::Link => "\u{1F517}",
            IconName::Copy => "\u{2750}",
            IconName::Download => "\u{21E3}",
            IconName::Upload => "\u{21E1}",
            IconName::Folder => "\u{1F4C1}",
            IconName::Tag => "\u{1F3F7}",
            IconName::Label => "\u{1F3F7}",
        }
    }
}

/// An icon component.
pub struct Icon {
    id: ElementId,
    icon: SharedString,
    size: IconSize,
    color: Option<Hsla>,
}

impl Icon {
    /// Create a new icon with a custom symbol/character.
    pub fn new(id: impl Into<ElementId>, icon: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            size: IconSize::Medium,
            color: None,
        }
    }

    /// Create an icon from a named icon.
    pub fn named(id: impl Into<ElementId>, name: IconName) -> Self {
        Self {
            id: id.into(),
            icon: name.symbol().into(),
            size: IconSize::Medium,
            color: None,
        }
    }

    /// Set the icon size.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Set a custom color for the icon.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();
        let size = self.size.px();
        let color = self.color.unwrap_or(colors.text_secondary);

        div()
            .id(self.id)
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .text_color(color)
            .text_size(px(size))
            .child(self.icon)
    }
}

/// An icon with a label next to it.
pub struct IconLabel {
    id: ElementId,
    icon: SharedString,
    label: SharedString,
    size: IconSize,
    gap: f32,
    reversed: bool,
}

impl IconLabel {
    /// Create a new icon with label.
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            size: IconSize::Medium,
            gap: 6.0,
            reversed: false,
        }
    }

    /// Create from a named icon.
    pub fn named(id: impl Into<ElementId>, name: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: name.symbol().into(),
            label: label.into(),
            size: IconSize::Medium,
            gap: 6.0,
            reversed: false,
        }
    }

    /// Set the icon size.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Set the gap between icon and label.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Put the label before the icon.
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }
}

impl RenderOnce for IconLabel {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let colors = ThemeColors::dark();
        let icon_size = self.size.px();

        let icon_element = div()
            .size(px(icon_size))
            .flex()
            .items_center()
            .justify_center()
            .text_color(colors.text_secondary)
            .text_size(px(icon_size))
            .child(self.icon);

        let label_element = div().text_color(colors.text_primary).child(self.label);

        let mut container = div().id(self.id).flex().items_center().gap(px(self.gap));

        if self.reversed {
            container = container.child(label_element).child(icon_element);
        } else {
            container = container.child(icon_element).child(label_element);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_sizes() {
        assert_eq!(IconSize::XSmall.px(), 12.0);
        assert_eq!(IconSize::Small.px(), 14.0);
        assert_eq!(IconSize::Medium.px(), 16.0);
        assert_eq!(IconSize::Large.px(), 20.0);
        assert_eq!(IconSize::XLarge.px(), 24.0);
    }

    #[test]
    fn icon_names_have_symbols() {
        assert!(!IconName::Mail.symbol().is_empty());
        assert!(!IconName::Star.symbol().is_empty());
        assert!(!IconName::Search.symbol().is_empty());
    }

    #[test]
    fn icon_builder() {
        let colors = ThemeColors::dark();
        let icon = Icon::new("test", "X")
            .size(IconSize::Large)
            .color(colors.error);

        assert_eq!(icon.size, IconSize::Large);
        assert!(icon.color.is_some());
    }

    #[test]
    fn icon_label_builder() {
        let icon_label = IconLabel::new("test", "X", "Close")
            .size(IconSize::Small)
            .gap(8.0)
            .reversed();

        assert_eq!(icon_label.size, IconSize::Small);
        assert_eq!(icon_label.gap, 8.0);
        assert!(icon_label.reversed);
    }
}
