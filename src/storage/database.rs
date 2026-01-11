//! Database connection pool and initialization.
//!
//! Provides a thread-safe wrapper around rusqlite for async operations.

use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use thiserror::Error;
use tokio::sync::Mutex;

use super::schema;

/// Errors that can occur during database operations.
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Database not initialized")]
    NotInitialized,

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for database operations.
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Thread-safe database connection wrapper.
///
/// Uses a Mutex to ensure only one operation accesses the connection at a time.
/// All operations are run via `spawn_blocking` to avoid blocking the async runtime.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens a database at the given path, creating it if necessary.
    ///
    /// Runs migrations to ensure the schema is up to date.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let conn = Connection::open(&path)?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
            Ok(conn)
        })
        .await
        .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))??;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.run_migrations().await?;

        Ok(db)
    }

    /// Opens an in-memory database for testing.
    pub async fn open_in_memory() -> Result<Self> {
        let conn = tokio::task::spawn_blocking(|| -> Result<Connection> {
            let conn = Connection::open_in_memory()?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            Ok(conn)
        })
        .await
        .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))??;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.run_migrations().await?;

        Ok(db)
    }

    /// Runs all schema migrations.
    async fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = conn.blocking_lock();

            for migration in schema::all_migrations() {
                conn.execute_batch(migration)?;
            }

            Ok(())
        })
        .await
        .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?
    }

    /// Executes a function with access to the database connection.
    ///
    /// The function runs in a blocking task to avoid blocking the async runtime.
    pub async fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            f(&conn)
        })
        .await
        .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?
    }

    /// Executes a function with mutable access to the database connection.
    ///
    /// Use this for transactions or operations that require mutable access.
    pub async fn with_conn_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = conn.blocking_lock();
            f(&mut conn)
        })
        .await
        .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?
    }

    /// Executes a transaction with the given function.
    ///
    /// The transaction is automatically committed on success or rolled back on error.
    pub async fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = conn.blocking_lock();
            let tx = conn.transaction()?;
            let result = f(&tx)?;
            tx.commit()?;
            Ok(result)
        })
        .await
        .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_in_memory_creates_schema() {
        let db = Database::open_in_memory().await.unwrap();

        let tables: Vec<String> = db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
                let rows = stmt.query_map([], |row| row.get(0))?;
                Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
            })
            .await
            .unwrap();

        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"emails".to_string()));
        assert!(tables.contains(&"threads".to_string()));
        assert!(tables.contains(&"labels".to_string()));
    }

    #[tokio::test]
    async fn with_conn_executes_query() {
        let db = Database::open_in_memory().await.unwrap();

        let count: i64 = db
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
    async fn transaction_commits_on_success() {
        let db = Database::open_in_memory().await.unwrap();

        db.transaction(|tx| {
            tx.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)",
                ["test_key", "test_value", "2025-01-01T00:00:00Z"],
            )?;
            Ok(())
        })
        .await
        .unwrap();

        let value: String = db
            .with_conn(|conn| {
                let value = conn.query_row(
                    "SELECT value FROM settings WHERE key = ?",
                    ["test_key"],
                    |row| row.get(0),
                )?;
                Ok(value)
            })
            .await
            .unwrap();

        assert_eq!(value, "test_value");
    }

    #[tokio::test]
    async fn transaction_rolls_back_on_error() {
        let db = Database::open_in_memory().await.unwrap();

        let result: Result<()> = db
            .transaction(|tx| {
                tx.execute(
                    "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)",
                    ["rollback_key", "rollback_value", "2025-01-01T00:00:00Z"],
                )?;
                Err(DatabaseError::MigrationFailed(
                    "intentional error".to_string(),
                ))
            })
            .await;

        assert!(result.is_err());

        let count: i64 = db
            .with_conn(|conn| {
                let count = conn.query_row(
                    "SELECT COUNT(*) FROM settings WHERE key = ?",
                    ["rollback_key"],
                    |row| row.get(0),
                )?;
                Ok(count)
            })
            .await
            .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn database_is_clone() {
        let db1 = Database::open_in_memory().await.unwrap();
        let db2 = db1.clone();

        db1.transaction(|tx| {
            tx.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)",
                ["clone_key", "clone_value", "2025-01-01T00:00:00Z"],
            )?;
            Ok(())
        })
        .await
        .unwrap();

        let value: String = db2
            .with_conn(|conn| {
                let value = conn.query_row(
                    "SELECT value FROM settings WHERE key = ?",
                    ["clone_key"],
                    |row| row.get(0),
                )?;
                Ok(value)
            })
            .await
            .unwrap();

        assert_eq!(value, "clone_value");
    }
}
