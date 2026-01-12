//! Domain layer types for The Heap email client.
//!
//! This module contains the core domain types used throughout the application,
//! including email, thread, account, label, contact, and screener entities.

mod account;
mod contact;
mod email;
mod label;
mod screener;
mod thread;
mod types;

pub use account::{Account, ProviderConfig, ProviderType};
pub use contact::Contact;
pub use email::{Address, Attachment, Email};
pub use label::{system_labels, Label};
pub use screener::{
    RuleType, ScreenerAction, ScreenerEntry, ScreenerRule, ScreenerStatus, SenderAnalysis,
    SenderType,
};
pub use thread::{Thread, ThreadSummary};
pub use types::{AccountId, EmailId, LabelId, MessageId, ThreadId};
