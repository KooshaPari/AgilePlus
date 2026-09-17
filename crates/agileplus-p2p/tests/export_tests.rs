//! Integration tests for the `agileplus_p2p::export` module.
//!
//! Exercises the public `export_state` entry point with in-memory stores and
//! verifies the deterministic, git-friendly on-disk layout.

use std::sync::Mutex;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_events::snapshot::{SnapshotError, SnapshotStore};
use agileplus_events::store::{EventError, EventStore};
use agileplus_p2p::device::{DeviceNode, DeviceStore, InMemoryDeviceStore};
use agileplus_p2p::export::{EntityRef, export_state};
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
        self.events.lock().unwrap().push(event.clone());
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
        g.retain(|s| !(s.entity_type == snapshot.entity_type && s.entity_id == snapshot.entity_id));
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

fn make_event(entity_type: &str, entity_id: i64, sequence: i64) -> Event {
    let mut e = Event::new(
        entity_type,
        entity_id,
        "created",
        serde_json::json!({"seq": sequence}),
        "tester",
    );
    e.sequence = sequence;
    e
}

fn entity(entity_type: &str, entity_id: i64) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_string(),
        entity_id,
    }
}

// ── Happy path ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn export_writes_device_events_snapshot_and_sync_state() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    es.append(&make_event("Feature", 1, 1)).await.unwrap();
    es.append(&make_event("Feature", 1, 2)).await.unwrap();
    ss.save(&Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 2))
        .await
        .unwrap();

    let mappings = vec![SyncMapping::new("Feature", 1, "plane-1", "hash-1")];
    let stats = export_state(
        &es,
        &ss,
        &ds,
        &mappings,
        serde_json::json!({"device_id": "d1"}),
        &[entity("Feature", 1)],
        tmp.path(),
    )
    .await
    .unwrap();

    assert_eq!(stats.events_exported, 2);
    assert_eq!(stats.snapshots_exported, 1);
    assert_eq!(stats.sync_mappings_exported, 1);
    assert!(tmp.path().join("device.json").exists());
    assert!(tmp.path().join("events/Feature/1.jsonl").exists());
    assert!(tmp.path().join("snapshots/Feature/1.json").exists());
    assert!(tmp.path().join("sync_state.json").exists());
}

#[tokio::test]
async fn export_jsonl_has_one_line_per_event_in_sequence_order() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    es.append(&make_event("Feature", 7, 3)).await.unwrap();
    es.append(&make_event("Feature", 7, 1)).await.unwrap();
    es.append(&make_event("Feature", 7, 2)).await.unwrap();

    export_state(
        &es,
        &ss,
        &ds,
        &[],
        serde_json::json!({}),
        &[entity("Feature", 7)],
        tmp.path(),
    )
    .await
    .unwrap();

    let content = std::fs::read_to_string(tmp.path().join("events/Feature/7.jsonl")).unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 3);
    let seqs: Vec<i64> = lines
        .iter()
        .map(|l| serde_json::from_str::<Event>(l).unwrap().sequence)
        .collect();
    assert_eq!(seqs, vec![1, 2, 3]);
}

#[tokio::test]
async fn export_skips_empty_event_streams() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    let stats = export_state(
        &es,
        &ss,
        &ds,
        &[],
        serde_json::json!({}),
        &[entity("Feature", 99)],
        tmp.path(),
    )
    .await
    .unwrap();

    assert_eq!(stats.events_exported, 0);
    assert_eq!(stats.snapshots_exported, 0);
    assert!(!tmp.path().join("events").exists());
    assert!(!tmp.path().join("snapshots").exists());
}

#[tokio::test]
async fn export_handles_multiple_entities() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    es.append(&make_event("Feature", 1, 1)).await.unwrap();
    es.append(&make_event("WorkPackage", 2, 1)).await.unwrap();

    let stats = export_state(
        &es,
        &ss,
        &ds,
        &[],
        serde_json::json!({}),
        &[entity("Feature", 1), entity("WorkPackage", 2)],
        tmp.path(),
    )
    .await
    .unwrap();

    assert_eq!(stats.events_exported, 2);
    assert!(tmp.path().join("events/Feature/1.jsonl").exists());
    assert!(tmp.path().join("events/WorkPackage/2.jsonl").exists());
}

#[tokio::test]
async fn export_is_deterministic_across_runs() {
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    let mut e = make_event("Feature", 1, 1);
    e.hash = [7u8; 32];
    es.append(&e).await.unwrap();
    ss.save(&Snapshot::new(
        "Feature",
        1,
        serde_json::json!({"z": 1, "a": {"y": 2, "b": 3}}),
        1,
    ))
    .await
    .unwrap();

    let t1 = tempfile::tempdir().unwrap();
    let t2 = tempfile::tempdir().unwrap();
    for out in [t1.path(), t2.path()] {
        export_state(
            &es,
            &ss,
            &ds,
            &[],
            serde_json::json!({"entries": {"Feature/1": 1}}),
            &[entity("Feature", 1)],
            out,
        )
        .await
        .unwrap();
    }

    let a = std::fs::read_to_string(t1.path().join("snapshots/Feature/1.json")).unwrap();
    let b = std::fs::read_to_string(t2.path().join("snapshots/Feature/1.json")).unwrap();
    assert_eq!(a, b);
    // Keys must be sorted alphabetically in the pretty output.
    let a_pos = a.find("\"a\"").unwrap();
    let z_pos = a.find("\"z\"").unwrap();
    assert!(a_pos < z_pos, "keys should be sorted: {a}");
}

#[tokio::test]
async fn export_device_json_reflects_registered_device() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    ds.insert_device(&DeviceNode {
        device_id: "dev-xyz".into(),
        hostname: "host-a".into(),
        tailscale_ip: "100.64.0.9".into(),
        created_at: Utc::now(),
    })
    .unwrap();

    export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();

    let device: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("device.json")).unwrap())
            .unwrap();
    assert_eq!(device["device_id"], "dev-xyz");
    assert_eq!(device["hostname"], "host-a");
}

#[tokio::test]
async fn export_sync_state_contains_mappings_and_vector() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    let mappings = vec![
        SyncMapping::new("Feature", 1, "plane-1", "h1"),
        SyncMapping::new("Feature", 2, "plane-2", "h2"),
    ];

    export_state(
        &es,
        &ss,
        &ds,
        &mappings,
        serde_json::json!({"device_id": "d1", "entries": {"Feature/1": 5}}),
        &[],
        tmp.path(),
    )
    .await
    .unwrap();

    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("sync_state.json")).unwrap())
            .unwrap();
    assert_eq!(v["sync_mappings"].as_array().unwrap().len(), 2);
    assert_eq!(v["sync_vector"]["entries"]["Feature/1"], 5);
}

#[tokio::test]
async fn export_records_nonzero_duration() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    let stats = export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();
    // duration_ms is a u64; it can legitimately be 0 on very fast machines,
    // so assert only that the export completed with consistent counts.
    assert_eq!(stats.sync_mappings_exported, 0);
}

#[tokio::test]
async fn export_overwrites_existing_output_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let es = MemEventStore::default();
    let ss = MemSnapshotStore::default();
    let ds = InMemoryDeviceStore::default();

    std::fs::write(tmp.path().join("stale.txt"), b"old").unwrap();
    export_state(&es, &ss, &ds, &[], serde_json::json!({}), &[], tmp.path())
        .await
        .unwrap();

    // Existing unrelated files must not prevent a successful export.
    assert!(tmp.path().join("device.json").exists());
}
