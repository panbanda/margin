//! IMAP/SMTP provider implementation.
//!
//! This module provides an [`EmailProvider`] implementation using standard IMAP
//! for fetching emails and SMTP for sending. This supports most email providers
//! that aren't Gmail (or Gmail via IMAP).
//!
//! # Authentication
//!
//! Credentials (username/password or OAuth tokens) are stored in the system keychain,
//! referenced by account ID. The provider handles connection management and
//! reconnection as needed.
//!
//! # Protocol Details
//!
//! - Uses IMAP4rev1 (RFC 3501) via `async-imap`
//! - Uses SMTP with STARTTLS or direct TLS via `lettre`
//! - Supports IDLE for push notifications (when available)

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    Change, EmailProvider, OutgoingEmail, Pagination, PendingChange, ProviderError, Result,
};
use crate::domain::{AccountId, Label, ProviderType, Thread, ThreadSummary};

/// IMAP/SMTP configuration.
#[derive(Debug, Clone)]
pub struct ImapConfig {
    /// IMAP server hostname.
    pub imap_host: String,
    /// IMAP server port (typically 993 for TLS, 143 for STARTTLS).
    pub imap_port: u16,
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port (typically 465 for TLS, 587 for STARTTLS).
    pub smtp_port: u16,
    /// Whether to use TLS (true) or STARTTLS (false).
    pub use_tls: bool,
}

impl ImapConfig {
    /// Creates a configuration for a typical TLS setup.
    pub fn tls(imap_host: impl Into<String>, smtp_host: impl Into<String>) -> Self {
        Self {
            imap_host: imap_host.into(),
            imap_port: 993,
            smtp_host: smtp_host.into(),
            smtp_port: 465,
            use_tls: true,
        }
    }

    /// Creates a configuration for a STARTTLS setup.
    pub fn starttls(imap_host: impl Into<String>, smtp_host: impl Into<String>) -> Self {
        Self {
            imap_host: imap_host.into(),
            imap_port: 143,
            smtp_host: smtp_host.into(),
            smtp_port: 587,
            use_tls: false,
        }
    }
}

/// IMAP/SMTP email provider.
///
/// Implements [`EmailProvider`] using standard IMAP for fetching and SMTP for sending.
///
/// # Example
///
/// ```ignore
/// use margin::providers::email::{ImapProvider, ImapConfig, EmailProvider, Pagination};
///
/// let config = ImapConfig::tls("imap.example.com", "smtp.example.com");
/// let mut provider = ImapProvider::new(account_id, config);
/// provider.authenticate().await?;
///
/// let threads = provider.fetch_threads("INBOX", Pagination::with_limit(50)).await?;
/// ```
pub struct ImapProvider {
    /// Account ID for keychain credential lookup.
    account_id: AccountId,
    /// Server configuration.
    config: ImapConfig,
    /// Whether the provider is authenticated and connected.
    authenticated: bool,
}

impl ImapProvider {
    /// Creates a new IMAP provider for the specified account.
    ///
    /// The provider is not authenticated until [`authenticate`](Self::authenticate) is called.
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID used to look up credentials in the keychain
    /// * `config` - Server configuration
    pub fn new(account_id: AccountId, config: ImapConfig) -> Self {
        Self {
            account_id,
            config,
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

    /// Returns the server configuration.
    pub fn config(&self) -> &ImapConfig {
        &self.config
    }
}

#[async_trait]
impl EmailProvider for ImapProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Imap
    }

    async fn authenticate(&mut self) -> Result<()> {
        // TODO: Implement IMAP connection and authentication
        // 1. Load credentials from keychain
        // 2. Connect to IMAP server (TLS or STARTTLS)
        // 3. Authenticate (LOGIN or AUTHENTICATE)
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

        // TODO: Implement IMAP FETCH
        // 1. SELECT folder
        // 2. SEARCH for messages
        // 3. FETCH envelope and flags
        // 4. Group by thread (References/In-Reply-To headers)
        Ok(vec![])
    }

    async fn fetch_thread(&self, _thread_id: &str) -> Result<Thread> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP FETCH for thread messages
        // 1. Parse thread_id to get message UIDs
        // 2. FETCH full messages (BODY[])
        // 3. Parse MIME structure
        Err(ProviderError::NotFound("not implemented".to_string()))
    }

    async fn fetch_changes_since(&self, _since: &DateTime<Utc>) -> Result<Vec<Change>> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement change detection
        // Options:
        // 1. CONDSTORE/QRESYNC extensions (if supported)
        // 2. Poll with SEARCH SINCE
        // 3. Compare UIDVALIDITY and UIDNEXT
        Ok(vec![])
    }

    async fn send_email(&self, _email: &OutgoingEmail) -> Result<String> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement SMTP sending via lettre
        // 1. Build RFC 5322 message from OutgoingEmail
        // 2. Connect to SMTP server
        // 3. Send message
        // 4. Optionally append to Sent folder via IMAP
        Err(ProviderError::Internal("not implemented".to_string()))
    }

    async fn archive(&self, _thread_ids: &[String]) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP MOVE or COPY+EXPUNGE
        // Move messages from INBOX to Archive folder
        Ok(())
    }

    async fn trash(&self, _thread_ids: &[String]) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP MOVE or COPY+EXPUNGE
        // Move messages to Trash folder
        Ok(())
    }

    async fn star(&self, _thread_id: &str, _starred: bool) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP STORE FLAGS
        // Add or remove \Flagged flag
        Ok(())
    }

    async fn mark_read(&self, _thread_id: &str, _read: bool) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP STORE FLAGS
        // Add or remove \Seen flag
        Ok(())
    }

    async fn apply_label(&self, _thread_id: &str, _label: &str) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP COPY
        // Copy message to label folder (IMAP doesn't have native labels)
        // Or use IMAP keywords if supported
        Ok(())
    }

    async fn fetch_labels(&self) -> Result<Vec<Label>> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Implement IMAP LIST
        // LIST "" "*" to get all folders
        // Map folders to labels
        Ok(vec![])
    }

    async fn push_change(&self, _change: &PendingChange) -> Result<()> {
        if !self.authenticated {
            return Err(ProviderError::Authentication(
                "not authenticated".to_string(),
            ));
        }

        // TODO: Dispatch to appropriate IMAP/SMTP operation based on change type
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ImapConfig {
        ImapConfig::tls("imap.example.com", "smtp.example.com")
    }

    #[test]
    fn imap_config_tls() {
        let config = ImapConfig::tls("imap.example.com", "smtp.example.com");
        assert_eq!(config.imap_host, "imap.example.com");
        assert_eq!(config.imap_port, 993);
        assert_eq!(config.smtp_host, "smtp.example.com");
        assert_eq!(config.smtp_port, 465);
        assert!(config.use_tls);
    }

    #[test]
    fn imap_config_starttls() {
        let config = ImapConfig::starttls("imap.example.com", "smtp.example.com");
        assert_eq!(config.imap_port, 143);
        assert_eq!(config.smtp_port, 587);
        assert!(!config.use_tls);
    }

    #[test]
    fn imap_provider_creation() {
        let provider = ImapProvider::new(AccountId::from("test-account"), test_config());
        assert_eq!(provider.account_id().0, "test-account");
        assert!(!provider.is_authenticated());
        assert_eq!(provider.config().imap_host, "imap.example.com");
    }

    #[test]
    fn imap_provider_type() {
        let provider = ImapProvider::new(AccountId::from("test-account"), test_config());
        assert_eq!(provider.provider_type(), ProviderType::Imap);
    }

    #[tokio::test]
    async fn imap_provider_authenticate() {
        let mut provider = ImapProvider::new(AccountId::from("test-account"), test_config());
        assert!(!provider.is_authenticated());

        let result = provider.authenticate().await;
        assert!(result.is_ok());
        assert!(provider.is_authenticated());
    }

    #[tokio::test]
    async fn imap_provider_requires_auth() {
        let provider = ImapProvider::new(AccountId::from("test-account"), test_config());

        let result = provider.fetch_threads("INBOX", Pagination::default()).await;
        assert!(matches!(result, Err(ProviderError::Authentication(_))));
    }

    #[tokio::test]
    async fn imap_provider_fetch_threads_empty() {
        let mut provider = ImapProvider::new(AccountId::from("test-account"), test_config());
        provider.authenticate().await.unwrap();

        let result = provider
            .fetch_threads("INBOX", Pagination::with_limit(10))
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn imap_provider_stub_operations() {
        let mut provider = ImapProvider::new(AccountId::from("test-account"), test_config());
        provider.authenticate().await.unwrap();

        // All stub operations should succeed (they're no-ops for now)
        assert!(provider.archive(&["thread-1".to_string()]).await.is_ok());
        assert!(provider.trash(&["thread-1".to_string()]).await.is_ok());
        assert!(provider.star("thread-1", true).await.is_ok());
        assert!(provider.mark_read("thread-1", true).await.is_ok());
        assert!(provider.apply_label("thread-1", "Work").await.is_ok());
        assert!(provider.fetch_labels().await.is_ok());
    }

    #[tokio::test]
    async fn imap_provider_fetch_changes_empty() {
        let mut provider = ImapProvider::new(AccountId::from("test-account"), test_config());
        provider.authenticate().await.unwrap();

        let result = provider.fetch_changes_since(&Utc::now()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
