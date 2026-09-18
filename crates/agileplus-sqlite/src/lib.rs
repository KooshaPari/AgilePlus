//! AgilePlus SQLite adapter — persistence layer.
//!
//! Implements `StoragePort` using rusqlite with WAL mode and foreign keys.
//! Traceability: WP06

#[path = "lib/content_storage.rs"]
mod content_storage;
pub mod event_store;
pub mod migrations;
pub mod rebuild;
pub mod repository;
pub mod seed;
#[path = "lib/storage_port.rs"]
mod storage_port;
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use agileplus_domain::error::DomainError;

    use super::*;

    #[tokio::test]
    async fn poisoned_lock_surfaces_as_storage_error() {
        let adapter = Arc::new(SqliteStorageAdapter::in_memory().expect("in-memory adapter"));
        let worker = Arc::clone(&adapter);

        // Hold the connection guard and panic, poisoning the mutex.
        let joined = std::thread::spawn(move || {
            let _guard = worker.conn_for_bench().expect("guard");
            panic!("poison the connection mutex");
        })
        .join();
        assert!(joined.is_err(), "worker thread must have panicked");

        match adapter.conn_for_bench() {
            Ok(_) => panic!("expected a poisoned lock"),
            Err(err) => assert!(
                matches!(err, DomainError::Storage(_)),
                "expected Storage error, got {err:?}"
            ),
        }

        // The async port surface must report the same failure instead of panicking.
        let err = StoragePort::list_all_features(&*adapter)
            .await
            .expect_err("port call must fail on a poisoned lock");
        assert!(
            matches!(err, DomainError::Storage(_)),
            "expected Storage error, got {err:?}"
        );
    }

    #[test]
    fn new_fails_when_parent_directory_is_missing() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "agileplus-sqlite-lib-{}-{nanos}",
            std::process::id()
        ));
        let path = root.join("absent").join("agileplus.db");

        let err = match SqliteStorageAdapter::new(&path) {
            Ok(_) => panic!("opening a database in a missing directory must fail"),
            Err(err) => err,
        };
        assert!(
            matches!(err, DomainError::Storage(_)),
            "expected Storage error, got {err:?}"
        );
    }
}
