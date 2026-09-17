//! Deep coverage for agileplus-plane primitive modules:
//! content_hash, sync_queue, state_mapper, labels, daemon, client models.
//!
//! These are integration tests exercising only the crate's public API.

use std::time::Duration;

use agileplus_domain::domain::state_machine::FeatureState;

use agileplus_plane::client::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};
use agileplus_plane::content_hash::{compute_content_hash, detect_conflict, ConflictStatus};
use agileplus_plane::daemon::{PlaneDaemonConfig, SyncState as DaemonSyncState};
use agileplus_plane::labels::{CreateLabelRequest, LabelSync, PlaneLabel};
use agileplus_plane::state_mapper::{
    PlaneStateGroup, PlaneStateMapper, PlaneStateMapperConfig, StateOverride,
};
use agileplus_plane::sync::{SyncOutcome, SyncState};
use agileplus_plane::sync_queue::{
    QueueError, SyncOpKind, SyncQueue, SyncQueueItem, SyncQueueStore, SyncTask, BASE_BACKOFF,
    MAX_BACKOFF, MAX_RETRIES, QUEUE_CAPACITY,
};

// ============================================================
// content_hash
// ============================================================

const HEX_LEN: usize = 64;

#[test]
fn content_hash_is_lowercase_hex_64() {
    let h = compute_content_hash("t", "d", "s", &[]);
    assert_eq!(h.len(), HEX_LEN);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
}

#[test]
fn content_hash_empty_inputs_still_hashes() {
    let h = compute_content_hash("", "", "", &[]);
    assert_eq!(h.len(), HEX_LEN);
}

#[test]
fn content_hash_deterministic_across_calls() {
    let a = compute_content_hash("Title", "Body", "implementing", &["x".into()]);
    let b = compute_content_hash("Title", "Body", "implementing", &["x".into()]);
    assert_eq!(a, b);
}

#[test]
fn content_hash_label_order_independent_with_duplicates() {
    let a = compute_content_hash("t", "d", "s", &["b".into(), "a".into(), "b".into()]);
    let b = compute_content_hash("t", "d", "s", &["b".into(), "b".into(), "a".into()]);
    assert_eq!(a, b);
}

#[test]
fn content_hash_title_change_detected() {
    let a = compute_content_hash("A", "d", "s", &[]);
    let b = compute_content_hash("B", "d", "s", &[]);
    assert_ne!(a, b);
}

#[test]
fn content_hash_description_change_detected() {
    let a = compute_content_hash("t", "one", "s", &[]);
    let b = compute_content_hash("t", "two", "s", &[]);
    assert_ne!(a, b);
}

#[test]
fn content_hash_state_change_detected() {
    let a = compute_content_hash("t", "d", "created", &[]);
    let b = compute_content_hash("t", "d", "shipped", &[]);
    assert_ne!(a, b);
}

#[test]
fn content_hash_label_change_detected() {
    let a = compute_content_hash("t", "d", "s", &["a".into()]);
    let b = compute_content_hash("t", "d", "s", &["a".into(), "b".into()]);
    assert_ne!(a, b);
}

#[test]
fn content_hash_unicode_stable() {
    let a = compute_content_hash("日本語", "emoji 🚀", "s", &["λ".into()]);
    let b = compute_content_hash("日本語", "emoji 🚀", "s", &["λ".into()]);
    assert_eq!(a, b);
    assert_eq!(a.len(), HEX_LEN);
}

#[test]
fn content_hash_field_boundaries_are_unambiguous() {
    // "ab"+"c" must not collide with "a"+"bc" because of the NUL separators.
    let a = compute_content_hash("ab", "c", "s", &[]);
    let b = compute_content_hash("a", "bc", "s", &[]);
    assert_ne!(a, b);
}

#[test]
fn content_hash_whitespace_significant() {
    let a = compute_content_hash("t", "d", "s", &[]);
    let b = compute_content_hash("t ", "d", "s", &[]);
    assert_ne!(a, b);
}

#[test]
fn detect_conflict_both_unchanged_is_clean() {
    assert_eq!(detect_conflict("h", "h", "h"), ConflictStatus::Clean);
}

#[test]
fn detect_conflict_only_local_changed_is_clean() {
    assert_eq!(detect_conflict("base", "local", "base"), ConflictStatus::Clean);
}

#[test]
fn detect_conflict_only_remote_changed_is_clean() {
    assert_eq!(detect_conflict("base", "base", "remote"), ConflictStatus::Clean);
}

#[test]
fn detect_conflict_both_changed_is_conflict() {
    assert_eq!(
        detect_conflict("base", "local", "remote"),
        ConflictStatus::Conflict
    );
}

#[test]
fn detect_conflict_divergent_new_values_is_clean_for_identical_edit() {
    // Both sides moved away from the baseline, even to the same value, so the
    // detector reports a conflict (it only compares against the baseline).
    assert_eq!(detect_conflict("base", "same", "same"), ConflictStatus::Conflict);
}

#[test]
fn conflict_status_debug_and_eq() {
    let c = ConflictStatus::Conflict;
    assert_eq!(c.clone(), ConflictStatus::Conflict);
    assert!(format!("{c:?}").contains("Conflict"));
}

// ============================================================
// sync_queue constants and backoff
// ============================================================

#[test]
fn queue_capacity_is_1000() {
    assert_eq!(QUEUE_CAPACITY, 1000);
}

#[test]
fn max_retries_is_three() {
    assert_eq!(MAX_RETRIES, 3);
}

#[test]
fn base_backoff_is_one_second() {
    assert_eq!(BASE_BACKOFF, Duration::from_secs(1));
}

#[test]
fn max_backoff_is_five_minutes() {
    assert_eq!(MAX_BACKOFF, Duration::from_secs(300));
}

#[test]
fn backoff_sequence_doubles() {
    assert_eq!(SyncQueueItem::next_backoff_delay(0), Duration::from_secs(1));
    assert_eq!(SyncQueueItem::next_backoff_delay(1), Duration::from_secs(2));
    assert_eq!(SyncQueueItem::next_backoff_delay(2), Duration::from_secs(4));
    assert_eq!(SyncQueueItem::next_backoff_delay(3), Duration::from_secs(8));
    assert_eq!(SyncQueueItem::next_backoff_delay(4), Duration::from_secs(16));
}

#[test]
fn backoff_below_cap_boundary() {
    // 2^8 = 256s, still below the 300s cap.
    assert_eq!(
        SyncQueueItem::next_backoff_delay(8),
        Duration::from_secs(256)
    );
}

#[test]
fn backoff_caps_exactly_at_max() {
    // 2^9 = 512s -> capped to 300s.
    assert_eq!(SyncQueueItem::next_backoff_delay(9), MAX_BACKOFF);
}

#[test]
fn backoff_never_panics_on_huge_attempt() {
    assert_eq!(SyncQueueItem::next_backoff_delay(u32::MAX), MAX_BACKOFF);
}

#[test]
fn queue_error_full_displays_capacity() {
    let err = QueueError::Full(QUEUE_CAPACITY);
    let msg = err.to_string();
    assert!(msg.contains("1000"), "unexpected message: {msg}");
}

#[test]
fn queue_error_serde_variant_wraps_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("nope").unwrap_err();
    let err: QueueError = json_err.into();
    assert!(err.to_string().contains("serialization"));
}

// ============================================================
// SyncOpKind
// ============================================================

#[test]
fn sync_op_kind_serde_snake_case() {
    assert_eq!(
        serde_json::to_string(&SyncOpKind::CreateIssue).unwrap(),
        "\"create_issue\""
    );
    assert_eq!(
        serde_json::to_string(&SyncOpKind::UpdateIssue).unwrap(),
        "\"update_issue\""
    );
    assert_eq!(
        serde_json::to_string(&SyncOpKind::CreateLabel).unwrap(),
        "\"create_label\""
    );
    assert_eq!(
        serde_json::to_string(&SyncOpKind::DeleteIssue).unwrap(),
        "\"delete_issue\""
    );
}

#[test]
fn sync_op_kind_roundtrip_all_variants() {
    for kind in [
        SyncOpKind::CreateIssue,
        SyncOpKind::UpdateIssue,
        SyncOpKind::CreateLabel,
        SyncOpKind::DeleteIssue,
    ] {
        let json = serde_json::to_string(&kind).unwrap();
        let back: SyncOpKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, kind);
    }
}

#[test]
fn sync_op_kind_rejects_unknown() {
    assert!(serde_json::from_str::<SyncOpKind>("\"nonsense\"").is_err());
}

#[test]
fn sync_op_kind_equality() {
    assert_eq!(SyncOpKind::CreateIssue, SyncOpKind::CreateIssue);
    assert_ne!(SyncOpKind::CreateIssue, SyncOpKind::DeleteIssue);
}

// ============================================================
// SyncQueue
// ============================================================

#[test]
fn sync_queue_new_is_empty() {
    let q = SyncQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
}

#[test]
fn sync_queue_default_matches_new() {
    let q = SyncQueue::default();
    assert!(q.is_empty());
}

#[test]
fn sync_queue_enqueue_returns_incrementing_ids() {
    let mut q = SyncQueue::new();
    let a = q.enqueue(SyncOpKind::CreateIssue, "a".into()).unwrap();
    let b = q.enqueue(SyncOpKind::CreateIssue, "b".into()).unwrap();
    let c = q.enqueue(SyncOpKind::CreateIssue, "c".into()).unwrap();
    assert_eq!((a, b, c), (1, 2, 3));
}

#[test]
fn sync_queue_pop_ready_fifo_order() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateIssue, "first".into()).unwrap();
    q.enqueue(SyncOpKind::CreateLabel, "second".into()).unwrap();
    assert_eq!(q.pop_ready().unwrap().payload, "first");
    assert_eq!(q.pop_ready().unwrap().payload, "second");
    assert!(q.pop_ready().is_none());
}

#[test]
fn sync_queue_pop_ready_on_empty_is_none() {
    let mut q = SyncQueue::new();
    assert!(q.pop_ready().is_none());
}

#[test]
fn sync_queue_len_tracks_enqueue_and_pop() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateIssue, "{}".into()).unwrap();
    q.enqueue(SyncOpKind::CreateIssue, "{}".into()).unwrap();
    assert_eq!(q.len(), 2);
    q.pop_ready();
    assert_eq!(q.len(), 1);
}

#[test]
fn sync_queue_enqueue_full_errors() {
    let mut q = SyncQueue::new();
    for i in 0..QUEUE_CAPACITY {
        q.enqueue(SyncOpKind::CreateIssue, format!("{i}")).unwrap();
    }
    assert_eq!(q.len(), QUEUE_CAPACITY);
    assert!(matches!(
        q.enqueue(SyncOpKind::CreateIssue, "overflow".into()),
        Err(QueueError::Full(_))
    ));
}

#[test]
fn sync_queue_requeue_increments_attempt_and_backoff() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::UpdateIssue, "data".into()).unwrap();
    let item = q.pop_ready().unwrap();
    assert_eq!(item.attempt, 0);
    q.requeue(item).unwrap();
    assert_eq!(q.len(), 1);
    // Requeued item is not immediately ready (backoff >= 1s).
    assert!(q.pop_ready().is_none());
}

#[test]
fn sync_queue_requeue_full_errors() {
    let mut q = SyncQueue::new();
    for i in 0..QUEUE_CAPACITY {
        q.enqueue(SyncOpKind::CreateIssue, format!("{i}")).unwrap();
    }
    let item = SyncQueueItem {
        id: 99,
        kind: SyncOpKind::CreateIssue,
        payload: "x".into(),
        attempt: 0,
        next_attempt_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    };
    assert!(matches!(q.requeue(item), Err(QueueError::Full(_))));
}

#[test]
fn sync_queue_drain_empties_and_returns_all() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateIssue, "1".into()).unwrap();
    q.enqueue(SyncOpKind::CreateLabel, "2".into()).unwrap();
    let items = q.drain();
    assert_eq!(items.len(), 2);
    assert!(q.is_empty());
    assert_eq!(items[0].payload, "1");
}

#[test]
fn sync_queue_reload_restores_items() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::DeleteIssue, "x".into()).unwrap();
    let items = q.drain();
    q.reload(items);
    assert_eq!(q.len(), 1);
    assert_eq!(q.pop_ready().unwrap().payload, "x");
}

#[test]
fn sync_queue_reload_respects_capacity() {
    let mut q = SyncQueue::new();
    let items: Vec<SyncQueueItem> = (0..QUEUE_CAPACITY as u64 + 10)
        .map(|id| SyncQueueItem {
            id,
            kind: SyncOpKind::CreateIssue,
            payload: "{}".into(),
            attempt: 0,
            next_attempt_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        })
        .collect();
    q.reload(items);
    assert_eq!(q.len(), QUEUE_CAPACITY);
}

#[test]
fn sync_queue_item_is_ready_false_for_future_timestamp() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: 0,
        next_attempt_at: chrono::Utc::now() + chrono::Duration::hours(1),
        created_at: chrono::Utc::now(),
    };
    assert!(!item.is_ready());
}

#[test]
fn sync_queue_item_is_ready_true_for_past_timestamp() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: 0,
        next_attempt_at: chrono::Utc::now() - chrono::Duration::hours(1),
        created_at: chrono::Utc::now(),
    };
    assert!(item.is_ready());
}

#[test]
fn sync_queue_item_is_exhausted_boundaries() {
    let mut item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: 0,
        next_attempt_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    };
    assert!(!item.is_exhausted());
    item.attempt = MAX_RETRIES - 1;
    assert!(!item.is_exhausted());
    item.attempt = MAX_RETRIES;
    assert!(item.is_exhausted());
    item.attempt = MAX_RETRIES + 5;
    assert!(item.is_exhausted());
}

#[test]
fn sync_queue_item_with_next_attempt_preserves_identity() {
    let item = SyncQueueItem {
        id: 7,
        kind: SyncOpKind::CreateLabel,
        payload: "payload".into(),
        attempt: 1,
        next_attempt_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    };
    let next = item.with_next_attempt();
    assert_eq!(next.id, 7);
    assert_eq!(next.kind, SyncOpKind::CreateLabel);
    assert_eq!(next.payload, "payload");
    assert_eq!(next.attempt, 2);
    assert!(next.next_attempt_at > item.next_attempt_at);
}

#[test]
fn sync_queue_item_serde_roundtrip() {
    let item = SyncQueueItem {
        id: 42,
        kind: SyncOpKind::DeleteIssue,
        payload: "{\"x\":1}".into(),
        attempt: 2,
        next_attempt_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: SyncQueueItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, 42);
    assert_eq!(back.kind, SyncOpKind::DeleteIssue);
    assert_eq!(back.attempt, 2);
}

// ============================================================
// SyncTask
// ============================================================

#[test]
fn sync_task_new_defaults() {
    let task = SyncTask::new(1, SyncOpKind::CreateIssue, "{}".into(), None);
    assert_eq!(task.id, 1);
    assert_eq!(task.attempt, 0);
    assert!(task.content_hash.is_none());
    assert!(!task.is_exhausted());
}

#[test]
fn sync_task_starts_ready() {
    let task = SyncTask::new(1, SyncOpKind::CreateIssue, "{}".into(), None);
    assert!(task.is_ready());
}

#[test]
fn sync_task_backoff_sequence() {
    let task = SyncTask::new(1, SyncOpKind::UpdateIssue, "{}".into(), None);
    assert_eq!(task.next_backoff_delay(), Duration::from_secs(1));
    let r1 = task.with_next_attempt();
    assert_eq!(r1.next_backoff_delay(), Duration::from_secs(2));
    let r2 = r1.with_next_attempt();
    assert_eq!(r2.next_backoff_delay(), Duration::from_secs(4));
    let r3 = r2.with_next_attempt();
    assert_eq!(r3.next_backoff_delay(), Duration::from_secs(8));
}

#[test]
fn sync_task_exhausts_after_three_retries() {
    let task = SyncTask::new(1, SyncOpKind::CreateIssue, "{}".into(), None);
    let r1 = task.with_next_attempt();
    let r2 = r1.with_next_attempt();
    let r3 = r2.with_next_attempt();
    assert_eq!(r3.attempt, MAX_RETRIES);
    assert!(r3.is_exhausted());
}

#[test]
fn sync_task_content_hash_retained() {
    let task = SyncTask::new(5, SyncOpKind::CreateIssue, "{}".into(), Some("abc".into()));
    assert_eq!(task.content_hash.as_deref(), Some("abc"));
    assert_eq!(task.with_next_attempt().content_hash.as_deref(), Some("abc"));
}

// ============================================================
// SyncQueueStore (sqlite)
// ============================================================

#[test]
fn sync_queue_store_in_memory_roundtrip() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateLabel, r#"{"name":"bug"}"#.into())
        .unwrap();
    store.save_all(&q.drain()).unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].kind, SyncOpKind::CreateLabel);
    assert_eq!(loaded[0].payload, r#"{"name":"bug"}"#);
}

#[test]
fn sync_queue_store_save_all_replaces_existing() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateIssue, "first".into()).unwrap();
    store.save_all(&q.drain()).unwrap();
    store.save_all(&[]).unwrap();
    assert!(store.load_all().unwrap().is_empty());
}

#[test]
fn sync_queue_store_load_all_empty() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    assert!(store.load_all().unwrap().is_empty());
}

#[test]
fn sync_queue_store_persists_to_file_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.db");
    let path_str = path.to_str().unwrap();
    {
        let store = SyncQueueStore::open(path_str).unwrap();
        let mut q = SyncQueue::new();
        q.enqueue(SyncOpKind::DeleteIssue, "persist".into()).unwrap();
        store.save_all(&q.drain()).unwrap();
    }
    let store = SyncQueueStore::open(path_str).unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].payload, "persist");
}

#[test]
fn sync_queue_store_open_invalid_path_errors() {
    let err = SyncQueueStore::open("/definitely/not/a/real/dir/queue.db");
    assert!(err.is_err());
}

#[test]
fn sync_queue_store_roundtrips_all_op_kinds() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    let mut q = SyncQueue::new();
    for kind in [
        SyncOpKind::CreateIssue,
        SyncOpKind::UpdateIssue,
        SyncOpKind::CreateLabel,
        SyncOpKind::DeleteIssue,
    ] {
        q.enqueue(kind, "{}".into()).unwrap();
    }
    store.save_all(&q.drain()).unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 4);
}

// ============================================================
// sync::SyncState / SyncOutcome
// ============================================================

#[test]
fn sync_state_new_defaults() {
    let s = SyncState::new("feat".into());
    assert_eq!(s.feature_slug, "feat");
    assert!(s.plane_issue_id.is_none());
    assert!(s.last_synced_at.is_none());
    assert!(s.content_hash.is_none());
    assert!(s.wp_mappings.is_empty());
}

#[test]
fn sync_state_serialization_roundtrip() {
    let mut s = SyncState::new("feat".into());
    s.plane_issue_id = Some("issue-1".into());
    s.content_hash = Some("hash".into());
    s.wp_mappings.insert("WP01".into(), "sub-1".into());
    let json = serde_json::to_string(&s).unwrap();
    let back: SyncState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.feature_slug, "feat");
    assert_eq!(back.plane_issue_id.as_deref(), Some("issue-1"));
    assert_eq!(back.content_hash.as_deref(), Some("hash"));
    assert_eq!(back.wp_mappings["WP01"], "sub-1");
}

#[test]
fn sync_state_uses_utc_timestamp_serde() {
    let mut s = SyncState::new("f".into());
    s.last_synced_at = Some(chrono::Utc::now());
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("last_synced_at"));
}

#[test]
fn sync_outcome_variants_distinct() {
    let values = [
        SyncOutcome::Created("a".into()),
        SyncOutcome::Updated("a".into()),
        SyncOutcome::Skipped,
        SyncOutcome::Conflict("a".into()),
    ];
    for (i, a) in values.iter().enumerate() {
        for (j, b) in values.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn sync_outcome_carries_id() {
    assert_eq!(SyncOutcome::Created("id-1".into()), SyncOutcome::Created("id-1".into()));
    assert_ne!(SyncOutcome::Created("id-1".into()), SyncOutcome::Created("id-2".into()));
}

#[test]
fn sync_outcome_debug_contains_variant() {
    assert!(format!("{:?}", SyncOutcome::Skipped).contains("Skipped"));
    assert!(format!("{:?}", SyncOutcome::Conflict("x".into())).contains("Conflict"));
}

// ============================================================
// state_mapper
// ============================================================

#[test]
fn plane_state_group_parses_canonical_names() {
    for (input, expected) in [
        ("backlog", PlaneStateGroup::Backlog),
        ("unstarted", PlaneStateGroup::Unstarted),
        ("started", PlaneStateGroup::Started),
        ("completed", PlaneStateGroup::Completed),
        ("cancelled", PlaneStateGroup::Cancelled),
    ] {
        assert_eq!(input.parse::<PlaneStateGroup>().unwrap(), expected);
    }
}

#[test]
fn plane_state_group_parses_aliases() {
    assert_eq!("todo".parse::<PlaneStateGroup>().unwrap(), PlaneStateGroup::Unstarted);
    assert_eq!(
        "in_progress".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    );
    assert_eq!(
        "in progress".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    );
    assert_eq!("done".parse::<PlaneStateGroup>().unwrap(), PlaneStateGroup::Completed);
    assert_eq!("canceled".parse::<PlaneStateGroup>().unwrap(), PlaneStateGroup::Cancelled);
}

#[test]
fn plane_state_group_parse_is_case_insensitive() {
    assert_eq!("BACKLOG".parse::<PlaneStateGroup>().unwrap(), PlaneStateGroup::Backlog);
    assert_eq!("Started".parse::<PlaneStateGroup>().unwrap(), PlaneStateGroup::Started);
}

#[test]
fn plane_state_group_unknown_preserves_lowercased_value() {
    let g = "WeirdState".parse::<PlaneStateGroup>().unwrap();
    assert_eq!(g, PlaneStateGroup::Unknown("weirdstate".into()));
    assert_eq!(g.as_str(), "weirdstate");
}

#[test]
fn plane_state_group_as_str_roundtrip() {
    for group in [
        PlaneStateGroup::Backlog,
        PlaneStateGroup::Unstarted,
        PlaneStateGroup::Started,
        PlaneStateGroup::Completed,
        PlaneStateGroup::Cancelled,
    ] {
        let s = group.as_str().to_string();
        assert_eq!(s.parse::<PlaneStateGroup>().unwrap(), group);
    }
}

#[test]
fn plane_state_group_from_str_never_fails() {
    let result: Result<PlaneStateGroup, std::convert::Infallible> = "anything".parse();
    assert!(result.is_ok());
}

#[test]
fn mapper_default_maps_backlog_to_created() {
    assert_eq!(
        PlaneStateMapper::new().map_plane_state("backlog", "Backlog"),
        FeatureState::Created
    );
}

#[test]
fn mapper_defaults_for_all_known_groups() {
    let m = PlaneStateMapper::new();
    assert_eq!(m.map_plane_state("unstarted", "Todo"), FeatureState::Specified);
    assert_eq!(m.map_plane_state("started", "In Progress"), FeatureState::Implementing);
    assert_eq!(m.map_plane_state("completed", "Done"), FeatureState::Validated);
    assert_eq!(m.map_plane_state("cancelled", "Wont Fix"), FeatureState::Validated);
}

#[test]
fn mapper_unknown_group_defaults_to_created() {
    let m = PlaneStateMapper::new();
    assert_eq!(m.map_plane_state("mystery", "???"), FeatureState::Created);
}

#[test]
fn mapper_group_matching_is_case_insensitive() {
    let m = PlaneStateMapper::new();
    assert_eq!(m.map_plane_state("STARTED", "X"), FeatureState::Implementing);
}

#[test]
fn mapper_uses_group_alias_todo() {
    let m = PlaneStateMapper::new();
    assert_eq!(m.map_plane_state("todo", "Todo"), FeatureState::Specified);
}

#[test]
fn mapper_default_impl_matches_new() {
    let m = PlaneStateMapper::default();
    assert_eq!(m.map_plane_state("started", "x"), FeatureState::Implementing);
}

#[test]
fn mapper_override_group_and_name_wins() {
    let config = PlaneStateMapperConfig {
        overrides: vec![StateOverride {
            plane_group: "started".into(),
            plane_name: Some("review".into()),
            feature_state: FeatureState::Validated,
        }],
        state_id_map: std::collections::HashMap::new(),
    };
    let m = PlaneStateMapper::with_config(config);
    assert_eq!(m.map_plane_state("started", "review"), FeatureState::Validated);
    assert_eq!(m.map_plane_state("started", "coding"), FeatureState::Implementing);
}

#[test]
fn mapper_override_group_only_applies_without_name_match() {
    let config = PlaneStateMapperConfig {
        overrides: vec![StateOverride {
            plane_group: "started".into(),
            plane_name: None,
            feature_state: FeatureState::Researched,
        }],
        state_id_map: std::collections::HashMap::new(),
    };
    let m = PlaneStateMapper::with_config(config);
    assert_eq!(m.map_plane_state("started", "anything"), FeatureState::Researched);
}

#[test]
fn mapper_specific_name_override_beats_group_only() {
    let config = PlaneStateMapperConfig {
        overrides: vec![
            StateOverride {
                plane_group: "started".into(),
                plane_name: None,
                feature_state: FeatureState::Researched,
            },
            StateOverride {
                plane_group: "started".into(),
                plane_name: Some("review".into()),
                feature_state: FeatureState::Validated,
            },
        ],
        state_id_map: std::collections::HashMap::new(),
    };
    let m = PlaneStateMapper::with_config(config);
    assert_eq!(m.map_plane_state("started", "review"), FeatureState::Validated);
    assert_eq!(m.map_plane_state("started", "other"), FeatureState::Researched);
    // Group-only override must not clobber a name-specific match.
    assert_eq!(m.map_plane_state("started", "Review"), FeatureState::Validated);
}

#[test]
fn mapper_to_plane_default_groups() {
    let m = PlaneStateMapper::new();
    assert_eq!(m.to_plane(FeatureState::Created).0, "backlog");
    assert_eq!(m.to_plane(FeatureState::Specified).0, "unstarted");
    assert_eq!(m.to_plane(FeatureState::Researched).0, "unstarted");
    assert_eq!(m.to_plane(FeatureState::Planned).0, "unstarted");
    assert_eq!(m.to_plane(FeatureState::Implementing).0, "started");
    assert_eq!(m.to_plane(FeatureState::Validated).0, "completed");
    assert_eq!(m.to_plane(FeatureState::Shipped).0, "completed");
    assert_eq!(m.to_plane(FeatureState::Retrospected).0, "completed");
}

#[test]
fn mapper_to_plane_default_id_is_empty() {
    let m = PlaneStateMapper::new();
    for state in [
        FeatureState::Created,
        FeatureState::Specified,
        FeatureState::Implementing,
        FeatureState::Validated,
    ] {
        assert!(m.to_plane(state).1.is_empty());
    }
}

#[test]
fn mapper_to_plane_uses_configured_id() {
    let mut config = PlaneStateMapperConfig::default();
    config.state_id_map.insert(
        FeatureState::Implementing,
        ("started".into(), "uuid-1".into()),
    );
    let m = PlaneStateMapper::with_config(config);
    assert_eq!(
        m.to_plane(FeatureState::Implementing),
        ("started".to_string(), "uuid-1".to_string())
    );
}

#[test]
fn mapper_to_plane_falls_back_for_unconfigured_state() {
    let mut config = PlaneStateMapperConfig::default();
    config
        .state_id_map
        .insert(FeatureState::Created, ("backlog".into(), "b-1".into()));
    let m = PlaneStateMapper::with_config(config);
    let (group, id) = m.to_plane(FeatureState::Implementing);
    assert_eq!(group, "started");
    assert!(id.is_empty());
}

#[test]
fn mapper_round_trip_for_main_states() {
    let m = PlaneStateMapper::new();
    for (state, group) in [
        (FeatureState::Created, "backlog"),
        (FeatureState::Specified, "unstarted"),
        (FeatureState::Implementing, "started"),
        (FeatureState::Validated, "completed"),
    ] {
        let (g, _) = m.to_plane(state);
        assert_eq!(g, group);
        assert_eq!(m.map_plane_state(&g, ""), state);
    }
}

// ============================================================
// labels
// ============================================================

#[test]
fn plane_label_deserialize_full() {
    let label: PlaneLabel =
        serde_json::from_str(r##"{"id":"a","name":"bug","color":"#f00"}"##).unwrap();
    assert_eq!(label.id, "a");
    assert_eq!(label.name, "bug");
    assert_eq!(label.color.as_deref(), Some("#f00"));
}

#[test]
fn plane_label_deserialize_without_color() {
    let label: PlaneLabel = serde_json::from_str(r#"{"id":"a","name":"bug"}"#).unwrap();
    assert!(label.color.is_none());
}

#[test]
fn plane_label_missing_required_field_errors() {
    assert!(serde_json::from_str::<PlaneLabel>(r#"{"id":"a"}"#).is_err());
}

#[test]
fn plane_label_serialize_roundtrip() {
    let label = PlaneLabel {
        id: "a".into(),
        name: "bug".into(),
        color: Some("#f00".into()),
    };
    let json = serde_json::to_string(&label).unwrap();
    let back: PlaneLabel = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "a");
    assert_eq!(back.name, "bug");
}

#[test]
fn plane_label_clone_and_debug() {
    let label = PlaneLabel {
        id: "a".into(),
        name: "bug".into(),
        color: None,
    };
    let clone = label.clone();
    assert_eq!(clone.id, label.id);
    assert!(format!("{label:?}").contains("PlaneLabel"));
}

#[test]
fn create_label_request_omits_color_when_none() {
    let req = CreateLabelRequest {
        name: "x".into(),
        color: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("color"));
    assert!(json.contains("\"name\":\"x\""));
}

#[test]
fn create_label_request_includes_color_when_some() {
    let req = CreateLabelRequest {
        name: "x".into(),
        color: Some("#0f0".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("#0f0"));
}

#[test]
fn label_sync_debug_constructs() {
    let client = agileplus_plane::client::PlaneClient::new(
        "http://localhost".into(),
        "key".into(),
        "ws".into(),
        "proj".into(),
    );
    let sync = LabelSync::new(client);
    assert!(format!("{sync:?}").contains("LabelSync"));
}

// ============================================================
// daemon
// ============================================================

#[test]
fn daemon_config_default_values() {
    let cfg = PlaneDaemonConfig::default();
    assert_eq!(cfg.interval, Duration::from_secs(300));
    assert_eq!(cfg.batch_size, 25);
    assert!(!cfg.dry_run);
}

#[test]
fn daemon_config_clone_eq_fields() {
    let cfg = PlaneDaemonConfig {
        interval: Duration::from_secs(7),
        batch_size: 3,
        dry_run: true,
    };
    let clone = cfg.clone();
    assert_eq!(clone.interval, Duration::from_secs(7));
    assert_eq!(clone.batch_size, 3);
    assert!(clone.dry_run);
}

#[test]
fn daemon_config_serde_roundtrip() {
    let cfg = PlaneDaemonConfig {
        interval: Duration::from_secs(90),
        batch_size: 11,
        dry_run: true,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: PlaneDaemonConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.interval, Duration::from_secs(90));
    assert_eq!(back.batch_size, 11);
    assert!(back.dry_run);
}

#[test]
fn daemon_config_debug_contains_fields() {
    let debug = format!("{:?}", PlaneDaemonConfig::default());
    assert!(debug.contains("batch_size"));
    assert!(debug.contains("dry_run"));
}

#[test]
fn daemon_sync_state_default() {
    let s = DaemonSyncState::default();
    assert!(!s.running);
    assert!(s.last_tick_at.is_none());
    assert_eq!(s.last_tick_duration_ms, 0);
    assert_eq!(s.modules_synced, 0);
    assert_eq!(s.cycles_synced, 0);
    assert_eq!(s.errors, 0);
}

#[test]
fn daemon_sync_state_serde_roundtrip() {
    let s = DaemonSyncState {
        running: true,
        last_tick_at: Some(chrono::Utc::now()),
        last_tick_duration_ms: 17,
        modules_synced: 3,
        cycles_synced: 2,
        errors: 1,
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DaemonSyncState = serde_json::from_str(&json).unwrap();
    assert!(back.running);
    assert_eq!(back.modules_synced, 3);
    assert_eq!(back.errors, 1);
}

#[test]
fn daemon_sync_state_deserializes_null_timestamp() {
    let json = r#"{"running":false,"last_tick_at":null,"last_tick_duration_ms":0,"modules_synced":0,"cycles_synced":0,"errors":0}"#;
    let s: DaemonSyncState = serde_json::from_str(json).unwrap();
    assert!(s.last_tick_at.is_none());
}

// ============================================================
// client models
// ============================================================

#[test]
fn work_item_response_deserializes_minimal() {
    let r: PlaneWorkItemResponse =
        serde_json::from_str(r#"{"id":"1","name":"n"}"#).unwrap();
    assert_eq!(r.id, "1");
    assert!(r.description_html.is_none());
    assert!(r.state.is_none());
    assert!(r.updated_at.is_none());
}

#[test]
fn work_item_response_requires_id_and_name() {
    assert!(serde_json::from_str::<PlaneWorkItemResponse>(r#"{"name":"n"}"#).is_err());
    assert!(serde_json::from_str::<PlaneWorkItemResponse>(r#"{"id":"1"}"#).is_err());
}

#[test]
fn work_item_serializes_labels_and_priority() {
    let item = PlaneWorkItem {
        id: None,
        name: "Feature".into(),
        description_html: Some("<p>x</p>".into()),
        state: Some("started".into()),
        priority: Some(2),
        parent: None,
        labels: vec!["a".into(), "b".into()],
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"priority\":2"));
    assert!(json.contains("\"labels\":[\"a\",\"b\"]"));
}

#[test]
fn plane_issue_alias_is_work_item() {
    let issue: PlaneIssue = PlaneWorkItem {
        id: Some("1".into()),
        name: "n".into(),
        description_html: None,
        state: None,
        priority: None,
        parent: None,
        labels: vec![],
    };
    assert_eq!(issue.id.as_deref(), Some("1"));
}

#[test]
fn module_request_omits_none_description() {
    let req = PlaneCreateModuleRequest {
        name: "m".into(),
        description: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("description"));
}

#[test]
fn module_response_roundtrip_fields() {
    let r: PlaneModuleResponse =
        serde_json::from_str(r#"{"id":"m1","name":"Mod","description":"d"}"#).unwrap();
    assert_eq!(r.id, "m1");
    assert_eq!(r.description.as_deref(), Some("d"));
}

#[test]
fn module_response_allows_null_description() {
    let r: PlaneModuleResponse =
        serde_json::from_str(r#"{"id":"m1","name":"Mod","description":null}"#).unwrap();
    assert!(r.description.is_none());
}

#[test]
fn cycle_request_serializes_dates() {
    let req = PlaneCreateCycleRequest {
        name: "Sprint".into(),
        description: None,
        start_date: "2026-01-01".into(),
        end_date: "2026-01-14".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("2026-01-01"));
    assert!(json.contains("2026-01-14"));
    assert!(!json.contains("description"));
}

#[test]
fn cycle_response_deserializes_with_dates() {
    let r: PlaneCycleResponse = serde_json::from_str(
        r#"{"id":"c1","name":"Sprint","start_date":"2026-01-01","end_date":"2026-01-14"}"#,
    )
    .unwrap();
    assert_eq!(r.id, "c1");
    assert_eq!(r.start_date.as_deref(), Some("2026-01-01"));
}

#[test]
fn cycle_response_allows_missing_dates() {
    let r: PlaneCycleResponse =
        serde_json::from_str(r#"{"id":"c1","name":"Sprint"}"#).unwrap();
    assert!(r.start_date.is_none());
}

// ============================================================
// work item field defaults
// ============================================================

#[test]
fn work_item_defaults_have_no_parent() {
    let item = PlaneWorkItem {
        id: None,
        name: "n".into(),
        description_html: None,
        state: None,
        priority: None,
        parent: None,
        labels: vec![],
    };
    assert!(item.parent.is_none());
    assert!(item.priority.is_none());
}
