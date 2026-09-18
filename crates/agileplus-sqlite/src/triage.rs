// SPDX-License-Identifier: MIT OR Apache-2.0
//! SQLite-backed triage adapter built on top of the backlog repository methods.

use std::path::Path;

use agileplus_domain::{
    domain::backlog::BacklogStatus,
    ports::{ContentStoragePort, TriageError, TriageOutcome, TriagePort, TriageTicket},
};
use async_trait::async_trait;

use crate::SqliteStorageAdapter;

/// Thin triage adapter that reuses the existing backlog persistence surface.
pub struct SqliteTriageAdapter {
    storage: SqliteStorageAdapter,
}

impl SqliteTriageAdapter {
    pub fn new(db_path: &Path) -> Result<Self, TriageError> {
        let storage = SqliteStorageAdapter::new(db_path).map_err(TriageError::from)?;
        Self::from_storage(storage)
    }

    pub fn in_memory() -> Result<Self, TriageError> {
        let storage = SqliteStorageAdapter::in_memory().map_err(TriageError::from)?;
        Self::from_storage(storage)
    }

    pub fn storage(&self) -> &SqliteStorageAdapter {
        &self.storage
    }

    fn from_storage(storage: SqliteStorageAdapter) -> Result<Self, TriageError> {
        ensure_backlog_storage(&storage)?;
        Ok(Self { storage })
    }
}

fn ensure_backlog_storage(storage: &SqliteStorageAdapter) -> Result<(), TriageError> {
    let sql = include_str!("migrations/016_create_backlog_items.sql")
        .split("-- DOWN")
        .next()
        .unwrap_or_default();
    storage
        .conn_for_bench()
        .map_err(TriageError::from)?
        .execute_batch(sql)
        .map_err(|err| TriageError::Storage(err.to_string()))
}

#[async_trait]
impl TriagePort for SqliteTriageAdapter {
    async fn next_ticket(&self) -> Result<TriageTicket, TriageError> {
        let item = self
            .storage
            .pop_next_backlog_item()
            .await
            .map_err(TriageError::from)?
            .ok_or(TriageError::NoTicketAvailable)?;

        Ok(item.into())
    }

    async fn record_outcome(&self, id: &str, outcome: TriageOutcome) -> Result<(), TriageError> {
        let parsed_id = id
            .parse::<i64>()
            .map_err(|_| TriageError::InvalidTicketId(id.to_string()))?;

        let existing = self
            .storage
            .get_backlog_item(parsed_id)
            .await
            .map_err(TriageError::from)?;
        if existing.is_none() {
            return Err(TriageError::TicketNotFound(id.to_string()));
        }

        let status = match outcome {
            TriageOutcome::Accepted => BacklogStatus::Triaged,
            TriageOutcome::Dismissed => BacklogStatus::Dismissed,
        };

        self.storage
            .update_backlog_status(parsed_id, status)
            .await
            .map_err(TriageError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use agileplus_domain::ports::{TriageError, TriagePort};

    use super::SqliteTriageAdapter;

    /// Create a fresh temp directory unique to this process and label.
    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "agileplus-sqlite-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn file_backed_adapter_ensures_backlog_storage_and_serves_tickets() {
        let dir = unique_temp_dir("triage-file");
        let path = dir.join("triage.db");

        let triage = SqliteTriageAdapter::new(&path).expect("file-backed triage adapter");
        // No tickets yet: the empty backlog must surface as NoTicketAvailable.
        let empty = TriagePort::next_ticket(&triage)
            .await
            .expect_err("empty backlog");
        assert!(matches!(empty, TriageError::NoTicketAvailable));

        {
            let conn = triage.storage().conn_for_bench().expect("lock");
            conn.execute(
                "INSERT INTO backlog_items
                 (title, description, intent, priority, status, source, feature_slug, tags_json, created_at, updated_at)
                 VALUES ('Needs triage','desc','bug','critical','new','sentry',NULL,'[\"p0\"]',?1,?1)",
                rusqlite::params![chrono::Utc::now().to_rfc3339()],
            )
            .expect("seed backlog item");
        }

        let ticket = TriagePort::next_ticket(&triage).await.expect("ticket");
        assert_eq!(ticket.title, "Needs triage");
        assert_eq!(
            ticket.priority,
            agileplus_domain::domain::backlog::BacklogPriority::Critical
        );
        assert_eq!(ticket.tags, vec!["p0".to_string()]);
        let id = ticket.id.clone();

        TriagePort::record_outcome(
            &triage,
            &id,
            agileplus_domain::ports::TriageOutcome::Accepted,
        )
        .await
        .expect("record accepted");
        assert!(
            matches!(
                TriagePort::next_ticket(&triage).await,
                Err(TriageError::NoTicketAvailable)
            ),
            "an accepted ticket must leave the new queue"
        );

        // Reopening the same file must not fail on the already-applied 016 DDL.
        let reopened = SqliteTriageAdapter::new(&path).expect("reopen");
        assert!(
            matches!(
                TriagePort::next_ticket(&reopened).await,
                Err(TriageError::NoTicketAvailable)
            ),
            "reopened adapter must see the same triaged state"
        );

        drop(reopened);
        drop(triage);
        cleanup(&dir);
    }

    #[test]
    fn new_fails_when_parent_directory_is_missing() {
        let dir = unique_temp_dir("triage-missing");
        let path = dir.join("absent").join("triage.db");

        let err = match SqliteTriageAdapter::new(&path) {
            Ok(_) => panic!("opening a database in a missing directory must fail"),
            Err(err) => err,
        };
        assert!(matches!(err, TriageError::Storage(_)), "got {err:?}");

        cleanup(&dir);
    }
}
