//! Real SQLite HTTP witnesses for authenticated work-item creation and audit.
use agileplus_api::{AppState, create_router};
use agileplus_domain::{
    config::AppConfig,
    credentials::{CredentialStore, InMemoryCredentialStore, format_api_key_hash, keys},
};
use agileplus_sqlite::SqliteStorageAdapter;
use axum_test::TestServer;
use serde_json::{Value, json};
use std::sync::Arc;
#[path = "api_integration/support/observability.rs"]
mod observability;
#[path = "api_integration/support/vcs.rs"]
mod vcs;
fn server(storage: Arc<SqliteStorageAdapter>) -> TestServer {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            "agileplus",
            keys::API_KEYS,
            &format_api_key_hash("journey-test-key"),
        )
        .unwrap();
    let state = AppState::new(
        storage,
        Arc::new(vcs::MockVcs::new()),
        Arc::new(observability::MockObs),
        Arc::new(AppConfig::default()),
        Arc::new(credentials),
    )
    .with_work_items();
    TestServer::new(create_router(state))
}
#[tokio::test]
async fn work_items_http_auth_chain_validation_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journey.sqlite");
    let storage = Arc::new(SqliteStorageAdapter::new(&path).unwrap());
    let s = server(storage.clone());
    let body = json!({"kind":"task","title":"JOURNEY-work-item"});
    s.post("/api/v1/work-items")
        .json(&body)
        .await
        .assert_status_unauthorized();
    s.get("/api/v1/work-items/1/audit-events")
        .await
        .assert_status_unauthorized();
    s.post("/api/v1/work-items/1/transition")
        .json(&json!({"state":"triaged"}))
        .await
        .assert_status_unauthorized();
    s.post("/api/v1/work-items")
        .add_header("X-API-Key", "journey-test-key")
        .json(&json!({"kind":"task","title":" "}))
        .await
        .assert_status_unprocessable_entity();
    s.post("/api/v1/work-items")
        .add_header("X-API-Key", "journey-test-key")
        .json(&json!({"kind":"task","title":"JOURNEY-invalid-cycle","cycle_id":99999}))
        .await
        .assert_status_unprocessable_entity();
    let r = s
        .post("/api/v1/work-items")
        .add_header("X-API-Key", "journey-test-key")
        .json(&body)
        .await;
    r.assert_status(axum::http::StatusCode::CREATED);
    let w: Value = r.json();
    let id = w["id"].as_i64().unwrap();
    let transition = format!("/api/v1/work-items/{id}/transition");
    let audit = format!("/api/v1/work-items/{id}/audit-events");
    s.post(&transition)
        .add_header("X-API-Key", "journey-test-key")
        .json(&json!({"state":"in_review"}))
        .await
        .assert_status_conflict();
    s.post(&transition)
        .add_header("X-API-Key", "journey-test-key")
        .json(&json!({"state":"accepted"}))
        .await
        .assert_status_unprocessable_entity();
    s.post(&transition)
        .add_header("X-API-Key", "journey-test-key")
        .json(&json!({"state":"triaged","actor":"forged"}))
        .await
        .assert_status_unprocessable_entity();
    s.post(&transition)
        .add_header("X-API-Key", "journey-test-key")
        .json(&json!({"state":"triaged"}))
        .await
        .assert_status_ok();
    let r = s
        .get(&audit)
        .add_header("X-API-Key", "journey-test-key")
        .await;
    r.assert_status_ok();
    let events: Vec<Value> = r.json();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1]["prev_hash"], events[0]["hash"]);
    assert_eq!(events[0]["actor"], "api-key");
    s.get("/api/v1/work-items/9999/audit-events")
        .add_header("X-API-Key", "journey-test-key")
        .await
        .assert_status_not_found();
    drop(s);
    drop(storage);
    let s = server(Arc::new(SqliteStorageAdapter::new(&path).unwrap()));
    let r = s
        .get(&audit)
        .add_header("X-API-Key", "journey-test-key")
        .await;
    r.assert_status_ok();
    assert_eq!(r.json::<Vec<Value>>(), events);
}
