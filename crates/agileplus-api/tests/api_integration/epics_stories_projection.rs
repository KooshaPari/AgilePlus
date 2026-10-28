//! Integration tests for the canonical epic/story projection.
//!
//! The dashboard consumes `/api/v1/projects/{slug}/epics` and
//! `/api/v1/epics/{id}/stories`. Both must carry the `requirement_id` that the
//! storage layer already persists, so the frontend can link a row to its
//! requirement without inventing one.
//!
//! Traceability: FR-STORE-EPIC

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::story::{Story, StoryStatus};
use chrono::Utc;

use crate::support::{MockStorage, TEST_API_KEY, setup_test_server_with_storage};

const KEY: &str = "X-API-Key";

/// Storage whose epic and story each carry a requirement link.
fn storage_with_requirement_links() -> MockStorage {
    let storage = MockStorage::with_test_data();
    let now = Utc::now();

    storage.epics.lock().expect("epics lock poisoned").clear();
    storage
        .epics
        .lock()
        .expect("epics lock poisoned")
        .push(Epic {
            id: 1,
            project_id: 1,
            title: "Requirement linked epic".to_string(),
            description: None,
            status: EpicStatus::Active,
            owner_id: None,
            requirement_id: Some("FR-EPIC-1".to_string()),
            created_at: now,
            updated_at: now,
        });

    storage
        .stories
        .lock()
        .expect("stories lock poisoned")
        .clear();
    storage
        .stories
        .lock()
        .expect("stories lock poisoned")
        .push(Story {
            id: 1,
            epic_id: 1,
            project_id: 1,
            title: "Requirement linked story".to_string(),
            description: None,
            status: StoryStatus::Todo,
            points: Some(3),
            assignee_id: None,
            requirement_id: Some("FR-STORY-1".to_string()),
            created_at: now,
            updated_at: now,
        });

    storage
}

#[tokio::test]
async fn list_project_epics_exposes_requirement_id() {
    let server = setup_test_server_with_storage(storage_with_requirement_links()).await;
    let resp = server
        .get("/api/v1/projects/test-project/epics")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();

    let body: serde_json::Value = resp.json();
    let arr = body.as_array().expect("epics response should be an array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "Requirement linked epic");
    assert_eq!(arr[0]["requirement_id"], "FR-EPIC-1");
}

#[tokio::test]
async fn list_epic_stories_exposes_requirement_id() {
    let server = setup_test_server_with_storage(storage_with_requirement_links()).await;
    let resp = server
        .get("/api/v1/epics/1/stories")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();

    let body: serde_json::Value = resp.json();
    let arr = body
        .as_array()
        .expect("stories response should be an array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "Requirement linked story");
    assert_eq!(arr[0]["requirement_id"], "FR-STORY-1");
}
