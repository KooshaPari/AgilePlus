//! Integration tests for webhook module.
//!
//! Covers: HMAC verification, payload parsing, event routing, edge cases.

use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use agileplus_plane::webhook::{
    PlaneEventType, PlaneInboundEvent, PlaneWebhookAction, PlaneWebhookCycle,
    PlaneWebhookIssue, PlaneWebhookModule, PlaneWebhookPayload, parse_webhook,
    verify_hmac_signature, verify_webhook_signature,
};

fn make_hmac_signature(secret: &[u8], body: &[u8]) -> String {
    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(body);
    let sig = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(sig))
}

// ── HMAC verification ───────────────────────────────────────

#[test]
fn hmac_valid_signature_accepted() {
    let secret = b"my_webhook_secret";
    let body = b"test payload content";
    let sig = make_hmac_signature(secret, body);
    assert!(verify_hmac_signature(secret, body, &sig));
}

#[test]
fn hmac_wrong_secret_rejects() {
    let body = b"test payload";
    let sig = make_hmac_signature(b"correct_secret", body);
    assert!(!verify_hmac_signature(b"wrong_secret", body, &sig));
}

#[test]
fn hmac_wrong_body_rejects() {
    let secret = b"secret";
    let sig = make_hmac_signature(secret, b"original");
    assert!(!verify_hmac_signature(secret, b"modified", &sig));
}

#[test]
fn hmac_missing_sha256_prefix_rejects() {
    let secret = b"secret";
    let body = b"body";
    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(body);
    let sig = hex::encode(mac.finalize().into_bytes());
    // Without sha256= prefix
    assert!(!verify_hmac_signature(secret, body, &sig));
}

#[test]
fn hmac_invalid_hex_rejects() {
    let secret = b"secret";
    let body = b"body";
    assert!(!verify_hmac_signature(secret, body, "sha256=not-hex!!!"));
}

#[test]
fn hmac_empty_body() {
    let secret = b"secret";
    let body = b"";
    let sig = make_hmac_signature(secret, body);
    assert!(verify_hmac_signature(secret, body, &sig));
}

#[test]
fn hmac_large_body() {
    let secret = b"secret";
    let body = vec![0u8; 100_000];
    let sig = make_hmac_signature(secret, &body);
    assert!(verify_hmac_signature(secret, &body, &sig));
}

#[test]
fn hmac_empty_secret() {
    let body = b"test";
    let sig = make_hmac_signature(b"", body);
    assert!(verify_hmac_signature(b"", body, &sig));
}

#[test]
fn verify_webhook_signature_is_alias() {
    let secret = b"test";
    let body = b"data";
    let sig = make_hmac_signature(secret, body);
    assert!(verify_webhook_signature(secret, body, &sig));
    assert_eq!(
        verify_hmac_signature(secret, body, &sig),
        verify_webhook_signature(secret, body, &sig)
    );
}

// ── parse_webhook: issue events ─────────────────────────────

#[test]
fn parse_issue_create() {
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {
            "id": "i1",
            "name": "New Issue",
            "labels": [],
            "description_html": null,
            "state": null,
            "project": null
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::IssueCreated(issue) => {
            assert_eq!(issue.id, "i1");
            assert_eq!(issue.name, "New Issue");
        }
        other => panic!("expected IssueCreated, got {:?}", other),
    }
}

#[test]
fn parse_issue_update() {
    let body = serde_json::json!({
        "event": "issues:update",
        "action": "update",
        "data": {
            "id": "i2",
            "name": "Updated",
            "state": "started",
            "labels": ["bug", "urgent"]
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::IssueUpdated(issue) => {
            assert_eq!(issue.id, "i2");
            assert_eq!(issue.name, "Updated");
            assert_eq!(issue.state, Some("started".into()));
            assert_eq!(issue.labels, vec!["bug", "urgent"]);
        }
        other => panic!("expected IssueUpdated, got {:?}", other),
    }
}

#[test]
fn parse_issue_delete() {
    let body = serde_json::json!({
        "event": "issues:delete",
        "action": "delete",
        "data": {
            "id": "i3",
            "name": "Deleted"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::IssueDeleted { issue_id } => {
            assert_eq!(issue_id, "i3");
        }
        other => panic!("expected IssueDeleted, got {:?}", other),
    }
}

#[test]
fn parse_issue_unknown_action_returns_bad_request() {
    let body = serde_json::json!({
        "event": "issues:mystery",
        "action": "mystery_action",
        "data": {
            "id": "i4",
            "name": "Test"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let result = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
}

// ── parse_webhook: module events ────────────────────────────

#[test]
fn parse_module_update() {
    let body = serde_json::json!({
        "event": "modules:update",
        "action": "update",
        "data": {
            "id": "mod-1",
            "name": "Auth Module",
            "description": "Auth"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::ModuleUpdated(module) => {
            assert_eq!(module.id, "mod-1");
            assert_eq!(module.name, "Auth Module");
        }
        other => panic!("expected ModuleUpdated, got {:?}", other),
    }
}

#[test]
fn parse_module_delete() {
    let body = serde_json::json!({
        "event": "modules:delete",
        "action": "delete",
        "data": {
            "id": "mod-2",
            "name": "Old Module"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::ModuleDeleted { module_id } => {
            assert_eq!(module_id, "mod-2");
        }
        other => panic!("expected ModuleDeleted, got {:?}", other),
    }
}

#[test]
fn parse_module_create_treated_as_update() {
    let body = serde_json::json!({
        "event": "modules:create",
        "action": "create",
        "data": {
            "id": "mod-new",
            "name": "New Module"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    assert!(matches!(event, PlaneInboundEvent::ModuleUpdated(_)));
}

// ── parse_webhook: cycle events ─────────────────────────────

#[test]
fn parse_cycle_update() {
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
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::CycleUpdated(cycle) => {
            assert_eq!(cycle.id, "cyc-1");
            assert_eq!(cycle.name, "Sprint 1");
            assert_eq!(cycle.start_date, Some("2026-01-01".into()));
            assert_eq!(cycle.end_date, Some("2026-01-14".into()));
        }
        other => panic!("expected CycleUpdated, got {:?}", other),
    }
}

#[test]
fn parse_cycle_delete() {
    let body = serde_json::json!({
        "event": "cycles:delete",
        "action": "delete",
        "data": {
            "id": "cyc-2",
            "name": "Old Sprint"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    match event {
        PlaneInboundEvent::CycleDeleted { cycle_id } => {
            assert_eq!(cycle_id, "cyc-2");
        }
        other => panic!("expected CycleDeleted, got {:?}", other),
    }
}

#[test]
fn parse_cycle_create_treated_as_update() {
    let body = serde_json::json!({
        "event": "cycles:create",
        "action": "create",
        "data": {
            "id": "cyc-new",
            "name": "New Sprint"
        }
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let event = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into()).unwrap();
    assert!(matches!(event, PlaneInboundEvent::CycleUpdated(_)));
}

// ── parse_webhook: error cases ──────────────────────────────

#[test]
fn parse_invalid_json_returns_400() {
    let body = Bytes::from("not json at all");
    let result = parse_webhook(b"", &HeaderMap::new(), &body);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
}

#[test]
fn parse_missing_data_returns_400() {
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create"
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let result = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into());
    assert!(result.is_err());
}

#[test]
fn parse_bad_signature_returns_401() {
    let secret = b"my_secret";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {"id": "i1", "name": "Test"}
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("x-plane-signature", "sha256=badbad".parse().unwrap());

    let result = parse_webhook(secret, &headers, &body_bytes.clone().into());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, StatusCode::UNAUTHORIZED);
}

#[test]
fn parse_valid_signature_accepted() {
    let secret = b"my_secret";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {"id": "i1", "name": "Test"}
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(&body_bytes);
    let sig = hex::encode(mac.finalize().into_bytes());

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-plane-signature",
        format!("sha256={}", sig).parse().unwrap(),
    );

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    assert!(matches!(event, PlaneInboundEvent::IssueCreated(_)));
}

#[test]
fn parse_empty_secret_skips_verification() {
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {"id": "i1", "name": "No Auth"}
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let result = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into());
    assert!(result.is_ok());
}

#[test]
fn parse_uppercase_signature_header() {
    let secret = b"secret";
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {"id": "i1", "name": "Test"}
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(&body_bytes);
    let sig = hex::encode(mac.finalize().into_bytes());

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Plane-Signature",
        format!("sha256={}", sig).parse().unwrap(),
    );

    let event = parse_webhook(secret, &headers, &body_bytes.into()).unwrap();
    assert!(matches!(event, PlaneInboundEvent::IssueCreated(_)));
}

#[test]
fn parse_no_signature_header_without_secret_succeeds() {
    let body = serde_json::json!({
        "event": "issues:create",
        "action": "create",
        "data": {"id": "i1", "name": "Test"}
    });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let result = parse_webhook(b"", &HeaderMap::new(), &body_bytes.into());
    assert!(result.is_ok());
}

// ── Serde roundtrips ────────────────────────────────────────

#[test]
fn plane_event_type_issue_activity() {
    let json = r#""issue_activity""#;
    let et: PlaneEventType = serde_json::from_str(json).unwrap();
    assert_eq!(et, PlaneEventType::IssueActivity);
}

#[test]
fn plane_event_type_unknown() {
    let et: PlaneEventType = serde_json::from_str(r#""something_else""#).unwrap();
    assert_eq!(et, PlaneEventType::Unknown);
}

#[test]
fn plane_webhook_action_variants() {
    assert_eq!(
        serde_json::from_str::<PlaneWebhookAction>(r#""create""#).unwrap(),
        PlaneWebhookAction::Create
    );
    assert_eq!(
        serde_json::from_str::<PlaneWebhookAction>(r#""update""#).unwrap(),
        PlaneWebhookAction::Update
    );
    assert_eq!(
        serde_json::from_str::<PlaneWebhookAction>(r#""delete""#).unwrap(),
        PlaneWebhookAction::Delete
    );
    assert_eq!(
        serde_json::from_str::<PlaneWebhookAction>(r#""archive""#).unwrap(),
        PlaneWebhookAction::Unknown
    );
}

#[test]
fn plane_webhook_payload_roundtrip() {
    let payload = PlaneWebhookPayload {
        event: "issues:create".into(),
        action: PlaneWebhookAction::Create,
        data: Some(serde_json::json!({"id": "1"})),
    };
    let json = serde_json::to_string(&payload).unwrap();
    let restored: PlaneWebhookPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.event, "issues:create");
    assert_eq!(restored.action, PlaneWebhookAction::Create);
    assert!(restored.data.is_some());
}

#[test]
fn plane_webhook_issue_defaults() {
    let json = r#"{
        "id": "i1",
        "name": "Test",
        "description_html": null,
        "state": null,
        "labels": [],
        "project": null
    }"#;
    let issue: PlaneWebhookIssue = serde_json::from_str(json).unwrap();
    assert_eq!(issue.id, "i1");
    assert!(issue.description_html.is_none());
    assert!(issue.state.is_none());
    assert!(issue.labels.is_empty());
    assert!(issue.project.is_none());
}

#[test]
fn plane_webhook_module_defaults() {
    let json = r#"{"id": "m1", "name": "Mod"}"#;
    let module: PlaneWebhookModule = serde_json::from_str(json).unwrap();
    assert!(module.description.is_none());
}

#[test]
fn plane_webhook_cycle_defaults() {
    let json = r#"{"id": "c1", "name": "Cyc"}"#;
    let cycle: PlaneWebhookCycle = serde_json::from_str(json).unwrap();
    assert!(cycle.start_date.is_none());
    assert!(cycle.end_date.is_none());
}

// ── PlaneInboundEvent variants ──────────────────────────────

#[test]
fn inbound_event_issue_created_debug() {
    let issue = PlaneWebhookIssue {
        id: "1".into(),
        name: "Test".into(),
        description_html: None,
        state: None,
        labels: vec![],
        project: None,
    };
    let event = PlaneInboundEvent::IssueCreated(issue);
    let debug = format!("{:?}", event);
    assert!(debug.contains("IssueCreated"));
}

#[test]
fn inbound_event_issue_deleted_debug() {
    let event = PlaneInboundEvent::IssueDeleted {
        issue_id: "1".into(),
    };
    let debug = format!("{:?}", event);
    assert!(debug.contains("IssueDeleted"));
}

#[test]
fn inbound_event_module_deleted_debug() {
    let event = PlaneInboundEvent::ModuleDeleted {
        module_id: "1".into(),
    };
    let debug = format!("{:?}", event);
    assert!(debug.contains("ModuleDeleted"));
}

#[test]
fn inbound_event_cycle_deleted_debug() {
    let event = PlaneInboundEvent::CycleDeleted {
        cycle_id: "1".into(),
    };
    let debug = format!("{:?}", event);
    assert!(debug.contains("CycleDeleted"));
}
