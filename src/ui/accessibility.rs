//! Accessibility support for the margin UI.
//!
//! Provides accessibility features including:
//! - Screen reader announcements
//! - High contrast mode
//! - Reduced motion preferences
//! - Focus management
//! - Semantic role annotations

use gpui::SharedString;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global accessibility settings.
static HIGH_CONTRAST_ENABLED: AtomicBool = AtomicBool::new(false);
static REDUCED_MOTION_ENABLED: AtomicBool = AtomicBool::new(false);
static SCREEN_READER_ENABLED: AtomicBool = AtomicBool::new(false);

/// Accessibility settings for the application.
#[derive(Debug, Clone)]
pub struct AccessibilitySettings {
    /// Whether high contrast mode is enabled.
    pub high_contrast: bool,
    /// Whether reduced motion is enabled.
    pub reduced_motion: bool,
    /// Whether screen reader support is enabled.
    pub screen_reader: bool,
    /// Minimum touch target size in pixels.
    pub min_touch_target: f32,
    /// Focus ring width in pixels.
    pub focus_ring_width: f32,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            high_contrast: false,
            reduced_motion: false,
            screen_reader: false,
            min_touch_target: 44.0,
            focus_ring_width: 2.0,
        }
    }
}

impl AccessibilitySettings {
    /// Creates settings from system preferences.
    pub fn from_system() -> Self {
        Self {
            high_contrast: detect_high_contrast(),
            reduced_motion: detect_reduced_motion(),
            screen_reader: detect_screen_reader(),
            ..Default::default()
        }
    }

    /// Applies these settings globally.
    pub fn apply(&self) {
        HIGH_CONTRAST_ENABLED.store(self.high_contrast, Ordering::SeqCst);
        REDUCED_MOTION_ENABLED.store(self.reduced_motion, Ordering::SeqCst);
        SCREEN_READER_ENABLED.store(self.screen_reader, Ordering::SeqCst);
    }
}

/// Returns whether high contrast mode is enabled.
pub fn is_high_contrast() -> bool {
    HIGH_CONTRAST_ENABLED.load(Ordering::SeqCst)
}

/// Returns whether reduced motion is enabled.
pub fn is_reduced_motion() -> bool {
    REDUCED_MOTION_ENABLED.load(Ordering::SeqCst)
}

/// Returns whether screen reader support is enabled.
pub fn is_screen_reader_active() -> bool {
    SCREEN_READER_ENABLED.load(Ordering::SeqCst)
}

/// Semantic roles for accessibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A button that can be activated.
    Button,
    /// A link that navigates to another location.
    Link,
    /// A text input field.
    TextInput,
    /// A search input field.
    Search,
    /// A checkbox that can be toggled.
    Checkbox,
    /// A radio button in a group.
    Radio,
    /// A list of items.
    List,
    /// An item within a list.
    ListItem,
    /// A menu of options.
    Menu,
    /// An item within a menu.
    MenuItem,
    /// A dialog or modal.
    Dialog,
    /// An alert or notification.
    Alert,
    /// A tab in a tab list.
    Tab,
    /// A panel associated with a tab.
    TabPanel,
    /// Main content area.
    Main,
    /// Navigation area.
    Navigation,
    /// Complementary/sidebar content.
    Complementary,
    /// A heading element.
    Heading,
    /// An image or icon.
    Image,
    /// A status message.
    Status,
    /// A progress indicator.
    Progress,
    /// A toolbar containing actions.
    Toolbar,
    /// A tooltip providing additional info.
    Tooltip,
    /// A tree view structure.
    Tree,
    /// An item in a tree view.
    TreeItem,
    /// A grid or table.
    Grid,
    /// A row in a grid.
    Row,
    /// A cell in a grid.
    Cell,
}

impl Role {
    /// Returns the ARIA role name.
    pub fn aria_name(&self) -> &'static str {
        match self {
            Role::Button => "button",
            Role::Link => "link",
            Role::TextInput => "textbox",
            Role::Search => "searchbox",
            Role::Checkbox => "checkbox",
            Role::Radio => "radio",
            Role::List => "list",
            Role::ListItem => "listitem",
            Role::Menu => "menu",
            Role::MenuItem => "menuitem",
            Role::Dialog => "dialog",
            Role::Alert => "alert",
            Role::Tab => "tab",
            Role::TabPanel => "tabpanel",
            Role::Main => "main",
            Role::Navigation => "navigation",
            Role::Complementary => "complementary",
            Role::Heading => "heading",
            Role::Image => "img",
            Role::Status => "status",
            Role::Progress => "progressbar",
            Role::Toolbar => "toolbar",
            Role::Tooltip => "tooltip",
            Role::Tree => "tree",
            Role::TreeItem => "treeitem",
            Role::Grid => "grid",
            Role::Row => "row",
            Role::Cell => "gridcell",
        }
    }
}

/// Accessibility state for interactive elements.
#[derive(Debug, Clone, Default)]
pub struct AccessibleState {
    /// Whether the element is disabled.
    pub disabled: bool,
    /// Whether the element is selected.
    pub selected: bool,
    /// Whether the element is expanded (for collapsible elements).
    pub expanded: Option<bool>,
    /// Whether the element is checked (for checkboxes/toggles).
    pub checked: Option<bool>,
    /// Whether the element is pressed (for toggle buttons).
    pub pressed: Option<bool>,
    /// Current value (for range inputs).
    pub value: Option<i32>,
    /// Minimum value (for range inputs).
    pub value_min: Option<i32>,
    /// Maximum value (for range inputs).
    pub value_max: Option<i32>,
    /// Current position in a set (1-indexed).
    pub pos_in_set: Option<usize>,
    /// Size of the set.
    pub set_size: Option<usize>,
    /// Level in a hierarchy (for headings, tree items).
    pub level: Option<u32>,
}

impl AccessibleState {
    /// Creates a new accessible state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Sets checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Sets pressed state.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = Some(pressed);
        self
    }

    /// Sets position in set.
    pub fn position(mut self, pos: usize, size: usize) -> Self {
        self.pos_in_set = Some(pos);
        self.set_size = Some(size);
        self
    }

    /// Sets hierarchy level.
    pub fn level(mut self, level: u32) -> Self {
        self.level = Some(level);
        self
    }
}

/// Describes an accessible element.
#[derive(Debug, Clone)]
pub struct AccessibleElement {
    /// Semantic role of the element.
    pub role: Role,
    /// Accessible label (what screen readers announce).
    pub label: SharedString,
    /// Additional description.
    pub description: Option<SharedString>,
    /// Current state.
    pub state: AccessibleState,
    /// Keyboard shortcut hint.
    pub shortcut: Option<SharedString>,
}

impl AccessibleElement {
    /// Creates a new accessible element.
    pub fn new(role: Role, label: impl Into<SharedString>) -> Self {
        Self {
            role,
            label: label.into(),
            description: None,
            state: AccessibleState::default(),
            shortcut: None,
        }
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the state.
    pub fn state(mut self, state: AccessibleState) -> Self {
        self.state = state;
        self
    }

    /// Sets a keyboard shortcut hint.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// Focus management utilities.
pub mod focus {
    /// Focus trap modes for dialogs and modals.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FocusTrapMode {
        /// Focus cycles within the trapped region.
        Cycle,
        /// Focus stops at boundaries.
        Stop,
        /// Focus can escape with specific keys.
        Escapable,
    }

    /// Focus direction for navigation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FocusDirection {
        /// Move to next focusable element.
        Next,
        /// Move to previous focusable element.
        Previous,
        /// Move up in a 2D layout.
        Up,
        /// Move down in a 2D layout.
        Down,
        /// Move left in a 2D layout.
        Left,
        /// Move right in a 2D layout.
        Right,
        /// Move to first focusable element.
        First,
        /// Move to last focusable element.
        Last,
    }
}

/// Announcements for screen readers.
pub mod announcements {
    use super::*;

    /// Priority level for announcements.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AnnouncementPriority {
        /// Polite: waits for current speech to finish.
        Polite,
        /// Assertive: interrupts current speech.
        Assertive,
    }

    /// A queued announcement for the screen reader.
    #[derive(Debug, Clone)]
    pub struct Announcement {
        /// The message to announce.
        pub message: SharedString,
        /// Priority of the announcement.
        pub priority: AnnouncementPriority,
    }

    impl Announcement {
        /// Creates a polite announcement.
        pub fn polite(message: impl Into<SharedString>) -> Self {
            Self {
                message: message.into(),
                priority: AnnouncementPriority::Polite,
            }
        }

        /// Creates an assertive announcement.
        pub fn assertive(message: impl Into<SharedString>) -> Self {
            Self {
                message: message.into(),
                priority: AnnouncementPriority::Assertive,
            }
        }
    }

    /// Common announcement messages.
    pub fn email_received(sender: &str, subject: &str) -> Announcement {
        Announcement::assertive(format!("New email from {}: {}", sender, subject))
    }

    pub fn email_sent() -> Announcement {
        Announcement::polite("Email sent successfully")
    }

    pub fn email_archived() -> Announcement {
        Announcement::polite("Email archived")
    }

    pub fn email_deleted() -> Announcement {
        Announcement::polite("Email moved to trash")
    }

    pub fn search_results(count: usize) -> Announcement {
        if count == 0 {
            Announcement::polite("No results found")
        } else if count == 1 {
            Announcement::polite("1 result found")
        } else {
            Announcement::polite(format!("{} results found", count))
        }
    }

    pub fn loading_started() -> Announcement {
        Announcement::polite("Loading")
    }

    pub fn loading_complete() -> Announcement {
        Announcement::polite("Loading complete")
    }

    pub fn error(message: &str) -> Announcement {
        Announcement::assertive(format!("Error: {}", message))
    }
}

/// High contrast color adjustments.
pub mod high_contrast {
    use gpui::{rgb, Hsla};

    /// High contrast color palette.
    pub struct HighContrastColors {
        pub background: Hsla,
        pub foreground: Hsla,
        pub accent: Hsla,
        pub error: Hsla,
        pub success: Hsla,
        pub warning: Hsla,
        pub border: Hsla,
        pub focus_ring: Hsla,
    }

    impl Default for HighContrastColors {
        fn default() -> Self {
            Self {
                background: rgb(0x000000).into(),
                foreground: rgb(0xffffff).into(),
                accent: rgb(0x00ffff).into(),
                error: rgb(0xff0000).into(),
                success: rgb(0x00ff00).into(),
                warning: rgb(0xffff00).into(),
                border: rgb(0xffffff).into(),
                focus_ring: rgb(0x00ffff).into(),
            }
        }
    }

    impl HighContrastColors {
        /// Windows high contrast black theme.
        pub fn windows_black() -> Self {
            Self::default()
        }

        /// Windows high contrast white theme.
        pub fn windows_white() -> Self {
            Self {
                background: rgb(0xffffff).into(),
                foreground: rgb(0x000000).into(),
                accent: rgb(0x0000ff).into(),
                error: rgb(0xff0000).into(),
                success: rgb(0x008000).into(),
                warning: rgb(0x808000).into(),
                border: rgb(0x000000).into(),
                focus_ring: rgb(0x0000ff).into(),
            }
        }
    }
}

/// Animation preferences for reduced motion.
pub mod motion {
    use std::time::Duration;

    /// Returns animation duration respecting reduced motion preference.
    pub fn duration(preferred: Duration) -> Duration {
        if super::is_reduced_motion() {
            Duration::ZERO
        } else {
            preferred
        }
    }

    /// Returns whether animations should be skipped entirely.
    pub fn should_skip() -> bool {
        super::is_reduced_motion()
    }

    /// Default animation durations.
    pub mod defaults {
        use std::time::Duration;

        pub const INSTANT: Duration = Duration::from_millis(0);
        pub const FAST: Duration = Duration::from_millis(100);
        pub const NORMAL: Duration = Duration::from_millis(200);
        pub const SLOW: Duration = Duration::from_millis(300);
        pub const VERY_SLOW: Duration = Duration::from_millis(500);
    }
}

// Platform detection stubs
fn detect_high_contrast() -> bool {
    // Would check system preferences
    false
}

fn detect_reduced_motion() -> bool {
    // Would check system preferences
    false
}

fn detect_screen_reader() -> bool {
    // Would detect screen reader presence
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessibility_settings_default() {
        let settings = AccessibilitySettings::default();
        assert!(!settings.high_contrast);
        assert!(!settings.reduced_motion);
        assert!(!settings.screen_reader);
        assert_eq!(settings.min_touch_target, 44.0);
    }

    #[test]
    fn role_aria_names() {
        assert_eq!(Role::Button.aria_name(), "button");
        assert_eq!(Role::List.aria_name(), "list");
        assert_eq!(Role::Dialog.aria_name(), "dialog");
    }

    #[test]
    fn accessible_state_builder() {
        let state = AccessibleState::new()
            .disabled(true)
            .selected(true)
            .expanded(false)
            .position(2, 5);

        assert!(state.disabled);
        assert!(state.selected);
        assert_eq!(state.expanded, Some(false));
        assert_eq!(state.pos_in_set, Some(2));
        assert_eq!(state.set_size, Some(5));
    }

    #[test]
    fn accessible_element_builder() {
        let element = AccessibleElement::new(Role::Button, "Save")
            .description("Save the current document")
            .shortcut("Ctrl+S");

        assert_eq!(element.role, Role::Button);
        assert!(element.description.is_some());
        assert!(element.shortcut.is_some());
    }

    #[test]
    fn announcements_format() {
        let search = announcements::search_results(5);
        assert!(search.message.contains("5"));

        let error = announcements::error("Connection failed");
        assert!(error.message.contains("Connection failed"));
    }

    #[test]
    fn motion_respects_preference() {
        use std::time::Duration;

        // When reduced motion is off, return preferred duration
        REDUCED_MOTION_ENABLED.store(false, Ordering::SeqCst);
        let dur = motion::duration(Duration::from_millis(200));
        assert_eq!(dur, Duration::from_millis(200));

        // When reduced motion is on, return zero
        REDUCED_MOTION_ENABLED.store(true, Ordering::SeqCst);
        let dur = motion::duration(Duration::from_millis(200));
        assert_eq!(dur, Duration::ZERO);

        // Reset
        REDUCED_MOTION_ENABLED.store(false, Ordering::SeqCst);
    }

    #[test]
    fn high_contrast_colors() {
        let hc = high_contrast::HighContrastColors::default();
        // Black background, white foreground for maximum contrast
        assert_eq!(hc.background, gpui::rgb(0x000000).into());
        assert_eq!(hc.foreground, gpui::rgb(0xffffff).into());
    }
}
