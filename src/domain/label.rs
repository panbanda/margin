//! Label domain types.
//!
//! Represents email labels (folders/tags) used for organization.

use serde::{Deserialize, Serialize};

use super::{AccountId, LabelId};

/// An email label (folder or tag).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Unique identifier for this label.
    pub id: LabelId,
    /// Account this label belongs to.
    pub account_id: AccountId,
    /// Display name of the label.
    pub name: String,
    /// Color for UI display (hex format, e.g., "#ff0000").
    pub color: Option<String>,
    /// Whether this is a system label (INBOX, SENT, etc.).
    pub is_system: bool,
    /// Provider-specific label ID for sync.
    pub provider_id: Option<String>,
}

/// Well-known system label IDs.
pub mod system_labels {
    use super::LabelId;

    /// Returns the inbox label ID.
    pub fn inbox() -> LabelId {
        LabelId::from("INBOX")
    }

    /// Returns the sent label ID.
    pub fn sent() -> LabelId {
        LabelId::from("SENT")
    }

    /// Returns the drafts label ID.
    pub fn drafts() -> LabelId {
        LabelId::from("DRAFTS")
    }

    /// Returns the trash label ID.
    pub fn trash() -> LabelId {
        LabelId::from("TRASH")
    }

    /// Returns the spam label ID.
    pub fn spam() -> LabelId {
        LabelId::from("SPAM")
    }

    /// Returns the starred label ID.
    pub fn starred() -> LabelId {
        LabelId::from("STARRED")
    }

    /// Returns the archive label ID.
    pub fn archive() -> LabelId {
        LabelId::from("ARCHIVE")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_serialization() {
        let label = Label {
            id: LabelId::from("label-1"),
            account_id: AccountId::from("account-1"),
            name: "Work".to_string(),
            color: Some("#0066cc".to_string()),
            is_system: false,
            provider_id: Some("Label_123".to_string()),
        };

        let json = serde_json::to_string(&label).unwrap();
        let deserialized: Label = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Work");
        assert_eq!(deserialized.color, Some("#0066cc".to_string()));
    }

    #[test]
    fn system_label_ids() {
        assert_eq!(system_labels::inbox().0, "INBOX");
        assert_eq!(system_labels::sent().0, "SENT");
        assert_eq!(system_labels::drafts().0, "DRAFTS");
        assert_eq!(system_labels::trash().0, "TRASH");
        assert_eq!(system_labels::spam().0, "SPAM");
        assert_eq!(system_labels::starred().0, "STARRED");
        assert_eq!(system_labels::archive().0, "ARCHIVE");
    }

    #[test]
    fn system_label_is_system_flag() {
        let inbox = Label {
            id: system_labels::inbox(),
            account_id: AccountId::from("account-1"),
            name: "Inbox".to_string(),
            color: None,
            is_system: true,
            provider_id: None,
        };

        assert!(inbox.is_system);
    }
}
