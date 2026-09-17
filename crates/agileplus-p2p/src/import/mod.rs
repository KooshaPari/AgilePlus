//! Git-backed state import — read deterministic files back into the event store.
//!
//! Reads from the same layout written by `export`:
//!   events/{entity_type}/{id}.jsonl  — JSONL, one event per line
//!   snapshots/{entity_type}/{id}.json — latest snapshot (pretty JSON)
//!   sync_state.json                  — SyncMapping entries and device sync vectors
//!
//! Skips duplicate events by hash comparison.  All events are collected before
//! any are applied (transaction-like semantics).
//!
//! Traceability: WP17 / T102

mod reader;

#[cfg(test)]
mod tests;

use std::path::Path;
use std::time::Instant;

use agileplus_events::snapshot::SnapshotStore;
use agileplus_events::store::EventStore;
use tracing::{debug, warn};

use reader::{read_events_from_dir, read_snapshots_from_dir, read_sync_mappings};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Deserialization error in {file}: {source}")]
    Deserialization {
        file: String,
        source: serde_json::Error,
    },

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Snapshot store error: {0}")]
    SnapshotStore(String),
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Statistics returned after a successful import.
#[derive(Debug, Default, Clone)]
pub struct ImportStats {
    pub events_imported: usize,
    pub snapshots_updated: usize,
    pub sync_mappings_merged: usize,
    pub duration_ms: u64,
}

// ── Core import function ──────────────────────────────────────────────────────

/// Import state from `input_dir` into the provided stores.
///
/// Events that already exist (matched by `hash` field) are silently skipped.
/// All events are collected before any writes occur (transaction-like).
pub async fn import_state<ES, SS>(
    input_dir: &Path,
    event_store: &ES,
    snapshot_store: &SS,
) -> Result<ImportStats, ImportError>
where
    ES: EventStore,
    SS: SnapshotStore,
{
    let started = Instant::now();
    let mut stats = ImportStats::default();

    // ── Collect phase ─────────────────────────────────────────────────────────
    let all_events = read_events_from_dir(&input_dir.join("events"))?;
    let all_snapshots = read_snapshots_from_dir(&input_dir.join("snapshots"))?;
    let all_mappings = read_sync_mappings(&input_dir.join("sync_state.json"))?;

    debug!(
        "Collected {} events, {} snapshots, {} mappings from {}",
        all_events.len(),
        all_snapshots.len(),
        all_mappings.len(),
        input_dir.display()
    );

    // ── Apply events (skip duplicates by hash) ────────────────────────────────
    for event in &all_events {
        // Check whether this sequence already exists for the entity stream.
        let latest_seq = event_store
            .get_latest_sequence(&event.entity_type, event.entity_id)
            .await
            .map_err(|e| ImportError::EventStore(e.to_string()))?;

        if event.sequence <= latest_seq {
            // Potentially a duplicate; verify by loading the exact event.
            let existing = event_store
                .get_events_since(&event.entity_type, event.entity_id, event.sequence - 1)
                .await
                .map_err(|e| ImportError::EventStore(e.to_string()))?;

            let already_present = existing
                .iter()
                .any(|e| e.sequence == event.sequence && e.hash == event.hash);

            if already_present {
                debug!(
                    "Skipping duplicate event {}/{} seq={}",
                    event.entity_type, event.entity_id, event.sequence
                );
                continue;
            }
        }

        event_store
            .append(event)
            .await
            .map_err(|e| ImportError::EventStore(e.to_string()))?;
        stats.events_imported += 1;
    }

    // ── Apply snapshots (latest-wins by event_sequence) ───────────────────────
    for snapshot in &all_snapshots {
        // Load existing snapshot for this entity to compare sequences.
        let existing = snapshot_store
            .load(&snapshot.entity_type, snapshot.entity_id)
            .await
            .map_err(|e| ImportError::SnapshotStore(e.to_string()))?;

        let should_update = match &existing {
            None => true,
            Some(ex) => snapshot.event_sequence > ex.event_sequence,
        };

        if should_update {
            snapshot_store
                .save(snapshot)
                .await
                .map_err(|e| ImportError::SnapshotStore(e.to_string()))?;
            stats.snapshots_updated += 1;
        } else {
            warn!(
                "Skipping older snapshot for {}/{} (imported seq={} <= existing seq={})",
                snapshot.entity_type,
                snapshot.entity_id,
                snapshot.event_sequence,
                existing.unwrap().event_sequence
            );
        }
    }

    stats.sync_mappings_merged = all_mappings.len();
    stats.duration_ms = started.elapsed().as_millis() as u64;
    Ok(stats)
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn import_error_io_display() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let e: ImportError = io.into();
        assert!(e.to_string().contains("IO error"));
    }

    #[test]
    fn import_error_deserialization_display() {
        let src = serde_json::from_str::<serde_json::Value>("x").unwrap_err();
        let e = ImportError::Deserialization {
            file: "events/Feature/1.jsonl".into(),
            source: src,
        };
        let s = e.to_string();
        assert!(s.contains("Deserialization error"));
        assert!(s.contains("events/Feature/1.jsonl"));
    }

    #[test]
    fn import_error_event_store_display() {
        let e = ImportError::EventStore("es".into());
        assert_eq!(e.to_string(), "Event store error: es");
    }

    #[test]
    fn import_error_snapshot_store_display() {
        let e = ImportError::SnapshotStore("ss".into());
        assert_eq!(e.to_string(), "Snapshot store error: ss");
    }

    #[test]
    fn import_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ImportError>();
    }

    #[test]
    fn import_stats_default_zeroed() {
        let s = ImportStats::default();
        assert_eq!(s.events_imported, 0);
        assert_eq!(s.snapshots_updated, 0);
        assert_eq!(s.sync_mappings_merged, 0);
        assert_eq!(s.duration_ms, 0);
    }

    #[test]
    fn import_stats_clone_and_debug() {
        let s = ImportStats {
            events_imported: 4,
            snapshots_updated: 2,
            sync_mappings_merged: 1,
            duration_ms: 7,
        };
        let c = s.clone();
        assert_eq!(c.events_imported, 4);
        assert!(format!("{s:?}").contains("events_imported"));
    }
}
