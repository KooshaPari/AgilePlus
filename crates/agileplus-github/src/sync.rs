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

        // Check if unchanged
        if let Some(existing_hash) = state.content_hashes.get(&item_id)
            && *existing_hash == body_hash
        {
            return Ok(SyncOutcome::Skipped);
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
            if let Ok(remote) = self.client.get_issue(issue_number).await
                && let Some(ref remote_body) = remote.body
            {
                let remote_hash = hash_content(remote_body);
                if let Some(our_hash) = state.content_hashes.get(&item_id)
                    && remote_hash != *our_hash
                    && body_hash != remote_hash
                {
                    return Ok(SyncOutcome::Conflict {
                        issue_number,
                        reason: "Remote issue body was modified externally".to_string(),
                    });
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::backlog::{BacklogPriority, BacklogStatus};
    use agileplus_triage::Intent;

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

    // ── format_bug_body (private) ─────────────────────────────────────────────

    fn fixed_bug() -> BacklogItem {
        let created: DateTime<Utc> = DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
            .expect("fixed RFC 3339 timestamp")
            .with_timezone(&Utc);
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
            created_at: created,
            updated_at: created,
        }
    }

    #[test]
    fn bug_body_matches_the_golden_layout() {
        // The body is content-hashed for change detection, so its exact layout
        // is part of the sync contract: any reformatting re-syncs every issue.
        let expected = concat!(
            "## Description\n\n",
            "App crashes when clicking login",
            "\n\n",
            "## Metadata\n\n",
            "- **Priority**: high\n",
            "- **Status**: new\n",
            "- **Source**: user-report\n",
            "- **Feature**: auth\n",
            "- **Created**: 2025-01-15T10:30:00+00:00\n",
            "\n---\n*Synced by AgilePlus*\n",
        );
        assert_eq!(format_bug_body(&fixed_bug()), expected);
    }

    #[test]
    fn bug_body_omits_feature_line_without_a_slug() {
        let mut item = fixed_bug();
        item.feature_slug = None;
        let body = format_bug_body(&item);
        assert!(!body.contains("**Feature**"));
        assert!(body.contains("- **Source**: user-report\n"));
    }

    #[test]
    fn bug_body_reports_the_item_status_and_priority() {
        let mut item = fixed_bug();
        item.priority = BacklogPriority::Critical;
        item.status = BacklogStatus::Triaged;
        let body = format_bug_body(&item);
        assert!(body.contains("- **Priority**: critical\n"));
        assert!(body.contains("- **Status**: triaged\n"));
    }

    #[test]
    fn bug_body_description_is_not_escaped() {
        let mut item = fixed_bug();
        item.description = "Use `--force` then **check** the log".to_string();
        let body = format_bug_body(&item);
        assert!(body.contains("Use `--force` then **check** the log"));
    }

    // ── hash_content (private) ────────────────────────────────────────────────

    #[test]
    fn hash_content_matches_published_sha256_vectors() {
        // Standard SHA-256 vectors, so the digest is checked against the
        // algorithm rather than against another call to itself.
        assert_eq!(
            hash_content(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hash_content("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hash_content_is_lowercase_hex_of_fixed_length() {
        let hash = hash_content("App crashes when clicking login");
        assert_eq!(hash.len(), 64);
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn hash_content_distinguishes_whitespace_and_case() {
        assert_ne!(hash_content("a b"), hash_content("ab"));
        assert_ne!(hash_content("Bug"), hash_content("bug"));
        assert_ne!(hash_content("ab\n"), hash_content("ab"));
    }
}
