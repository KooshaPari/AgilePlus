//! Deep coverage for the agileplus-plane webhook module (T047):
//! HMAC verification, payload parsing, and the axum handler.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::HeaderName;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use agileplus_plane::webhook::{
    handle_plane_webhook, parse_webhook, verify_hmac_signature, verify_webhook_signature,
    PlaneEventType, PlaneInboundEvent, PlaneWebhookAction, PlaneWebhookCycle, PlaneWebhookIssue,
    PlaneWebhookModule, PlaneWebhookPayload,
};

fn sign(secret: &[u8], body: &[u8]) -> String {
    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

fn headers_with(name: &str, value: &str) -> HeaderMap {
    let mut h = HeaderMap::new();
    let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap();
    h.insert(header_name, HeaderValue::from_str(value).unwrap());
    h
}

fn parse(body: &str) -> Result<PlaneInboundEvent, (StatusCode, String)> {
    parse_webhook(b"", &HeaderMap::new(), &Bytes::from(body.to_string()))
}

// ============================================================
// HMAC verification
// ============================================================

#[test]
fn hmac_accepts_valid_signature() {
    let sig = sign(b"secret", b"body");
    assert!(verify_hmac_signature(b"secret", b"body", &sig));
}

#[test]
fn hmac_alias_accepts_valid_signature() {
    let sig = sign(b"secret", b"body");
    assert!(verify_webhook_signature(b"secret", b"body", &sig));
}

#[test]
fn hmac_rejects_wrong_secret() {
    let sig = sign(b"secret", b"body");
    assert!(!verify_hmac_signature(b"other", b"body", &sig));
}

#[test]
fn hmac_rejects_tampered_body() {
    let sig = sign(b"secret", b"body");
    assert!(!verify_hmac_signature(b"secret", b"body-tampered", &sig));
}

#[test]
fn hmac_rejects_missing_prefix() {
    let sig = sign(b"secret", b"body");
    let bare = sig.trim_start_matches("sha256=");
    // The verifier requires the literal `sha256=` prefix.
    assert!(!verify_hmac_signature(b"secret", b"body", bare));
}

#[test]
fn hmac_rejects_empty_header() {
    assert!(!verify_hmac_signature(b"secret", b"body", ""));
}

#[test]
fn hmac_rejects_non_hex_payload() {
    assert!(!verify_hmac_signature(b"secret", b"body", "sha256=zzzz"));
}

#[test]
fn hmac_rejects_wrong_prefix() {
    let sig = sign(b"secret", b"body").replace("sha256=", "sha1=");
    // "sha1=<hex>" strips nothing, then falls back to decoding the whole
    // string as hex, which fails because of the non-hex characters.
    assert!(!verify_hmac_signature(b"secret", b"body", &sig));
}

#[test]
fn hmac_accepts_uppercase_hex() {
    let sig = sign(b"secret", b"body");
    let hex_part = sig.trim_start_matches("sha256=").to_uppercase();
    let mixed = format!("sha256={hex_part}");
    assert!(verify_hmac_signature(b"secret", b"body", &mixed));
}

#[test]
fn hmac_handles_empty_body() {
    let sig = sign(b"secret", b"");
    assert!(verify_hmac_signature(b"secret", b"", &sig));
}

#[test]
fn hmac_handles_large_body() {
    let body = vec![b'x'; 1_000_000];
    let sig = sign(b"secret", &body);
    assert!(verify_hmac_signature(b"secret", &body, &sig));
}

#[test]
fn hmac_handles_empty_secret() {
    let sig = sign(b"", b"body");
    assert!(verify_hmac_signature(b"", b"body", &sig));
}

// ============================================================
// parse_webhook: signature gate
// ============================================================

#[test]
fn parse_with_secret_and_no_signature_is_401() {
    let body = Bytes::from(r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#);
    let result = parse_webhook(b"secret", &HeaderMap::new(), &body);
    let (code, msg) = result.unwrap_err();
    assert_eq!(code, StatusCode::UNAUTHORIZED);
    assert!(msg.contains("signature"));
}

#[test]
fn parse_with_secret_and_valid_signature_succeeds() {
    let raw = r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#;
    let body = Bytes::from(raw);
    let headers = headers_with("x-plane-signature", &sign(b"secret", raw.as_bytes()));
    assert!(parse_webhook(b"secret", &headers, &body).is_ok());
}

#[test]
fn parse_with_secret_and_bad_signature_is_401() {
    let raw = r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#;
    let body = Bytes::from(raw);
    let headers = headers_with("x-plane-signature", "sha256=deadbeef");
    let (code, _) = parse_webhook(b"secret", &headers, &body).unwrap_err();
    assert_eq!(code, StatusCode::UNAUTHORIZED);
}

#[test]
fn parse_accepts_mixed_case_signature_header_name() {
    let raw = r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#;
    let body = Bytes::from(raw);
    let headers = headers_with("X-Plane-Signature", &sign(b"secret", raw.as_bytes()));
    assert!(parse_webhook(b"secret", &headers, &body).is_ok());
}

// ============================================================
// parse_webhook: body validation
// ============================================================

#[test]
fn parse_bad_json_is_400() {
    let (code, _) = parse("not json").unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
}

#[test]
fn parse_empty_body_is_400() {
    let (code, _) = parse("").unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
}

#[test]
fn parse_missing_data_field_is_400() {
    let (code, msg) = parse(r#"{"event":"issue","action":"create"}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
    assert!(msg.contains("missing data"));
}

#[test]
fn parse_null_data_is_400() {
    let (code, _) = parse(r#"{"event":"issue","action":"create","data":null}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
}

#[test]
fn parse_missing_event_field_is_400() {
    let (code, _) = parse(r#"{"action":"create","data":{}}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
}

// ============================================================
// parse_webhook: issue events
// ============================================================

#[test]
fn parse_issue_create() {
    let event = parse(
        r#"{"event":"issue","action":"create","data":{"id":"i1","name":"Title","labels":["a"]}}"#,
    )
    .unwrap();
    match event {
        PlaneInboundEvent::IssueCreated(issue) => {
            assert_eq!(issue.id, "i1");
            assert_eq!(issue.name, "Title");
            assert_eq!(issue.labels, vec!["a".to_string()]);
        }
        other => panic!("expected IssueCreated, got {other:?}"),
    }
}

#[test]
fn parse_issue_update() {
    let event = parse(
        r#"{"event":"issue","action":"update","data":{"id":"i2","name":"Updated"}}"#,
    )
    .unwrap();
    assert!(matches!(event, PlaneInboundEvent::IssueUpdated(i) if i.id == "i2"));
}

#[test]
fn parse_issue_delete_carries_id() {
    let event =
        parse(r#"{"event":"issue","action":"delete","data":{"id":"i3","name":"Gone"}}"#).unwrap();
    match event {
        PlaneInboundEvent::IssueDeleted { issue_id } => assert_eq!(issue_id, "i3"),
        other => panic!("expected IssueDeleted, got {other:?}"),
    }
}

#[test]
fn parse_issue_unknown_action_is_400() {
    let (code, msg) = parse(r#"{"event":"issue","action":"archive","data":{"id":"1","name":"n"}}"#)
        .unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
    assert!(msg.contains("unknown action"));
}

#[test]
fn parse_issue_missing_name_is_400() {
    let (code, msg) = parse(r#"{"event":"issue","action":"create","data":{"id":"1"}}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
    assert!(msg.contains("invalid issue data"));
}

#[test]
fn parse_issue_data_wrong_shape_is_400() {
    let (code, _) = parse(r#"{"event":"issue","action":"create","data":[]}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
}

#[test]
fn parse_issue_optional_fields_default() {
    let event = parse(r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#)
        .unwrap();
    match event {
        PlaneInboundEvent::IssueCreated(issue) => {
            assert!(issue.description_html.is_none());
            assert!(issue.state.is_none());
            assert!(issue.labels.is_empty());
            assert!(issue.project.is_none());
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn parse_issue_full_fields() {
    let event = parse(
        r#"{"event":"issue","action":"update","data":{"id":"1","name":"n","description_html":"<p>d</p>","state":"started","labels":["x","y"],"project":"p1"}}"#,
    )
    .unwrap();
    match event {
        PlaneInboundEvent::IssueUpdated(issue) => {
            assert_eq!(issue.description_html.as_deref(), Some("<p>d</p>"));
            assert_eq!(issue.state.as_deref(), Some("started"));
            assert_eq!(issue.labels.len(), 2);
            assert_eq!(issue.project.as_deref(), Some("p1"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn parse_event_prefixed_issues_uses_issue_path() {
    let event = parse(
        r#"{"event":"issues:create","action":"create","data":{"id":"1","name":"n"}}"#,
    )
    .unwrap();
    assert!(matches!(event, PlaneInboundEvent::IssueCreated(_)));
}

// ============================================================
// parse_webhook: module events
// ============================================================

#[test]
fn parse_module_update() {
    let event = parse(
        r#"{"event":"module","action":"update","data":{"id":"m1","name":"Mod","description":"d"}}"#,
    )
    .unwrap();
    match event {
        PlaneInboundEvent::ModuleUpdated(m) => {
            assert_eq!(m.id, "m1");
            assert_eq!(m.description.as_deref(), Some("d"));
        }
        other => panic!("expected ModuleUpdated, got {other:?}"),
    }
}

#[test]
fn parse_module_delete() {
    let event = parse(r#"{"event":"module","action":"delete","data":{"id":"m2","name":"Mod"}}"#)
        .unwrap();
    assert!(matches!(event, PlaneInboundEvent::ModuleDeleted { module_id } if module_id == "m2"));
}

#[test]
fn parse_module_create_treated_as_update() {
    let event = parse(r#"{"event":"module","action":"create","data":{"id":"m3","name":"Mod"}}"#)
        .unwrap();
    assert!(matches!(event, PlaneInboundEvent::ModuleUpdated(m) if m.id == "m3"));
}

#[test]
fn parse_module_missing_id_is_400() {
    let (code, msg) =
        parse(r#"{"event":"module","action":"update","data":{"name":"Mod"}}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
    assert!(msg.contains("invalid module data"));
}

#[test]
fn parse_module_event_prefix_variants() {
    let event =
        parse(r#"{"event":"module:update","action":"update","data":{"id":"m","name":"n"}}"#)
            .unwrap();
    assert!(matches!(event, PlaneInboundEvent::ModuleUpdated(_)));
}

// ============================================================
// parse_webhook: cycle events
// ============================================================

#[test]
fn parse_cycle_update() {
    let event = parse(
        r#"{"event":"cycle","action":"update","data":{"id":"c1","name":"Sprint","start_date":"2026-01-01","end_date":"2026-01-14"}}"#,
    )
    .unwrap();
    match event {
        PlaneInboundEvent::CycleUpdated(c) => {
            assert_eq!(c.id, "c1");
            assert_eq!(c.start_date.as_deref(), Some("2026-01-01"));
            assert_eq!(c.end_date.as_deref(), Some("2026-01-14"));
        }
        other => panic!("expected CycleUpdated, got {other:?}"),
    }
}

#[test]
fn parse_cycle_delete() {
    let event =
        parse(r#"{"event":"cycle","action":"delete","data":{"id":"c2","name":"Old"}}"#).unwrap();
    assert!(matches!(event, PlaneInboundEvent::CycleDeleted { cycle_id } if cycle_id == "c2"));
}

#[test]
fn parse_cycle_create_treated_as_update() {
    let event =
        parse(r#"{"event":"cycle","action":"create","data":{"id":"c3","name":"New"}}"#).unwrap();
    assert!(matches!(event, PlaneInboundEvent::CycleUpdated(c) if c.id == "c3"));
}

#[test]
fn parse_cycle_missing_name_is_400() {
    let (code, msg) =
        parse(r#"{"event":"cycle","action":"update","data":{"id":"c"}}"#).unwrap_err();
    assert_eq!(code, StatusCode::BAD_REQUEST);
    assert!(msg.contains("invalid cycle data"));
}

#[test]
fn parse_cycle_optional_dates_default() {
    let event = parse(r#"{"event":"cycle","action":"update","data":{"id":"c","name":"n"}}"#)
        .unwrap();
    match event {
        PlaneInboundEvent::CycleUpdated(c) => {
            assert!(c.start_date.is_none());
            assert!(c.end_date.is_none());
        }
        other => panic!("unexpected {other:?}"),
    }
}

// ============================================================
// Webhook payload serde
// ============================================================

#[test]
fn webhook_action_serde_snake_case() {
    assert_eq!(
        serde_json::to_string(&PlaneWebhookAction::Create).unwrap(),
        "\"create\""
    );
    assert_eq!(
        serde_json::to_string(&PlaneWebhookAction::Update).unwrap(),
        "\"update\""
    );
    assert_eq!(
        serde_json::to_string(&PlaneWebhookAction::Delete).unwrap(),
        "\"delete\""
    );
}

#[test]
fn webhook_action_unknown_is_other() {
    let action: PlaneWebhookAction = serde_json::from_str("\"whatever\"").unwrap();
    assert_eq!(action, PlaneWebhookAction::Unknown);
}

#[test]
fn webhook_action_roundtrip() {
    for action in [
        PlaneWebhookAction::Create,
        PlaneWebhookAction::Update,
        PlaneWebhookAction::Delete,
    ] {
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(serde_json::from_str::<PlaneWebhookAction>(&json).unwrap(), action);
    }
}

#[test]
fn event_type_issue_activity_serde() {
    let t: PlaneEventType = serde_json::from_str("\"issue_activity\"").unwrap();
    assert_eq!(t, PlaneEventType::IssueActivity);
}

#[test]
fn event_type_unknown_maps_to_other() {
    let t: PlaneEventType = serde_json::from_str("\"something_else\"").unwrap();
    assert_eq!(t, PlaneEventType::Unknown);
}

#[test]
fn payload_deserializes_event_and_action() {
    let p: PlaneWebhookPayload =
        serde_json::from_str(r#"{"event":"issue","action":"update","data":{"x":1}}"#).unwrap();
    assert_eq!(p.event, "issue");
    assert_eq!(p.action, PlaneWebhookAction::Update);
    assert!(p.data.is_some());
}

#[test]
fn payload_data_defaults_to_none() {
    let p: PlaneWebhookPayload =
        serde_json::from_str(r#"{"event":"issue","action":"create"}"#).unwrap();
    assert!(p.data.is_none());
}

#[test]
fn payload_roundtrips() {
    let p = PlaneWebhookPayload {
        event: "cycle".into(),
        action: PlaneWebhookAction::Delete,
        data: Some(serde_json::json!({"id":"c1"})),
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: PlaneWebhookPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(back.event, "cycle");
    assert_eq!(back.action, PlaneWebhookAction::Delete);
}

#[test]
fn webhook_issue_serde_defaults() {
    let issue: PlaneWebhookIssue = serde_json::from_str(r#"{"id":"1","name":"n"}"#).unwrap();
    assert!(issue.labels.is_empty());
    assert!(issue.state.is_none());
}

#[test]
fn webhook_issue_requires_id_and_name() {
    assert!(serde_json::from_str::<PlaneWebhookIssue>(r#"{"id":"1"}"#).is_err());
}

#[test]
fn webhook_module_serde() {
    let m: PlaneWebhookModule =
        serde_json::from_str(r#"{"id":"m","name":"n","description":"d"}"#).unwrap();
    assert_eq!(m.description.as_deref(), Some("d"));
}

#[test]
fn webhook_cycle_serde() {
    let c: PlaneWebhookCycle =
        serde_json::from_str(r#"{"id":"c","name":"n","start_date":"2026-01-01"}"#).unwrap();
    assert_eq!(c.start_date.as_deref(), Some("2026-01-01"));
    assert!(c.end_date.is_none());
}

#[test]
fn inbound_event_debug_and_clone() {
    let event = PlaneInboundEvent::IssueDeleted {
        issue_id: "x".into(),
    };
    let cloned = event.clone();
    assert!(format!("{cloned:?}").contains("IssueDeleted"));
}

// ============================================================
// axum handler
// ============================================================

#[tokio::test]
async fn handler_returns_200_for_valid_event() {
    let raw = r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#;
    let response = handle_plane_webhook(
        State(Vec::<u8>::new()),
        HeaderMap::new(),
        Bytes::from(raw),
    )
    .await
    .into_response();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn handler_returns_401_for_bad_signature() {
    let raw = r#"{"event":"issue","action":"create","data":{"id":"1","name":"n"}}"#;
    let headers = headers_with("x-plane-signature", "sha256=deadbeef");
    let response = handle_plane_webhook(
        State(b"secret".to_vec()),
        headers,
        Bytes::from(raw),
    )
    .await
    .into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn handler_returns_400_for_bad_json() {
    let response =
        handle_plane_webhook(State(Vec::new()), HeaderMap::new(), Bytes::from("nope"))
            .await
            .into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn handler_returns_400_for_missing_data() {
    let raw = r#"{"event":"issue","action":"create"}"#;
    let response = handle_plane_webhook(State(Vec::new()), HeaderMap::new(), Bytes::from(raw))
        .await
        .into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn handler_accepts_valid_signature_when_secret_configured() {
    let raw = r#"{"event":"module","action":"update","data":{"id":"m","name":"n"}}"#;
    let headers = headers_with("x-plane-signature", &sign(b"topsecret", raw.as_bytes()));
    let response = handle_plane_webhook(
        State(b"topsecret".to_vec()),
        headers,
        Bytes::from(raw),
    )
    .await
    .into_response();
    assert_eq!(response.status(), StatusCode::OK);
}
