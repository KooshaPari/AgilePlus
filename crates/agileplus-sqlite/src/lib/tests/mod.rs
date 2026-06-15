use super::*;
use agileplus_domain::domain::{
    audit::{AuditEntry, hash_entry},
    backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus, Intent},
    event::Event,
    feature::Feature,
    governance::{
        Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck, PolicyDefinition,
        PolicyDomain, PolicyRule,
    },
    sync_mapping::{SyncDirection, SyncMapping},
    metric::Metric,
    state_machine::FeatureState,
    work_package::{DependencyType, WorkPackage, WpDependency, WpState},
};
use agileplus_events::EventStore;
use agileplus_domain::ports::{ContentStoragePort, StoragePort};

mod feature_work_packages;
mod governance_metrics;
mod modules_cycles;

fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter")
}

fn make_audit_entry(feature_id: i64, prev_hash: [u8; 32]) -> AuditEntry {
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "agent".into(),
        transition: "created->specified".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    entry
}

fn make_date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

fn make_backlog_item(
    title: &str,
    intent: Intent,
    priority: BacklogPriority,
    status: BacklogStatus,
    source: &str,
) -> BacklogItem {
    let mut item = BacklogItem::from_triage(
        title.to_string(),
        format!("{title} description"),
        intent,
        source.to_string(),
    );
    item.priority = priority;
    item.status = status;
    item.feature_slug = Some("feat-coverage".to_string());
    item.tags = vec!["coverage".to_string(), priority.to_string()];
    item
}

#[tokio::test]
async fn backlog_create_get_list_update_and_pop() {
    let adapter = make_adapter();
    let low_id = adapter
        .create_backlog_item(&make_backlog_item(
            "docs cleanup",
            Intent::Docs,
            BacklogPriority::Low,
            BacklogStatus::New,
            "manual",
        ))
        .await
        .expect("create low backlog");
    let critical_id = adapter
        .create_backlog_item(&make_backlog_item(
            "production bug",
            Intent::Bug,
            BacklogPriority::Critical,
            BacklogStatus::New,
            "inbox",
        ))
        .await
        .expect("create critical backlog");

    let fetched = adapter
        .get_backlog_item(critical_id)
        .await
        .expect("get backlog")
        .expect("backlog exists");
    assert_eq!(fetched.title, "production bug");
    assert_eq!(fetched.intent, Intent::Bug);
    assert_eq!(fetched.priority, BacklogPriority::Critical);
    assert_eq!(fetched.tags, vec!["coverage", "critical"]);

    let filtered = adapter
        .list_backlog_items(&BacklogFilters {
            intent: Some(Intent::Bug),
            status: Some(BacklogStatus::New),
            priority: Some(BacklogPriority::Critical),
            feature_slug: Some("feat-coverage".to_string()),
            source: Some("inbox".to_string()),
            sort: BacklogSort::Priority,
            limit: Some(1),
        })
        .await
        .expect("list filtered backlog");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, Some(critical_id));

    adapter
        .update_backlog_priority(low_id, BacklogPriority::High)
        .await
        .expect("update priority");
    adapter
        .update_backlog_status(low_id, BacklogStatus::InProgress)
        .await
        .expect("update status");
    let updated = adapter
        .get_backlog_item(low_id)
        .await
        .expect("get updated backlog")
        .expect("updated backlog exists");
    assert_eq!(updated.priority, BacklogPriority::High);
    assert_eq!(updated.status, BacklogStatus::InProgress);

    let popped = adapter
        .pop_next_backlog_item()
        .await
        .expect("pop next backlog")
        .expect("new item available");
    assert_eq!(popped.id, Some(critical_id));
    assert_eq!(popped.status, BacklogStatus::Triaged);

    assert!(adapter.get_backlog_item(9999).await.expect("missing lookup").is_none());
}

#[tokio::test]
async fn event_store_queries_sequence_since_and_ranges() {
    let adapter = make_adapter();
    let mut first = Event::new("feature", 7, "created", serde_json::json!({"n": 1}), "agent");
    first.sequence = 1;
    first.hash = [1u8; 32];
    let mut second = Event::new("feature", 7, "updated", serde_json::json!({"n": 2}), "agent");
    second.sequence = 2;
    second.prev_hash = first.hash;
    second.hash = [2u8; 32];

    let first_id = adapter.append(&first).await.expect("append first");
    let second_id = adapter.append(&second).await.expect("append second");
    assert!(second_id > first_id);

    let all = adapter
        .get_events("feature", 7)
        .await
        .expect("get events");
    assert_eq!(all.iter().map(|e| e.sequence).collect::<Vec<_>>(), vec![1, 2]);

    let since = adapter
        .get_events_since("feature", 7, 1)
        .await
        .expect("get events since");
    assert_eq!(since.len(), 1);
    assert_eq!(since[0].event_type, "updated");

    let ranged = adapter
        .get_events_by_range(
            "feature",
            7,
            &first.timestamp.to_rfc3339(),
            &second.timestamp.to_rfc3339(),
        )
        .await
        .expect("get events by range");
    assert_eq!(ranged.len(), 2);
    assert_eq!(
        adapter
            .get_latest_sequence("feature", 7)
            .await
            .expect("latest sequence"),
        2
    );
    assert_eq!(
        adapter
            .get_latest_sequence("missing", 7)
            .await
            .expect("missing latest sequence"),
        0
    );
}

#[tokio::test]
async fn sync_mapping_upsert_get_by_plane_id_and_delete() {
    let adapter = make_adapter();
    let mut mapping = SyncMapping::new("feature", 42, "plane-1", "hash-a");
    mapping.sync_direction = SyncDirection::Push;
    mapping.conflict_count = 2;

    adapter
        .upsert_sync_mapping(&mapping)
        .await
        .expect("insert mapping");
    let fetched = adapter
        .get_sync_mapping("feature", 42)
        .await
        .expect("get mapping")
        .expect("mapping exists");
    assert_eq!(fetched.plane_issue_id, "plane-1");
    assert_eq!(fetched.sync_direction, SyncDirection::Push);
    assert_eq!(fetched.conflict_count, 2);

    mapping.plane_issue_id = "plane-2".to_string();
    mapping.content_hash = "hash-b".to_string();
    mapping.sync_direction = SyncDirection::Pull;
    adapter
        .upsert_sync_mapping(&mapping)
        .await
        .expect("update mapping");

    let by_plane = adapter
        .get_sync_mapping_by_plane_id("feature", "plane-2")
        .await
        .expect("get by plane")
        .expect("updated mapping exists");
    assert_eq!(by_plane.entity_id, 42);
    assert_eq!(by_plane.content_hash, "hash-b");
    assert_eq!(by_plane.sync_direction, SyncDirection::Pull);

    adapter
        .delete_sync_mapping("feature", 42)
        .await
        .expect("delete mapping");
    assert!(adapter
        .get_sync_mapping("feature", 42)
        .await
        .expect("get deleted mapping")
        .is_none());
}
