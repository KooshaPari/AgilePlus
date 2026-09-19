//! Request-mapping coverage for the Plane client back-compat alias surface in
//! `client/resources/work_items.rs`.
//!
//! The `issue`-named aliases (`create_issue`, `update_issue`, `get_issue`,
//! `list_issues`, `add_issue_to_module`, `add_issue_to_cycle`) exist so the
//! outbound/sync code can keep talking in "issue" terms while the client moved
//! to "work item" naming. Aliases must not drift: each one has to issue the
//! *same* HTTP request as the method it forwards to, against the same Plane
//! endpoint, with the same JSON body, and propagate the same errors.
//!
//! This file drives the aliases against a real HTTP mock server so the
//! wire-level request (method, path, auth header, body) is observed rather
//! than assumed.

use agileplus_plane::client::PlaneClient;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> PlaneClient {
    PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into())
}

// ============================================================
// add_issue_to_module
// ============================================================

#[tokio::test]
async fn add_issue_to_module_posts_issue_ids_to_the_module_issues_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-7/module-issues/",
        ))
        .and(header("X-API-Key", "k"))
        .and(body_json(serde_json::json!({ "issues": ["wi-9"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    client(&server)
        .add_issue_to_module("mod-7", "wi-9")
        .await
        .expect("alias must POST the issue id to the module-issues endpoint");
}

#[tokio::test]
async fn add_issue_to_module_propagates_failure_status_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-7/module-issues/",
        ))
        .respond_with(ResponseTemplate::new(422).set_body_string("module is archived"))
        .mount(&server)
        .await;

    let err = client(&server)
        .add_issue_to_module("mod-7", "wi-9")
        .await
        .expect_err("non-2xx must surface as an error");

    let msg = err.to_string();
    assert!(
        msg.contains("add work item to module"),
        "unexpected error: {msg}"
    );
    assert!(msg.contains("422"), "status must be reported: {msg}");
    assert!(
        msg.contains("module is archived"),
        "body must be reported: {msg}"
    );
}

// ============================================================
// add_issue_to_cycle
// ============================================================

#[tokio::test]
async fn add_issue_to_cycle_posts_issue_ids_to_the_cycle_issues_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-3/cycle-issues/",
        ))
        .and(header("X-API-Key", "k"))
        .and(body_json(serde_json::json!({ "issues": ["wi-9"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    client(&server)
        .add_issue_to_cycle("cyc-3", "wi-9")
        .await
        .expect("alias must POST the issue id to the cycle-issues endpoint");
}

#[tokio::test]
async fn add_issue_to_cycle_propagates_failure_status_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-3/cycle-issues/",
        ))
        .respond_with(ResponseTemplate::new(409).set_body_string("cycle closed"))
        .mount(&server)
        .await;

    let err = client(&server)
        .add_issue_to_cycle("cyc-3", "wi-9")
        .await
        .expect_err("non-2xx must surface as an error");

    let msg = err.to_string();
    assert!(
        msg.contains("add work item to cycle"),
        "unexpected error: {msg}"
    );
    assert!(msg.contains("409"), "status must be reported: {msg}");
    assert!(msg.contains("cycle closed"), "body must be reported: {msg}");
}

// ============================================================
// Alias identity: alias and canonical method hit the same URL
// ============================================================

#[tokio::test]
async fn module_alias_issues_the_same_request_as_the_canonical_method() {
    let server = MockServer::start().await;
    // Exactly one request expected: the alias. If it drifted to a different
    // path or body, the mock would not match and the call would fail.
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-1/module-issues/",
        ))
        .and(body_json(serde_json::json!({ "issues": ["wi-1"] })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({})))
        .expect(2)
        .mount(&server)
        .await;

    let c = client(&server);
    c.add_issue_to_module("mod-1", "wi-1").await.unwrap();
    c.add_work_item_to_module("mod-1", "wi-1").await.unwrap();

    server.verify().await;
}

#[tokio::test]
async fn cycle_alias_issues_the_same_request_as_the_canonical_method() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/cycle-issues/",
        ))
        .and(body_json(serde_json::json!({ "issues": ["wi-1"] })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({})))
        .expect(2)
        .mount(&server)
        .await;

    let c = client(&server);
    c.add_issue_to_cycle("cyc-1", "wi-1").await.unwrap();
    c.add_work_item_to_cycle("cyc-1", "wi-1").await.unwrap();

    server.verify().await;
}
