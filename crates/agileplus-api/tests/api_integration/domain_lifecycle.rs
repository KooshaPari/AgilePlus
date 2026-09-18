// SPDX-License-Identifier: MIT OR Apache-2.0
//! Epic and story lifecycle behaviour over the real HTTP surface.
//!
//! Covers the *successful* status transitions (the invalid ones are pinned by
//! `api_integration.rs`) plus the story-title validation branch, and asserts
//! that each transition is persisted rather than only echoed back.
//!
//! Traceability: WP12-T080 (domain-wiring)

use axum::http::StatusCode;

use crate::support::{TEST_API_KEY, setup_test_server};

const KEY: &str = "X-API-Key";

#[tokio::test]
async fn transition_epic_active_to_review_succeeds_and_persists() {
    let server = setup_test_server().await;
    // Epic 1 is seeded Active; Active -> Review is a permitted edge.
    let resp = server
        .post("/api/v1/epics/1/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_status": "review" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(
        body["status"], "review",
        "response should report the new status"
    );

    let reread = server
        .get("/api/v1/epics/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    assert_eq!(
        reread.json::<serde_json::Value>()["status"],
        "review",
        "the transition must be persisted, not only echoed"
    );
}

#[tokio::test]
async fn transition_story_todo_to_in_progress_succeeds_and_persists() {
    let server = setup_test_server().await;
    // Story 1 is seeded Todo; Todo -> InProgress is a permitted edge.
    let resp = server
        .post("/api/v1/stories/1/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_status": "in_progress" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(body["status"], "in_progress");

    let reread = server
        .get("/api/v1/stories/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    assert_eq!(reread.json::<serde_json::Value>()["status"], "in_progress");
}

#[tokio::test]
async fn create_story_blank_title_is_400() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/stories")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({
            "epic_id": 1,
            "project_id": 1,
            "title": "   ",
            "points": 3
        }))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json();
    assert!(
        body["error"].as_str().unwrap_or_default().contains("title"),
        "the validation message should name the offending field, got: {body}"
    );
}
