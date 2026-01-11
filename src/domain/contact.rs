//! Contact domain types.
//!
//! Represents contacts extracted from email interactions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A contact extracted from email interactions.
///
/// Contacts are automatically created and updated based on email activity,
/// tracking frequency of communication and allowing VIP marking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Unique identifier for this contact.
    pub id: String,
    /// Email address (unique).
    pub email: String,
    /// Display name.
    pub name: Option<String>,
    /// Number of emails exchanged with this contact.
    pub frequency: u32,
    /// Date of last email interaction.
    pub last_contacted: Option<DateTime<Utc>>,
    /// Whether this contact is marked as VIP.
    pub is_vip: bool,
    /// User notes about this contact.
    pub notes: Option<String>,
}

impl Contact {
    /// Creates a new contact with just an email address.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.into(),
            name: None,
            frequency: 1,
            last_contacted: None,
            is_vip: false,
            notes: None,
        }
    }

    /// Creates a new contact with email and name.
    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.into(),
            name: Some(name.into()),
            frequency: 1,
            last_contacted: None,
            is_vip: false,
            notes: None,
        }
    }

    /// Returns the display name or email if no name is set.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.email)
    }

    /// Increments the contact frequency and updates last contacted time.
    pub fn record_interaction(&mut self) {
        self.frequency = self.frequency.saturating_add(1);
        self.last_contacted = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_new() {
        let contact = Contact::new("test@example.com");
        assert_eq!(contact.email, "test@example.com");
        assert!(contact.name.is_none());
        assert_eq!(contact.frequency, 1);
        assert!(!contact.is_vip);
    }

    #[test]
    fn contact_with_name() {
        let contact = Contact::with_name("test@example.com", "Test User");
        assert_eq!(contact.email, "test@example.com");
        assert_eq!(contact.name, Some("Test User".to_string()));
    }

    #[test]
    fn contact_display_name_with_name() {
        let contact = Contact::with_name("test@example.com", "Test User");
        assert_eq!(contact.display_name(), "Test User");
    }

    #[test]
    fn contact_display_name_without_name() {
        let contact = Contact::new("test@example.com");
        assert_eq!(contact.display_name(), "test@example.com");
    }

    #[test]
    fn contact_record_interaction() {
        let mut contact = Contact::new("test@example.com");
        assert_eq!(contact.frequency, 1);
        assert!(contact.last_contacted.is_none());

        contact.record_interaction();
        assert_eq!(contact.frequency, 2);
        assert!(contact.last_contacted.is_some());

        contact.record_interaction();
        assert_eq!(contact.frequency, 3);
    }

    #[test]
    fn contact_serialization() {
        let contact = Contact::with_name("test@example.com", "Test User");
        let json = serde_json::to_string(&contact).unwrap();
        let deserialized: Contact = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.email, "test@example.com");
        assert_eq!(deserialized.name, Some("Test User".to_string()));
    }
}
