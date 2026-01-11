//! Account CRUD operations.
//!
//! Provides database operations for account entities.

use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};

use crate::domain::{Account, AccountId, ProviderConfig, ProviderType};
use crate::storage::database::{Database, Result};

use std::time::Duration;

/// Inserts a new account into the database.
pub async fn insert(db: &Database, account: &Account) -> Result<()> {
    let account = account.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        let provider_type = match account.provider_type {
            ProviderType::Gmail => "gmail",
            ProviderType::Imap => "imap",
        };
        let provider_config = serde_json::to_string(&account.provider_config).unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO accounts (
                id, email, display_name, provider_type, provider_config,
                sync_enabled, sync_interval_seconds, signature,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
            )
            "#,
            params![
                account.id.0,
                account.email,
                account.display_name,
                provider_type,
                provider_config,
                account.sync_enabled as i32,
                account.sync_interval.as_secs() as i32,
                account.signature,
                now,
                now,
            ],
        )?;

        Ok(())
    })
    .await
}

/// Retrieves an account by its ID.
pub async fn get_by_id(db: &Database, account_id: &AccountId) -> Result<Option<Account>> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, email, display_name, provider_type, provider_config,
                sync_enabled, sync_interval_seconds, signature
            FROM accounts
            WHERE id = ?1
            "#,
        )?;

        let result = stmt.query_row([&account_id.0], row_to_account).optional()?;
        Ok(result)
    })
    .await
}

/// Retrieves an account by email address.
pub async fn get_by_email(db: &Database, email: &str) -> Result<Option<Account>> {
    let email = email.to_string();

    db.with_conn(move |conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, email, display_name, provider_type, provider_config,
                sync_enabled, sync_interval_seconds, signature
            FROM accounts
            WHERE email = ?1
            "#,
        )?;

        let result = stmt.query_row([&email], row_to_account).optional()?;
        Ok(result)
    })
    .await
}

/// Retrieves all accounts.
pub async fn get_all(db: &Database) -> Result<Vec<Account>> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, email, display_name, provider_type, provider_config,
                sync_enabled, sync_interval_seconds, signature
            FROM accounts
            ORDER BY email
            "#,
        )?;

        let rows = stmt.query_map([], row_to_account)?;
        let accounts: std::result::Result<Vec<_>, _> = rows.collect();
        Ok(accounts?)
    })
    .await
}

/// Updates an account's display name.
pub async fn set_display_name(
    db: &Database,
    account_id: &AccountId,
    display_name: Option<&str>,
) -> Result<()> {
    let account_id = account_id.clone();
    let display_name = display_name.map(|s| s.to_string());

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE accounts SET display_name = ?1, updated_at = ?2 WHERE id = ?3",
            params![display_name, now, account_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Updates an account's sync settings.
pub async fn set_sync_settings(
    db: &Database,
    account_id: &AccountId,
    enabled: bool,
    interval: Duration,
) -> Result<()> {
    let account_id = account_id.clone();

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE accounts SET sync_enabled = ?1, sync_interval_seconds = ?2, updated_at = ?3 WHERE id = ?4",
            params![enabled as i32, interval.as_secs() as i32, now, account_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Updates an account's signature.
pub async fn set_signature(
    db: &Database,
    account_id: &AccountId,
    signature: Option<&str>,
) -> Result<()> {
    let account_id = account_id.clone();
    let signature = signature.map(|s| s.to_string());

    db.with_conn(move |conn| {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE accounts SET signature = ?1, updated_at = ?2 WHERE id = ?3",
            params![signature, now, account_id.0],
        )?;
        Ok(())
    })
    .await
}

/// Deletes an account and all associated data.
pub async fn delete(db: &Database, account_id: &AccountId) -> Result<()> {
    let account_id = account_id.clone();

    db.transaction(move |tx| {
        // Delete in order to respect foreign key constraints
        tx.execute(
            "DELETE FROM embeddings WHERE email_id IN (SELECT id FROM emails WHERE account_id = ?1)",
            [&account_id.0],
        )?;
        tx.execute(
            "DELETE FROM attachments WHERE email_id IN (SELECT id FROM emails WHERE account_id = ?1)",
            [&account_id.0],
        )?;
        tx.execute("DELETE FROM emails WHERE account_id = ?1", [&account_id.0])?;
        tx.execute("DELETE FROM threads WHERE account_id = ?1", [&account_id.0])?;
        tx.execute("DELETE FROM labels WHERE account_id = ?1", [&account_id.0])?;
        tx.execute("DELETE FROM drafts WHERE account_id = ?1", [&account_id.0])?;
        tx.execute(
            "DELETE FROM pending_changes WHERE account_id = ?1",
            [&account_id.0],
        )?;
        tx.execute("DELETE FROM sync_state WHERE account_id = ?1", [&account_id.0])?;
        tx.execute("DELETE FROM accounts WHERE id = ?1", [&account_id.0])?;

        Ok(())
    })
    .await
}

/// Counts total accounts.
pub async fn count(db: &Database) -> Result<u32> {
    db.with_conn(|conn| {
        let count: u32 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        Ok(count)
    })
    .await
}

/// Checks if an account with the given email exists.
pub async fn exists_by_email(db: &Database, email: &str) -> Result<bool> {
    let email = email.to_string();

    db.with_conn(move |conn| {
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM accounts WHERE email = ?1",
            [&email],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    })
    .await
}

fn row_to_account(row: &Row<'_>) -> std::result::Result<Account, rusqlite::Error> {
    let provider_type_str: String = row.get(3)?;
    let provider_config_json: String = row.get(4)?;
    let sync_interval_secs: i32 = row.get(6)?;

    let provider_type = match provider_type_str.as_str() {
        "gmail" => ProviderType::Gmail,
        "imap" => ProviderType::Imap,
        _ => ProviderType::Imap, // Default fallback
    };

    let provider_config: ProviderConfig =
        serde_json::from_str(&provider_config_json).unwrap_or(ProviderConfig::Gmail {});

    Ok(Account {
        id: AccountId(row.get(0)?),
        email: row.get(1)?,
        display_name: row.get(2)?,
        provider_type,
        provider_config,
        sync_enabled: row.get::<_, i32>(5)? != 0,
        sync_interval: Duration::from_secs(sync_interval_secs as u64),
        signature: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_account() -> Account {
        Account {
            id: AccountId::from("account-1"),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            provider_type: ProviderType::Gmail,
            provider_config: ProviderConfig::Gmail {},
            sync_enabled: true,
            sync_interval: Duration::from_secs(300),
            signature: Some("-- \nTest User".to_string()),
        }
    }

    fn make_imap_account() -> Account {
        Account {
            id: AccountId::from("account-2"),
            email: "imap@example.com".to_string(),
            display_name: None,
            provider_type: ProviderType::Imap,
            provider_config: ProviderConfig::Imap {
                imap_host: "imap.example.com".to_string(),
                imap_port: 993,
                smtp_host: "smtp.example.com".to_string(),
                smtp_port: 587,
                use_tls: true,
            },
            sync_enabled: true,
            sync_interval: Duration::from_secs(600),
            signature: None,
        }
    }

    #[tokio::test]
    async fn insert_and_get_account() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        insert(&db, &account).await.unwrap();

        let retrieved = get_by_id(&db, &account.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, account.id);
        assert_eq!(retrieved.email, account.email);
        assert_eq!(retrieved.display_name, account.display_name);
        assert_eq!(retrieved.provider_type, ProviderType::Gmail);
    }

    #[tokio::test]
    async fn insert_imap_account() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_imap_account();

        insert(&db, &account).await.unwrap();

        let retrieved = get_by_id(&db, &account.id).await.unwrap().unwrap();
        assert_eq!(retrieved.provider_type, ProviderType::Imap);

        if let ProviderConfig::Imap { imap_host, .. } = retrieved.provider_config {
            assert_eq!(imap_host, "imap.example.com");
        } else {
            panic!("Expected IMAP config");
        }
    }

    #[tokio::test]
    async fn get_by_email_address() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        insert(&db, &account).await.unwrap();

        let retrieved = get_by_email(&db, "test@example.com").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, account.id);
    }

    #[tokio::test]
    async fn get_nonexistent_account_returns_none() {
        let db = Database::open_in_memory().await.unwrap();

        let result = get_by_id(&db, &AccountId::from("nonexistent"))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_all_accounts() {
        let db = Database::open_in_memory().await.unwrap();

        insert(&db, &make_test_account()).await.unwrap();
        insert(&db, &make_imap_account()).await.unwrap();

        let accounts = get_all(&db).await.unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn update_display_name() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        insert(&db, &account).await.unwrap();

        set_display_name(&db, &account.id, Some("New Name"))
            .await
            .unwrap();

        let retrieved = get_by_id(&db, &account.id).await.unwrap().unwrap();
        assert_eq!(retrieved.display_name, Some("New Name".to_string()));
    }

    #[tokio::test]
    async fn update_sync_settings() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        insert(&db, &account).await.unwrap();

        set_sync_settings(&db, &account.id, false, Duration::from_secs(900))
            .await
            .unwrap();

        let retrieved = get_by_id(&db, &account.id).await.unwrap().unwrap();
        assert!(!retrieved.sync_enabled);
        assert_eq!(retrieved.sync_interval, Duration::from_secs(900));
    }

    #[tokio::test]
    async fn update_signature() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        insert(&db, &account).await.unwrap();

        set_signature(&db, &account.id, Some("New Signature"))
            .await
            .unwrap();

        let retrieved = get_by_id(&db, &account.id).await.unwrap().unwrap();
        assert_eq!(retrieved.signature, Some("New Signature".to_string()));
    }

    #[tokio::test]
    async fn delete_account() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        insert(&db, &account).await.unwrap();
        assert!(get_by_id(&db, &account.id).await.unwrap().is_some());

        delete(&db, &account.id).await.unwrap();
        assert!(get_by_id(&db, &account.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn count_accounts() {
        let db = Database::open_in_memory().await.unwrap();

        assert_eq!(count(&db).await.unwrap(), 0);

        insert(&db, &make_test_account()).await.unwrap();
        assert_eq!(count(&db).await.unwrap(), 1);

        insert(&db, &make_imap_account()).await.unwrap();
        assert_eq!(count(&db).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn check_exists_by_email() {
        let db = Database::open_in_memory().await.unwrap();
        let account = make_test_account();

        assert!(!exists_by_email(&db, "test@example.com").await.unwrap());

        insert(&db, &account).await.unwrap();

        assert!(exists_by_email(&db, "test@example.com").await.unwrap());
        assert!(!exists_by_email(&db, "other@example.com").await.unwrap());
    }
}
