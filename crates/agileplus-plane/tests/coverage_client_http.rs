//! Deep coverage for the agileplus-plane REST client: endpoint behaviour,
//! error propagation, pagination, raw calls, label sync, and the token bucket.

use agileplus_plane::client::{
    PlaneClient, PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneIssue, PlaneWorkItem,
    PlaneWorkItemResponse, TokenBucket,
};
use agileplus_plane::labels::{LabelSync, PlaneLabel};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> PlaneClient {
    PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into())
}

fn work_item(name: &str) -> PlaneWorkItem {
    PlaneWorkItem {
        id: None,
        name: name.into(),
        description_html: Some("<p>x</p>".into()),
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec!["agileplus".into()],
    }
}

fn item_json(id: &str, name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "description_html": null,
        "state": null,
        "updated_at": null
    })
}

// ============================================================
// TokenBucket
// ============================================================

#[test]
fn token_bucket_starts_full() {
    let mut b = TokenBucket::new(3.0, 1.0);
    assert!(b.try_acquire());
    assert!(b.try_acquire());
    assert!(b.try_acquire());
    assert!(!b.try_acquire());
}

#[test]
fn token_bucket_time_until_available_zero_when_fresh() {
    let b = TokenBucket::new(5.0, 1.0);
    assert_eq!(b.time_until_available(), std::time::Duration::ZERO);
}

#[test]
fn token_bucket_time_until_available_positive_when_empty() {
    let mut b = TokenBucket::new(1.0, 0.5);
    assert!(b.try_acquire());
    assert!(b.time_until_available() > std::time::Duration::ZERO);
}

#[test]
fn token_bucket_zero_capacity_never_acquires_without_refill() {
    let mut b = TokenBucket::new(0.0, 0.0);
    assert!(!b.try_acquire());
}

#[test]
fn token_bucket_refills_over_time() {
    let mut b = TokenBucket::new(1.0, 1000.0);
    assert!(b.try_acquire());
    // With a fast refill rate, a short sleep is enough to regain a token.
    std::thread::sleep(std::time::Duration::from_millis(20));
    assert!(b.try_acquire());
}

#[test]
fn token_bucket_debug_impl() {
    let b = TokenBucket::new(1.0, 1.0);
    assert!(format!("{b:?}").contains("TokenBucket"));
}

// ============================================================
// work item CRUD
// ============================================================

#[tokio::test]
async fn create_work_item_posts_to_root_and_sends_api_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .and(header("X-API-Key", "k"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("wi-1", "Feature")))
        .mount(&server)
        .await;

    let resp = client(&server).create_work_item(&work_item("Feature")).await.unwrap();
    assert_eq!(resp.id, "wi-1");
    assert_eq!(resp.name, "Feature");
}

#[tokio::test]
async fn create_work_item_error_propagates_status() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .mount(&server)
        .await;

    let err = client(&server)
        .create_work_item(&work_item("x"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("400"));
}

#[tokio::test]
async fn create_work_item_bad_json_body_is_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    assert!(client(&server).create_work_item(&work_item("x")).await.is_err());
}

#[tokio::test]
async fn update_work_item_patches_item_url() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-9/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("wi-9", "Updated")))
        .mount(&server)
        .await;

    let resp = client(&server)
        .update_work_item("wi-9", &work_item("Updated"))
        .await
        .unwrap();
    assert_eq!(resp.id, "wi-9");
}

#[tokio::test]
async fn update_work_item_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-9/"))
        .respond_with(ResponseTemplate::new(404).set_body_string("missing"))
        .mount(&server)
        .await;

    assert!(client(&server).update_work_item("wi-9", &work_item("x")).await.is_err());
}

#[tokio::test]
async fn get_work_item_uses_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-3/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("wi-3", "Fetched")))
        .mount(&server)
        .await;

    let resp = client(&server).get_work_item("wi-3").await.unwrap();
    assert_eq!(resp.name, "Fetched");
}

#[tokio::test]
async fn get_work_item_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/nope/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    assert!(client(&server).get_work_item("nope").await.is_err());
}

#[tokio::test]
async fn list_work_items_parses_bare_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            item_json("1", "One"),
            item_json("2", "Two")
        ])))
        .mount(&server)
        .await;

    let items = client(&server).list_work_items().await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].name, "Two");
}

#[tokio::test]
async fn list_work_items_parses_paginated_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [item_json("1", "One")],
            "next": "cursor-2"
        })))
        .mount(&server)
        .await;

    let items = client(&server).list_work_items().await.unwrap();
    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn list_work_items_empty_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    assert!(client(&server).list_work_items().await.unwrap().is_empty());
}

#[tokio::test]
async fn list_work_items_malformed_body_is_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .mount(&server)
        .await;

    assert!(client(&server).list_work_items().await.is_err());
}

#[tokio::test]
async fn create_sub_issue_posts_with_parent() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("child-1", "Child")))
        .mount(&server)
        .await;

    let resp = client(&server)
        .create_sub_issue("parent-1", "Child", Some("<p>c</p>".into()))
        .await
        .unwrap();
    assert_eq!(resp.id, "child-1");
}

#[tokio::test]
async fn create_sub_issue_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(422).set_body_string("nope"))
        .mount(&server)
        .await;

    assert!(client(&server).create_sub_issue("p", "c", None).await.is_err());
}

#[tokio::test]
async fn issue_aliases_delegate_to_work_item_methods() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("i-1", "Issue")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([item_json("i-1", "Issue")])))
        .mount(&server)
        .await;

    let c = client(&server);
    assert_eq!(c.create_issue(&work_item("Issue")).await.unwrap().id, "i-1");
    assert_eq!(c.list_issues().await.unwrap().len(), 1);
}

#[tokio::test]
async fn get_issue_alias_works() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/x/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("x", "X")))
        .mount(&server)
        .await;

    assert_eq!(client(&server).get_issue("x").await.unwrap().id, "x");
}

#[tokio::test]
async fn update_issue_alias_works() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/x/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json("x", "X2")))
        .mount(&server)
        .await;

    let issue: PlaneIssue = work_item("X2");
    assert_eq!(client(&server).update_issue("x", &issue).await.unwrap().id, "x");
}

// ============================================================
// modules
// ============================================================

fn module_req() -> PlaneCreateModuleRequest {
    PlaneCreateModuleRequest {
        name: "Auth".into(),
        description: Some("auth module".into()),
    }
}

#[tokio::test]
async fn create_module_posts_and_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "m-1", "name": "Auth", "description": "auth module"
        })))
        .mount(&server)
        .await;

    let resp = client(&server).create_module(&module_req()).await.unwrap();
    assert_eq!(resp.id, "m-1");
    assert_eq!(resp.name, "Auth");
}

#[tokio::test]
async fn create_module_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("err"))
        .mount(&server)
        .await;

    assert!(client(&server).create_module(&module_req()).await.is_err());
}

#[tokio::test]
async fn update_module_patches_module_url() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    assert!(client(&server).update_module("m-1", &module_req()).await.is_ok());
}

#[tokio::test]
async fn update_module_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;

    assert!(client(&server).update_module("m-1", &module_req()).await.is_err());
}

#[tokio::test]
async fn delete_module_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    assert!(client(&server).delete_module("m-1").await.is_ok());
}

#[tokio::test]
async fn delete_module_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("err"))
        .mount(&server)
        .await;

    assert!(client(&server).delete_module("m-1").await.is_err());
}

#[tokio::test]
async fn add_work_item_to_module_posts_module_issues() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/module-issues/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    assert!(client(&server).add_work_item_to_module("m-1", "wi-1").await.is_ok());
}

#[tokio::test]
async fn add_work_item_to_module_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/module-issues/"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .mount(&server)
        .await;

    assert!(client(&server).add_work_item_to_module("m-1", "wi-1").await.is_err());
}

#[tokio::test]
async fn delete_work_item_from_module_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/module-issues/wi-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    assert!(client(&server).delete_work_item_from_module("m-1", "wi-1").await.is_ok());
}

#[tokio::test]
async fn delete_work_item_from_module_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/m-1/module-issues/wi-1/"))
        .respond_with(ResponseTemplate::new(404).set_body_string("gone"))
        .mount(&server)
        .await;

    assert!(client(&server).delete_work_item_from_module("m-1", "wi-1").await.is_err());
}

// ============================================================
// cycles
// ============================================================

fn cycle_req() -> PlaneCreateCycleRequest {
    PlaneCreateCycleRequest {
        name: "Sprint 1".into(),
        description: None,
        start_date: "2026-01-01".into(),
        end_date: "2026-01-14".into(),
    }
}

#[tokio::test]
async fn create_cycle_posts_and_parses_dates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "c-1", "name": "Sprint 1",
            "start_date": "2026-01-01", "end_date": "2026-01-14"
        })))
        .mount(&server)
        .await;

    let resp = client(&server).create_cycle(&cycle_req()).await.unwrap();
    assert_eq!(resp.id, "c-1");
    assert_eq!(resp.end_date.as_deref(), Some("2026-01-14"));
}

#[tokio::test]
async fn create_cycle_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("err"))
        .mount(&server)
        .await;

    assert!(client(&server).create_cycle(&cycle_req()).await.is_err());
}

#[tokio::test]
async fn update_cycle_patches_cycle_url() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    assert!(client(&server).update_cycle("c-1", &cycle_req()).await.is_ok());
}

#[tokio::test]
async fn update_cycle_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .mount(&server)
        .await;

    assert!(client(&server).update_cycle("c-1", &cycle_req()).await.is_err());
}

#[tokio::test]
async fn delete_cycle_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    assert!(client(&server).delete_cycle("c-1").await.is_ok());
}

#[tokio::test]
async fn delete_cycle_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("err"))
        .mount(&server)
        .await;

    assert!(client(&server).delete_cycle("c-1").await.is_err());
}

#[tokio::test]
async fn add_work_item_to_cycle_posts_cycle_issues() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/cycle-issues/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    assert!(client(&server).add_work_item_to_cycle("c-1", "wi-1").await.is_ok());
}

#[tokio::test]
async fn add_work_item_to_cycle_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/cycle-issues/"))
        .respond_with(ResponseTemplate::new(409).set_body_string("conflict"))
        .mount(&server)
        .await;

    assert!(client(&server).add_work_item_to_cycle("c-1", "wi-1").await.is_err());
}

#[tokio::test]
async fn delete_work_item_from_cycle_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/cycle-issues/wi-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    assert!(client(&server).delete_work_item_from_cycle("c-1", "wi-1").await.is_ok());
}

#[tokio::test]
async fn delete_work_item_from_cycle_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c-1/cycle-issues/wi-1/"))
        .respond_with(ResponseTemplate::new(404).set_body_string("gone"))
        .mount(&server)
        .await;

    assert!(client(&server).delete_work_item_from_cycle("c-1", "wi-1").await.is_err());
}

// ============================================================
// raw calls and labels
// ============================================================

#[tokio::test]
async fn get_raw_returns_body_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/anything"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hello"))
        .mount(&server)
        .await;

    let body = client(&server).get_raw(&format!("{}/anything", server.uri())).await.unwrap();
    assert_eq!(body, "hello");
}

#[tokio::test]
async fn get_raw_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/anything"))
        .respond_with(ResponseTemplate::new(503).set_body_string("down"))
        .mount(&server)
        .await;

    let err = client(&server).get_raw(&format!("{}/anything", server.uri())).await.unwrap_err();
    assert!(err.to_string().contains("503"));
}

#[tokio::test]
async fn post_raw_sends_body_and_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/anything"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"ok\":true}"))
        .mount(&server)
        .await;

    let body = client(&server)
        .post_raw(&format!("{}/anything", server.uri()), r#"{"name":"x"}"#)
        .await
        .unwrap();
    assert!(body.contains("ok"));
}

#[tokio::test]
async fn post_raw_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/anything"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .mount(&server)
        .await;

    assert!(client(&server)
        .post_raw(&format!("{}/anything", server.uri()), "{}")
        .await
        .is_err());
}

#[test]
fn labels_url_builds_expected_path() {
    let c = PlaneClient::new("https://api.plane.so".into(), "k".into(), "ws".into(), "p".into());
    assert_eq!(
        c.labels_url(),
        "https://api.plane.so/api/v1/workspaces/ws/projects/p/labels/"
    );
}

#[tokio::test]
async fn fetch_remote_labels_parses_bare_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "1", "name": "bug"},
            {"id": "2", "name": "feature", "color": "#f00"}
        ])))
        .mount(&server)
        .await;

    let labels = LabelSync::new(client(&server)).fetch_remote_labels().await.unwrap();
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[1].name, "feature");
}

#[tokio::test]
async fn fetch_remote_labels_parses_wrapped_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": "1", "name": "bug"}]
        })))
        .mount(&server)
        .await;

    let labels = LabelSync::new(client(&server)).fetch_remote_labels().await.unwrap();
    assert_eq!(labels.len(), 1);
}

#[tokio::test]
async fn fetch_remote_labels_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("err"))
        .mount(&server)
        .await;

    assert!(LabelSync::new(client(&server)).fetch_remote_labels().await.is_err());
}

#[tokio::test]
async fn create_remote_label_posts_and_parses() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "lbl-1", "name": "urgent", "color": "#f00"
        })))
        .mount(&server)
        .await;

    let label = LabelSync::new(client(&server))
        .create_remote_label("urgent", Some("#f00"))
        .await
        .unwrap();
    assert_eq!(label.id, "lbl-1");
}

#[tokio::test]
async fn create_remote_label_error_propagates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .mount(&server)
        .await;

    assert!(LabelSync::new(client(&server))
        .create_remote_label("x", None)
        .await
        .is_err());
}

#[tokio::test]
async fn sync_labels_to_remote_creates_missing_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "existing-1", "name": "bug"}
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "new-1", "name": "feature"
        })))
        .mount(&server)
        .await;

    let map = LabelSync::new(client(&server))
        .sync_labels_to_remote(&["bug".into(), "feature".into()])
        .await
        .unwrap();
    assert_eq!(map.get("bug").map(String::as_str), Some("existing-1"));
    assert_eq!(map.get("feature").map(String::as_str), Some("new-1"));
}

#[tokio::test]
async fn sync_labels_from_remote_returns_names() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "1", "name": "bug"},
            {"id": "2", "name": "feature"}
        ])))
        .mount(&server)
        .await;

    let names = LabelSync::new(client(&server))
        .sync_labels_from_remote()
        .await
        .unwrap();
    assert_eq!(names, vec!["bug".to_string(), "feature".to_string()]);
}

#[tokio::test]
async fn client_sync_labels_delegates_to_label_sync() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "n1", "name": "fresh"
        })))
        .mount(&server)
        .await;

    let map = client(&server).sync_labels(&["fresh".into()]).await.unwrap();
    assert_eq!(map.get("fresh").map(String::as_str), Some("n1"));
}

#[tokio::test]
async fn client_get_labels_returns_labels() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/labels/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "1", "name": "bug"}
        ])))
        .mount(&server)
        .await;

    let labels: Vec<PlaneLabel> = client(&server).get_labels().await.unwrap();
    assert_eq!(labels.len(), 1);
}

#[test]
fn work_item_response_from_json_roundtrip() {
    let resp: PlaneWorkItemResponse =
        serde_json::from_str(r#"{"id":"a","name":"n","state":"started"}"#).unwrap();
    assert_eq!(resp.state.as_deref(), Some("started"));
}
