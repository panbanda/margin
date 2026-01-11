//! Core identifier types for domain entities.
//!
//! These newtype wrappers provide type safety for entity identifiers,
//! preventing accidental mixing of different ID types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for an email account.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub String);

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AccountId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AccountId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Unique identifier for an email thread (conversation).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreadId(pub String);

impl fmt::Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ThreadId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ThreadId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Unique identifier for an individual email.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailId(pub String);

impl fmt::Display for EmailId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EmailId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EmailId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// RFC 5322 Message-ID header value.
///
/// This is the unique identifier assigned by the originating mail system,
/// used for threading via In-Reply-To and References headers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MessageId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MessageId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Unique identifier for a label (folder/tag).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LabelId(pub String);

impl fmt::Display for LabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for LabelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for LabelId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_id_display() {
        let id = AccountId("test-account".to_string());
        assert_eq!(id.to_string(), "test-account");
    }

    #[test]
    fn thread_id_equality() {
        let id1 = ThreadId::from("thread-1");
        let id2 = ThreadId::from("thread-1".to_string());
        assert_eq!(id1, id2);
    }

    #[test]
    fn email_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(EmailId::from("email-1"));
        assert!(set.contains(&EmailId::from("email-1")));
    }

    #[test]
    fn message_id_from_str() {
        let id: MessageId = "<unique@example.com>".into();
        assert_eq!(id.0, "<unique@example.com>");
    }

    #[test]
    fn label_id_clone() {
        let id = LabelId::from("inbox");
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }
}
