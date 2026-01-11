//! Theme definitions for margin

use gpui::{rgb, Hsla};

/// Color palette for the application
#[derive(Clone, Debug)]
pub struct ThemeColors {
    // Backgrounds
    pub background: Hsla,
    pub surface: Hsla,
    pub surface_elevated: Hsla,

    // Text
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,

    // Borders
    pub border: Hsla,
    pub border_focused: Hsla,

    // Accents
    pub accent: Hsla,
    pub accent_hover: Hsla,

    // Status colors
    pub success: Hsla,
    pub warning: Hsla,
    pub error: Hsla,

    // Email-specific
    pub unread: Hsla,
    pub starred: Hsla,
}

impl ThemeColors {
    /// Dark theme colors
    pub fn dark() -> Self {
        Self {
            // Backgrounds
            background: rgb(0x1a1a1a).into(),
            surface: rgb(0x242424).into(),
            surface_elevated: rgb(0x2e2e2e).into(),

            // Text
            text_primary: rgb(0xffffff).into(),
            text_secondary: rgb(0xa0a0a0).into(),
            text_muted: rgb(0x666666).into(),

            // Borders
            border: rgb(0x3a3a3a).into(),
            border_focused: rgb(0x4a9eff).into(),

            // Accents
            accent: rgb(0x4a9eff).into(),
            accent_hover: rgb(0x6aafff).into(),

            // Status
            success: rgb(0x4caf50).into(),
            warning: rgb(0xffc107).into(),
            error: rgb(0xf44336).into(),

            // Email-specific
            unread: rgb(0x4a9eff).into(),
            starred: rgb(0xffc107).into(),
        }
    }

    /// Light theme colors
    pub fn light() -> Self {
        Self {
            // Backgrounds
            background: rgb(0xffffff).into(),
            surface: rgb(0xf5f5f5).into(),
            surface_elevated: rgb(0xffffff).into(),

            // Text
            text_primary: rgb(0x1a1a1a).into(),
            text_secondary: rgb(0x666666).into(),
            text_muted: rgb(0x999999).into(),

            // Borders
            border: rgb(0xe0e0e0).into(),
            border_focused: rgb(0x1a73e8).into(),

            // Accents
            accent: rgb(0x1a73e8).into(),
            accent_hover: rgb(0x4285f4).into(),

            // Status
            success: rgb(0x34a853).into(),
            warning: rgb(0xfbbc04).into(),
            error: rgb(0xea4335).into(),

            // Email-specific
            unread: rgb(0x1a73e8).into(),
            starred: rgb(0xfbbc04).into(),
        }
    }
}

/// Theme mode
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Application theme
#[derive(Clone, Debug)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ThemeColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            colors: ThemeColors::dark(),
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            colors: ThemeColors::light(),
        }
    }

    /// Toggle between light and dark
    pub fn toggle(&mut self) {
        match self.mode {
            ThemeMode::Dark => *self = Self::light(),
            ThemeMode::Light => *self = Self::dark(),
        }
    }
}
