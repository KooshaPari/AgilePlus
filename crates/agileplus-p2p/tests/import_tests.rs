//! Integration tests for the `agileplus_p2p::import` module.
//!
//! Verifies the collect-then-apply import semantics, duplicate detection by
//! hash, latest-wins snapshot resolution, and error paths.

use std::sync::Mutex;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_events::snapshot::{SnapshotError, SnapshotStore};
use agileplus_events::store::{EventError, EventStore};
use agileplus_p2p::import::{ImportError, import_state};
use async_trait::async_trait;
use chrono::Utc;

// ── In-memory stores ──────────────────────────────────────────────────────────

#[derive(Default)]
struct MemEventStore {
    events: Mutex<Vec<Event>>,
}

#[async_trait]
impl EventStore for MemEventStore {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        let mut g = self.events.lock().unwrap();
        g.push(event.clone());
        Ok(event.sequence)
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<Event>, EventError> {
        let g = self.events.lock().unwrap();
        let mut out: Vec<Event> = g
            .iter()
            .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
            .cloned()
            .collect();
        out.sort_by_key(|e| e.sequence);
        Ok(out)
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        let g = self.events.lock().unwrap();
        Ok(g.iter()
            .filter(|e| {
                e.entity_type == entity_type && e.entity_id == entity_id && e.sequence > sequence
            })
            .cloned()
            .collect())
    }

    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: chrono::DateTime<Utc>,
        to: chrono::DateTime<Utc>,
    ) -> Result<Vec<Event>, EventError> {
        let g = self.events.lock().unwrap();
        Ok(g.iter()
            .filter(|e| {
                e.entity_type == entity_type
                    && e.entity_id == entity_id
                    && e.timestamp >= from
                    && e.timestamp <= to
            })
            .cloned()
            .collect())
    }

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError> {
        let g = self.events.lock().unwrap();
        Ok(g.iter()
            .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
            .map(|e| e.sequence)
            .max()
            .unwrap_or(0))
    }
}

#[derive(Default)]
struct MemSnapshotStore {
    snapshots: Mutex<Vec<Snapshot>>,
}

#[async_trait]
impl SnapshotStore for MemSnapshotStore {
    async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
        let mut g = self.snapshots.lock().unwrap();
        g.retain(|s| {
            !(s.entity_type == snapshot.entity_type && s.entity_id == snapshot.entity_id)
        });
        g.push(snapshot.clone());
        Ok(())
    }

    async fn load(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<Snapshot>, SnapshotError> {
        Ok(self
            .snapshots
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.entity_type == entity_type && s.entity_id == entity_id)
            .cloned())
    }

    async fn delete_before(
        &self,
        _entity_type: &str,
        _entity_id: i64,
        _sequence: i64,
    ) -> Result<(), SnapshotError> {
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_event(entity_type: &str, entity_id: i64, sequence: i64) -> Event {
    let mut e = Event::new(
        entity_type,
        entity_id,
        "created",
        serde_json::json!({"seq": sequence}),
        "tester",
    );
    e.sequence = sequence;
    // Distinct hash per sequence so dedup logic can distinguish them.
    e.hash[0] = sequence as u8;
    e
}

fn write_jsonl(dir: &std::path::Path, entity: &str, id: i64, events: &[Event]) {
    let d = dir.join("events").join(entity);
    std::fs::create_dir_all(&d).unwrap();
    let body: String = events
        .iter()
        .map(|e| format!("{}\n", serde_json::to_string(e).unwrap()))
        .collect();
    std::fs::write(d.join(format!("{id}.jsonl")), body).unwrap();
}

fn write_snapshot(dir: &std::path::Path, snap: &Snapshot) {
    let d = dir.join("snapshots").join(&snap.entity_type);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(
        d.join(format!("{}.json", snap.entity_id)),
        serde_json::to_string_pretty(snap).unwrap(),
    )
    .unwrap();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn import_empty_dir_reports_zero_counts() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 0);
    assert_eq!(stats.snapshots_updated, 0);
    assert_eq!(stats.sync_mappings_merged, 0);
}

#[tokio::test]
async fn import_single_event() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    write_jsonl(tmp.path(), "Feature", 1, &[make_event("Feature", 1, 1)]);

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 1);
    assert_eq!(es.get_events("Feature", 1).await.unwrap().len(), 1);
}

#[tokio::test]
async fn import_multiple_events_in_sequence_order() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    write_jsonl(
        tmp.path(),
        "Feature",
        1,
        &[
            make_event("Feature", 1, 1),
            make_event("Feature", 1, 2),
            make_event("Feature", 1, 3),
        ],
    );

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 3);
    let seqs: Vec<i64> = es
        .get_events("Feature", 1)
        .await
        .unwrap()
        .iter()
        .map(|e| e.sequence)
        .collect();
    assert_eq!(seqs, vec![1, 2, 3]);
}

#[tokio::test]
async fn import_skips_exact_duplicate_events() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ev = make_event("Feature", 1, 1);
    es.append(&ev).await.unwrap();
    write_jsonl(tmp.path(), "Feature", 1, &[ev]);

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 0);
}

#[tokio::test]
async fn import_appends_when_same_sequence_but_different_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();

    let existing = make_event("Feature", 1, 1); // hash[0] = 1
    es.append(&existing).await.unwrap();

    let mut incoming = make_event("Feature", 1, 1);
    incoming.hash[0] = 99; // different content hash
    write_jsonl(tmp.path(), "Feature", 1, &[incoming]);

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 1);
}

#[tokio::test]
async fn import_ignores_blank_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let d = tmp.path().join("events").join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    let ev = make_event("Feature", 1, 1);
    std::fs::write(
        d.join("1.jsonl"),
        format!("\n\n{}\n\n", serde_json::to_string(&ev).unwrap()),
    )
    .unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 1);
}

#[tokio::test]
async fn import_ignores_non_jsonl_files() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let d = tmp.path().join("events").join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("notes.txt"), "not an event").unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 0);
}

#[tokio::test]
async fn import_ignores_stray_files_at_events_root() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let d = tmp.path().join("events");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("README.md"), "docs").unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 0);
}

#[tokio::test]
async fn import_malformed_jsonl_reports_file_and_line() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let d = tmp.path().join("events").join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.jsonl"), "{ not valid json }\n").unwrap();

    let err = import_state(tmp.path(), &es, &ss).await.unwrap_err();
    match err {
        ImportError::Deserialization { file, .. } => {
            assert!(file.contains("1.jsonl"), "file was: {file}");
            assert!(file.contains(":1"), "should reference line 1: {file}");
        }
        other => panic!("expected Deserialization, got {other:?}"),
    }
}

#[tokio::test]
async fn import_multiple_entities_from_separate_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    write_jsonl(tmp.path(), "Feature", 1, &[make_event("Feature", 1, 1)]);
    write_jsonl(tmp.path(), "Epic", 2, &[make_event("Epic", 2, 1)]);

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 2);
}

#[tokio::test]
async fn import_snapshot_created_when_absent() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    write_snapshot(tmp.path(), &Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 5));

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.snapshots_updated, 1);
    assert_eq!(ss.load("Feature", 1).await.unwrap().unwrap().event_sequence, 5);
}

#[tokio::test]
async fn import_updates_snapshot_when_newer() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    ss.save(&Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1))
        .await
        .unwrap();
    write_snapshot(tmp.path(), &Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 9));

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.snapshots_updated, 1);
    assert_eq!(ss.load("Feature", 1).await.unwrap().unwrap().event_sequence, 9);
}

#[tokio::test]
async fn import_keeps_existing_snapshot_when_imported_is_older() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    ss.save(&Snapshot::new("Feature", 1, serde_json::json!({"v": 9}), 9))
        .await
        .unwrap();
    write_snapshot(tmp.path(), &Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1));

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.snapshots_updated, 0);
    assert_eq!(ss.load("Feature", 1).await.unwrap().unwrap().event_sequence, 9);
}

#[tokio::test]
async fn import_snapshot_with_equal_sequence_is_not_applied() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    ss.save(&Snapshot::new("Feature", 1, serde_json::json!({"v": 3}), 3))
        .await
        .unwrap();
    write_snapshot(tmp.path(), &Snapshot::new("Feature", 1, serde_json::json!({"v": 4}), 3));

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.snapshots_updated, 0);
}

#[tokio::test]
async fn import_ignores_non_json_snapshot_files() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let d = tmp.path().join("snapshots").join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.jsonl"), "{}").unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.snapshots_updated, 0);
}

#[tokio::test]
async fn import_malformed_snapshot_reports_error() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let d = tmp.path().join("snapshots").join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.json"), "!!!not json!!!").unwrap();

    let err = import_state(tmp.path(), &es, &ss).await.unwrap_err();
    assert!(matches!(err, ImportError::Deserialization { .. }));
}

#[tokio::test]
async fn import_sync_mappings_counted() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let mappings = vec![
        SyncMapping::new("Feature", 1, "p1", "h1"),
        SyncMapping::new("Feature", 2, "p2", "h2"),
        SyncMapping::new("Epic", 3, "p3", "h3"),
    ];
    std::fs::write(
        tmp.path().join("sync_state.json"),
        serde_json::json!({"sync_mappings": mappings, "sync_vector": {}}).to_string(),
    )
    .unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.sync_mappings_merged, 3);
}

#[tokio::test]
async fn import_sync_state_without_mappings_key_is_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    std::fs::write(tmp.path().join("sync_state.json"), "{}").unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.sync_mappings_merged, 0);
}

#[tokio::test]
async fn import_malformed_sync_state_reports_error() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    std::fs::write(tmp.path().join("sync_state.json"), "nope").unwrap();

    let err = import_state(tmp.path(), &es, &ss).await.unwrap_err();
    assert!(matches!(err, ImportError::Deserialization { .. }));
}

#[tokio::test]
async fn import_is_idempotent_on_second_run() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    write_jsonl(
        tmp.path(),
        "Feature",
        1,
        &[make_event("Feature", 1, 1), make_event("Feature", 1, 2)],
    );

    let first = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(first.events_imported, 2);
    let second = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(second.events_imported, 0);
    assert_eq!(es.get_events("Feature", 1).await.unwrap().len(), 2);
}

#[tokio::test]
async fn import_combines_events_snapshots_and_mappings() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    write_jsonl(tmp.path(), "Feature", 1, &[make_event("Feature", 1, 1)]);
    write_snapshot(tmp.path(), &Snapshot::new("Feature", 1, serde_json::json!({}), 1));
    std::fs::write(
        tmp.path().join("sync_state.json"),
        serde_json::json!({"sync_mappings": [SyncMapping::new("Feature", 1, "p", "h")]})
            .to_string(),
    )
    .unwrap();

    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 1);
    assert_eq!(stats.snapshots_updated, 1);
    assert_eq!(stats.sync_mappings_merged, 1);
}
