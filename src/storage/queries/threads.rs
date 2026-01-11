//! Thread query operations.
//!
//! Provides database operations for thread entities.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};

use crate::domain::{AccountId, Address, LabelId, ThreadId, ThreadSummary};
use crate::storage::database::{Database, Result};

/// Inserts or updates a thread in the database.
pub async fn upsert(db: &Database, summary: &ThreadSummary) -> Result<()> {
    let summary = summary.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        let participant_emails = serde_json::to_string(&[&summary.from.email]).unwrap_or_default();
        let participant_names = serde_json::to_string(&[&summary.from.name]).unwrap_or_default();
        let labels_json = serde_json::to_string(&summary.labels).unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO threads (
                id, account_id, subject, snippet, participant_emails, participant_names,
                last_message_date, message_count, unread_count, is_starred, labels,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
            )
            ON CONFLICT(id) DO UPDATE SET
                subject = excluded.subject,
                snippet = excluded.snippet,
                participant_emails = excluded.participant_emails,
                participant_names = excluded.participant_names,
                last_message_date = excluded.last_message_date,
                message_count = excluded.message_count,
                unread_count = excluded.unread_count,
                is_starred = excluded.is_starred,
                labels = excluded.labels,
                updated_at = excluded.updated_at
            "#,
            params![
                summary.id.0,
                summary.account_id.0,
                summary.subject,
                summary.snippet,
                participant_emails,
                participant_names,
                summary.last_message_date.to_rfc3339(),
                summary.message_count,
                summary.unread_count,
                summary.is_starred as i32,
                labels_json,
                now,
                now,
            ],
        )?;

        Ok(())
    })
    .await
}

/// Retrieves a thread summary by its ID.
pub async fn get_by_id(db: &Database, thread_id: &ThreadId) -> Result<Option<ThreadSummary>> {
    let thread_id = thread_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, subject, snippet, participant_emails, participant_names,
                last_message_date, message_count, unread_count, is_starred, labels
            FROM threads
            WHERE id = ?1
            "#,
        )?;

        let result = stmt.query_row([&thread_id.0], row_to_summary).optional()?;
        Ok(result)
    })
    .await
}

/// Retrieves thread summaries for an account, ordered by date descending.
pub async fn get_by_account(
    db: &Database,
    account_id: &AccountId,
    limit: u32,
    offset: u32,
) -> Result<Vec<ThreadSummary>> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, subject, snippet, participant_emails, participant_names,
                last_message_date, message_count, unread_count, is_starred, labels
            FROM threads
            WHERE account_id = ?1
            ORDER BY last_message_date DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )?;

        let rows = stmt.query_map(params![account_id.0, limit, offset], row_to_summary)?;
        let threads: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(threads?)
    })
    .await
}

/// Retrieves unread thread summaries for an account.
pub async fn get_unread(
    db: &Database,
    account_id: &AccountId,
    limit: u32,
) -> Result<Vec<ThreadSummary>> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, subject, snippet, participant_emails, participant_names,
                last_message_date, message_count, unread_count, is_starred, labels
            FROM threads
            WHERE account_id = ?1 AND unread_count > 0
            ORDER BY last_message_date DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![account_id.0, limit], row_to_summary)?;
        let threads: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(threads?)
    })
    .await
}

/// Retrieves starred thread summaries for an account.
pub async fn get_starred(
    db: &Database,
    account_id: &AccountId,
    limit: u32,
) -> Result<Vec<ThreadSummary>> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, subject, snippet, participant_emails, participant_names,
                last_message_date, message_count, unread_count, is_starred, labels
            FROM threads
            WHERE account_id = ?1 AND is_starred = 1
            ORDER BY last_message_date DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![account_id.0, limit], row_to_summary)?;
        let threads: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(threads?)
    })
    .await
}

/// Retrieves threads with a specific label.
pub async fn get_by_label(
    db: &Database,
    account_id: &AccountId,
    label_id: &LabelId,
    limit: u32,
) -> Result<Vec<ThreadSummary>> {
    let account_id = account_id.clone();
    let label_pattern = format!("%\"{}\"%%", label_id.0);

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, account_id, subject, snippet, participant_emails, participant_names,
                last_message_date, message_count, unread_count, is_starred, labels
            FROM threads
            WHERE account_id = ?1 AND labels LIKE ?2
            ORDER BY last_message_date DESC
            LIMIT ?3
            "#,
        )?;

        let rows = stmt.query_map(params![account_id.0, label_pattern, limit], row_to_summary)?;
        let threads: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(threads?)
    })
    .await
}

/// Updates the starred status of a thread.
pub async fn set_starred(db: &Database, thread_id: &ThreadId, is_starred: bool) -> Result<()> {
    let thread_id = thread_id.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE threads SET is_starred = ?1, updated_at = ?2 WHERE id = ?3",
            params![is_starred as i32, now, thread_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Updates the unread count of a thread.
pub async fn set_unread_count(db: &Database, thread_id: &ThreadId, count: u32) -> Result<()> {
    let thread_id = thread_id.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE threads SET unread_count = ?1, updated_at = ?2 WHERE id = ?3",
            params![count, now, thread_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Deletes a thread and all its emails.
pub async fn delete(db: &Database, thread_id: &ThreadId) -> Result<()> {
    let thread_id = thread_id.clone();

    db.transaction(move |tx| {
        tx.execute("DELETE FROM emails WHERE thread_id = ?1", [&thread_id.0])?;
        tx.execute("DELETE FROM threads WHERE id = ?1", [&thread_id.0])?;
        Ok(())
    })
    .await
}

/// Counts total threads for an account.
pub async fn count_by_account(db: &Database, account_id: &AccountId) -> Result<u32> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM threads WHERE account_id = ?1",
            [&account_id.0],
            |row| row.get(0),
        )?;
        Ok(count)
    })
    .await
}

/// Counts unread threads for an account.
pub async fn count_unread(db: &Database, account_id: &AccountId) -> Result<u32> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM threads WHERE account_id = ?1 AND unread_count > 0",
            [&account_id.0],
            |row| row.get(0),
        )?;
        Ok(count)
    })
    .await
}

fn row_to_summary(row: &Row<'_>) -> std::result::Result<ThreadSummary, rusqlite::Error> {
    let participant_emails_json: String = row.get(4)?;
    let participant_names_json: String = row.get(5)?;
    let labels_json: String = row.get(10)?;
    let date_str: String = row.get(6)?;

    let participant_emails: Vec<String> =
        serde_json::from_str(&participant_emails_json).unwrap_or_default();
    let participant_names: Vec<Option<String>> =
        serde_json::from_str(&participant_names_json).unwrap_or_default();
    let labels: Vec<LabelId> = serde_json::from_str(&labels_json).unwrap_or_default();

    let date = DateTime::parse_from_rfc3339(&date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let from = if let Some(email) = participant_emails.first() {
        let name = participant_names.first().and_then(|n| n.clone());
        Address {
            email: email.clone(),
            name,
        }
    } else {
        Address::new("unknown@unknown.com")
    };

    Ok(ThreadSummary {
        id: ThreadId(row.get(0)?),
        account_id: AccountId(row.get(1)?),
        subject: row.get(2)?,
        snippet: row.get(3)?,
        from,
        last_message_date: date,
        message_count: row.get(7)?,
        unread_count: row.get(8)?,
        is_starred: row.get::<_, i32>(9)? != 0,
        labels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_summary() -> ThreadSummary {
        ThreadSummary {
            id: ThreadId::from("thread-1"),
            account_id: AccountId::from("account-1"),
            subject: Some("Test Subject".to_string()),
            snippet: "Test snippet...".to_string(),
            from: Address::with_name("sender@example.com", "Sender"),
            last_message_date: Utc::now(),
            message_count: 3,
            unread_count: 1,
            is_starred: false,
            labels: vec![LabelId::from("INBOX")],
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
    async fn upsert_and_get_thread() {
        let db = setup_db_with_account().await;
        let summary = make_test_summary();

        upsert(&db, &summary).await.unwrap();

        let retrieved = get_by_id(&db, &summary.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, summary.id);
        assert_eq!(retrieved.subject, summary.subject);
        assert_eq!(retrieved.message_count, 3);
    }

    #[tokio::test]
    async fn upsert_updates_existing() {
        let db = setup_db_with_account().await;
        let mut summary = make_test_summary();

        upsert(&db, &summary).await.unwrap();

        summary.unread_count = 5;
        summary.snippet = "Updated snippet".to_string();
        upsert(&db, &summary).await.unwrap();

        let retrieved = get_by_id(&db, &summary.id).await.unwrap().unwrap();
        assert_eq!(retrieved.unread_count, 5);
        assert_eq!(retrieved.snippet, "Updated snippet");
    }

    #[tokio::test]
    async fn get_nonexistent_thread_returns_none() {
        let db = Database::open_in_memory().await.unwrap();

        let result = get_by_id(&db, &ThreadId::from("nonexistent"))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_threads_by_account() {
        let db = setup_db_with_account().await;

        let mut summary1 = make_test_summary();
        summary1.id = ThreadId::from("thread-1");

        let mut summary2 = make_test_summary();
        summary2.id = ThreadId::from("thread-2");

        upsert(&db, &summary1).await.unwrap();
        upsert(&db, &summary2).await.unwrap();

        let threads = get_by_account(&db, &AccountId::from("account-1"), 10, 0)
            .await
            .unwrap();
        assert_eq!(threads.len(), 2);
    }

    #[tokio::test]
    async fn get_unread_threads() {
        let db = setup_db_with_account().await;

        let mut summary1 = make_test_summary();
        summary1.id = ThreadId::from("thread-1");
        summary1.unread_count = 1;

        let mut summary2 = make_test_summary();
        summary2.id = ThreadId::from("thread-2");
        summary2.unread_count = 0;

        upsert(&db, &summary1).await.unwrap();
        upsert(&db, &summary2).await.unwrap();

        let threads = get_unread(&db, &AccountId::from("account-1"), 10)
            .await
            .unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id, ThreadId::from("thread-1"));
    }

    #[tokio::test]
    async fn get_starred_threads() {
        let db = setup_db_with_account().await;

        let mut summary1 = make_test_summary();
        summary1.id = ThreadId::from("thread-1");
        summary1.is_starred = true;

        let mut summary2 = make_test_summary();
        summary2.id = ThreadId::from("thread-2");
        summary2.is_starred = false;

        upsert(&db, &summary1).await.unwrap();
        upsert(&db, &summary2).await.unwrap();

        let threads = get_starred(&db, &AccountId::from("account-1"), 10)
            .await
            .unwrap();
        assert_eq!(threads.len(), 1);
        assert!(threads[0].is_starred);
    }

    #[tokio::test]
    async fn set_starred_status() {
        let db = setup_db_with_account().await;
        let summary = make_test_summary();

        upsert(&db, &summary).await.unwrap();
        assert!(
            !get_by_id(&db, &summary.id)
                .await
                .unwrap()
                .unwrap()
                .is_starred
        );

        set_starred(&db, &summary.id, true).await.unwrap();
        assert!(
            get_by_id(&db, &summary.id)
                .await
                .unwrap()
                .unwrap()
                .is_starred
        );
    }

    #[tokio::test]
    async fn delete_thread() {
        let db = setup_db_with_account().await;
        let summary = make_test_summary();

        upsert(&db, &summary).await.unwrap();
        assert!(get_by_id(&db, &summary.id).await.unwrap().is_some());

        delete(&db, &summary.id).await.unwrap();
        assert!(get_by_id(&db, &summary.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn count_threads() {
        let db = setup_db_with_account().await;

        let mut summary1 = make_test_summary();
        summary1.id = ThreadId::from("thread-1");

        let mut summary2 = make_test_summary();
        summary2.id = ThreadId::from("thread-2");

        upsert(&db, &summary1).await.unwrap();
        upsert(&db, &summary2).await.unwrap();

        let count = count_by_account(&db, &AccountId::from("account-1"))
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
