//! AgilePlus SQLite adapter — persistence layer.
//!
//! Implements `StoragePort` using rusqlite with WAL mode and foreign keys.
//! Traceability: WP06

pub mod event_store;
pub mod migrations;
pub mod rebuild;
pub mod repository;
pub mod seed;
#[path = "lib/storage_port.rs"]
mod storage_port;
#[path = "lib/content_storage.rs"]
mod content_storage;
pub mod triage;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use agileplus_domain::error::DomainError;
#[allow(unused_imports)]
use agileplus_domain::ports::StoragePort;

use crate::migrations::MigrationRunner;

/// SQLite-backed storage adapter.
///
/// Uses a single write-serialized connection protected by a Mutex.
/// WAL mode is enabled to allow concurrent reads; all writes are serialized.
pub struct SqliteStorageAdapter {
    conn: Arc<Mutex<Connection>>,
}

pub use triage::SqliteTriageAdapter;

impl SqliteStorageAdapter {
    /// Open a file-backed database, enable WAL + FK pragma, and run all migrations.
    pub fn new(db_path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| DomainError::Storage(format!("failed to open db: {e}")))?;
        Self::configure_and_migrate(conn, true)
    }

    /// Open an in-memory database (for tests).
    pub fn in_memory() -> Result<Self, DomainError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DomainError::Storage(format!("failed to open in-memory db: {e}")))?;
        Self::configure_and_migrate(conn, false)
    }

    fn configure_and_migrate(conn: Connection, enable_wal: bool) -> Result<Self, DomainError> {
        // Enable WAL mode for file-backed databases (not applicable to in-memory).
        // WAL mode doesn't work reliably with in-memory databases, so skip it for tests.
        if enable_wal {
            conn.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| DomainError::Storage(format!("WAL pragma failed: {e}")))?;
        }

        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::Storage(format!("FK pragma failed: {e}")))?;

        // Run migrations
        let runner = MigrationRunner::new(&conn);
        runner.run_all()?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get a locked guard to the connection.
    pub(crate) fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.conn
            .lock()
            .map_err(|e| DomainError::Storage(format!("mutex poisoned: {e}")))
    }

    /// Expose a locked connection guard for benchmarks and test helpers.
    ///
    /// This method is intentionally public so that benchmark crates can access
    /// the underlying rusqlite `Connection` to call repository functions directly
    /// without going through the async `StoragePort` trait.
    pub fn conn_for_bench(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.lock()
    }
}

#[cfg(test)]
#[path = "lib/tests_persistence.rs"]
mod tests_persistence;
