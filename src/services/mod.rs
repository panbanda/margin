//! Business services layer.
//!
//! This module contains the core services that orchestrate business logic,
//! coordinating between providers, storage, and domain types.
//!
//! # Architecture
//!
//! Services sit between the application layer and the infrastructure layer:
//!
//! ```text
//! Application Layer (UI, Actions, Events)
//!          |
//!          v
//!    Services Layer  <-- You are here
//!          |
//!          v
//! Infrastructure (Providers, Storage)
//! ```
//!
//! # Services Overview
//!
//! - [`EmailService`]: Orchestrates email operations across providers and storage
//! - [`AiService`]: Manages AI provider interactions for summarization, drafts, and search
//! - [`SyncService`]: Handles synchronization between remote providers and local storage

mod ai_service;
mod email_service;
mod sync_service;

pub use ai_service::{
    AiService, AiSettings, Category, DraftSuggestion, SearchResult, Summary, SummarySettings,
};
pub use email_service::{Draft, EmailService, Pagination, ViewType};
pub use sync_service::{SyncResult, SyncService, SyncSettings, SyncStatus};
