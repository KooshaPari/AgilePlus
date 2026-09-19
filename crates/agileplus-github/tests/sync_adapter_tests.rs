// Integration tests for `GitHubSyncAdapter` and `GitHubSyncState` sync
// behaviour, driven over a local in-process mock server
// (`tests/common/mod.rs`). Covers the create / skip / update / conflict
// decision table, state bookkeeping, and polling. No network access.

mod common;

use agileplus_github::client::GitHubClient;
use agileplus_github::sync::{GitHubSyncAdapter, GitHubSyncState, SyncOutcome};
use common::{MockGitHub, MockResponse, RecordedRequest, sample_bug, sha256_hex};

/// Body text used to seed `content_hashes` in the update/conflict scenarios.
const PREVIOUS_BODY: &str = "## Description\n\nwhat we last pushed\n";

fn adapter_for(server: &MockGitHub) -> GitHubSyncAdapter {
    GitHubSyncAdapter::new(GitHubClient::new(
        server.base_url().to_string(),
        "ghp_test_token".to_string(),
        "acme".to_string(),
        "widgets".to_string(),
    ))
}

fn state_with_remote_hash(item_id: i64, issue_number: i64, remote_body: &str) -> GitHubSyncState {
    let mut state = GitHubSyncState::default();
    state.issue_mappings.insert(item_id, issue_number);
    state
        .content_hashes
        .insert(item_id, sha256_hex(remote_body));
    state
}

fn paths(requests: &[RecordedRequest]) -> Vec<(String, String)> {
    requests
        .iter()
        .map(|request| (request.method.clone(), request.path.clone()))
        .collect()
}

// ── sync_bug: create path ──────────────────────────────────────────────────

#[tokio::test]
async fn sync_bug_creates_an_issue_and_records_the_mapping() {
    let server = MockGitHub::start(|_| MockResponse::issue(101, "Login crash", None, "open")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();
    let item = sample_bug(1);

    let outcome = adapter.sync_bug(&mut state, &item).await.unwrap();

    // The issue number comes from the API response, not from the request.
    assert_eq!(outcome, SyncOutcome::Created(101));
    assert_eq!(state.issue_mappings[&1], 101);
    assert!(state.last_synced_at.is_some());
    assert_eq!(server.request_count(), 1);
}

#[tokio::test]
async fn sync_bug_posts_the_bug_title_labels_and_formatted_body() {
    let server = MockGitHub::start(|_| MockResponse::issue(101, "t", None, "open")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();

    adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    let request = &server.requests()[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/repos/acme/widgets/issues");
    let body = request.json();
    assert_eq!(body["title"], "[Bug] Login crash");
    assert_eq!(body["labels"][0], "bug");
    assert_eq!(body["labels"][1], "agileplus");
    assert_eq!(body["labels"][2], "priority:high");
    let posted_body = body["body"].as_str().expect("body must be a string");
    assert!(posted_body.starts_with("## Description\n\nApp crashes when clicking login"));
    assert!(posted_body.ends_with("*Synced by AgilePlus*\n"));
}

#[tokio::test]
async fn sync_bug_records_the_hash_of_the_body_it_actually_sent() {
    let server = MockGitHub::start(|_| MockResponse::issue(101, "t", None, "open")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();

    adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    let posted_body = server.requests()[0].json()["body"]
        .as_str()
        .expect("body must be a string")
        .to_string();
    assert_eq!(state.content_hashes[&1], sha256_hex(&posted_body));
}

#[tokio::test]
async fn sync_bug_without_an_item_id_fails_before_any_request() {
    let server = MockGitHub::start(|_| MockResponse::issue(101, "t", None, "open")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();
    let mut item = sample_bug(1);
    item.id = None;

    let error = adapter.sync_bug(&mut state, &item).await.unwrap_err();

    assert_eq!(error.to_string(), "backlog item must have an ID");
    assert_eq!(server.request_count(), 0);
    assert!(state.issue_mappings.is_empty());
}

#[tokio::test]
async fn sync_bug_surfaces_create_failures_and_leaves_state_untouched() {
    let server =
        MockGitHub::start(|_| MockResponse::error("500 Internal Server Error", "boom")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();

    let error = adapter
        .sync_bug(&mut state, &sample_bug(1))
        .await
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "GitHub API error 500 Internal Server Error: boom"
    );
    assert!(state.issue_mappings.is_empty());
    assert!(state.content_hashes.is_empty());
    assert!(state.last_synced_at.is_none());
}

// ── sync_bug: skip path ────────────────────────────────────────────────────

#[tokio::test]
async fn sync_bug_skips_an_unchanged_item_without_any_request() {
    let server = MockGitHub::start(|_| MockResponse::issue(101, "t", None, "open")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();
    let item = sample_bug(1);

    let created = adapter.sync_bug(&mut state, &item).await.unwrap();
    let skipped = adapter.sync_bug(&mut state, &item).await.unwrap();

    assert_eq!(created, SyncOutcome::Created(101));
    assert_eq!(skipped, SyncOutcome::Skipped);
    assert_eq!(
        server.request_count(),
        1,
        "a skipped item must not issue a second HTTP request"
    );
}

// ── sync_bug: update path ──────────────────────────────────────────────────

#[tokio::test]
async fn sync_bug_updates_when_the_remote_body_still_matches_our_hash() {
    let server = MockGitHub::start(|request| {
        if request.method == "GET" {
            MockResponse::issue(55, "Login crash", Some(PREVIOUS_BODY), "open")
        } else {
            MockResponse::issue(55, "[Bug] Login crash", Some(PREVIOUS_BODY), "open")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = state_with_remote_hash(1, 55, PREVIOUS_BODY);

    let outcome = adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    assert_eq!(outcome, SyncOutcome::Updated(55));
    let seen = paths(&server.requests());
    assert_eq!(
        seen,
        vec![
            (
                "GET".to_string(),
                "/repos/acme/widgets/issues/55".to_string()
            ),
            (
                "PATCH".to_string(),
                "/repos/acme/widgets/issues/55".to_string()
            ),
        ]
    );
    assert_eq!(server.requests()[1].json()["title"], "[Bug] Login crash");
    assert!(state.last_synced_at.is_some());
}

#[tokio::test]
async fn sync_bug_records_the_new_hash_after_a_successful_update() {
    let server = MockGitHub::start(|request| {
        if request.method == "GET" {
            MockResponse::issue(55, "t", Some(PREVIOUS_BODY), "open")
        } else {
            MockResponse::issue(55, "t", Some("patched"), "open")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = state_with_remote_hash(1, 55, PREVIOUS_BODY);

    adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    let patch_body = server.requests()[1].json()["body"]
        .as_str()
        .expect("body must be a string")
        .to_string();
    assert_eq!(state.content_hashes[&1], sha256_hex(&patch_body));
    assert_ne!(state.content_hashes[&1], sha256_hex(PREVIOUS_BODY));
}

#[tokio::test]
async fn sync_bug_updates_when_the_remote_body_is_null() {
    let server = MockGitHub::start(|request| {
        if request.method == "GET" {
            MockResponse::issue(55, "t", None, "open")
        } else {
            MockResponse::issue(55, "t", Some("now set"), "open")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = state_with_remote_hash(1, 55, PREVIOUS_BODY);

    let outcome = adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    assert_eq!(outcome, SyncOutcome::Updated(55));
    assert_eq!(server.request_count(), 2);
}

#[tokio::test]
async fn sync_bug_updates_when_the_conflict_probe_fails() {
    let server = MockGitHub::start(|request| {
        if request.method == "GET" {
            MockResponse::error("500 Internal Server Error", "probe down")
        } else {
            MockResponse::issue(55, "t", Some("patched"), "open")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = state_with_remote_hash(1, 55, PREVIOUS_BODY);

    let outcome = adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    assert_eq!(outcome, SyncOutcome::Updated(55));
    assert_eq!(server.request_count(), 2);
}

#[tokio::test]
async fn sync_bug_surfaces_update_failures_without_advancing_the_hash() {
    let server = MockGitHub::start(|request| {
        if request.method == "GET" {
            MockResponse::issue(55, "t", Some(PREVIOUS_BODY), "open")
        } else {
            MockResponse::error("422 Unprocessable Entity", r#"{"message":"nope"}"#)
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = state_with_remote_hash(1, 55, PREVIOUS_BODY);
    let stale_hash = state.content_hashes[&1].clone();

    let error = adapter
        .sync_bug(&mut state, &sample_bug(1))
        .await
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        r#"GitHub API error 422 Unprocessable Entity: {"message":"nope"}"#
    );
    assert_eq!(state.content_hashes[&1], stale_hash);
    assert!(state.last_synced_at.is_none());
}

// ── sync_bug: conflict path ────────────────────────────────────────────────

#[tokio::test]
async fn sync_bug_reports_a_conflict_when_the_remote_body_diverged() {
    let server =
        MockGitHub::start(|_| MockResponse::issue(55, "t", Some("edited by someone else"), "open"))
            .await;
    let adapter = adapter_for(&server);
    let mut state = state_with_remote_hash(1, 55, PREVIOUS_BODY);
    let stale_hash = state.content_hashes[&1].clone();

    let outcome = adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    assert_eq!(
        outcome,
        SyncOutcome::Conflict {
            issue_number: 55,
            reason: "Remote issue body was modified externally".to_string(),
        }
    );
    assert_eq!(
        paths(&server.requests()),
        vec![(
            "GET".to_string(),
            "/repos/acme/widgets/issues/55".to_string()
        )],
        "a conflict must not attempt a write"
    );
    assert_eq!(state.content_hashes[&1], stale_hash);
    assert!(state.last_synced_at.is_none());
}

#[tokio::test]
async fn sync_bug_updates_when_no_hash_was_recorded_yet() {
    let server = MockGitHub::start(|request| {
        if request.method == "GET" {
            MockResponse::issue(55, "t", Some("body we never recorded"), "open")
        } else {
            MockResponse::issue(55, "t", Some("patched"), "open")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    // Mapping present but no stored hash: the conflict check cannot run, so the
    // write proceeds rather than being reported as a conflict.
    let mut state = GitHubSyncState::default();
    state.issue_mappings.insert(1, 55);

    let outcome = adapter.sync_bug(&mut state, &sample_bug(1)).await.unwrap();

    assert_eq!(outcome, SyncOutcome::Updated(55));
    assert_eq!(server.request_count(), 2);
}

// ── poll_status_changes ────────────────────────────────────────────────────

#[tokio::test]
async fn poll_status_changes_returns_each_item_with_its_remote_state() {
    let server = MockGitHub::start(|request| {
        if request.path.ends_with("/issues/10") {
            MockResponse::issue(10, "Ten", None, "open")
        } else {
            MockResponse::issue(11, "Eleven", None, "closed")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();
    state.issue_mappings.insert(1, 10);
    state.issue_mappings.insert(2, 11);

    let mut changes = adapter.poll_status_changes(&state).await.unwrap();
    changes.sort();

    assert_eq!(
        changes,
        vec![(1, "open".to_string()), (2, "closed".to_string())]
    );
    assert_eq!(server.request_count(), 2);
}

#[tokio::test]
async fn poll_status_changes_with_no_mappings_makes_no_requests() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let adapter = adapter_for(&server);
    let state = GitHubSyncState::default();

    let changes = adapter.poll_status_changes(&state).await.unwrap();

    assert!(changes.is_empty());
    assert_eq!(server.request_count(), 0);
}

#[tokio::test]
async fn poll_status_changes_omits_items_whose_lookup_fails() {
    let server = MockGitHub::start(|request| {
        if request.path.ends_with("/issues/10") {
            MockResponse::issue(10, "Ten", None, "open")
        } else {
            MockResponse::error("404 Not Found", "Not Found")
        }
    })
    .await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();
    state.issue_mappings.insert(1, 10);
    state.issue_mappings.insert(2, 11);

    let changes = adapter.poll_status_changes(&state).await.unwrap();

    assert_eq!(changes, vec![(1, "open".to_string())]);
}

#[tokio::test]
async fn poll_status_changes_returns_empty_ok_when_every_lookup_fails() {
    let server = MockGitHub::start(|_| MockResponse::error("403 Forbidden", "rate limited")).await;
    let adapter = adapter_for(&server);
    let mut state = GitHubSyncState::default();
    state.issue_mappings.insert(1, 10);

    let changes = adapter.poll_status_changes(&state).await.unwrap();

    assert!(
        changes.is_empty(),
        "lookup failures are swallowed, not fatal"
    );
    assert_eq!(server.request_count(), 1);
}
