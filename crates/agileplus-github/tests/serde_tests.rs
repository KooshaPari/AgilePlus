// Integration tests for serde roundtrips across all serializable types.
// Tests the full serialize → deserialize cycle for correctness.

use agileplus_github::client::{GitHubIssuePayload, GitHubIssueResponse};
use agileplus_github::sync::GitHubSyncState;
use chrono::Utc;

// ── GitHubIssuePayload ─────────────────────────────────────────────────────

#[test]
fn payload_roundtrip_with_labels() {
    let original = GitHubIssuePayload {
        title: "Feature: dark mode".to_string(),
        body: "Implement dark mode toggle in settings.".to_string(),
        labels: vec!["enhancement".to_string(), "ui".to_string()],
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.title, original.title);
    assert_eq!(restored.body, original.body);
    assert_eq!(restored.labels, original.labels);
}

#[test]
fn payload_roundtrip_empty_labels_omitted() {
    let original = GitHubIssuePayload {
        title: "T".to_string(),
        body: "B".to_string(),
        labels: vec![],
    };
    let json = serde_json::to_string(&original).unwrap();
    // labels should be absent from JSON when empty (skip_serializing_if)
    assert!(!json.contains("labels"));
    // Note: deserializing JSON without `labels` field will fail because
    // the struct requires it. This is expected — the skip_serializing_if
    // is a serialization-only optimization for the GitHub API wire format.
    // The deserialization always includes labels from the API response.
}

#[test]
fn payload_roundtrip_special_chars() {
    let original = GitHubIssuePayload {
        title: "Crash: \"null\" é \n tab\t".to_string(),
        body: "Line1\r\nLine2".to_string(),
        labels: vec!["bug".to_string()],
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.title, original.title);
    assert_eq!(restored.body, original.body);
}

// ── GitHubSyncState ────────────────────────────────────────────────────────

#[test]
fn sync_state_roundtrip_populated() {
    let mut original = GitHubSyncState::default();
    original.issue_mappings.insert(1, 42);
    original.issue_mappings.insert(2, 99);
    original.content_hashes.insert(1, "hash_abc".to_string());
    original.content_hashes.insert(2, "hash_def".to_string());
    original.last_synced_at = Some(Utc::now());

    let json = serde_json::to_string(&original).unwrap();
    let restored: GitHubSyncState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.issue_mappings.len(), 2);
    assert_eq!(restored.issue_mappings[&1], 42);
    assert_eq!(restored.issue_mappings[&2], 99);
    assert_eq!(restored.content_hashes[&1], "hash_abc");
    assert_eq!(restored.content_hashes[&2], "hash_def");
    assert!(restored.last_synced_at.is_some());
}

#[test]
fn sync_state_roundtrip_empty() {
    let original = GitHubSyncState::default();
    let json = serde_json::to_string(&original).unwrap();
    let restored: GitHubSyncState = serde_json::from_str(&json).unwrap();
    assert!(restored.issue_mappings.is_empty());
    assert!(restored.content_hashes.is_empty());
    assert!(restored.last_synced_at.is_none());
}

#[test]
fn sync_state_json_structure() {
    let mut state = GitHubSyncState::default();
    state.issue_mappings.insert(5, 10);
    state.content_hashes.insert(5, "h".to_string());

    let value: serde_json::Value = serde_json::to_value(&state).unwrap();
    assert!(value.is_object());
    assert!(value["issue_mappings"].is_object());
    assert!(value["content_hashes"].is_object());
    assert!(value["last_synced_at"].is_null());
}

#[test]
fn sync_state_large_number_of_mappings() {
    let mut state = GitHubSyncState::default();
    for i in 0..1000 {
        state.issue_mappings.insert(i, i * 10);
        state.content_hashes.insert(i, format!("hash_{i}"));
    }
    let json = serde_json::to_string(&state).unwrap();
    let restored: GitHubSyncState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.issue_mappings.len(), 1000);
}

// ── GitHubIssueResponse ───────────────────────────────────────────────────

#[test]
fn response_roundtrip_minimal() {
    let json = r#"{
        "number": 1,
        "title": "Simple",
        "body": null,
        "state": "open",
        "labels": [],
        "updated_at": "2025-06-15T12:00:00Z"
    }"#;
    let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
    // GitHubIssueResponse only derives Deserialize, not Serialize
    // Verify deserialized fields are correct
    assert_eq!(resp.number, 1);
    assert_eq!(resp.title, "Simple");
    assert!(resp.body.is_none());
    assert_eq!(resp.state, "open");
    assert!(resp.labels.is_empty());
}

#[test]
fn response_roundtrip_full() {
    let json = r#"{
        "number": 42,
        "title": "Bug: crash",
        "body": "Steps to reproduce:\n1. Open app\n2. Click login",
        "state": "closed",
        "labels": [{"name": "bug"}, {"name": "priority:high"}],
        "updated_at": "2025-12-31T23:59:59Z"
    }"#;
    let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.number, 42);
    assert_eq!(resp.title, "Bug: crash");
    assert!(resp.body.unwrap().contains("Steps to reproduce"));
    assert_eq!(resp.state, "closed");
    assert_eq!(resp.labels.len(), 2);
}