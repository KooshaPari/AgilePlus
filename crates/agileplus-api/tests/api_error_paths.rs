// SPDX-License-Identifier: MIT OR Apache-2.0
//! Failure and edge-case behaviour that no other suite drives.
//!
//! Three groups:
//!
//! 1. **Storage failure surfacing.** Handlers must convert a `DomainError`
//!    into the fixed `{"error": "internal server error"}` envelope without
//!    echoing the storage message, while `/detailed-health` must report the
//!    same failure as an `unavailable` service entry with its reason.
//! 2. **Query-parameter rejections.** The branch/event/cycle endpoints take
//!    typed query params; malformed values must be rejected rather than
//!    silently defaulted.
//! 3. **The `since=<n>m` relative window.** The minutes arm of the event
//!    filter is exercised with a genuinely stale event, so a wrong unit or a
//!    reversed comparison window would fail the test.
//!
//! Traceability: WP11-T065, WP11-T068, WP15-T086

#![allow(dead_code)]

#[path = "api_integration/support/mod.rs"]
mod support;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::cycle::{Cycle, CycleState};
use axum_test::TestResponse;
use chrono::{Duration, NaiveDate, Utc};

use crate::support::{MockStorage, TEST_API_KEY, setup_test_server_with_storage};

const KEY: &str = "X-API-Key";

/// A storage-layer failure message that a client must never see.
const STORAGE_FAILURE: &str = "sqlite: disk I/O error at /var/db/agileplus.sqlite";

/// `MockStorage::with_test_data()` with one `StoragePort` method failing.
fn storage_failing_at(method: &'static str) -> MockStorage {
    let storage = MockStorage::with_test_data();
    storage.fail_on(method, STORAGE_FAILURE);
    storage
}

/// Every handler-side storage failure must produce the same fixed envelope.
fn assert_generic_500(resp: &TestResponse) {
    resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    let content_type = resp
        .headers()
        .get("content-type")
        .expect("error responses carry a content type")
        .to_str()
        .expect("content type is ASCII");
    assert!(
        content_type.contains("application/json"),
        "error responses must stay JSON, got: {content_type}"
    );
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"], "internal server error");
    assert!(
        !body.to_string().contains("sqlite"),
        "the storage message must not leak to the client, got: {body}"
    );
}

// ── 1. Storage failures ──────────────────────────────────────────────────────

#[tokio::test]
async fn features_list_returns_generic_500_when_storage_scan_fails() {
    let server = setup_test_server_with_storage(storage_failing_at("list_all_features")).await;
    let resp = server.get("/api/v1/features").add_header(KEY, TEST_API_KEY).await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn events_list_returns_generic_500_when_storage_scan_fails() {
    let server = setup_test_server_with_storage(storage_failing_at("list_all_features")).await;
    let resp = server.get("/api/v1/events").add_header(KEY, TEST_API_KEY).await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn events_detail_returns_generic_500_when_storage_scan_fails() {
    let server = setup_test_server_with_storage(storage_failing_at("list_all_features")).await;
    let resp = server
        .get("/api/v1/events/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    assert_generic_500(&resp);
}

/// `/detailed-health` is the one endpoint that *does* surface the storage
/// reason: it is an operator probe, not a client-facing resource.
#[tokio::test]
async fn detailed_health_marks_storage_unavailable_and_reports_the_reason() {
    let server = setup_test_server_with_storage(storage_failing_at("list_all_features")).await;
    let resp = server.get("/detailed-health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();

    assert_eq!(
        body["status"], "unavailable",
        "an unavailable dependency must dominate the overall status, got: {body}"
    );
    assert_eq!(body["services"]["sqlite"]["status"], "unavailable");
    assert_eq!(
        body["services"]["sqlite"]["error"],
        format!("Storage error: {STORAGE_FAILURE}"),
        "the probe must report the storage reason verbatim"
    );
    assert!(
        body["services"]["sqlite"].get("latency_ms").is_none(),
        "a failed probe has no latency, got: {body}"
    );
    assert_eq!(
        body["api"]["status"], "healthy",
        "the HTTP API itself is still up, got: {body}"
    );
    assert!(body["timestamp"].is_string());
}

#[tokio::test]
async fn module_tree_returns_generic_500_when_module_lookup_fails() {
    let server =
        setup_test_server_with_storage(storage_failing_at("get_module_with_features")).await;
    let resp = server
        .get("/api/modules/1/tree")
        .add_header(KEY, TEST_API_KEY)
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn module_detail_returns_generic_500_when_module_lookup_fails() {
    let server =
        setup_test_server_with_storage(storage_failing_at("get_module_with_features")).await;
    let resp = server
        .get("/api/modules/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    assert_generic_500(&resp);
}

/// The HTML module page takes the same storage path as the JSON tree, so a
/// storage failure must surface as a 500 rather than an empty page.
#[tokio::test]
async fn modules_page_returns_generic_500_when_module_lookup_fails() {
    let storage = MockStorage::default();
    storage.fail_on("get_module_with_features", STORAGE_FAILURE);
    let now = Utc::now();
    storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .push(agileplus_domain::domain::module::Module {
            id: 1,
            slug: "platform".to_string(),
            friendly_name: "Platform".to_string(),
            description: None,
            parent_module_id: None,
            created_at: now,
            updated_at: now,
        });

    let server = setup_test_server_with_storage(storage).await;
    let resp = server.get("/modules").await;
    assert_generic_500(&resp);
}

// ── 2. Query-parameter rejections ────────────────────────────────────────────

#[tokio::test]
async fn branches_reject_non_boolean_remote_param() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server
        .get("/api/v1/branches?remote=maybe")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn branches_accept_boolean_remote_param() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server
        .get("/api/v1/branches?remote=true")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn events_reject_non_numeric_limit_param() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server
        .get("/api/v1/events?limit=not-a-number")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn events_reject_negative_offset_param() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server
        .get("/api/v1/events?offset=-1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cycles_reject_non_numeric_id_in_detail_page() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server.get("/cycles/not-a-number").await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ── 3. Relative `since` windows ──────────────────────────────────────────────

/// Seed a feature with a single audit entry `age` old, so the event query has
/// an event whose position relative to "now" is exact.
fn storage_with_event_aged(age: Duration) -> MockStorage {
    let storage = MockStorage::with_test_data();
    let timestamp = Utc::now() - age;
    storage
        .audit
        .lock()
        .expect("audit lock poisoned")
        .push(AuditEntry {
            id: 903,
            feature_id: 1,
            wp_id: None,
            timestamp,
            actor: "stale-bot".to_string(),
            transition: "specified".to_string(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [7u8; 32],
            event_id: None,
            archived_to: None,
        });
    storage
}

/// `since=<n>m` means `n` **minutes**. A 45-minute-old event is inside a
/// 90-minute window and outside a 30-minute window; if the unit were wrong or
/// the comparison reversed, one of the two assertions would fail.
#[tokio::test]
async fn events_since_minutes_window_selects_events_by_age() {
    let server = setup_test_server_with_storage(storage_with_event_aged(Duration::minutes(45))).await;

    let inside = server
        .get("/api/v1/events?actor=stale-bot&since=90m")
        .add_header(KEY, TEST_API_KEY)
        .await;
    inside.assert_status_ok();
    let inside: Vec<serde_json::Value> = inside.json();
    assert_eq!(inside.len(), 1, "90m must include a 45-minute-old event");
    assert_eq!(inside[0]["actor"], "stale-bot");

    let outside = server
        .get("/api/v1/events?actor=stale-bot&since=30m")
        .add_header(KEY, TEST_API_KEY)
        .await;
    outside.assert_status_ok();
    let outside: Vec<serde_json::Value> = outside.json();
    assert!(
        outside.is_empty(),
        "30m must exclude a 45-minute-old event, got: {outside:?}"
    );
}

#[tokio::test]
async fn events_since_rejects_non_numeric_hour_suffix() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server
        .get("/api/v1/events?since=xh")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"], "invalid since: xh");
}

#[tokio::test]
async fn events_since_rejects_bare_number_without_unit() {
    let server = setup_test_server_with_storage(MockStorage::default()).await;
    let resp = server
        .get("/api/v1/events?since=24")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"], "invalid since: 24");
}

// ── 4. Cycle detail page without a module scope ──────────────────────────────

#[tokio::test]
async fn cycle_detail_page_without_module_scope_omits_the_scope_line() {
    let storage = MockStorage::default();
    let now = Utc::now();
    storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .push(Cycle {
            id: 1,
            name: "Unscoped Cycle".to_string(),
            description: None,
            state: CycleState::Active,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            end_date: NaiveDate::from_ymd_opt(2027, 1, 1).expect("valid date"),
            module_scope_id: None,
            created_at: now,
            updated_at: now,
        });

    let server = setup_test_server_with_storage(storage).await;
    let resp = server.get("/cycles/1").await;
    resp.assert_status_ok();
    let body = resp.text();

    assert!(body.contains("Unscoped Cycle"), "got: {body}");
    assert!(body.contains("days remaining"), "got: {body}");
    assert!(
        !body.contains("Scope:"),
        "a cycle with no module scope must not render a scope line, got: {body}"
    );
    assert!(
        body.contains("Features (0)"),
        "an empty cycle still renders its feature table, got: {body}"
    );
}
