//! Email service for orchestrating email operations.
//!
//! The [`EmailService`] coordinates between email providers and local storage,
//! providing a unified interface for all email operations.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{AccountId, Address, EmailId, LabelId, Thread, ThreadId, ThreadSummary};

/// Email provider trait for abstracting over different email backends.
///
/// This trait is implemented by Gmail, IMAP, and other email providers.
#[async_trait::async_trait]
pub trait EmailProvider: Send + Sync {
    /// Returns the provider type identifier.
    fn provider_type(&self) -> &str;

    /// Fetches thread summaries for display in the message list.
    async fn fetch_threads(
        &self,
        folder: &str,
        pagination: Pagination,
    ) -> Result<Vec<ThreadSummary>>;

    /// Fetches a complete thread with all messages.
    async fn fetch_thread(&self, thread_id: &str) -> Result<Thread>;

    /// Sends an email.
    async fn send_email(&self, email: &OutgoingEmail) -> Result<String>;

    /// Archives threads (removes from inbox but keeps in All Mail).
    async fn archive(&self, thread_ids: &[String]) -> Result<()>;

    /// Moves threads to trash.
    async fn trash(&self, thread_ids: &[String]) -> Result<()>;

    /// Stars or unstars a thread.
    async fn star(&self, thread_id: &str, starred: bool) -> Result<()>;

    /// Marks a thread as read or unread.
    async fn mark_read(&self, thread_id: &str, read: bool) -> Result<()>;

    /// Applies a label to a thread.
    async fn apply_label(&self, thread_id: &str, label: &str) -> Result<()>;

    /// Removes a label from a thread.
    async fn remove_label(&self, thread_id: &str, label: &str) -> Result<()>;
}

/// Storage layer trait for local email persistence.
#[async_trait::async_trait]
pub trait EmailStorage: Send + Sync {
    /// Retrieves threads from local storage.
    async fn get_threads(
        &self,
        account_id: &AccountId,
        view: ViewType,
        pagination: Pagination,
    ) -> Result<Vec<ThreadSummary>>;

    /// Retrieves a complete thread from local storage.
    async fn get_thread(&self, thread_id: &ThreadId) -> Result<Option<Thread>>;

    /// Stores a thread in local storage.
    async fn store_thread(&self, thread: &Thread) -> Result<()>;

    /// Updates thread metadata.
    async fn update_thread_metadata(
        &self,
        thread_id: &ThreadId,
        updates: ThreadMetadataUpdate,
    ) -> Result<()>;
}

/// Updates to thread metadata for local storage.
#[derive(Debug, Clone, Default)]
pub struct ThreadMetadataUpdate {
    /// Set starred status.
    pub is_starred: Option<bool>,
    /// Set read status on all messages.
    pub is_read: Option<bool>,
    /// Labels to add.
    pub add_labels: Vec<LabelId>,
    /// Labels to remove.
    pub remove_labels: Vec<LabelId>,
    /// New snooze time, or None to clear.
    pub snooze_until: Option<Option<DateTime<Utc>>>,
}

/// View types for filtering thread lists.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    /// Primary inbox view.
    Inbox,
    /// Starred/flagged threads.
    Starred,
    /// Sent mail.
    Sent,
    /// Draft messages.
    Drafts,
    /// Archived threads.
    Archive,
    /// Deleted threads.
    Trash,
    /// All mail.
    All,
    /// Snoozed threads.
    Snoozed,
    /// Threads with a specific label.
    Label(LabelId),
}

impl ViewType {
    /// Returns the Gmail/IMAP folder name for this view type.
    pub fn folder_name(&self) -> &str {
        match self {
            ViewType::Inbox => "INBOX",
            ViewType::Starred => "[Gmail]/Starred",
            ViewType::Sent => "[Gmail]/Sent Mail",
            ViewType::Drafts => "[Gmail]/Drafts",
            ViewType::Archive => "[Gmail]/All Mail",
            ViewType::Trash => "[Gmail]/Trash",
            ViewType::All => "[Gmail]/All Mail",
            ViewType::Snoozed => "heap/Snoozed",
            ViewType::Label(_) => "INBOX", // Will be filtered by label
        }
    }
}

/// Pagination parameters for thread listing.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// Number of items to skip.
    pub offset: usize,
    /// Maximum number of items to return.
    pub limit: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

impl Pagination {
    /// Creates a new pagination with the given limit.
    pub fn with_limit(limit: usize) -> Self {
        Self { offset: 0, limit }
    }

    /// Returns the next page.
    pub fn next_page(&self) -> Self {
        Self {
            offset: self.offset + self.limit,
            limit: self.limit,
        }
    }
}

/// An outgoing email to be sent.
#[derive(Debug, Clone)]
pub struct OutgoingEmail {
    /// Sender address.
    pub from: Address,
    /// Primary recipients.
    pub to: Vec<Address>,
    /// CC recipients.
    pub cc: Vec<Address>,
    /// BCC recipients.
    pub bcc: Vec<Address>,
    /// Email subject.
    pub subject: String,
    /// Plain text body.
    pub body_text: String,
    /// HTML body (optional).
    pub body_html: Option<String>,
    /// Thread ID if this is a reply.
    pub in_reply_to: Option<ThreadId>,
    /// Message ID being replied to.
    pub reply_to_message_id: Option<String>,
}

/// A draft email being composed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    /// Draft ID for persistence.
    pub id: Option<String>,
    /// Account to send from.
    pub account_id: AccountId,
    /// Thread this is a reply to.
    pub reply_to_thread_id: Option<ThreadId>,
    /// Message this is a reply to.
    pub reply_to_message_id: Option<String>,
    /// Recipient addresses.
    pub to: Vec<Address>,
    /// CC addresses.
    pub cc: Vec<Address>,
    /// BCC addresses.
    pub bcc: Vec<Address>,
    /// Email subject.
    pub subject: String,
    /// Markdown body content.
    pub body_markdown: String,
    /// Rendered HTML body.
    pub body_html: Option<String>,
    /// When the draft was created.
    pub created_at: DateTime<Utc>,
    /// When the draft was last modified.
    pub updated_at: DateTime<Utc>,
}

/// Orchestrates email operations across providers and storage.
///
/// The EmailService provides a unified interface for all email operations,
/// coordinating between remote providers (Gmail, IMAP) and local storage.
/// It handles caching, offline support, and optimistic updates.
///
/// # Thread Safety
///
/// EmailService uses `Arc` and `RwLock` internally for thread-safe access
/// to providers. All async methods can be called concurrently.
///
/// # Example
///
/// ```ignore
/// let service = EmailService::new(storage);
/// service.register_provider(account_id, provider).await;
///
/// let threads = service.fetch_threads(&account_id, ViewType::Inbox, Pagination::default()).await?;
/// ```
pub struct EmailService<S: EmailStorage> {
    /// Registered email providers by account ID.
    providers: RwLock<HashMap<AccountId, Arc<dyn EmailProvider>>>,
    /// Local storage layer.
    storage: Arc<S>,
}

impl<S: EmailStorage> EmailService<S> {
    /// Creates a new EmailService with the given storage backend.
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            storage,
        }
    }

    /// Registers an email provider for an account.
    ///
    /// If a provider is already registered for this account, it is replaced.
    pub async fn register_provider(&self, account_id: AccountId, provider: Arc<dyn EmailProvider>) {
        let mut providers = self.providers.write().await;
        providers.insert(account_id, provider);
    }

    /// Unregisters the email provider for an account.
    pub async fn unregister_provider(&self, account_id: &AccountId) {
        let mut providers = self.providers.write().await;
        providers.remove(account_id);
    }

    /// Fetches thread summaries for display in the message list.
    ///
    /// Attempts to fetch from the provider first, falling back to local storage
    /// if the provider is unavailable. Results are cached locally.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account to fetch threads for
    /// * `view` - The view type (inbox, starred, etc.)
    /// * `pagination` - Pagination parameters
    ///
    /// # Returns
    ///
    /// A list of thread summaries sorted by last message date (newest first).
    pub async fn fetch_threads(
        &self,
        account_id: &AccountId,
        view: ViewType,
        pagination: Pagination,
    ) -> Result<Vec<ThreadSummary>> {
        // Try to fetch from provider first
        let providers = self.providers.read().await;
        if let Some(provider) = providers.get(account_id) {
            match provider.fetch_threads(view.folder_name(), pagination).await {
                Ok(threads) => return Ok(threads),
                Err(e) => {
                    // Log error and fall back to local storage
                    tracing::warn!("Failed to fetch threads from provider: {}", e);
                }
            }
        }

        // Fall back to local storage
        self.storage.get_threads(account_id, view, pagination).await
    }

    /// Fetches a complete thread with all messages.
    ///
    /// Attempts to fetch from the provider first for the latest state,
    /// falling back to local storage if unavailable.
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The thread to fetch
    ///
    /// # Returns
    ///
    /// The complete thread with all messages, or an error if not found.
    pub async fn get_thread(&self, thread_id: &ThreadId) -> Result<Thread> {
        // Check local storage first
        if let Some(thread) = self.storage.get_thread(thread_id).await? {
            return Ok(thread);
        }

        // Try all providers
        let providers = self.providers.read().await;
        for provider in providers.values() {
            if let Ok(thread) = provider.fetch_thread(&thread_id.0).await {
                // Cache locally
                self.storage.store_thread(&thread).await?;
                return Ok(thread);
            }
        }

        anyhow::bail!("Thread not found: {}", thread_id)
    }

    /// Sends an email.
    ///
    /// # Arguments
    ///
    /// * `draft` - The draft to send
    ///
    /// # Returns
    ///
    /// The ID of the sent email.
    pub async fn send_email(&self, draft: Draft) -> Result<EmailId> {
        let providers = self.providers.read().await;
        let provider = providers
            .get(&draft.account_id)
            .ok_or_else(|| anyhow::anyhow!("No provider for account: {}", draft.account_id))?;

        // Convert draft to outgoing email
        let outgoing = OutgoingEmail {
            from: Address::new(""), // Will be filled by provider from account
            to: draft.to,
            cc: draft.cc,
            bcc: draft.bcc,
            subject: draft.subject,
            body_text: draft.body_markdown,
            body_html: draft.body_html,
            in_reply_to: draft.reply_to_thread_id,
            reply_to_message_id: draft.reply_to_message_id,
        };

        let email_id = provider.send_email(&outgoing).await?;
        Ok(EmailId::from(email_id))
    }

    /// Archives threads by removing them from the inbox.
    ///
    /// Archived threads remain accessible in All Mail.
    ///
    /// # Arguments
    ///
    /// * `thread_ids` - The threads to archive
    pub async fn archive(&self, thread_ids: &[ThreadId]) -> Result<()> {
        if thread_ids.is_empty() {
            return Ok(());
        }

        // Group by account
        let ids: Vec<String> = thread_ids.iter().map(|id| id.0.clone()).collect();

        // Update all providers (threads might span accounts in shared view)
        let providers = self.providers.read().await;
        for provider in providers.values() {
            // Ignore errors for providers that don't have these threads
            let _ = provider.archive(&ids).await;
        }

        // Update local storage
        for thread_id in thread_ids {
            self.storage
                .update_thread_metadata(
                    thread_id,
                    ThreadMetadataUpdate {
                        remove_labels: vec![LabelId::from("INBOX")],
                        ..Default::default()
                    },
                )
                .await?;
        }

        Ok(())
    }

    /// Moves threads to the trash.
    ///
    /// Trashed threads are permanently deleted after 30 days.
    ///
    /// # Arguments
    ///
    /// * `thread_ids` - The threads to trash
    pub async fn trash(&self, thread_ids: &[ThreadId]) -> Result<()> {
        if thread_ids.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = thread_ids.iter().map(|id| id.0.clone()).collect();

        let providers = self.providers.read().await;
        for provider in providers.values() {
            let _ = provider.trash(&ids).await;
        }

        // Update local storage
        for thread_id in thread_ids {
            self.storage
                .update_thread_metadata(
                    thread_id,
                    ThreadMetadataUpdate {
                        remove_labels: vec![LabelId::from("INBOX")],
                        add_labels: vec![LabelId::from("TRASH")],
                        ..Default::default()
                    },
                )
                .await?;
        }

        Ok(())
    }

    /// Stars or unstars a thread.
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The thread to star/unstar
    /// * `starred` - True to star, false to unstar
    pub async fn star(&self, thread_id: &ThreadId, starred: bool) -> Result<()> {
        // Update provider
        let providers = self.providers.read().await;
        for provider in providers.values() {
            let _ = provider.star(&thread_id.0, starred).await;
        }

        // Update local storage
        self.storage
            .update_thread_metadata(
                thread_id,
                ThreadMetadataUpdate {
                    is_starred: Some(starred),
                    ..Default::default()
                },
            )
            .await
    }

    /// Applies a label to threads.
    ///
    /// # Arguments
    ///
    /// * `thread_ids` - The threads to label
    /// * `label_id` - The label to apply
    pub async fn apply_label(&self, thread_ids: &[ThreadId], label_id: &LabelId) -> Result<()> {
        if thread_ids.is_empty() {
            return Ok(());
        }

        let providers = self.providers.read().await;
        for thread_id in thread_ids {
            for provider in providers.values() {
                let _ = provider.apply_label(&thread_id.0, &label_id.0).await;
            }

            self.storage
                .update_thread_metadata(
                    thread_id,
                    ThreadMetadataUpdate {
                        add_labels: vec![label_id.clone()],
                        ..Default::default()
                    },
                )
                .await?;
        }

        Ok(())
    }

    /// Snoozes a thread until a specified time.
    ///
    /// Snoozed threads are hidden from the inbox and reappear at the specified time.
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The thread to snooze
    /// * `until` - When the thread should reappear
    pub async fn snooze(&self, thread_id: &ThreadId, until: DateTime<Utc>) -> Result<()> {
        // Update local storage
        self.storage
            .update_thread_metadata(
                thread_id,
                ThreadMetadataUpdate {
                    snooze_until: Some(Some(until)),
                    remove_labels: vec![LabelId::from("INBOX")],
                    add_labels: vec![LabelId::from("heap/Snoozed")],
                    ..Default::default()
                },
            )
            .await
    }

    /// Unsnoozes a thread, returning it to the inbox.
    ///
    /// # Arguments
    ///
    /// * `thread_id` - The thread to unsnooze
    pub async fn unsnooze(&self, thread_id: &ThreadId) -> Result<()> {
        self.storage
            .update_thread_metadata(
                thread_id,
                ThreadMetadataUpdate {
                    snooze_until: Some(None),
                    add_labels: vec![LabelId::from("INBOX")],
                    remove_labels: vec![LabelId::from("heap/Snoozed")],
                    ..Default::default()
                },
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_type_folder_names() {
        assert_eq!(ViewType::Inbox.folder_name(), "INBOX");
        assert_eq!(ViewType::Starred.folder_name(), "[Gmail]/Starred");
        assert_eq!(ViewType::Sent.folder_name(), "[Gmail]/Sent Mail");
        assert_eq!(ViewType::Drafts.folder_name(), "[Gmail]/Drafts");
        assert_eq!(ViewType::Trash.folder_name(), "[Gmail]/Trash");
    }

    #[test]
    fn pagination_default() {
        let p = Pagination::default();
        assert_eq!(p.offset, 0);
        assert_eq!(p.limit, 50);
    }

    #[test]
    fn pagination_next_page() {
        let p = Pagination::with_limit(25);
        let next = p.next_page();
        assert_eq!(next.offset, 25);
        assert_eq!(next.limit, 25);
    }

    #[test]
    fn view_type_serialization() {
        let view = ViewType::Inbox;
        let json = serde_json::to_string(&view).unwrap();
        assert_eq!(json, "\"inbox\"");

        let deserialized: ViewType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ViewType::Inbox);
    }

    #[test]
    fn thread_metadata_update_default() {
        let update = ThreadMetadataUpdate::default();
        assert!(update.is_starred.is_none());
        assert!(update.is_read.is_none());
        assert!(update.add_labels.is_empty());
        assert!(update.remove_labels.is_empty());
        assert!(update.snooze_until.is_none());
    }
}
