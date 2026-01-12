//! UI components and views
//!
//! This module contains the gpui-based user interface for margin.
//! The UI is organized into:
//! - `theme`: Color schemes and styling
//! - `components`: Reusable UI primitives
//! - `views`: Full-screen application views
//! - `keybindings`: Keyboard shortcut management
//! - `accessibility`: Screen reader, high contrast, and motion preferences

pub mod accessibility;
pub mod components;
pub mod keybindings;
pub mod theme;
pub mod views;

pub use accessibility::{
    is_high_contrast, is_reduced_motion, is_screen_reader_active, AccessibilitySettings,
    AccessibleElement, AccessibleState, Role,
};
pub use keybindings::{
    Key, KeyBinding, KeyContext, KeyResult, KeybindingConfig, KeybindingManager, Keystroke,
    Modifiers,
};
pub use theme::{Theme, ThemeColors, ThemeMode};
pub use views::MainWindow;
