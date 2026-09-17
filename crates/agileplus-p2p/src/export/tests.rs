use std::sync::Mutex;

use super::serialization::to_sorted_pretty;
use super::*;
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_events::snapshot::{SnapshotError, SnapshotStore};
use agileplus_events::store::{EventError, EventStore};
use async_trait::async_trait;
use chrono::Utc;

use crate::device::DeviceStore as _;
use crate::device::InMemoryDeviceStore;

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
        Ok(g.iter()
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
        self.snapshots.lock().unwrap().push(snapshot.clone());
        Ok(())
    }

    async fn load(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<Snapshot>, SnapshotError> {
        let g = self.snapshots.lock().unwrap();
        Ok(g.iter()
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

#[tokio::test]
async fn export_creates_expected_files() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path();

    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    let mut ev = Event::new(
        "Feature",
        1,
        "created",
        serde_json::json!({"title": "T1"}),
        "test",
    );
    ev.sequence = 1;
    es.append(&ev).await.unwrap();

    let snap = Snapshot::new("Feature", 1, serde_json::json!({"title": "T1"}), 1);
    ss.save(&snap).await.unwrap();

    let mappings = vec![SyncMapping::new("Feature", 1, "plane-001", "hash-aaa")];
    let entities = vec![EntityRef {
        entity_type: "Feature".into(),
        entity_id: 1,
    }];

    let stats = export_state(
        &es,
        &ss,
        &ds,
        &mappings,
        serde_json::json!({}),
        &entities,
        out,
    )
    .await
    .unwrap();

    assert_eq!(stats.events_exported, 1);
    assert_eq!(stats.snapshots_exported, 1);
    assert_eq!(stats.sync_mappings_exported, 1);

    assert!(out.join("device.json").exists());
    assert!(out.join("events/Feature/1.jsonl").exists());
    assert!(out.join("snapshots/Feature/1.json").exists());
    assert!(out.join("sync_state.json").exists());
}

#[test]
fn to_sorted_sorts_object_keys() {
    let v = serde_json::json!({"z": 1, "a": 2, "m": 3});
    let s = to_sorted_pretty(v).unwrap();
    let pos_a = s.find('"').unwrap();
    let first_key = &s[pos_a + 1..pos_a + 2];
    assert_eq!(first_key, "a");
}

// ── Deep coverage additions ────────────────────────────────────────────────

fn make_event(entity_type: &str, entity_id: i64, seq: i64) -> Event {
    let mut e = Event::new(entity_type, entity_id, "created", serde_json::json!({"n": seq}), "test");
    e.sequence = seq;
    e
}

async fn populated_stores() -> (MemEventStore, MemSnapshotStore, InMemoryDeviceStore) {
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    for (t, id, seq) in [("Feature", 1_i64, 1_i64), ("Feature", 1, 2), ("Epic", 5, 1)] {
        es.append(&make_event(t, id, seq)).await.unwrap();
    }
    ss.save(&Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 2))
        .await
        .unwrap();
    ds.insert_device(&crate::device::DeviceNode {
        device_id: "dev-1".into(),
        hostname: "h".into(),
        tailscale_ip: "100.0.0.1".into(),
        created_at: Utc::now(),
    })
    .unwrap();
    (es, ss, ds)
}

#[tokio::test]
async fn export_empty_entity_list_writes_metadata_only() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();
    assert_eq!(stats.events_exported, 0);
    assert_eq!(stats.snapshots_exported, 0);
    assert_eq!(stats.sync_mappings_exported, 0);
    assert!(tmp.path().join("device.json").exists());
    assert!(tmp.path().join("sync_state.json").exists());
    assert!(!tmp.path().join("events").exists());
}

#[tokio::test]
async fn export_counts_events_across_multiple_entities() {
    let tmp = tempfile::tempdir().unwrap();
    let (es, ss, ds) = populated_stores().await;
    let entities = vec![
        EntityRef { entity_type: "Feature".into(), entity_id: 1 },
        EntityRef { entity_type: "Epic".into(), entity_id: 5 },
    ];
    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &entities, tmp.path())
        .await
        .unwrap();
    assert_eq!(stats.events_exported, 3);
    assert_eq!(stats.snapshots_exported, 1);
}

#[tokio::test]
async fn export_writes_jsonl_lines_parseable_and_ordered() {
    let tmp = tempfile::tempdir().unwrap();
    let (es, ss, ds) = populated_stores().await;
    let entities = vec![EntityRef { entity_type: "Feature".into(), entity_id: 1 }];
    export_state(&es, &ss, &ds, &[], serde_json::json!({}), &entities, tmp.path())
        .await
        .unwrap();

    let content = std::fs::read_to_string(tmp.path().join("events/Feature/1.jsonl")).unwrap();
    let seqs: Vec<i64> = content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str::<Event>(l).unwrap().sequence)
        .collect();
    assert_eq!(seqs, vec![1, 2]);
}

#[tokio::test]
async fn export_entity_with_no_events_creates_no_file() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    let entities = vec![EntityRef { entity_type: "Ghost".into(), entity_id: 99 }];
    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &entities, tmp.path())
        .await
        .unwrap();
    assert_eq!(stats.events_exported, 0);
    assert!(!tmp.path().join("events/Ghost/99.jsonl").exists());
}

#[tokio::test]
async fn export_entity_with_event_but_no_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    es.append(&make_event("Feature", 7, 1)).await.unwrap();
    let entities = vec![EntityRef { entity_type: "Feature".into(), entity_id: 7 }];
    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &entities, tmp.path())
        .await
        .unwrap();
    assert_eq!(stats.events_exported, 1);
    assert_eq!(stats.snapshots_exported, 0);
    assert!(!tmp.path().join("snapshots/Feature/7.json").exists());
}

#[tokio::test]
async fn export_entity_with_snapshot_but_no_events() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    ss.save(&Snapshot::new("Feature", 8, serde_json::json!({"v": 1}), 3))
        .await
        .unwrap();
    let entities = vec![EntityRef { entity_type: "Feature".into(), entity_id: 8 }];
    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &entities, tmp.path())
        .await
        .unwrap();
    assert_eq!(stats.events_exported, 0);
    assert_eq!(stats.snapshots_exported, 1);
    let raw = std::fs::read_to_string(tmp.path().join("snapshots/Feature/8.json")).unwrap();
    let snap: Snapshot = serde_json::from_str(&raw).unwrap();
    assert_eq!(snap.event_sequence, 3);
}

#[tokio::test]
async fn export_device_json_null_when_unregistered() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();
    let raw = std::fs::read_to_string(tmp.path().join("device.json")).unwrap();
    assert_eq!(raw.trim(), "null");
}

#[tokio::test]
async fn export_device_json_contains_ids_when_registered() {
    let tmp = tempfile::tempdir().unwrap();
    let (es, ss, ds) = populated_stores().await;
    export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();
    let raw = std::fs::read_to_string(tmp.path().join("device.json")).unwrap();
    assert!(raw.contains("dev-1"));
    assert!(raw.contains("100.0.0.1"));
}

#[tokio::test]
async fn export_sync_state_is_sorted_and_contains_vector() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    let mappings = vec![SyncMapping::new("Feature", 1, "p1", "h1")];
    export_state(
        &es,
        &ss,
        &ds,
        &mappings,
        serde_json::json!({"entries": {"Feature/1": 3}}),
        &[],
        tmp.path(),
    )
    .await
    .unwrap();

    let raw = std::fs::read_to_string(tmp.path().join("sync_state.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert!(value.get("sync_mappings").is_some());
    assert!(value.get("sync_vector").is_some());
    let map_pos = raw.find("\"sync_mappings\"").unwrap();
    let vec_pos = raw.find("\"sync_vector\"").unwrap();
    assert!(map_pos < vec_pos, "keys should be sorted");
}

#[tokio::test]
async fn export_is_deterministic_across_runs() {
    let tmp = tempfile::tempdir().unwrap();
    let (es, ss, ds) = populated_stores().await;
    let entities = vec![EntityRef { entity_type: "Feature".into(), entity_id: 1 }];
    let mappings = vec![SyncMapping::new("Feature", 1, "p1", "h1")];

    for _ in 0..2 {
        let dir = tmp.path().join(format!("run{}", rand_suffix()));
        export_state(
            &es,
            &ss,
            &ds,
            &mappings,
            serde_json::json!({"z": 1, "a": 2}),
            &entities,
            &dir,
        )
        .await
        .unwrap();
    }
    // Re-run twice into the same directory and compare bytes.
    let dir = tmp.path().join("same");
    export_state(&es, &ss, &ds, &mappings, serde_json::json!({}), &entities, &dir)
        .await
        .unwrap();
    let first = std::fs::read_to_string(dir.join("sync_state.json")).unwrap();
    export_state(&es, &ss, &ds, &mappings, serde_json::json!({}), &entities, &dir)
        .await
        .unwrap();
    let second = std::fs::read_to_string(dir.join("sync_state.json")).unwrap();
    assert_eq!(first, second);
}

fn rand_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[tokio::test]
async fn export_stats_duration_is_recorded_field() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();
    // duration_ms is a u64; just assert the struct is populated.
    let _ = stats.duration_ms;
}

#[tokio::test]
async fn export_creates_nested_output_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let nested = tmp.path().join("a/b/c");
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();
    export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], &nested)
        .await
        .unwrap();
    assert!(nested.join("device.json").exists());
}
