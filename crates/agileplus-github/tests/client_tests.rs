// Integration tests for client.rs — construction, URL building, and response deserialization.
// Supplements inline tests in src/client.rs.

use agileplus_github::client::*;

// ── GitHubClient construction ──────────────────────────────────────────────

#[test]
fn client_construction() {
    let client = GitHubClient::new(
        "https://api.github.com".to_string(),
        "ghp_test123".to_string(),
        "KooshaPari".to_string(),
        "AgilePlus".to_string(),
    );
    // Verify the client was constructed (Debug doesn't panic)
    let debug = format!("{client:?}");
    assert!(debug.contains("GitHubClient"));
}

#[test]
fn client_clone() {
    let client = GitHubClient::new(
        "https://api.github.com".to_string(),
        "token".to_string(),
        "owner".to_string(),
        "repo".to_string(),
    );
    let cloned = client.clone();
    let debug = format!("{cloned:?}");
    assert!(debug.contains("GitHubClient"));
}

#[test]
fn client_different_base_urls() {
    let c1 = GitHubClient::new(
        "https://api.github.com".to_string(),
        "t".to_string(),
        "o".to_string(),
        "r".to_string(),
    );
    let c2 = GitHubClient::new(
        "https://github.example.com/api/v3".to_string(),
        "t".to_string(),
        "o".to_string(),
        "r".to_string(),
    );
    // Both construct successfully
    let d1 = format!("{c1:?}");
    let d2 = format!("{c2:?}");
    assert_ne!(d1, d2);
}

// ── GitHubIssuePayload serialization edge cases ───────────────────────────

#[test]
fn payload_unicode_title() {
    let payload = GitHubIssuePayload {
        title: "Unicode: émojis 🚀".to_string(),
        body: "Description".to_string(),
        labels: vec!["bug".to_string()],
    };
    let json = serde_json::to_string(&payload).unwrap();
    let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.title, "Unicode: émojis 🚀");
}

#[test]
fn payload_long_body() {
    let long_body = "x".repeat(10_000);
    let payload = GitHubIssuePayload {
        title: "Long".to_string(),
        body: long_body.clone(),
        labels: vec!["bug".to_string()],
    };
    let json = serde_json::to_string(&payload).unwrap();
    let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.body.len(), 10_000);
}

#[test]
fn payload_many_labels() {
    let labels: Vec<String> = (0..100).map(|i| format!("label-{i}")).collect();
    let payload = GitHubIssuePayload {
        title: "Many labels".to_string(),
        body: "".to_string(),
        labels: labels.clone(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.labels.len(), 100);
}

#[test]
fn payload_json_structure() {
    let payload = GitHubIssuePayload {
        title: "Title".to_string(),
        body: "Body".to_string(),
        labels: vec!["bug".to_string()],
    };
    let json: serde_json::Value = serde_json::to_value(&payload).unwrap();
    assert_eq!(json["title"], "Title");
    assert_eq!(json["body"], "Body");
    assert_eq!(json["labels"][0], "bug");
}

// ── GitHubIssueResponse deserialization edge cases ────────────────────────

#[test]
fn response_with_no_labels() {
    let json = r#"{
        "number": 1,
        "title": "No labels",
        "body": null,
        "state": "open",
        "labels": [],
        "updated_at": "2025-01-01T00:00:00Z"
    }"#;
    let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
    assert!(resp.labels.is_empty());
}

#[test]
fn response_with_many_labels() {
    let json = r#"{
        "number": 1,
        "title": "Many",
        "body": null,
        "state": "open",
        "labels": [{"name": "a"}, {"name": "b"}, {"name": "c"}],
        "updated_at": "2025-01-01T00:00:00Z"
    }"#;
    let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.labels.len(), 3);
    assert_eq!(resp.labels[0].name, "a");
    assert_eq!(resp.labels[2].name, "c");
}

#[test]
fn response_number_various() {
    for num in [0, 1, 42, 999, 100_000] {
        let json = format!(
            r#"{{"number":{num},"title":"t","body":null,"state":"open","labels":[],"updated_at":"2025-01-01T00:00:00Z"}}"#
        );
        let resp: GitHubIssueResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp.number, num);
    }
}

#[test]
fn response_unicode_title() {
    let json = r#"{
        "number": 1,
        "title": "Fix émojis 🎉",
        "body": null,
        "state": "open",
        "labels": [],
        "updated_at": "2025-01-01T00:00:00Z"
    }"#;
    let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.title, "Fix émojis 🎉");
}

#[test]
fn response_body_multiline() {
    let json = r#"{
        "number": 1,
        "title": "t",
        "body": "Line 1\nLine 2\nLine 3",
        "state": "open",
        "labels": [],
        "updated_at": "2025-01-01T00:00:00Z"
    }"#;
    let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
    let body = resp.body.unwrap();
    assert!(body.contains('\n'));
    assert_eq!(body.lines().count(), 3);
}

#[test]
fn response_label_debug() {
    let label = GitHubLabel {
        name: "bug".to_string(),
    };
    let debug = format!("{label:?}");
    assert!(debug.contains("bug"));
}

#[test]
fn response_state_values() {
    for state in ["open", "closed"] {
        let json = format!(
            r#"{{"number":1,"title":"t","body":null,"state":"{state}","labels":[],"updated_at":"2025-01-01T00:00:00Z"}}"#
        );
        let resp: GitHubIssueResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp.state, state);
    }
}