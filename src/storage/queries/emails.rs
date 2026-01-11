//! Email CRUD operations.
//!
//! Provides database operations for email entities.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};

use crate::domain::{AccountId, Address, Email, EmailId, LabelId, MessageId, ThreadId};
use crate::storage::database::{Database, Result};

/// Inserts a new email into the database.
pub async fn insert(db: &Database, email: &Email) -> Result<()> {
    let email = email.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        let references_json = serde_json::to_string(&email.references).unwrap_or_default();
        let to_json = serde_json::to_string(&email.to).unwrap_or_default();
        let cc_json = serde_json::to_string(&email.cc).unwrap_or_default();
        let bcc_json = serde_json::to_string(&email.bcc).unwrap_or_default();
        let labels_json = serde_json::to_string(&email.labels).unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO emails (
                id, account_id, thread_id, message_id, in_reply_to, references_json,
                from_address, from_name, to_addresses, cc_addresses, bcc_addresses,
                subject, body_text, body_html, snippet, date,
                is_read, is_starred, is_draft, labels, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21, ?22
            )
            "#,
            params![
                email.id.0,
                email.account_id.0,
                email.thread_id.0,
                email.message_id.0,
                email.in_reply_to.as_ref().map(|m| &m.0),
                references_json,
                email.from.email,
                email.from.name,
                to_json,
                cc_json,
                bcc_json,
                email.subject,
                email.body_text,
                email.body_html,
                email.snippet,
                email.date.to_rfc3339(),
                email.is_read as i32,
                email.is_starred as i32,
                email.is_draft as i32,
                labels_json,
                now,
                now,
            ],
        )?;

        Ok(())
    })
    .await
}

/// Retrieves an email by its ID.
pub async fn get_by_id(db: &Database, email_id: &EmailId) -> Result<Option<Email>> {
    let email_id = email_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, thread_id, message_id, in_reply_to, references_json,
                from_address, from_name, to_addresses, cc_addresses, bcc_addresses,
                subject, body_text, body_html, snippet, date,
                is_read, is_starred, is_draft, labels
            FROM emails
            WHERE id = ?1
            "#,
        )?;

        let result = stmt.query_row([&email_id.0], row_to_email).optional()?;
        Ok(result)
    })
    .await
}

/// Retrieves all emails in a thread.
pub async fn get_by_thread(db: &Database, thread_id: &ThreadId) -> Result<Vec<Email>> {
    let thread_id = thread_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, thread_id, message_id, in_reply_to, references_json,
                from_address, from_name, to_addresses, cc_addresses, bcc_addresses,
                subject, body_text, body_html, snippet, date,
                is_read, is_starred, is_draft, labels
            FROM emails
            WHERE thread_id = ?1
            ORDER BY date ASC
            "#,
        )?;

        let rows = stmt.query_map([&thread_id.0], row_to_email)?;
        let emails: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(emails?)
    })
    .await
}

/// Retrieves emails for an account, ordered by date descending.
pub async fn get_by_account(
    db: &Database,
    account_id: &AccountId,
    limit: u32,
    offset: u32,
) -> Result<Vec<Email>> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, thread_id, message_id, in_reply_to, references_json,
                from_address, from_name, to_addresses, cc_addresses, bcc_addresses,
                subject, body_text, body_html, snippet, date,
                is_read, is_starred, is_draft, labels
            FROM emails
            WHERE account_id = ?1
            ORDER BY date DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )?;

        let rows = stmt.query_map(params![account_id.0, limit, offset], row_to_email)?;
        let emails: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(emails?)
    })
    .await
}

/// Updates the read status of an email.
pub async fn set_read(db: &Database, email_id: &EmailId, is_read: bool) -> Result<()> {
    let email_id = email_id.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE emails SET is_read = ?1, updated_at = ?2 WHERE id = ?3",
            params![is_read as i32, now, email_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Updates the starred status of an email.
pub async fn set_starred(db: &Database, email_id: &EmailId, is_starred: bool) -> Result<()> {
    let email_id = email_id.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE emails SET is_starred = ?1, updated_at = ?2 WHERE id = ?3",
            params![is_starred as i32, now, email_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Deletes an email by its ID.
pub async fn delete(db: &Database, email_id: &EmailId) -> Result<()> {
    let email_id = email_id.clone();

    db.with_conn(move |conn| {
        conn.execute("DELETE FROM emails WHERE id = ?1", [&email_id.0])?;
        Ok(())
    })
    .await
}

/// Counts emails in a thread.
pub async fn count_in_thread(db: &Database, thread_id: &ThreadId) -> Result<u32> {
    let thread_id = thread_id.clone();

    db.with_conn(move |conn| {
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE thread_id = ?1",
            [&thread_id.0],
            |row| row.get(0),
        )?;
        Ok(count)
    })
    .await
}

/// Counts unread emails in a thread.
pub async fn count_unread_in_thread(db: &Database, thread_id: &ThreadId) -> Result<u32> {
    let thread_id = thread_id.clone();

    db.with_conn(move |conn| {
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM emails WHERE thread_id = ?1 AND is_read = 0",
            [&thread_id.0],
            |row| row.get(0),
        )?;
        Ok(count)
    })
    .await
}

fn row_to_email(row: &Row<'_>) -> std::result::Result<Email, rusqlite::Error> {
    let references_json: String = row.get(5)?;
    let to_json: String = row.get(8)?;
    let cc_json: String = row.get(9)?;
    let bcc_json: String = row.get(10)?;
    let labels_json: String = row.get(19)?;
    let date_str: String = row.get(15)?;

    let references: Vec<MessageId> = serde_json::from_str(&references_json).unwrap_or_default();
    let to: Vec<Address> = serde_json::from_str(&to_json).unwrap_or_default();
    let cc: Vec<Address> = serde_json::from_str(&cc_json).unwrap_or_default();
    let bcc: Vec<Address> = serde_json::from_str(&bcc_json).unwrap_or_default();
    let labels: Vec<LabelId> = serde_json::from_str(&labels_json).unwrap_or_default();

    let date = DateTime::parse_from_rfc3339(&date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let in_reply_to: Option<String> = row.get(4)?;
    let from_name: Option<String> = row.get(7)?;

    Ok(Email {
        id: EmailId(row.get(0)?),
        account_id: AccountId(row.get(1)?),
        thread_id: ThreadId(row.get(2)?),
        message_id: MessageId(row.get(3)?),
        in_reply_to: in_reply_to.map(MessageId),
        references,
        from: Address {
            email: row.get(6)?,
            name: from_name,
        },
        to,
        cc,
        bcc,
        subject: row.get(11)?,
        body_text: row.get(12)?,
        body_html: row.get(13)?,
        snippet: row.get(14)?,
        date,
        is_read: row.get::<_, i32>(16)? != 0,
        is_starred: row.get::<_, i32>(17)? != 0,
        is_draft: row.get::<_, i32>(18)? != 0,
        labels,
        attachments: vec![], // Loaded separately if needed
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_email() -> Email {
        Email {
            id: EmailId::from("email-1"),
            account_id: AccountId::from("account-1"),
            thread_id: ThreadId::from("thread-1"),
            message_id: MessageId::from("<msg-1@example.com>"),
            in_reply_to: None,
            references: vec![],
            from: Address::with_name("sender@example.com", "Sender"),
            to: vec![Address::new("recipient@example.com")],
            cc: vec![],
            bcc: vec![],
            subject: Some("Test Subject".to_string()),
            body_text: Some("Test body".to_string()),
            body_html: None,
            snippet: "Test body".to_string(),
            date: Utc::now(),
            is_read: false,
            is_starred: false,
            is_draft: false,
            labels: vec![LabelId::from("INBOX")],
            attachments: vec![],
        }
    }

    async fn setup_db_with_account() -> Database {
        let db = Database::open_in_memory().await.unwrap();

        db.with_conn(|conn| {
            conn.execute(
                r#"
                INSERT INTO accounts (id, email, provider_type, provider_config, created_at, updated_at)
                VALUES ('account-1', 'test@example.com', 'gmail', '{}', '2025-01-01', '2025-01-01')
                "#,
                [],
            )?;
            Ok(())
        })
        .await
        .unwrap();

        db
    }

    #[tokio::test]
    async fn insert_and_get_email() {
        let db = setup_db_with_account().await;
        let email = make_test_email();

        insert(&db, &email).await.unwrap();

        let retrieved = get_by_id(&db, &email.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, email.id);
        assert_eq!(retrieved.subject, email.subject);
        assert_eq!(retrieved.from.email, email.from.email);
    }

    #[tokio::test]
    async fn get_nonexistent_email_returns_none() {
        let db = Database::open_in_memory().await.unwrap();

        let result = get_by_id(&db, &EmailId::from("nonexistent")).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_emails_by_thread() {
        let db = setup_db_with_account().await;

        let mut email1 = make_test_email();
        email1.id = EmailId::from("email-1");

        let mut email2 = make_test_email();
        email2.id = EmailId::from("email-2");

        insert(&db, &email1).await.unwrap();
        insert(&db, &email2).await.unwrap();

        let emails = get_by_thread(&db, &ThreadId::from("thread-1"))
            .await
            .unwrap();
        assert_eq!(emails.len(), 2);
    }

    #[tokio::test]
    async fn set_read_status() {
        let db = setup_db_with_account().await;
        let email = make_test_email();

        insert(&db, &email).await.unwrap();
        assert!(!get_by_id(&db, &email.id).await.unwrap().unwrap().is_read);

        set_read(&db, &email.id, true).await.unwrap();
        assert!(get_by_id(&db, &email.id).await.unwrap().unwrap().is_read);
    }

    #[tokio::test]
    async fn set_starred_status() {
        let db = setup_db_with_account().await;
        let email = make_test_email();

        insert(&db, &email).await.unwrap();
        assert!(!get_by_id(&db, &email.id).await.unwrap().unwrap().is_starred);

        set_starred(&db, &email.id, true).await.unwrap();
        assert!(get_by_id(&db, &email.id).await.unwrap().unwrap().is_starred);
    }

    #[tokio::test]
    async fn delete_email() {
        let db = setup_db_with_account().await;
        let email = make_test_email();

        insert(&db, &email).await.unwrap();
        assert!(get_by_id(&db, &email.id).await.unwrap().is_some());

        delete(&db, &email.id).await.unwrap();
        assert!(get_by_id(&db, &email.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn count_emails_in_thread() {
        let db = setup_db_with_account().await;

        let mut email1 = make_test_email();
        email1.id = EmailId::from("email-1");

        let mut email2 = make_test_email();
        email2.id = EmailId::from("email-2");

        insert(&db, &email1).await.unwrap();
        insert(&db, &email2).await.unwrap();

        let count = count_in_thread(&db, &ThreadId::from("thread-1"))
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn count_unread_emails() {
        let db = setup_db_with_account().await;

        let mut email1 = make_test_email();
        email1.id = EmailId::from("email-1");
        email1.is_read = false;

        let mut email2 = make_test_email();
        email2.id = EmailId::from("email-2");
        email2.is_read = true;

        insert(&db, &email1).await.unwrap();
        insert(&db, &email2).await.unwrap();

        let count = count_unread_in_thread(&db, &ThreadId::from("thread-1"))
            .await
            .unwrap();
        assert_eq!(count, 1);
    }
}
