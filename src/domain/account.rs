//! Account domain types.
//!
//! Represents email accounts and their provider-specific configurations.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::AccountId;

/// An email account configured in the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier for this account.
    pub id: AccountId,
    /// Email address for this account.
    pub email: String,
    /// Display name shown in the UI.
    pub display_name: Option<String>,
    /// Type of email provider.
    pub provider_type: ProviderType,
    /// Provider-specific configuration.
    pub provider_config: ProviderConfig,
    /// Whether automatic sync is enabled.
    pub sync_enabled: bool,
    /// Interval between sync operations.
    #[serde(with = "duration_serde")]
    pub sync_interval: Duration,
    /// Email signature for this account.
    pub signature: Option<String>,
}

/// Type of email provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Gmail API provider.
    Gmail,
    /// Standard IMAP/SMTP provider.
    Imap,
}

/// Provider-specific configuration.
///
/// OAuth tokens and passwords are stored in the system keychain,
/// not in this configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProviderConfig {
    /// Gmail API configuration.
    Gmail {
        // OAuth tokens stored in keychain, referenced by account ID.
    },
    /// IMAP/SMTP configuration.
    Imap {
        /// IMAP server hostname.
        imap_host: String,
        /// IMAP server port.
        imap_port: u16,
        /// SMTP server hostname.
        smtp_host: String,
        /// SMTP server port.
        smtp_port: u16,
        /// Whether to use TLS.
        use_tls: bool,
    },
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_serialization() {
        let account = Account {
            id: AccountId::from("test-id"),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            provider_type: ProviderType::Gmail,
            provider_config: ProviderConfig::Gmail {},
            sync_enabled: true,
            sync_interval: Duration::from_secs(300),
            signature: None,
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: Account = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.email, "test@example.com");
        assert_eq!(deserialized.sync_interval, Duration::from_secs(300));
    }

    #[test]
    fn imap_config_serialization() {
        let config = ProviderConfig::Imap {
            imap_host: "imap.example.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            use_tls: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("imap.example.com"));

        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        if let ProviderConfig::Imap { imap_port, .. } = deserialized {
            assert_eq!(imap_port, 993);
        } else {
            panic!("Expected Imap config");
        }
    }

    #[test]
    fn provider_type_equality() {
        assert_eq!(ProviderType::Gmail, ProviderType::Gmail);
        assert_ne!(ProviderType::Gmail, ProviderType::Imap);
    }
}
