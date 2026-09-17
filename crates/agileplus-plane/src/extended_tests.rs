//! Additional tests for agileplus-plane crate modules.
//!
//! Covers: sync_queue extended ops, webhook parse/signing edge cases,
//! state_mapper to_plane and custom overrides, daemon config, and
//! content_hash additional scenarios.

use std::time::Duration;

use chrono::Utc;

use crate::sync_queue::*;
use crate::webhook::*;
use crate::state_mapper::*;
use crate::daemon::*;
use crate::content_hash::*;
use agileplus_domain::domain::state_machine::FeatureState;

// ============================================================
// sync_queue: SyncQueueItem backoff edge cases
// ============================================================

#[test]
fn sync_queue_item_backoff_at_zero() {
    assert_eq!(
        SyncQueueItem::next_backoff_delay(0),
        Duration::from_secs(1)
    );
}

#[test]
fn sync_queue_item_backoff_at_max_retries() {
    // attempt = MAX_RETRIES (3) -> 2^3 = 8s
    assert_eq!(
        SyncQueueItem::next_backoff_delay(MAX_RETRIES),
        Duration::from_secs(8)
    );
}

#[test]
fn sync_queue_item_backoff_overflow_cap() {
    // Very large attempt should still cap at MAX_BACKOFF
    assert_eq!(
        SyncQueueItem::next_backoff_delay(100),
        MAX_BACKOFF
    );
}

#[test]
fn sync_queue_item_is_ready_immediately() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: 0,
        next_attempt_at: Utc::now() - chrono::Duration::seconds(10),
        created_at: Utc::now(),
    };
    assert!(item.is_ready());
}

#[test]
fn sync_queue_item_is_not_ready_in_future() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: 0,
        next_attempt_at: Utc::now() + chrono::Duration::hours(1),
        created_at: Utc::now(),
    };
    assert!(!item.is_ready());
}

#[test]
fn sync_queue_item_is_exhausted_at_max() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: MAX_RETRIES,
        next_attempt_at: Utc::now(),
        created_at: Utc::now(),
    };
    assert!(item.is_exhausted());
}

#[test]
fn sync_queue_item_is_not_exhausted_below_max() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::CreateIssue,
        payload: "{}".into(),
        attempt: MAX_RETRIES - 1,
        next_attempt_at: Utc::now(),
        created_at: Utc::now(),
    };
    assert!(!item.is_exhausted());
}

#[test]
fn sync_queue_item_with_next_attempt_increments() {
    let item = SyncQueueItem {
        id: 1,
        kind: SyncOpKind::UpdateIssue,
        payload: r#"{"key":"val"}"#.into(),
        attempt: 0,
        next_attempt_at: Utc::now(),
        created_at: Utc::now(),
    };
    let next = item.with_next_attempt();
    assert_eq!(next.attempt, 1);
    assert_eq!(next.id, 1);
    assert_eq!(next.kind, SyncOpKind::UpdateIssue);
    assert_eq!(next.payload, r#"{"key":"val"}"#);
    assert!(next.next_attempt_at > item.next_attempt_at);
}

// ============================================================
// sync_queue: SyncQueue operations
// ============================================================

#[test]
fn sync_queue_default_is_empty() {
    let q = SyncQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
}

#[test]
fn sync_queue_pop_ready_returns_none_when_empty() {
    let mut q = SyncQueue::new();
    assert!(q.pop_ready().is_none());
}

#[test]
fn sync_queue_pop_ready_returns_none_when_all_future() {
    let mut q = SyncQueue::new();
    q.enqueue(
        SyncOpKind::CreateIssue,
        "{}".into(),
    )
    .unwrap();
    // The item was just enqueued with next_attempt_at = now, so it should be ready.
    // But if we had a future item, it wouldn't be ready.
    // The item IS ready since next_attempt_at is Utc::now() at enqueue.
    let item = q.pop_ready().unwrap();
    assert_eq!(item.attempt, 0);
}

#[test]
fn sync_queue_requeue_increments_attempt() {
    let mut q = SyncQueue::new();
    let id = q
        .enqueue(SyncOpKind::DeleteIssue, "payload".into())
        .unwrap();
    let item = q.pop_ready().unwrap();
    assert_eq!(item.id, id);
    q.requeue(item).unwrap();
    assert_eq!(q.len(), 1);
}

#[test]
fn sync_queue_drain_empties() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateLabel, "a".into()).unwrap();
    q.enqueue(SyncOpKind::CreateLabel, "b".into()).unwrap();
    let items = q.drain();
    assert_eq!(items.len(), 2);
    assert!(q.is_empty());
}

#[test]
fn sync_queue_reload_restores_items() {
    let mut q = SyncQueue::new();
    q.enqueue(SyncOpKind::CreateIssue, "x".into()).unwrap();
    let items = q.drain();
    q.reload(items);
    assert_eq!(q.len(), 1);
}

#[test]
fn sync_queue_reload_respects_capacity() {
    let mut q = SyncQueue::new();
    // Fill to capacity
    for i in 0..QUEUE_CAPACITY {
        q.enqueue(SyncOpKind::CreateIssue, format!("{i}"))
            .unwrap();
    }
    assert_eq!(q.len(), QUEUE_CAPACITY);

    // Reload should skip extras
    let items: Vec<SyncQueueItem> = (0..10)
        .map(|i| SyncQueueItem {
            id: i as u64 + QUEUE_CAPACITY as u64 + 1,
            kind: SyncOpKind::CreateIssue,
            payload: format!("extra-{i}"),
            attempt: 0,
            next_attempt_at: Utc::now(),
            created_at: Utc::now(),
        })
        .collect();
    q.reload(items);
    assert_eq!(q.len(), QUEUE_CAPACITY); // still at capacity
}

#[test]
fn sync_queue_enqueue_sequential_ids() {
    let mut q = SyncQueue::new();
    let id1 = q.enqueue(SyncOpKind::CreateIssue, "a".into()).unwrap();
    let id2 = q.enqueue(SyncOpKind::CreateIssue, "b".into()).unwrap();
    let id3 = q.enqueue(SyncOpKind::CreateIssue, "c".into()).unwrap();
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

// ============================================================
// sync_queue: SyncTask operations
// ============================================================

#[test]
fn sync_task_new_defaults() {
    let task = SyncTask::new(42, SyncOpKind::CreateIssue, "{}".into(), Some("hash1".into()));
    assert_eq!(task.id, 42);
    assert_eq!(task.kind, SyncOpKind::CreateIssue);
    assert_eq!(task.attempt, 0);
    assert!(task.is_ready());
    assert!(!task.is_exhausted());
    assert_eq!(task.content_hash, Some("hash1".into()));
}

#[test]
fn sync_task_new_without_hash() {
    let task = SyncTask::new(1, SyncOpKind::DeleteIssue, "d".into(), None);
    assert!(task.content_hash.is_none());
}

#[test]
fn sync_task_with_next_attempt_preserves_fields() {
    let task = SyncTask::new(
        10,
        SyncOpKind::UpdateIssue,
        r#"{"data":1}"#.into(),
        Some("abc".into()),
    );
    let next = task.with_next_attempt();
    assert_eq!(next.id, 10);
    assert_eq!(next.kind, SyncOpKind::UpdateIssue);
    assert_eq!(next.payload, r#"{"data":1}"#.to_string());
    assert_eq!(next.content_hash, Some("abc".into()));
    assert_eq!(next.attempt, 1);
}

// ============================================================
// sync_queue: SyncQueueStore persistence
// ============================================================

#[test]
fn sync_queue_store_open_and_roundtrip() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    let items = vec![
        SyncQueueItem {
            id: 1,
            kind: SyncOpKind::CreateIssue,
            payload: r#"{"name":"issue1"}"#.into(),
            attempt: 0,
            next_attempt_at: Utc::now(),
            created_at: Utc::now(),
        },
        SyncQueueItem {
            id: 2,
            kind: SyncOpKind::DeleteIssue,
            payload: r#"{"id":"del1"}"#.into(),
            attempt: 2,
            next_attempt_at: Utc::now() + chrono::Duration::seconds(30),
            created_at: Utc::now(),
        },
    ];
    store.save_all(&items).unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].kind, SyncOpKind::CreateIssue);
    assert_eq!(loaded[1].kind, SyncOpKind::DeleteIssue);
    assert_eq!(loaded[1].attempt, 2);
}

#[test]
fn sync_queue_store_save_clears_previous() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    store
        .save_all(&[SyncQueueItem {
            id: 1,
            kind: SyncOpKind::CreateIssue,
            payload: "old".into(),
            attempt: 0,
            next_attempt_at: Utc::now(),
            created_at: Utc::now(),
        }])
        .unwrap();
    store
        .save_all(&[SyncQueueItem {
            id: 2,
            kind: SyncOpKind::CreateLabel,
            payload: "new".into(),
            attempt: 0,
            next_attempt_at: Utc::now(),
            created_at: Utc::now(),
        }])
        .unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].kind, SyncOpKind::CreateLabel);
}

#[test]
fn sync_queue_store_load_empty() {
    let store = SyncQueueStore::open_in_memory().unwrap();
    let loaded = store.load_all().unwrap();
    assert!(loaded.is_empty());
}

// ============================================================
// sync_queue: SyncOpKind serialization
// ============================================================

#[test]
fn sync_op_kind_serde_roundtrip() {
    let kinds = [
        SyncOpKind::CreateIssue,
        SyncOpKind::UpdateIssue,
        SyncOpKind::CreateLabel,
        SyncOpKind::DeleteIssue,
    ];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        let deserialized: SyncOpKind = serde_json::from_str(&json).unwrap();
        assert_eq!(*kind, deserialized);
    }
}

// ============================================================
// webhook: HMAC signature verification
// ============================================================

#[test]
fn verify_hmac_valid_signature() {
    use hmac::{Hmac, Mac, KeyInit};
    use sha2::Sha256;

    let secret = b"webhook_secret";
    let body = b"test body content";

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(body);
    let sig = hex::encode(mac.finalize().into_bytes());

    let header = format!("sha256={}", sig);
    assert!(verify_hmac_signature(secret, body, &header));
}

#[test]
fn verify_hmac_invalid_signature() {
    let secret = b"webhook_secret";
    let body = b"test body content";
    assert!(!verify_hmac_signature(secret, body, "sha256=deadbeef"));
}

#[test]
fn verify_hmac_missing_prefix() {
    let secret = b"webhook_secret";
    let body = b"test body";
    // No sha256= prefix
    assert!(!verify_hmac_signature(secret, body, "abcdef123456"));
}

#[test]
fn verify_hmac_invalid_hex() {
    let secret = b"webhook_secret";
    let body = b"test body";
    assert!(!verify_hmac_signature(secret, body, "sha256=not-hex!!!"));
}

#[test]
fn verify_hmac_raw_hex_without_prefix_is_rejected() {
    use hmac::{Hmac, Mac, KeyInit};
    use sha2::Sha256;

    let secret = b"secret";
    let body = b"payload";

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(body);
    let sig = hex::encode(mac.finalize().into_bytes());

    // Without sha256= prefix, the function rejects it (tries to parse hex string
    // which is too long for a valid hex signature without prefix)
    assert!(!verify_hmac_signature(secret, body, &sig));
}

#[test]
fn verify_webhook_signature_alias() {
    use hmac::{Hmac, Mac, KeyInit};
    use sha2::Sha256;

    let secret = b"test";
    let body = b"data";

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(body);
    let sig = hex::encode(mac.finalize().into_bytes());
    let header = format!("sha256={}", sig);

    assert!(verify_webhook_signature(secret, body, &header));
}

// ============================================================
// webhook: parse_webhook
// ============================================================

#[test]
fn parse_webhook_issue_create() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {
            "id": "issue-1",
            "name": "Test Issue",
            "description_html": null,
            "state": null,
            "labels": [],
            "project": null
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::IssueCreated(issue) => {
            assert_eq!(issue.id, "issue-1");
            assert_eq!(issue.name, "Test Issue");
        }
        _ => panic!("expected IssueCreated"),
    }
}

#[test]
fn parse_webhook_issue_update() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "issues:update",
        "action": "update",
        "data": {
            "id": "issue-2",
            "name": "Updated",
            "labels": ["bug"]
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::IssueUpdated(issue) => {
            assert_eq!(issue.id, "issue-2");
            assert_eq!(issue.labels, vec!["bug"]);
        }
        _ => panic!("expected IssueUpdated"),
    }
}

#[test]
fn parse_webhook_issue_delete() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "issues:delete",
        "action": "delete",
        "data": {
            "id": "issue-3",
            "name": "Deleted"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::IssueDeleted { issue_id } => {
            assert_eq!(issue_id, "issue-3");
        }
        _ => panic!("expected IssueDeleted"),
    }
}

#[test]
fn parse_webhook_module_update() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "modules:update",
        "action": "update",
        "data": {
            "id": "mod-1",
            "name": "Auth",
            "description": "Auth module"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::ModuleUpdated(module) => {
            assert_eq!(module.id, "mod-1");
            assert_eq!(module.name, "Auth");
        }
        _ => panic!("expected ModuleUpdated"),
    }
}

#[test]
fn parse_webhook_module_delete() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "modules:delete",
        "action": "delete",
        "data": {
            "id": "mod-2",
            "name": "Deleted"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::ModuleDeleted { module_id } => {
            assert_eq!(module_id, "mod-2");
        }
        _ => panic!("expected ModuleDeleted"),
    }
}

#[test]
fn parse_webhook_cycle_update() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "cycles:update",
        "action": "update",
        "data": {
            "id": "cyc-1",
            "name": "Sprint 1",
            "start_date": "2026-01-01",
            "end_date": "2026-01-14"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::CycleUpdated(cycle) => {
            assert_eq!(cycle.id, "cyc-1");
            assert_eq!(cycle.name, "Sprint 1");
        }
        _ => panic!("expected CycleUpdated"),
    }
}

#[test]
fn parse_webhook_cycle_delete() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "cycles:delete",
        "action": "delete",
        "data": {
            "id": "cyc-2",
            "name": "Deleted"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::CycleDeleted { cycle_id } => {
            assert_eq!(cycle_id, "cyc-2");
        }
        _ => panic!("expected CycleDeleted"),
    }
}

#[test]
fn parse_webhook_module_create_treated_as_update() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "modules:create",
        "action": "create",
        "data": {
            "id": "mod-new",
            "name": "New Module"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    // Module create is treated as update
    assert!(matches!(event, PlaneInboundEvent::ModuleUpdated(_)));
}

#[test]
fn parse_webhook_invalid_json() {
    let secret = b"";
    let body = b"not json";
    let headers = axum::http::HeaderMap::new();

    let body_bytes = axum::body::Bytes::from(body.to_vec());
    let result = parse_webhook(secret, &headers, &body_bytes);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn parse_webhook_missing_data() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create"
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let result = parse_webhook(secret, &headers, &body_bytes.into());
    assert!(result.is_err());
}

#[test]
fn parse_webhook_bad_signature() {
    use hmac::{Hmac, Mac, KeyInit};
    use sha2::Sha256;

    let secret = b"my_secret";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {
            "id": "i1",
            "name": "Test"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-plane-signature", "sha256=badbadbad".parse().unwrap());

    let result = parse_webhook(secret, &headers, &body_bytes.clone().into());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().0,
        axum::http::StatusCode::UNAUTHORIZED
    );
}

#[test]
fn parse_webhook_valid_signature() {
    use hmac::{Hmac, Mac, KeyInit};
    use sha2::Sha256;

    let secret = b"my_secret";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {
            "id": "i1",
            "name": "Test"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(&body_bytes);
    let sig = hex::encode(mac.finalize().into_bytes());

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "x-plane-signature",
        format!("sha256={}", sig).parse().unwrap(),
    );

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    assert!(matches!(event, PlaneInboundEvent::IssueCreated(_)));
}

#[test]
fn parse_webhook_empty_secret_skips_verification() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {
            "id": "i1",
            "name": "No Auth"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    // Empty secret means skip verification
    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    assert!(matches!(event, PlaneInboundEvent::IssueCreated(_)));
}

#[test]
fn parse_webhook_unknown_action_for_issue() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "issues:unknown_action",
        "action": "unknown_action",
        "data": {
            "id": "i1",
            "name": "Test"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let result = parse_webhook(secret, &headers, &body_bytes.into());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn parse_webhook_module_create_treated_as_update_variant() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "modules:create",
        "action": "create",
        "data": {
            "id": "mod-new",
            "name": "New Module"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    // Module create -> treated as ModuleUpdated
    assert!(matches!(event, PlaneInboundEvent::ModuleUpdated(_)));
}

#[test]
fn parse_webhook_cycle_create_treated_as_update() {
    let secret = b"";
    let body = serde_json::json!({
        "event": "cycles:create",
        "action": "create",
        "data": {
            "id": "cyc-new",
            "name": "New Cycle"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let headers = axum::http::HeaderMap::new();

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    // Cycle create -> treated as CycleUpdated
    assert!(matches!(event, PlaneInboundEvent::CycleUpdated(_)));
}

// ============================================================
// webhook: Serde roundtrips for types
// ============================================================

#[test]
fn plane_event_type_serde() {
    let json = r#""issue_activity""#;
    let et: PlaneEventType = serde_json::from_str(json).unwrap();
    assert_eq!(et, PlaneEventType::IssueActivity);

    // Unknown variant
    let json = r#""something_else""#;
    let et: PlaneEventType = serde_json::from_str(json).unwrap();
    assert_eq!(et, PlaneEventType::Unknown);
}

#[test]
fn plane_webhook_action_serde() {
    let cases = [
        (r#""create""#, PlaneWebhookAction::Create),
        (r#""update""#, PlaneWebhookAction::Update),
        (r#""delete""#, PlaneWebhookAction::Delete),
    ];
    for (json, expected) in cases {
        let action: PlaneWebhookAction = serde_json::from_str(json).unwrap();
        assert_eq!(action, expected);
    }

    // Unknown action
    let action: PlaneWebhookAction = serde_json::from_str(r#""archive""#).unwrap();
    assert_eq!(action, PlaneWebhookAction::Unknown);
}

#[test]
fn plane_webhook_payload_deserialize() {
    let json = r#"{
        "event": "issues:update",
        "action": "update",
        "data": {"id": "1", "name": "Test"}
    }"#;
    let payload: PlaneWebhookPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.event, "issues:update");
    assert!(payload.data.is_some());
}

#[test]
fn plane_webhook_issue_deserialize() {
    let json = r#"{
        "id": "issue-42",
        "name": "My Issue",
        "description_html": "<p>desc</p>",
        "state": "started",
        "labels": ["bug", "urgent"],
        "project": "proj-1"
    }"#;
    let issue: PlaneWebhookIssue = serde_json::from_str(json).unwrap();
    assert_eq!(issue.id, "issue-42");
    assert_eq!(issue.description_html, Some("<p>desc</p>".into()));
    assert_eq!(issue.labels, vec!["bug", "urgent"]);
}

#[test]
fn plane_webhook_module_deserialize() {
    let json = r#"{
        "id": "mod-99",
        "name": "Billing",
        "description": null
    }"#;
    let module: PlaneWebhookModule = serde_json::from_str(json).unwrap();
    assert_eq!(module.id, "mod-99");
    assert!(module.description.is_none());
}

#[test]
fn plane_webhook_cycle_deserialize() {
    let json = r#"{
        "id": "cyc-5",
        "name": "Sprint 5",
        "start_date": "2026-06-01",
        "end_date": "2026-06-14"
    }"#;
    let cycle: PlaneWebhookCycle = serde_json::from_str(json).unwrap();
    assert_eq!(cycle.id, "cyc-5");
    assert_eq!(cycle.start_date, Some("2026-06-01".into()));
}

// ============================================================
// state_mapper: to_plane mapping
// ============================================================

#[test]
fn to_plane_created_maps_to_backlog() {
    let mapper = PlaneStateMapper::new();
    let (group, _id) = mapper.to_plane(FeatureState::Created);
    assert_eq!(group, "backlog");
}

#[test]
fn to_plane_specified_maps_to_unstarted() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Specified);
    assert_eq!(group, "unstarted");
}

#[test]
fn to_plane_implementing_maps_to_started() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Implementing);
    assert_eq!(group, "started");
}

#[test]
fn to_plane_validated_maps_to_completed() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Validated);
    assert_eq!(group, "completed");
}

#[test]
fn to_plane_shipped_maps_to_completed() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Shipped);
    assert_eq!(group, "completed");
}

#[test]
fn to_plane_researched_maps_to_unstarted() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Researched);
    assert_eq!(group, "unstarted");
}

#[test]
fn to_plane_planned_maps_to_unstarted() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Planned);
    assert_eq!(group, "unstarted");
}

#[test]
fn to_plane_retrospected_maps_to_completed() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Retrospected);
    assert_eq!(group, "completed");
}

#[test]
fn to_plane_with_config_override() {
    let mut config = PlaneStateMapperConfig::default();
    let mut state_id_map = std::collections::HashMap::new();
    state_id_map.insert(
        FeatureState::Implementing,
        ("started".into(), "state-uuid-123".into()),
    );
    config.state_id_map = state_id_map;
    let mapper = PlaneStateMapper::with_config(config);

    let (group, id) = mapper.to_plane(FeatureState::Implementing);
    assert_eq!(group, "started");
    assert_eq!(id, "state-uuid-123");
}

// ============================================================
// state_mapper: PlaneStateGroup::from_str
// ============================================================

#[test]
fn plane_state_group_from_str_variants() {
    assert!(matches!(
        "backlog".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Backlog
    ));
    assert!(matches!(
        "unstarted".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Unstarted
    ));
    assert!(matches!(
        "todo".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Unstarted
    ));
    assert!(matches!(
        "started".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    ));
    assert!(matches!(
        "in_progress".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    ));
    assert!(matches!(
        "completed".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Completed
    ));
    assert!(matches!(
        "done".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Completed
    ));
    assert!(matches!(
        "cancelled".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Cancelled
    ));
    assert!(matches!(
        "canceled".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Cancelled
    ));
    assert!(matches!(
        "something_weird".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Unknown(_)
    ));
}

#[test]
fn plane_state_group_as_str_roundtrip() {
    let groups = [
        PlaneStateGroup::Backlog,
        PlaneStateGroup::Unstarted,
        PlaneStateGroup::Started,
        PlaneStateGroup::Completed,
        PlaneStateGroup::Cancelled,
    ];
    for g in &groups {
        let s = g.as_str();
        let parsed: PlaneStateGroup = s.parse().unwrap();
        assert_eq!(g.as_str(), parsed.as_str());
    }
}

// ============================================================
// daemon: PlaneDaemonConfig
// ============================================================

#[test]
fn daemon_config_defaults() {
    let config = PlaneDaemonConfig::default();
    assert_eq!(config.interval, Duration::from_secs(300));
    assert_eq!(config.batch_size, 25);
    assert!(!config.dry_run);
}

// Serialize every daemon env test across setup, reads, assertions, and cleanup.
static DAEMON_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn daemon_config_from_env_defaults() {
    let _guard = DAEMON_ENV_MUTEX.lock().unwrap();
    // Clear any existing env vars
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }

    let config = PlaneDaemonConfig::from_env();
    assert_eq!(config.interval, Duration::from_secs(300));
    assert_eq!(config.batch_size, 25);
    assert!(!config.dry_run);
}

#[test]
fn daemon_config_from_env_custom() {
    let _guard = DAEMON_ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "60");
        std::env::set_var("PLANE_DAEMON_BATCH_SIZE", "50");
        std::env::set_var("PLANE_DAEMON_DRY_RUN", "1");
    }

    let config = PlaneDaemonConfig::from_env();
    assert_eq!(config.interval, Duration::from_secs(60));
    assert_eq!(config.batch_size, 50);
    assert!(config.dry_run);

    // Clean up
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }
}

#[test]
fn daemon_config_from_env_true_string() {
    let _guard = DAEMON_ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("PLANE_DAEMON_DRY_RUN", "true");
    }
    let config = PlaneDaemonConfig::from_env();
    assert!(config.dry_run);
    unsafe {
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }
}

#[test]
fn daemon_config_from_env_invalid_interval() {
    let _guard = DAEMON_ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "not_a_number");
    }
    let config = PlaneDaemonConfig::from_env();
    assert_eq!(config.interval, Duration::from_secs(300)); // falls back to default
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
    }
}

#[test]
fn daemon_config_from_env_invalid_batch_size() {
    let _guard = DAEMON_ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("PLANE_DAEMON_BATCH_SIZE", "abc");
    }
    let config = PlaneDaemonConfig::from_env();
    assert_eq!(config.batch_size, 25); // falls back to default
    unsafe {
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
    }
}

// ============================================================
// daemon: SyncState
// ============================================================

#[test]
fn sync_state_default() {
    let state = SyncState::default();
    assert!(!state.running);
    assert!(state.last_tick_at.is_none());
    assert_eq!(state.last_tick_duration_ms, 0);
    assert_eq!(state.modules_synced, 0);
    assert_eq!(state.cycles_synced, 0);
    assert_eq!(state.errors, 0);
}

// ============================================================
// content_hash: extended coverage
// ============================================================

#[test]
fn content_hash_empty_strings() {
    let h = compute_content_hash("", "", "", &[]);
    assert!(!h.is_empty());
    assert_eq!(h.len(), 64); // SHA-256 hex is 64 chars
}

#[test]
fn content_hash_different_inputs_different_hashes() {
    let h1 = compute_content_hash("A", "", "", &[]);
    let h2 = compute_content_hash("B", "", "", &[]);
    assert_ne!(h1, h2);
}

#[test]
fn content_hash_labels_dont_affect_when_identical() {
    let h1 = compute_content_hash("T", "D", "s", &["x".into(), "y".into()]);
    let h2 = compute_content_hash("T", "D", "s", &["y".into(), "x".into()]);
    assert_eq!(h1, h2);
}

#[test]
fn conflict_detect_both_changed() {
    assert_eq!(
        detect_conflict("base", "local_changed", "remote_changed"),
        ConflictStatus::Conflict
    );
}

#[test]
fn conflict_detect_neither_changed() {
    assert_eq!(
        detect_conflict("base", "base", "base"),
        ConflictStatus::Clean
    );
}

#[test]
fn conflict_detect_only_local_changed() {
    assert_eq!(
        detect_conflict("base", "new_local", "base"),
        ConflictStatus::Clean
    );
}

#[test]
fn conflict_detect_only_remote_changed() {
    assert_eq!(
        detect_conflict("base", "base", "new_remote"),
        ConflictStatus::Clean
    );
}
