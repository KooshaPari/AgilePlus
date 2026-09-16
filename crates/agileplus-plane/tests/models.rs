//! Integration tests for client models and InMemoryPlaneClient.
//!
//! Covers: serialization/deserialization, field defaults, mock client operations.

use agileplus_plane::PlaneClient;
use agileplus_plane::client::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};

// ── PlaneWorkItem serialization ─────────────────────────────

#[test]
fn work_item_serializes_all_fields() {
    let item = PlaneWorkItem {
        id: Some("wi-1".into()),
        name: "Test Item".into(),
        description_html: Some("<p>desc</p>".into()),
        state: Some("started".into()),
        priority: Some(2),
        parent: Some("parent-1".into()),
        labels: vec!["bug".into(), "urgent".into()],
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("wi-1"));
    assert!(json.contains("Test Item"));
    assert!(json.contains("<p>desc</p>"));
    assert!(json.contains("started"));
    assert!(json.contains("parent-1"));
}

#[test]
fn work_item_serializes_with_none_fields() {
    let item = PlaneWorkItem {
        id: None,
        name: "Minimal".into(),
        description_html: None,
        state: None,
        priority: None,
        parent: None,
        labels: vec![],
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("Minimal"));
    assert!(json.contains("null")); // null for Option fields
}

#[test]
fn work_item_deserializes() {
    let json = r#"{
        "id": "wi-1",
        "name": "Test",
        "description_html": "<p>hi</p>",
        "state": "started",
        "priority": 3,
        "parent": "p-1",
        "labels": ["bug"]
    }"#;
    let item: PlaneWorkItem = serde_json::from_str(json).unwrap();
    assert_eq!(item.id, Some("wi-1".into()));
    assert_eq!(item.name, "Test");
    assert_eq!(item.priority, Some(3));
    assert_eq!(item.labels, vec!["bug"]);
}

#[test]
fn work_item_clone() {
    let item = PlaneWorkItem {
        id: Some("1".into()),
        name: "Test".into(),
        description_html: None,
        state: None,
        priority: None,
        parent: None,
        labels: vec![],
    };
    let cloned = item.clone();
    assert_eq!(cloned.id, item.id);
    assert_eq!(cloned.name, item.name);
}

#[test]
fn work_item_debug_format() {
    let item = PlaneWorkItem {
        id: None,
        name: "Test".into(),
        description_html: None,
        state: None,
        priority: None,
        parent: None,
        labels: vec![],
    };
    let debug = format!("{:?}", item);
    assert!(debug.contains("PlaneWorkItem"));
}

#[test]
fn plane_issue_is_alias() {
    let issue: PlaneIssue = PlaneWorkItem {
        id: None,
        name: "Alias".into(),
        description_html: None,
        state: None,
        priority: None,
        parent: None,
        labels: vec![],
    };
    assert_eq!(issue.name, "Alias");
}

// ── PlaneWorkItemResponse deserialization ────────────────────

#[test]
fn work_item_response_deserializes_all_fields() {
    let json = r#"{
        "id": "resp-1",
        "name": "Created",
        "description_html": "<p>desc</p>",
        "state": "started",
        "updated_at": "2026-01-01T00:00:00Z"
    }"#;
    let resp: PlaneWorkItemResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.id, "resp-1");
    assert_eq!(resp.name, "Created");
    assert!(resp.description_html.is_some());
    assert!(resp.updated_at.is_some());
}

#[test]
fn work_item_response_optional_fields() {
    let json = r#"{
        "id": "resp-2",
        "name": "Minimal"
    }"#;
    let resp: PlaneWorkItemResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.id, "resp-2");
    assert!(resp.description_html.is_none());
    assert!(resp.state.is_none());
    assert!(resp.updated_at.is_none());
}

#[test]
fn work_item_response_debug_format() {
    let resp = PlaneWorkItemResponse {
        id: "1".into(),
        name: "Test".into(),
        description_html: None,
        state: None,
        updated_at: None,
    };
    let debug = format!("{:?}", resp);
    assert!(debug.contains("PlaneWorkItemResponse"));
}

// ── PlaneCreateModuleRequest ────────────────────────────────

#[test]
fn create_module_request_serializes_with_description() {
    let req = PlaneCreateModuleRequest {
        name: "Auth".into(),
        description: Some("Authentication module".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("Auth"));
    assert!(json.contains("Authentication"));
}

#[test]
fn create_module_request_omits_none_description() {
    let req = PlaneCreateModuleRequest {
        name: "Auth".into(),
        description: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("Auth"));
    assert!(!json.contains("description"));
}

#[test]
fn create_module_request_clone() {
    let req = PlaneCreateModuleRequest {
        name: "Mod".into(),
        description: Some("desc".into()),
    };
    let cloned = req.clone();
    assert_eq!(cloned.name, "Mod");
    assert_eq!(cloned.description, Some("desc".into()));
}

#[test]
fn create_module_request_debug_format() {
    let req = PlaneCreateModuleRequest {
        name: "Mod".into(),
        description: None,
    };
    let debug = format!("{:?}", req);
    assert!(debug.contains("PlaneCreateModuleRequest"));
}

// ── PlaneModuleResponse deserialization ──────────────────────

#[test]
fn module_response_deserializes() {
    let json = r#"{"id": "mod-1", "name": "Auth", "description": "Auth module"}"#;
    let resp: PlaneModuleResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.id, "mod-1");
    assert_eq!(resp.name, "Auth");
    assert_eq!(resp.description, Some("Auth module".into()));
}

#[test]
fn module_response_none_description() {
    let json = r#"{"id": "mod-2", "name": "Test"}"#;
    let resp: PlaneModuleResponse = serde_json::from_str(json).unwrap();
    assert!(resp.description.is_none());
}

#[test]
fn module_response_clone() {
    let resp = PlaneModuleResponse {
        id: "m1".into(),
        name: "Mod".into(),
        description: None,
    };
    let cloned = resp.clone();
    assert_eq!(cloned.id, "m1");
}

// ── PlaneCreateCycleRequest ─────────────────────────────────

#[test]
fn create_cycle_request_serializes() {
    let req = PlaneCreateCycleRequest {
        name: "Sprint 1".into(),
        description: Some("First sprint".into()),
        start_date: "2026-01-01".into(),
        end_date: "2026-01-14".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("Sprint 1"));
    assert!(json.contains("2026-01-01"));
    assert!(json.contains("2026-01-14"));
}

#[test]
fn create_cycle_request_omits_none_description() {
    let req = PlaneCreateCycleRequest {
        name: "Sprint 2".into(),
        description: None,
        start_date: "2026-02-01".into(),
        end_date: "2026-02-14".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("description"));
}

#[test]
fn create_cycle_request_clone() {
    let req = PlaneCreateCycleRequest {
        name: "S1".into(),
        description: None,
        start_date: "2026-01-01".into(),
        end_date: "2026-01-14".into(),
    };
    let cloned = req.clone();
    assert_eq!(cloned.name, "S1");
    assert_eq!(cloned.start_date, "2026-01-01");
}

// ── PlaneCycleResponse deserialization ───────────────────────

#[test]
fn cycle_response_deserializes() {
    let json = r#"{
        "id": "cyc-1",
        "name": "Sprint 1",
        "start_date": "2026-01-01",
        "end_date": "2026-01-14"
    }"#;
    let resp: PlaneCycleResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.id, "cyc-1");
    assert_eq!(resp.name, "Sprint 1");
    assert_eq!(resp.start_date, Some("2026-01-01".into()));
}

#[test]
fn cycle_response_none_dates() {
    let json = r#"{"id": "cyc-2", "name": "Sprint 2"}"#;
    let resp: PlaneCycleResponse = serde_json::from_str(json).unwrap();
    assert!(resp.start_date.is_none());
    assert!(resp.end_date.is_none());
}

#[test]
fn cycle_response_clone() {
    let resp = PlaneCycleResponse {
        id: "c1".into(),
        name: "Sprint".into(),
        start_date: None,
        end_date: None,
    };
    let cloned = resp.clone();
    assert_eq!(cloned.id, "c1");
}

// ── PlaneClient construction ────────────────────────────────

#[test]
fn plane_client_construction() {
    let client = PlaneClient::new(
        "http://localhost".into(),
        "api-key".into(),
        "workspace".into(),
        "project".into(),
    );
    let debug = format!("{:?}", client);
    assert!(debug.contains("PlaneClient"));
}

#[test]
fn plane_client_clone() {
    let client = PlaneClient::new(
        "http://localhost".into(),
        "api-key".into(),
        "workspace".into(),
        "project".into(),
    );
    let cloned = client.clone();
    let debug = format!("{:?}", cloned);
    assert!(debug.contains("PlaneClient"));
}

// ── TokenBucket (re-exported) ───────────────────────────────

#[test]
fn token_bucket_acquire_and_deplete() {
    let mut bucket = agileplus_plane::client::TokenBucket::new(3.0, 1.0);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(!bucket.try_acquire()); // exhausted
}

#[test]
fn token_bucket_refills_over_time() {
    let mut bucket = agileplus_plane::client::TokenBucket::new(2.0, 100.0);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(!bucket.try_acquire());
    // Time-based test: after some time, tokens refill
    // With rate=100 tokens/sec, after a brief pause we should have tokens
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert!(bucket.try_acquire());
}

#[test]
fn token_bucket_time_until_available() {
    let mut bucket = agileplus_plane::client::TokenBucket::new(1.0, 1.0);
    assert!(bucket.try_acquire());
    let wait = bucket.time_until_available();
    assert!(wait > std::time::Duration::ZERO);
    assert!(wait <= std::time::Duration::from_secs(2)); // 1 token / 1.0 rate = 1s
}
