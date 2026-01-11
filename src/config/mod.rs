//! Configuration and settings management.
//!
//! This module provides application settings types and persistence.
//! Settings are stored in the user's config directory as JSON.

mod settings;

pub use settings::{
    AiSettings, AppearanceSettings, ComposeSettings, Density, KeybindingSettings,
    NewEmailNotification, NotificationSettings, PrivacySettings, ProviderSettings, QuietHours,
    SearchSettings, Settings, SummarySettings, SyncSettings, Theme, Tone,
};
