//! GitHub Issues sync logic with conflict detection.
//!
//! Syncs bugs to GitHub Issues with structured markdown bodies.
//! Detects body conflicts via SHA-256 hashing to prevent overwriting.
//!
//! Traceability: WP19-T110, T111, T112

use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::client::{GitHubClient, GitHubIssuePayload};
use crate::map::{GhIssue, GhPullRequest, issue_to_story, pr_to_story};
use agileplus_domain::domain::backlog::BacklogItem;
use agileplus_domain::domain::story::Story;
use agileplus_domain::error::DomainError;

#[async_trait]
pub trait GhDataSource: Send + Sync {
    async fn list_issues(&self) -> Result<Vec<GhIssue>>;
    async fn list_prs(&self) -> Result<Vec<GhPullRequest>>;
}

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub stories: Vec<Story>,
    pub skipped: Vec<(u64, String)>,
}

pub struct LiveGhDataSource {
    _api_base: String,
    _token: String,
    _owner: String,
    _repo: String,
}

impl LiveGhDataSource {
    pub fn new(api_base: &str, token: String, owner: &str, repo: &str) -> Self {
        Self {
            _api_base: api_base.to_string(),
            _token: token,
            _owner: owner.to_string(),
            _repo: repo.to_string(),
        }
    }
}

#[async_trait]
impl GhDataSource for LiveGhDataSource {
    async fn list_issues(&self) -> Result<Vec<GhIssue>> {
        Ok(vec![])
    }

    async fn list_prs(&self) -> Result<Vec<GhPullRequest>> {
        Ok(vec![])
    }
}

pub async fn sync_repository(
    source: &dyn GhDataSource,
    project_id: i64,
    epic_id: i64,
) -> Result<SyncReport> {
    let mut report = SyncReport::default();

    for issue in source.list_issues().await? {
        match issue_to_story(&issue, epic_id, project_id) {
            Ok(story) => report.stories.push(story),
            Err(DomainError::Validation(message)) => report
                .skipped
                .push((issue.number.try_into().unwrap(), message)),
            Err(error) => {
                return Err(anyhow!(
                    "unexpected error mapping issue #{}: {error}",
                    issue.number
                ));
            }
        }
    }

    for pr in source.list_prs().await? {
        match pr_to_story(&pr, epic_id, project_id) {
            Ok(story) => report.stories.push(story),
            Err(DomainError::Validation(message)) => report
                .skipped
                .push((pr.number.try_into().unwrap(), message)),
            Err(error) => {
                return Err(anyhow!(
                    "unexpected error mapping PR #{}: {error}",
                    pr.number
                ));
            }
        }
    }

    Ok(report)
}

/// Sync state for GitHub Issues tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSyncState {
    /// Maps backlog item ID → GitHub issue number
    pub issue_mappings: HashMap<i64, i64>,
    /// Content hashes for conflict detection
    pub content_hashes: HashMap<i64, String>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// GitHub sync adapter.
#[derive(Debug)]
pub struct GitHubSyncAdapter {
    client: GitHubClient,
}

/// Outcome of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    Created(i64),
    Updated(i64),
    Skipped,
    Conflict { issue_number: i64, reason: String },
}

impl GitHubSyncAdapter {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    /// Sync a backlog bug item to GitHub Issues.
    pub async fn sync_bug(
        &self,
        state: &mut GitHubSyncState,
        item: &BacklogItem,
    ) -> Result<SyncOutcome> {
        let item_id = item.id.context("backlog item must have an ID")?;

        let body = format_bug_body(item);
        let body_hash = hash_content(&body);

        // Local content is unchanged, but the remote may have been edited
        // since our last successful sync. Verify it before skipping.
        if let (Some(existing_hash), Some(&issue_number)) = (
            state.content_hashes.get(&item_id),
            state.issue_mappings.get(&item_id),
        ) && *existing_hash == body_hash
        {
            let remote = self.client.get_issue(issue_number).await?;
            return Ok(classify_existing_sync(
                issue_number,
                Some(existing_hash),
                &body_hash,
                remote.body.as_deref().unwrap_or_default(),
            )
            .into());
        }

        let labels = vec![
            "bug".to_string(),
            "agileplus".to_string(),
            format!("priority:{}", item.priority),
        ];

        let payload = GitHubIssuePayload {
            title: format!("[Bug] {}", item.title),
            body: body.clone(),
            labels,
        };

        let outcome = if let Some(&issue_number) = state.issue_mappings.get(&item_id) {
            // Conflict check: fetch remote and compare hashes
            let remote = self.client.get_issue(issue_number).await?;
            let remote_hash = hash_content(remote.body.as_deref().unwrap_or_default());
            if let Some(our_hash) = state.content_hashes.get(&item_id)
                && remote_hash != *our_hash
                && body_hash != remote_hash
            {
                return Ok(SyncOutcome::Conflict {
                    issue_number,
                    reason: "Remote issue body was modified externally".to_string(),
                });
            }

            let resp = self.client.update_issue(issue_number, &payload).await?;
            SyncOutcome::Updated(resp.number)
        } else {
            let resp = self.client.create_issue(&payload).await?;
            state.issue_mappings.insert(item_id, resp.number);
            SyncOutcome::Created(resp.number)
        };

        state.content_hashes.insert(item_id, body_hash);
        state.last_synced_at = Some(Utc::now());

        Ok(outcome)
    }

    /// Poll GitHub for status changes and return items that changed.
    pub async fn poll_status_changes(&self, state: &GitHubSyncState) -> Result<Vec<(i64, String)>> {
        let mut changes = Vec::new();

        for (&item_id, &issue_number) in &state.issue_mappings {
            match self.client.get_issue(issue_number).await {
                Ok(issue) => {
                    changes.push((item_id, issue.state));
                }
                Err(e) => {
                    tracing::warn!(issue_number = issue_number, error = %e, "github issue poll failed");
                }
            }
        }

        Ok(changes)
    }
}

/// Format a backlog item as a structured GitHub issue body.
fn format_bug_body(item: &BacklogItem) -> String {
    let mut body = String::new();
    body.push_str("## Description\n\n");
    body.push_str(&item.description);
    body.push_str("\n\n");
    body.push_str("## Metadata\n\n");
    body.push_str(&format!("- **Priority**: {}\n", item.priority));
    body.push_str(&format!("- **Status**: {}\n", item.status));
    body.push_str(&format!("- **Source**: {}\n", item.source));
    if let Some(ref slug) = item.feature_slug {
        body.push_str(&format!("- **Feature**: {slug}\n"));
    }
    body.push_str(&format!(
        "- **Created**: {}\n",
        item.created_at.to_rfc3339()
    ));
    body.push_str("\n---\n*Synced by AgilePlus*\n");
    body
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

fn classify_existing_sync(
    issue_number: i64,
    previous_hash: Option<&String>,
    local_hash: &str,
    remote_body: &str,
) -> ExistingSyncDecision {
    let remote_hash = hash_content(remote_body);
    if previous_hash.is_some_and(|previous| previous != &remote_hash) && local_hash != remote_hash {
        ExistingSyncDecision::Conflict { issue_number }
    } else {
        ExistingSyncDecision::Skipped
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExistingSyncDecision {
    Skipped,
    Conflict { issue_number: i64 },
}

impl From<ExistingSyncDecision> for SyncOutcome {
    fn from(decision: ExistingSyncDecision) -> Self {
        match decision {
            ExistingSyncDecision::Skipped => Self::Skipped,
            ExistingSyncDecision::Conflict { issue_number } => Self::Conflict {
                issue_number,
                reason: "Remote issue body was modified externally".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::backlog::{BacklogPriority, BacklogStatus};
    use agileplus_triage::Intent;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;

    async fn spawn_issue_server(
        status: &str,
        body: &str,
    ) -> (
        std::net::SocketAddr,
        Arc<Mutex<Vec<String>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded_requests = Arc::clone(&requests);
        let status = status.to_string();
        let body = body.to_string();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let bytes_read = stream.read(&mut buffer).await.unwrap();
                assert_ne!(bytes_read, 0, "connection closed before request headers");
                request.extend_from_slice(&buffer[..bytes_read]);
            }
            let request = String::from_utf8_lossy(&request);
            recorded_requests
                .lock()
                .await
                .push(request.lines().next().unwrap().to_string());
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        (address, requests, server)
    }

    fn sample_bug() -> BacklogItem {
        BacklogItem {
            id: Some(1),
            title: "Login crash".to_string(),
            description: "App crashes when clicking login".to_string(),
            intent: Intent::Bug,
            priority: BacklogPriority::High,
            status: BacklogStatus::New,
            source: "user-report".to_string(),
            feature_slug: Some("auth".to_string()),
            tags: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn bug_body_format() {
        let item = sample_bug();
        let body = format_bug_body(&item);
        assert!(body.contains("## Description"));
        assert!(body.contains("Login crash") || body.contains("App crashes"));
        assert!(body.contains("Priority"));
        assert!(body.contains("high"));
        assert!(body.contains("Feature**: auth"));
        assert!(body.contains("Synced by AgilePlus"));
    }

    #[test]
    fn hash_deterministic() {
        assert_eq!(hash_content("abc"), hash_content("abc"));
        assert_ne!(hash_content("abc"), hash_content("def"));
    }

    #[test]
    fn sync_state_roundtrip() {
        let mut state = GitHubSyncState::default();
        state.issue_mappings.insert(1, 42);
        state.content_hashes.insert(1, "abc123".to_string());

        let json = serde_json::to_string(&state).unwrap();
        let restored: GitHubSyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.issue_mappings[&1], 42);
    }

    #[test]
    fn skipped_when_unchanged() {
        // Test that content hash matching would signal skip
        let body = format_bug_body(&sample_bug());
        let h1 = hash_content(&body);
        let h2 = hash_content(&body);
        assert_eq!(h1, h2); // Same content → same hash → skip
    }

    #[test]
    fn unchanged_local_content_still_detects_a_remote_edit() {
        let item = sample_bug();
        let local_hash = hash_content(&format_bug_body(&item));

        assert!(matches!(
            classify_existing_sync(42, Some(&local_hash), &local_hash, "edited remotely"),
            ExistingSyncDecision::Conflict { issue_number: 42 }
        ));
    }

    #[tokio::test]
    async fn unchanged_local_content_fetches_remote_once_and_reports_remote_edit() {
        let body = r#"{"number":42,"title":"Login crash","body":"edited remotely","state":"open","labels":[],"updated_at":"2026-08-30T00:00:00Z"}"#;
        let (address, requests, server) = spawn_issue_server("200 OK", body).await;

        let item = sample_bug();
        let local_hash = hash_content(&format_bug_body(&item));
        let mut state = GitHubSyncState::default();
        state.issue_mappings.insert(item.id.unwrap(), 42);
        state.content_hashes.insert(item.id.unwrap(), local_hash);
        let adapter = GitHubSyncAdapter::new(GitHubClient::new(
            format!("http://{address}"),
            "test-token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        ));

        let outcome = adapter.sync_bug(&mut state, &item).await.unwrap();
        server.await.unwrap();

        assert!(matches!(
            outcome,
            SyncOutcome::Conflict {
                issue_number: 42,
                ..
            }
        ));
        assert_eq!(
            requests.lock().await.as_slice(),
            ["GET /repos/owner/repo/issues/42 HTTP/1.1"]
        );
    }

    #[tokio::test]
    async fn changed_local_content_propagates_remote_fetch_failure() {
        let (address, requests, server) =
            spawn_issue_server("500 Internal Server Error", "boom").await;
        let mut item = sample_bug();
        let previous_hash = hash_content(&format_bug_body(&item));
        item.description.push_str(" changed locally");
        let mut state = GitHubSyncState::default();
        state.issue_mappings.insert(item.id.unwrap(), 42);
        state.content_hashes.insert(item.id.unwrap(), previous_hash);
        let adapter = GitHubSyncAdapter::new(GitHubClient::new(
            format!("http://{address}"),
            "token".into(),
            "owner".into(),
            "repo".into(),
        ));

        let error = adapter.sync_bug(&mut state, &item).await.unwrap_err();
        server.await.unwrap();

        assert!(error.to_string().contains("GitHub API error 500"));
        assert_eq!(requests.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn changed_local_content_treats_missing_remote_body_as_empty_for_conflicts() {
        let body = r#"{"number":42,"title":"Login crash","body":null,"state":"open","labels":[],"updated_at":"2026-08-30T00:00:00Z"}"#;
        let (address, requests, server) = spawn_issue_server("200 OK", body).await;
        let mut item = sample_bug();
        let previous_hash = hash_content(&format_bug_body(&item));
        item.description.push_str(" changed locally");
        let mut state = GitHubSyncState::default();
        state.issue_mappings.insert(item.id.unwrap(), 42);
        state.content_hashes.insert(item.id.unwrap(), previous_hash);
        let adapter = GitHubSyncAdapter::new(GitHubClient::new(
            format!("http://{address}"),
            "token".into(),
            "owner".into(),
            "repo".into(),
        ));

        let outcome = adapter.sync_bug(&mut state, &item).await.unwrap();
        server.await.unwrap();

        assert!(matches!(
            outcome,
            SyncOutcome::Conflict {
                issue_number: 42,
                ..
            }
        ));
        assert_eq!(requests.lock().await.len(), 1);
    }

    struct FakeSource {
        issues: Vec<GhIssue>,
        prs: Vec<GhPullRequest>,
    }

    #[async_trait]
    impl GhDataSource for FakeSource {
        async fn list_issues(&self) -> Result<Vec<GhIssue>, anyhow::Error> {
            Ok(self.issues.clone())
        }

        async fn list_prs(&self) -> Result<Vec<GhPullRequest>, anyhow::Error> {
            Ok(self.prs.clone())
        }
    }

    #[tokio::test]
    async fn sync_repository_maps_issues_and_prs() {
        let source = FakeSource {
            issues: vec![GhIssue {
                number: 7,
                title: "Fix auth".to_string(),
                body: None,
                state: "open".to_string(),
                user_login: None,
                user_email: None,
                user_avatar_url: None,
            }],
            prs: vec![GhPullRequest {
                number: 8,
                title: "Implement sync".to_string(),
                body: None,
                state: "closed".to_string(),
                merged: true,
                user_login: None,
                user_email: None,
                user_avatar_url: None,
            }],
        };

        let report = sync_repository(&source, 10, 20).await.unwrap();

        assert_eq!(report.stories.len(), 2);
        assert_eq!(report.stories[0].project_id, 10);
        assert_eq!(report.stories[0].epic_id, 20);
        assert_eq!(
            report.stories[0].requirement_id.as_deref(),
            Some("gh:issue:7")
        );
        assert_eq!(report.stories[1].requirement_id.as_deref(), Some("gh:pr:8"));
        assert!(report.skipped.is_empty());
    }

    #[tokio::test]
    async fn sync_repository_reports_skipped_numbers_as_u64() {
        let source = FakeSource {
            issues: vec![GhIssue {
                number: 7,
                title: "   ".to_string(),
                body: None,
                state: "open".to_string(),
                user_login: None,
                user_email: None,
                user_avatar_url: None,
            }],
            prs: Vec::new(),
        };

        let report = sync_repository(&source, 10, 20).await.unwrap();

        assert_eq!(report.stories.len(), 0);
        assert_eq!(
            report.skipped,
            vec![(7_u64, "story title must not be empty".to_string())]
        );
    }
}
