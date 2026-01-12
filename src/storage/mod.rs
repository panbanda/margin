//! Database and credential storage.
//!
//! This module provides the storage layer for The Heap, including:
//!
//! - SQLite database for emails, threads, accounts, and other data
//! - OS keychain integration for secure credential storage
//! - Async-safe database operations via tokio::task::spawn_blocking

mod database;
mod keychain;
pub mod queries;
mod schema;

pub use database::{Database, DatabaseError, Result};
pub use keychain::{KeychainAccess, KeychainError};

use std::sync::Arc;

/// Combined storage layer with database and keychain access.
///
/// This is the main entry point for storage operations.
#[derive(Debug, Clone)]
pub struct StorageLayer {
    db: Database,
    keychain: KeychainAccess,
}

impl StorageLayer {
    /// Creates a new storage layer with the given database path.
    pub async fn new(db_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let db = Database::open(db_path).await?;
        let keychain = KeychainAccess::new();

        Ok(Self { db, keychain })
    }

    /// Creates a storage layer with an in-memory database for testing.
    pub async fn in_memory() -> Result<Self> {
        let db = Database::open_in_memory().await?;
        let keychain = KeychainAccess::with_service("com.panbanda.heap.test");

        Ok(Self { db, keychain })
    }

    /// Returns a reference to the database.
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Returns a reference to the keychain.
    pub fn keychain(&self) -> &KeychainAccess {
        &self.keychain
    }

    /// Wraps the storage layer in an Arc for shared ownership.
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn storage_layer_in_memory() {
        let storage = StorageLayer::in_memory().await.unwrap();

        // Verify database is accessible
        let count: i64 = storage
            .db()
            .with_conn(|conn| {
                let count =
                    conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
                Ok(count)
            })
            .await
            .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn storage_layer_keychain_service() {
        let storage = StorageLayer::in_memory().await.unwrap();
        assert_eq!(storage.keychain().service_name(), "com.panbanda.heap.test");
    }

    #[tokio::test]
    async fn storage_layer_into_arc() {
        let storage = StorageLayer::in_memory().await.unwrap();
        let arc_storage = storage.into_arc();

        let count: i64 = arc_storage
            .db()
            .with_conn(|conn| {
                let count =
                    conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
                Ok(count)
            })
            .await
            .unwrap();

        assert_eq!(count, 0);
    }
}
