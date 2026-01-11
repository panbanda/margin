//! Email provider implementations.
//!
//! This module contains the [`EmailProvider`] trait and implementations for
//! different email backends:
//!
//! - [`GmailProvider`] - Gmail API with OAuth 2.0
//! - [`ImapProvider`] - Standard IMAP/SMTP
//!
//! # Architecture
//!
//! The email provider abstraction allows the application to work with different
//! email services through a common interface. Each provider handles:
//!
//! - Authentication (OAuth, username/password)
//! - Fetching emails and threads
//! - Sending emails
//! - Syncing changes bidirectionally
//!
//! # Example
//!
//! ```ignore
//! use margin::providers::email::{EmailProvider, GmailProvider, ImapProvider, Pagination};
//! use margin::domain::ProviderType;
//!
//! async fn list_inbox(provider: &dyn EmailProvider) {
//!     let threads = provider
//!         .fetch_threads("INBOX", Pagination::with_limit(50))
//!         .await
//!         .expect("failed to fetch threads");
//!
//!     for thread in threads {
//!         println!("{}: {}", thread.from.display(), thread.subject.unwrap_or_default());
//!     }
//! }
//! ```

mod gmail;
mod imap;
mod traits;

pub use gmail::GmailProvider;
pub use imap::{ImapConfig, ImapProvider};
pub use traits::{
    Change, EmailProvider, EmailUpdate, NewEmailData, OutgoingAttachment, OutgoingEmail,
    Pagination, PendingChange, PendingChangeType, ProviderError, Result,
};
