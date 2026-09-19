// SPDX-License-Identifier: MIT OR Apache-2.0
//! `VcsPort` contract and failure behaviour.
//!
//! Every git-facing handler (`/api/v1/branches/*`, `/api/v1/worktrees`) calls
//! into `VcsPort` and maps a `DomainError` through `ApiError::from`. Until now
//! no suite made a VCS call fail, so all eight `Err` arms were unexecuted, and
//! the only observable VCS behaviour was the stub's empty/None replies.
//!
//! Three groups:
//!
//! 1. **Failures.** A failing git operation must surface as the fixed
//!    `{"error": "internal server error"}` envelope, must stay JSON, and must
//!    not echo the underlying git/IO message.
//! 2. **Pass-through.** The values a handler received (path, query, body) must
//!    be the values it handed to the port — asserted against the port's own
//!    call log rather than the stub's canned reply.
//! 3. **Result mapping.** A *populated* port result (branches, worktrees, a
//!    merge with a commit and conflicts) must be rendered faithfully.
//!
//! Traceability: WP11-T065, FR-D01

use agileplus_domain::ports::vcs::{BranchInfo, MergeResult, WorktreeInfo};
use axum::http::StatusCode;
use axum_test::TestResponse;
use std::path::PathBuf;

use crate::support::{MockStorage, MockVcs, TEST_API_KEY, setup_test_server_with_ports};

const KEY: &str = "X-API-Key";

/// A git-layer failure message a client must never see.
const GIT_FAILURE: &str = "git: cannot lock ref 'refs/heads/main' at /Users/secret/repo";

fn vcs_failing_at(operation: &'static str) -> MockVcs {
    let vcs = MockVcs::new();
    vcs.fail_on(operation, GIT_FAILURE);
    vcs
}

async fn server_with_vcs(vcs: MockVcs) -> axum_test::TestServer {
    setup_test_server_with_ports(MockStorage::with_test_data(), vcs).await
}

/// Every VCS failure must produce the same fixed, leak-free envelope.
fn assert_generic_500(resp: &TestResponse) {
    resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
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
        !body.to_string().contains("refs/heads"),
        "the git message must not leak to the client, got: {body}"
    );
    assert!(
        !body.to_string().contains("/Users/secret"),
        "the repository path must not leak to the client, got: {body}"
    );
}

// ── 1. Failures ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_branches_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("list_branches")).await;
    let resp = server
        .get("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn create_branch_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("create_branch")).await;
    let resp = server
        .post("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "feat/x" }))
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn checkout_branch_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("checkout_branch")).await;
    let resp = server
        .post("/api/v1/branches/checkout")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "feat/x" }))
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn delete_branch_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("delete_branch")).await;
    let resp = server
        .post("/api/v1/branches/delete")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "feat/x", "remote": "origin" }))
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn sync_branches_surfaces_a_merge_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("merge_to_target")).await;
    let resp = server
        .post("/api/v1/branches/sync")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "source": "main", "target": "canary" }))
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn list_worktrees_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("list_worktrees")).await;
    let resp = server
        .get("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn add_worktree_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("create_worktree")).await;
    let resp = server
        .post("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "feature_slug": "feat/z", "wp_id": "WP-Z" }))
        .await;
    assert_generic_500(&resp);
}

#[tokio::test]
async fn remove_worktree_surfaces_a_vcs_failure_as_a_generic_500() {
    let server = server_with_vcs(vcs_failing_at("cleanup_worktree")).await;
    let resp = server
        .delete("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "path": "/tmp/wt" }))
        .await;
    assert_generic_500(&resp);
}

// ── 2. Pass-through ──────────────────────────────────────────────────────────

#[tokio::test]
async fn list_branches_forwards_pattern_and_remote_flag_to_the_port() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    server
        .get("/api/v1/branches?pattern=feat/*&remote=true")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status_ok();

    let calls = vcs.calls_of("list_branches");
    assert_eq!(calls.len(), 1, "exactly one port call");
    assert_eq!(
        calls[0].args,
        vec!["feat/*".to_string(), "true".to_string()],
        "the pattern and remote flag must reach the port unchanged"
    );
}

#[tokio::test]
async fn list_branches_defaults_remote_to_false_and_pattern_to_absent() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    server
        .get("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status_ok();

    let calls = vcs.calls_of("list_branches");
    assert_eq!(
        calls[0].args,
        vec!["<none>".to_string(), "false".to_string()],
        "an absent pattern must stay absent and remote must default to false"
    );
}

#[tokio::test]
async fn create_branch_forwards_an_explicit_base_to_the_port() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    let resp = server
        .post("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "feat/y", "base": "develop" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    assert_eq!(
        resp.json::<serde_json::Value>()["message"],
        "Created branch feat/y from develop"
    );
    assert_eq!(
        vcs.calls_of("create_branch")[0].args,
        vec!["feat/y".to_string(), "develop".to_string()]
    );
}

#[tokio::test]
async fn delete_branch_forwards_force_and_remote_to_the_port() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    server
        .post("/api/v1/branches/delete")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "old", "force": true, "remote": "upstream" }))
        .await
        .assert_status_ok();

    assert_eq!(
        vcs.calls_of("delete_branch")[0].args,
        vec!["old".to_string(), "true".to_string(), "upstream".to_string()],
        "force must not default away when explicitly set"
    );
}

#[tokio::test]
async fn sync_branches_forwards_source_and_target_to_the_port() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    server
        .post("/api/v1/branches/sync")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "source": "release/1.0", "target": "main" }))
        .await
        .assert_status_ok();

    assert_eq!(
        vcs.calls_of("merge_to_target")[0].args,
        vec!["release/1.0".to_string(), "main".to_string()]
    );
}

#[tokio::test]
async fn add_worktree_forwards_the_slug_and_wp_id_to_the_port() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    server
        .post("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "feature_slug": "feat/deep", "wp_id": "WP-42" }))
        .await
        .assert_status(StatusCode::CREATED);

    assert_eq!(
        vcs.calls_of("create_worktree")[0].args,
        vec!["feat/deep".to_string(), "WP-42".to_string()]
    );
}

#[tokio::test]
async fn remove_worktree_forwards_the_path_to_the_port() {
    let vcs = MockVcs::new();
    let server = server_with_vcs(vcs.clone()).await;

    let resp = server
        .delete("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "path": "/srv/wt/feat-deep" }))
        .await;
    resp.assert_status_ok();
    assert_eq!(
        resp.json::<serde_json::Value>()["message"],
        "Removed worktree /srv/wt/feat-deep"
    );
    assert_eq!(
        vcs.calls_of("cleanup_worktree")[0].args,
        vec!["/srv/wt/feat-deep".to_string()]
    );
}

// ── 3. Result mapping ────────────────────────────────────────────────────────

#[tokio::test]
async fn list_branches_renders_every_branch_field() {
    let vcs = MockVcs::new();
    vcs.with_branches(vec![
        BranchInfo {
            name: "main".to_string(),
            commit: "abc1234".to_string(),
            is_remote: false,
        },
        BranchInfo {
            name: "origin/main".to_string(),
            commit: "def5678".to_string(),
            is_remote: true,
        },
    ]);
    let server = server_with_vcs(vcs).await;

    let resp = server
        .get("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: Vec<serde_json::Value> = resp.json();
    assert_eq!(body.len(), 2);
    assert_eq!(body[0]["name"], "main");
    assert_eq!(body[0]["commit"], "abc1234");
    assert_eq!(body[0]["is_remote"], false);
    assert_eq!(body[1]["name"], "origin/main");
    assert_eq!(body[1]["is_remote"], true);
}

#[tokio::test]
async fn list_worktrees_renders_every_worktree_field() {
    let vcs = MockVcs::new();
    vcs.with_worktrees(vec![WorktreeInfo {
        path: PathBuf::from("/srv/wt/feat-a"),
        commit: "cafe123".to_string(),
        branch: "feat/a".to_string(),
        feature_slug: "feat-a".to_string(),
        wp_id: "WP01".to_string(),
    }]);
    let server = server_with_vcs(vcs).await;

    let resp = server
        .get("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: Vec<serde_json::Value> = resp.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["path"], "/srv/wt/feat-a");
    assert_eq!(body[0]["branch"], "feat/a");
    assert_eq!(body[0]["commit"], "cafe123");
    assert_eq!(body[0]["feature_slug"], "feat-a");
    assert_eq!(body[0]["wp_id"], "WP01");
}

#[tokio::test]
async fn add_worktree_derives_the_branch_from_a_nested_path() {
    let vcs = MockVcs::new();
    vcs.with_worktree_path("/srv/wtrees/feat-a/WP02");
    let server = server_with_vcs(vcs).await;

    let resp = server
        .post("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "feature_slug": "feat-a", "wp_id": "WP02" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["path"], "/srv/wtrees/feat-a/WP02");
    assert_eq!(
        body["branch"], "WP02",
        "the branch label is the last path segment"
    );
    assert_eq!(
        body["commit"], "",
        "the port returns no commit, so the field is empty rather than absent"
    );
}

/// A path with no final segment (`/`) has no branch label. The handler must
/// fall back to an empty string instead of panicking or omitting the field.
#[tokio::test]
async fn add_worktree_falls_back_to_an_empty_branch_for_a_root_path() {
    let vcs = MockVcs::new();
    vcs.with_worktree_path("/");
    let server = server_with_vcs(vcs).await;

    let resp = server
        .post("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "feature_slug": "feat-root", "wp_id": "WP-R" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["path"], "/");
    assert_eq!(body["branch"], "");
    assert_eq!(body["feature_slug"], "feat-root");
}

#[tokio::test]
async fn sync_branches_echoes_a_merged_commit_from_the_port() {
    let vcs = MockVcs::new();
    vcs.with_merge_result(MergeResult {
        success: true,
        conflicts: vec![],
        merged_commit: Some("9f8e7d6".to_string()),
        commit: Some("9f8e7d6".to_string()),
        message: Some("fast-forward".to_string()),
    });
    let server = server_with_vcs(vcs).await;

    let resp = server
        .post("/api/v1/branches/sync")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "source": "main", "target": "canary" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["merged_commit"], "9f8e7d6");
}

#[tokio::test]
async fn sync_branches_reports_a_failed_merge_without_a_commit() {
    let vcs = MockVcs::new();
    vcs.with_merge_result(MergeResult {
        success: false,
        conflicts: vec![],
        merged_commit: None,
        commit: None,
        message: Some("merge conflict".to_string()),
    });
    let server = server_with_vcs(vcs).await;

    let resp = server
        .post("/api/v1/branches/sync")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "source": "main", "target": "canary" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["success"], false,
        "a port-reported failure is a successful HTTP call with success=false"
    );
    assert!(body["merged_commit"].is_null());
}
