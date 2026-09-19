// Integration tests for the HTTP layer of `client.rs` — request construction,
// header handling, success parsing, and failure mapping against a local
// in-process mock server (`tests/common/mod.rs`). No network access.

mod common;

use agileplus_github::client::{GitHubClient, GitHubIssuePayload};
use common::{MockGitHub, MockResponse, closed_loopback_url, issue_json};

fn client_for(server: &MockGitHub) -> GitHubClient {
    GitHubClient::new(
        server.base_url().to_string(),
        "ghp_test_token".to_string(),
        "acme".to_string(),
        "widgets".to_string(),
    )
}

fn payload(title: &str) -> GitHubIssuePayload {
    GitHubIssuePayload {
        title: title.to_string(),
        body: "Reproduces on startup.".to_string(),
        labels: vec!["bug".to_string(), "agileplus".to_string()],
    }
}

// ── create_issue: request construction ─────────────────────────────────────

#[tokio::test]
async fn create_issue_posts_to_the_repo_issues_endpoint() {
    let server = MockGitHub::start(|_| MockResponse::issue(42, "Fix crash", None, "open")).await;
    let client = client_for(&server);

    let response = client.create_issue(&payload("Fix crash")).await.unwrap();

    assert_eq!(response.number, 42);
    assert_eq!(response.title, "Fix crash");
    assert_eq!(server.request_count(), 1);
    let request = &server.requests()[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/repos/acme/widgets/issues");
}

#[tokio::test]
async fn create_issue_sends_bearer_token_accept_and_user_agent() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let client = client_for(&server);

    client.create_issue(&payload("t")).await.unwrap();

    let request = &server.requests()[0];
    assert_eq!(
        request.header("authorization"),
        Some("Bearer ghp_test_token")
    );
    assert_eq!(
        request.header("accept"),
        Some("application/vnd.github+json")
    );
    assert_eq!(request.header("user-agent"), Some("agileplus"));
}

#[tokio::test]
async fn create_issue_sends_the_payload_as_the_json_body() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let client = client_for(&server);

    client
        .create_issue(&payload("Crash on start"))
        .await
        .unwrap();

    let body = server.requests()[0].json();
    assert_eq!(body["title"], "Crash on start");
    assert_eq!(body["body"], "Reproduces on startup.");
    assert_eq!(body["labels"][0], "bug");
    assert_eq!(body["labels"][1], "agileplus");
}

#[tokio::test]
async fn create_issue_omits_the_labels_key_when_empty() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let client = client_for(&server);
    let payload = GitHubIssuePayload {
        title: "No labels".to_string(),
        body: "Body".to_string(),
        labels: Vec::new(),
    };

    client.create_issue(&payload).await.unwrap();

    let body = server.requests()[0].json();
    assert!(
        body.get("labels").is_none(),
        "empty labels must be omitted from the wire payload, got {body}"
    );
}

// ── create_issue: failure paths ────────────────────────────────────────────

#[tokio::test]
async fn create_issue_maps_non_success_status_to_api_error() {
    let server = MockGitHub::start(|_| {
        MockResponse::error(
            "422 Unprocessable Entity",
            r#"{"message":"Validation Failed"}"#,
        )
    })
    .await;
    let client = client_for(&server);

    let error = client.create_issue(&payload("bad")).await.unwrap_err();

    assert_eq!(
        error.to_string(),
        r#"GitHub API error 422 Unprocessable Entity: {"message":"Validation Failed"}"#
    );
}

#[tokio::test]
async fn create_issue_reports_parse_failure_for_non_json_success_body() {
    let server = MockGitHub::start(|_| MockResponse::json("this is not json")).await;
    let client = client_for(&server);

    let error = client.create_issue(&payload("t")).await.unwrap_err();

    assert_eq!(error.to_string(), "parsing GitHub response");
}

#[tokio::test]
async fn create_issue_reports_transport_failure_when_nothing_listens() {
    let client = GitHubClient::new(
        closed_loopback_url().await,
        "token".to_string(),
        "acme".to_string(),
        "widgets".to_string(),
    );

    let error = client.create_issue(&payload("t")).await.unwrap_err();

    assert_eq!(error.to_string(), "GitHub create issue request failed");
}

// ── update_issue: request construction ─────────────────────────────────────

#[tokio::test]
async fn update_issue_patches_the_issue_url_with_the_payload() {
    let server =
        MockGitHub::start(|_| MockResponse::issue(7, "Updated", Some("new"), "open")).await;
    let client = client_for(&server);

    let response = client.update_issue(7, &payload("Updated")).await.unwrap();

    assert_eq!(response.number, 7);
    assert_eq!(response.body.as_deref(), Some("new"));
    assert_eq!(server.request_count(), 1);
    let request = &server.requests()[0];
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/repos/acme/widgets/issues/7");
    assert_eq!(
        request.header("authorization"),
        Some("Bearer ghp_test_token")
    );
    assert_eq!(request.json()["title"], "Updated");
}

#[tokio::test]
async fn update_issue_maps_non_success_status_to_api_error() {
    let server = MockGitHub::start(|_| MockResponse::error("404 Not Found", "Not Found")).await;
    let client = client_for(&server);

    let error = client.update_issue(7, &payload("t")).await.unwrap_err();

    assert_eq!(
        error.to_string(),
        "GitHub API error 404 Not Found: Not Found"
    );
}

// ── get_issue: request construction ────────────────────────────────────────

#[tokio::test]
async fn get_issue_sends_a_bodyless_get_request() {
    let server =
        MockGitHub::start(|_| MockResponse::issue(5, "Open bug", Some("Steps"), "open")).await;
    let client = client_for(&server);

    let response = client.get_issue(5).await.unwrap();

    assert_eq!(response.number, 5);
    assert_eq!(response.title, "Open bug");
    assert_eq!(response.body.as_deref(), Some("Steps"));
    assert_eq!(response.state, "open");
    assert_eq!(response.labels[0].name, "bug");
    let request = &server.requests()[0];
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/repos/acme/widgets/issues/5");
    assert_eq!(request.body, "");
    assert_eq!(
        request.header("authorization"),
        Some("Bearer ghp_test_token")
    );
}

#[tokio::test]
async fn get_issue_serialises_the_issue_number_into_the_path() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "closed")).await;
    let client = client_for(&server);

    client.get_issue(1_234_567).await.unwrap();

    assert_eq!(
        server.requests()[0].path,
        "/repos/acme/widgets/issues/1234567"
    );
}

#[tokio::test]
async fn get_issue_error_message_ends_with_colon_for_empty_error_body() {
    let server = MockGitHub::start(|_| MockResponse::error("500 Internal Server Error", "")).await;
    let client = client_for(&server);

    let error = client.get_issue(5).await.unwrap_err();

    assert_eq!(
        error.to_string(),
        "GitHub API error 500 Internal Server Error: "
    );
}

#[tokio::test]
async fn get_issue_reports_parse_failure_for_truncated_json() {
    let server = MockGitHub::start(|_| MockResponse::json(r#"{"number":"#)).await;
    let client = client_for(&server);

    let error = client.get_issue(5).await.unwrap_err();

    assert_eq!(error.to_string(), "parsing GitHub response");
}

#[tokio::test]
async fn get_issue_reports_parse_failure_when_required_field_is_absent() {
    let server = MockGitHub::start(|_| {
        MockResponse::json(r#"{"number":5,"title":"t","body":null,"labels":[]}"#)
    })
    .await;
    let client = client_for(&server);

    let error = client.get_issue(5).await.unwrap_err();

    assert_eq!(error.to_string(), "parsing GitHub response");
}

#[tokio::test]
async fn get_issue_reports_transport_failure_when_nothing_listens() {
    let client = GitHubClient::new(
        closed_loopback_url().await,
        "token".to_string(),
        "acme".to_string(),
        "widgets".to_string(),
    );

    let error = client.get_issue(5).await.unwrap_err();

    assert_eq!(error.to_string(), "GitHub get issue request failed");
}

// ── base URL / owner / repo interpolation ──────────────────────────────────

#[tokio::test]
async fn owner_and_repo_are_interpolated_verbatim_into_the_path() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let client = GitHubClient::new(
        server.base_url().to_string(),
        "token".to_string(),
        "Acme.Corp-Dev".to_string(),
        "widgets_2".to_string(),
    );

    client.create_issue(&payload("t")).await.unwrap();

    assert_eq!(
        server.requests()[0].path,
        "/repos/Acme.Corp-Dev/widgets_2/issues"
    );
}

#[tokio::test]
async fn base_url_trailing_slash_is_not_normalised() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let client = GitHubClient::new(
        format!("{}/", server.base_url()),
        "token".to_string(),
        "acme".to_string(),
        "widgets".to_string(),
    );

    client.create_issue(&payload("t")).await.unwrap();

    // The client concatenates `base_url` verbatim, so a trailing slash yields a
    // doubled separator rather than a normalised path.
    assert_eq!(server.requests()[0].path, "//repos/acme/widgets/issues");
}

#[tokio::test]
async fn base_url_is_used_verbatim_including_any_path_prefix() {
    let server = MockGitHub::start(|_| MockResponse::issue(1, "t", None, "open")).await;
    let client = GitHubClient::new(
        format!("{}/api/v3", server.base_url()),
        "token".to_string(),
        "acme".to_string(),
        "widgets".to_string(),
    );

    client.get_issue(9).await.unwrap();

    assert_eq!(
        server.requests()[0].path,
        "/api/v3/repos/acme/widgets/issues/9"
    );
}

// ── response field handling ────────────────────────────────────────────────

#[tokio::test]
async fn get_issue_treats_a_null_body_as_none() {
    let server =
        MockGitHub::start(|_| MockResponse::json(issue_json(3, "No body", None, "open"))).await;
    let client = client_for(&server);

    let response = client.get_issue(3).await.unwrap();

    assert!(response.body.is_none());
    assert_eq!(response.updated_at, "2025-01-15T10:30:00Z");
}
