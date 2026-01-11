//! Gmail API provider implementation.
//!
//! This module provides an [`EmailProvider`] implementation using the Gmail REST API.
//! It handles OAuth 2.0 authentication, fetching emails via the Gmail API, and
//! sending emails via the Gmail API.
//!
//! # Authentication
//!
//! Gmail uses OAuth 2.0 for authentication. Access tokens and refresh tokens are
//! stored in the system keychain, referenced by account ID. The provider handles
//! token refresh automatically when tokens expire.
//!
//! # API Usage
//!
//! This provider uses the Gmail API v1:
//! - `users.threads.list` for fetching thread summaries
//! - `users.threads.get` for fetching complete threads
//! - `users.history.list` for incremental sync
//! - `users.messages.send` for sending emails
//! - `users.labels.list` for fetching labels

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    Change, EmailProvider, OutgoingEmail, Pagination, PendingChange, ProviderError, Result,
};
use crate::domain::{AccountId, Label, ProviderType, Thread, ThreadSummary};

/// Gmail API provider.
///
/// Implements [`EmailProvider`] using the Gmail REST API with OAuth 2.0 authentication.
///
/// # Example
///
/// ```ignore
/// use margin::providers::email::{GmailProvider, EmailProvider, Pagination};
///
/// let mut provider = GmailProvider::new(account_id);
/// provider.authenticate().await?;
///
/// let threads = provider.fetch_threads("INBOX", Pagination::with_limit(50)).await?;
/// ```
pub struct GmailProvider {
    /// Account ID for keychain credential lookup.
    account_id: AccountId,
    /// HTTP client for API requests.
    #[allow(dead_code)]
    client: reqwest::Client,
    /// Current OAuth access token (refreshed as needed).
    #[allow(dead_code)]
    access_token: Option<String>,
    /// Whether the provider is authenticated.
    authenticated: bool,
}

impl GmailProvider {
    /// Creates a new Gmail provider for the specified account.
    ///
    /// The provider is not authenticated until [`authenticate`](Self::authenticate) is called.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID used to look up OAuth tokens in the keychain
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id,
            client: reqwest::Client::new(),
            access_token: None,
            authenticated: false,
        }
    }

    /// Returns whether the provider is currently authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Returns the account ID for this provider.
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }
}

#[async_trait]
impl EmailProvider for GmailProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Gmail
    }

    async fn authenticate(&mut self) -> Result<()> {
        // TODO: Implement OAuth token retrieval from keychain and refresh if needed
        // 1. Load refresh token from keychain
        // 2. Exchange refresh token for access token
        // 3. Store new access token
        self.authenticated = true;
        Ok(())
    }

    async fn fetch_threads(
        &self,
        _folder: &str,
        _pagination: Pagination,
    ) -> Result<Vec<ThreadSummary>> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.list
        // GET https://gmail.googleapis.com/gmail/v1/users/me/threads
        // Query params: labelIds, maxResults, pageToken
        Ok(vec![])
    }

    async fn fetch_thread(&self, _thread_id: &str) -> Result<Thread> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.get
        // GET https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}
        // Query params: format=full
        Err(ProviderError::NotFound("not implemented".to_string()))
    }

    async fn fetch_changes_since(&self, _since: &DateTime<Utc>) -> Result<Vec<Change>> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API history.list
        // GET https://gmail.googleapis.com/gmail/v1/users/me/history
        // Query params: startHistoryId, historyTypes
        Ok(vec![])
    }

    async fn send_email(&self, _email: &OutgoingEmail) -> Result<String> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API messages.send
        // POST https://gmail.googleapis.com/gmail/v1/users/me/messages/send
        // Body: raw (base64 encoded RFC 5322 message)
        Err(ProviderError::Internal("not implemented".to_string()))
    }

    async fn archive(&self, _thread_ids: &[String]) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.modify
        // POST https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}/modify
        // Body: { "removeLabelIds": ["INBOX"] }
        Ok(())
    }

    async fn trash(&self, _thread_ids: &[String]) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.trash
        // POST https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}/trash
        Ok(())
    }

    async fn star(&self, _thread_id: &str, _starred: bool) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.modify
        // POST https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}/modify
        // Body: { "addLabelIds": ["STARRED"] } or { "removeLabelIds": ["STARRED"] }
        Ok(())
    }

    async fn mark_read(&self, _thread_id: &str, _read: bool) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.modify
        // POST https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}/modify
        // Body: { "removeLabelIds": ["UNREAD"] } or { "addLabelIds": ["UNREAD"] }
        Ok(())
    }

    async fn apply_label(&self, _thread_id: &str, _label: &str) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API threads.modify
        // POST https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}/modify
        // Body: { "addLabelIds": [label] }
        Ok(())
    }

    async fn fetch_labels(&self) -> Result<Vec<Label>> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement Gmail API labels.list
        // GET https://gmail.googleapis.com/gmail/v1/users/me/labels
        Ok(vec![])
    }

    async fn push_change(&self, _change: &PendingChange) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Dispatch to appropriate Gmail API based on change type
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_provider_creation() {
        let provider = GmailProvider::new(AccountId::from("test-account"));
        assert_eq!(provider.account_id().0, "test-account");
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn gmail_provider_type() {
        let provider = GmailProvider::new(AccountId::from("test-account"));
        assert_eq!(provider.provider_type(), ProviderType::Gmail);
    }

    #[tokio::test]
    async fn gmail_provider_authenticate() {
        let mut provider = GmailProvider::new(AccountId::from("test-account"));
        assert!(!provider.is_authenticated());

        let result = provider.authenticate().await;
        assert!(result.is_ok());
        assert!(provider.is_authenticated());
    }

    #[tokio::test]
    async fn gmail_provider_requires_auth() {
        let provider = GmailProvider::new(AccountId::from("test-account"));

        let result = provider.fetch_threads("INBOX", Pagination::default()).await;
        assert!(matches!(result, Err(ProviderError::Authentication(_))));
    }

    #[tokio::test]
    async fn gmail_provider_fetch_threads_empty() {
        let mut provider = GmailProvider::new(AccountId::from("test-account"));
        provider.authenticate().await.unwrap();

        let result = provider
            .fetch_threads("INBOX", Pagination::with_limit(10))
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn gmail_provider_stub_operations() {
        let mut provider = GmailProvider::new(AccountId::from("test-account"));
        provider.authenticate().await.unwrap();

        // All stub operations should succeed (they're no-ops for now)
        assert!(provider.archive(&["thread-1".to_string()]).await.is_ok());
        assert!(provider.trash(&["thread-1".to_string()]).await.is_ok());
        assert!(provider.star("thread-1", true).await.is_ok());
        assert!(provider.mark_read("thread-1", true).await.is_ok());
        assert!(provider.apply_label("thread-1", "Work").await.is_ok());
        assert!(provider.fetch_labels().await.is_ok());
    }
}
