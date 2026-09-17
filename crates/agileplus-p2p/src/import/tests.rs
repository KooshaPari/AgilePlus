use std::sync::Mutex;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_events::snapshot::SnapshotError;
use agileplus_events::store::EventError;
use async_trait::async_trait;
use chrono::Utc;

use super::*;

#[derive(Default)]
struct MemEventStore {
    events: Mutex<Vec<Event>>,
}

#[async_trait]
impl EventStore for MemEventStore {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        self.events.lock().unwrap().push(event.clone());
        Ok(event.sequence)
    }

    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError> {
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
            .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
            .cloned()
            .collect())
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
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
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
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
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
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
        let mut snapshots = self.snapshots.lock().unwrap();
        snapshots.retain(|s| {
            !(s.entity_type == snapshot.entity_type && s.entity_id == snapshot.entity_id)
        });
        snapshots.push(snapshot.clone());
        Ok(())
    }

    async fn load(&self, entity_type: &str, entity_id: i64) -> Result<Option<Snapshot>, SnapshotError> {
        let snapshots = self.snapshots.lock().unwrap();
        Ok(snapshots
            .iter()
            .filter(|s| s.entity_type == entity_type && s.entity_id == entity_id)
            .max_by_key(|s| s.event_sequence)
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

fn make_event(entity_type: &str, entity_id: i64, sequence: i64) -> Event {
    let mut event = Event::new(entity_type, entity_id, "created", serde_json::json!({}), "test");
    event.sequence = sequence;
    event
}

#[tokio::test]
async fn import_new_events() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let events_dir = dir.join("events/Feature");
    std::fs::create_dir_all(&events_dir).unwrap();
    let event = make_event("Feature", 1, 1);
    let line = serde_json::to_string(&event).unwrap();
    std::fs::write(events_dir.join("1.jsonl"), format!("{line}\n")).unwrap();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();
    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.events_imported, 1);
}

#[tokio::test]
async fn import_skips_duplicate_events() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let event = make_event("Feature", 1, 1);
    let event_store = MemEventStore::default();
    event_store.append(&event).await.unwrap();
    let snapshot_store = MemSnapshotStore::default();

    let events_dir = dir.join("events/Feature");
    std::fs::create_dir_all(&events_dir).unwrap();
    let line = serde_json::to_string(&event).unwrap();
    std::fs::write(events_dir.join("1.jsonl"), format!("{line}\n")).unwrap();

    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.events_imported, 0);
}

#[tokio::test]
async fn import_updates_snapshot_latest_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();

    let old_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
    snapshot_store.save(&old_snapshot).await.unwrap();

    let snapshots_dir = dir.join("snapshots/Feature");
    std::fs::create_dir_all(&snapshots_dir).unwrap();
    let new_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 5);
    let json = serde_json::to_string_pretty(&new_snapshot).unwrap();
    std::fs::write(snapshots_dir.join("1.json"), json).unwrap();

    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.snapshots_updated, 1);

    let loaded = snapshot_store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 5);
}

#[tokio::test]
async fn import_does_not_downgrade_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();

    let new_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 10}), 10);
    snapshot_store.save(&new_snapshot).await.unwrap();

    let snapshots_dir = dir.join("snapshots/Feature");
    std::fs::create_dir_all(&snapshots_dir).unwrap();
    let old_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
    let json = serde_json::to_string_pretty(&old_snapshot).unwrap();
    std::fs::write(snapshots_dir.join("1.json"), json).unwrap();

    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.snapshots_updated, 0);
}

#[tokio::test]
async fn import_sync_mappings_counted() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let mappings = vec![
        agileplus_domain::domain::sync_mapping::SyncMapping::new("Feature", 1, "p1", "h1"),
        agileplus_domain::domain::sync_mapping::SyncMapping::new("Feature", 2, "p2", "h2"),
    ];
    let sync_state = serde_json::json!({ "sync_mappings": mappings, "sync_vector": {} });
    std::fs::write(
        dir.join("sync_state.json"),
        serde_json::to_string_pretty(&sync_state).unwrap(),
    )
    .unwrap();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();
    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.sync_mappings_merged, 2);
}

// ── Deep reader coverage ───────────────────────────────────────────────────

use super::reader::{
    read_events_from_dir, read_snapshots_from_dir, read_sync_mappings,
};

fn write_event_jsonl(dir: &std::path::Path, entity_type: &str, id: i64, lines: &[String]) {
    let d = dir.join(entity_type);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(format!("{id}.jsonl")), lines.join("\n")).unwrap();
}

#[test]
fn read_events_from_missing_dir_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let events = read_events_from_dir(&tmp.path().join("nope")).unwrap();
    assert!(events.is_empty());
}

#[test]
fn read_events_from_empty_dir_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let events = read_events_from_dir(tmp.path()).unwrap();
    assert!(events.is_empty());
}

#[test]
fn read_events_skips_blank_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let e1 = serde_json::to_string(&make_event("Feature", 1, 1)).unwrap();
    let e2 = serde_json::to_string(&make_event("Feature", 1, 2)).unwrap();
    write_event_jsonl(tmp.path(), "Feature", 1, &[e1, String::new(), "   ".into(), e2]);
    let events = read_events_from_dir(tmp.path()).unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn read_events_ignores_non_jsonl_files() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path().join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("ignore.txt"), "not json").unwrap();
    std::fs::write(d.join("1.jsonl"), serde_json::to_string(&make_event("Feature", 1, 1)).unwrap()).unwrap();
    let events = read_events_from_dir(tmp.path()).unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn read_events_ignores_top_level_files() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("loose.jsonl"), "{}").unwrap();
    let events = read_events_from_dir(tmp.path()).unwrap();
    assert!(events.is_empty());
}

#[test]
fn read_events_sorted_by_type_id_sequence() {
    let tmp = tempfile::tempdir().unwrap();
    // Insert out of order across entity types.
    write_event_jsonl(tmp.path(), "Feature", 2, &[serde_json::to_string(&make_event("Feature", 2, 1)).unwrap()]);
    write_event_jsonl(tmp.path(), "Epic", 1, &[serde_json::to_string(&make_event("Epic", 1, 1)).unwrap()]);
    write_event_jsonl(
        tmp.path(),
        "Feature",
        1,
        &[
            serde_json::to_string(&make_event("Feature", 1, 2)).unwrap(),
            serde_json::to_string(&make_event("Feature", 1, 1)).unwrap(),
        ],
    );
    let events = read_events_from_dir(tmp.path()).unwrap();
    let triples: Vec<(String, i64, i64)> = events
        .iter()
        .map(|e| (e.entity_type.clone(), e.entity_id, e.sequence))
        .collect();
    assert_eq!(
        triples,
        vec![
            ("Epic".to_string(), 1, 1),
            ("Feature".to_string(), 1, 1),
            ("Feature".to_string(), 1, 2),
            ("Feature".to_string(), 2, 1),
        ]
    );
}

#[test]
fn read_events_malformed_line_reports_file_and_line() {
    let tmp = tempfile::tempdir().unwrap();
    write_event_jsonl(tmp.path(), "Feature", 1, &["{not json".to_string()]);
    let err = read_events_from_dir(tmp.path()).unwrap_err();
    match err {
        super::ImportError::Deserialization { file, .. } => {
            assert!(file.contains("1.jsonl"), "file={file}");
            assert!(file.contains(":1"), "line number missing: {file}");
        }
        other => panic!("expected Deserialization, got {other:?}"),
    }
}

#[test]
fn read_events_multiple_files_same_entity_merged() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path().join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("a.jsonl"), serde_json::to_string(&make_event("Feature", 1, 1)).unwrap()).unwrap();
    std::fs::write(d.join("b.jsonl"), serde_json::to_string(&make_event("Feature", 1, 2)).unwrap()).unwrap();
    let events = read_events_from_dir(tmp.path()).unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn read_snapshots_missing_dir_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(read_snapshots_from_dir(&tmp.path().join("nope")).unwrap().is_empty());
}

#[test]
fn read_snapshots_parses_files() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path().join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    let snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 3}), 3);
    std::fs::write(d.join("1.json"), serde_json::to_string_pretty(&snap).unwrap()).unwrap();
    let snaps = read_snapshots_from_dir(tmp.path()).unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].event_sequence, 3);
}

#[test]
fn read_snapshots_ignores_non_json() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path().join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.jsonl"), "nope").unwrap();
    assert!(read_snapshots_from_dir(tmp.path()).unwrap().is_empty());
}

#[test]
fn read_snapshots_malformed_reports_error() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path().join("Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.json"), "{bad").unwrap();
    let err = read_snapshots_from_dir(tmp.path()).unwrap_err();
    assert!(matches!(err, super::ImportError::Deserialization { .. }));
}

#[test]
fn read_sync_mappings_missing_file_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let mappings = read_sync_mappings(&tmp.path().join("none.json")).unwrap();
    assert!(mappings.is_empty());
}

#[test]
fn read_sync_mappings_missing_key_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sync_state.json");
    std::fs::write(&path, serde_json::json!({"other": 1}).to_string()).unwrap();
    assert!(read_sync_mappings(&path).unwrap().is_empty());
}

#[test]
fn read_sync_mappings_parses_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sync_state.json");
    let mappings = vec![
        agileplus_domain::domain::sync_mapping::SyncMapping::new("Feature", 1, "p1", "h1"),
        agileplus_domain::domain::sync_mapping::SyncMapping::new("Epic", 2, "p2", "h2"),
    ];
    std::fs::write(
        &path,
        serde_json::json!({"sync_mappings": mappings}).to_string(),
    )
    .unwrap();
    let parsed = read_sync_mappings(&path).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].plane_issue_id, "p1");
}

#[test]
fn read_sync_mappings_malformed_json_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sync_state.json");
    std::fs::write(&path, "{not json").unwrap();
    let err = read_sync_mappings(&path).unwrap_err();
    assert!(matches!(err, super::ImportError::Deserialization { .. }));
}

#[test]
fn read_sync_mappings_wrong_shape_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sync_state.json");
    std::fs::write(&path, serde_json::json!({"sync_mappings": 5}).to_string()).unwrap();
    assert!(read_sync_mappings(&path).is_err());
}

#[tokio::test]
async fn import_empty_dir_is_noop() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 0);
    assert_eq!(stats.snapshots_updated, 0);
    assert_eq!(stats.sync_mappings_merged, 0);
}

#[tokio::test]
async fn import_malformed_event_propagates_error() {
    let tmp = tempfile::tempdir().unwrap();
    write_event_jsonl(&tmp.path().join("events"), "Feature", 1, &["{bad".to_string()]);
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let err = import_state(tmp.path(), &es, &ss).await.unwrap_err();
    assert!(matches!(err, super::ImportError::Deserialization { .. }));
}

#[tokio::test]
async fn import_non_contiguous_newer_event_is_applied() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    // Existing seq 1, importing seq 2 (newer) should be applied.
    let es = MemEventStore::default();
    es.append(&make_event("Feature", 1, 1)).await.unwrap();
    write_event_jsonl(&dir.join("events"), "Feature", 1, &[serde_json::to_string(&make_event("Feature", 1, 2)).unwrap()]);
    let ss = MemSnapshotStore::default();
    let stats = import_state(dir, &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 1);
}

#[tokio::test]
async fn import_equal_sequence_same_hash_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let e = make_event("Feature", 1, 1);
    let es = MemEventStore::default();
    es.append(&e).await.unwrap();
    write_event_jsonl(&dir.join("events"), "Feature", 1, &[serde_json::to_string(&e).unwrap()]);
    let ss = MemSnapshotStore::default();
    let stats = import_state(dir, &es, &ss).await.unwrap();
    assert_eq!(stats.events_imported, 0);
}

#[tokio::test]
async fn import_snapshot_equal_sequence_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    ss.save(&Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 5))
        .await
        .unwrap();
    let d = dir.join("snapshots/Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(
        d.join("1.json"),
        serde_json::to_string_pretty(&Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 5)).unwrap(),
    )
    .unwrap();
    let stats = import_state(dir, &es, &ss).await.unwrap();
    assert_eq!(stats.snapshots_updated, 0);
}

#[tokio::test]
async fn import_stats_duration_field_present() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let stats = import_state(tmp.path(), &es, &ss).await.unwrap();
    let _ = stats.duration_ms;
}

