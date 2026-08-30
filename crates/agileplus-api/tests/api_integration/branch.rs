use axum::http::StatusCode;
use serde_json::Value;

use super::support::{TEST_API_KEY, setup_test_server};

#[tokio::test]
async fn branch_endpoints_work() {
    let server = setup_test_server().await;

    let created = server
        .post("/api/v1/branches")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"name":"feat/demo","base":"main"}))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body: Value = created.json();
    assert_eq!(body["message"], "Created branch feat/demo from main");

    let sync = server
        .post("/api/v1/branches/sync")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"source":"main","target":"canary"}))
        .await;
    sync.assert_status_ok();
    let body: Value = sync.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["source"], "main");
    assert_eq!(body["target"], "canary");

    let listed = server
        .get("/api/v1/branches")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    listed.assert_status_ok();
}

#[tokio::test]
async fn branch_mutations_require_authentication() {
    let server = setup_test_server().await;

    server
        .post("/api/v1/branches")
        .json(&serde_json::json!({"name":"feat/no-auth"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    server
        .post("/api/v1/branches/checkout")
        .json(&serde_json::json!({"name":"feat/no-auth"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    server
        .post("/api/v1/branches/delete")
        .json(&serde_json::json!({"name":"feat/no-auth"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    server
        .post("/api/v1/branches/sync")
        .json(&serde_json::json!({"source":"main","target":"canary"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn branch_actions_return_contract_messages() {
    let server = setup_test_server().await;

    let created = server
        .post("/api/v1/branches")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"name":"feat/default-base"}))
        .await;
    created.assert_status(StatusCode::CREATED);
    assert_eq!(
        created.json::<Value>()["message"],
        "Created branch feat/default-base from main"
    );

    let checkout = server
        .post("/api/v1/branches/checkout")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"name":"feat/default-base"}))
        .await;
    checkout.assert_status_ok();
    assert_eq!(
        checkout.json::<Value>()["message"],
        "Checked out branch feat/default-base"
    );

    let local_delete = server
        .post("/api/v1/branches/delete")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"name":"feat/default-base","force":true}))
        .await;
    local_delete.assert_status_ok();
    assert_eq!(
        local_delete.json::<Value>()["message"],
        "Deleted branch feat/default-base"
    );

    let remote_delete = server
        .post("/api/v1/branches/delete")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"name":"feat/default-base","remote":"origin"}))
        .await;
    remote_delete.assert_status_ok();
    assert_eq!(
        remote_delete.json::<Value>()["message"],
        "Deleted remote branch origin/feat/default-base"
    );
}

#[tokio::test]
async fn malformed_branch_requests_are_rejected() {
    let server = setup_test_server().await;

    server
        .post("/api/v1/branches")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"base":"main"}))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    server
        .post("/api/v1/branches/sync")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"source":"main"}))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}
