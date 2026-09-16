//! GitHub Issues REST API client with rate limiting.
//!
//! Traceability: WP19-T109

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// GitHub Issues API client with token bucket rate limiter.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    base_url: String,
    token: String,
    owner: String,
    repo: String,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<TokenBucket>>,
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_acquire(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64((1.0 - self.tokens) / self.refill_rate)
        }
    }
}

/// GitHub Issue create/update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssuePayload {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels: Vec<String>,
}

/// GitHub Issue response.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubIssueResponse {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
}

impl GitHubClient {
    /// Create a new GitHub client. Rate limited to 60 req/min.
    pub fn new(base_url: String, token: String, owner: String, repo: String) -> Self {
        Self {
            base_url,
            token,
            owner,
            repo,
            client: reqwest::Client::new(),
            rate_limiter: Arc::new(Mutex::new(TokenBucket::new(60.0, 1.0))),
        }
    }

    async fn acquire_token(&self) -> Result<()> {
        loop {
            let mut limiter = self.rate_limiter.lock().await;
            if limiter.try_acquire() {
                return Ok(());
            }
            let wait = limiter.time_until_available();
            drop(limiter);
            tokio::time::sleep(wait).await;
        }
    }

    fn issues_url(&self) -> String {
        format!(
            "{}/repos/{}/{}/issues",
            self.base_url, self.owner, self.repo
        )
    }

    fn issue_url(&self, number: i64) -> String {
        format!(
            "{}/repos/{}/{}/issues/{}",
            self.base_url, self.owner, self.repo, number
        )
    }

    /// Create a new issue.
    pub async fn create_issue(&self, payload: &GitHubIssuePayload) -> Result<GitHubIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .post(self.issues_url())
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .json(payload)
            .send()
            .await
            .context("GitHub create issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        resp.json().await.context("parsing GitHub response")
    }

    /// Update an existing issue.
    pub async fn update_issue(
        &self,
        number: i64,
        payload: &GitHubIssuePayload,
    ) -> Result<GitHubIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .patch(self.issue_url(number))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .json(payload)
            .send()
            .await
            .context("GitHub update issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        resp.json().await.context("parsing GitHub response")
    }

    /// Get an issue by number.
    pub async fn get_issue(&self, number: i64) -> Result<GitHubIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .get(self.issue_url(number))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .send()
            .await
            .context("GitHub get issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        resp.json().await.context("parsing GitHub response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_bucket_basic() {
        let mut bucket = TokenBucket::new(5.0, 1.0);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
    }

    #[test]
    fn github_payload_serialize() {
        let payload = GitHubIssuePayload {
            title: "Bug: crash on start".to_string(),
            body: "## Description\nApp crashes".to_string(),
            labels: vec!["bug".to_string(), "agileplus".to_string()],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Bug: crash on start"));
        assert!(json.contains("labels"));
    }

    #[test]
    fn empty_labels_omitted() {
        let payload = GitHubIssuePayload {
            title: "Test".to_string(),
            body: "Body".to_string(),
            labels: vec![],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("labels"));
    }

    // ── TokenBucket ────────────────────────────────────────────────────────────

    #[test]
    fn token_bucket_exhausts_after_max_tokens() {
        let mut bucket = TokenBucket::new(3.0, 0.0); // no refill
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire()); // exhausted
    }

    #[test]
    fn token_bucket_refills_over_time() {
        let mut bucket = TokenBucket::new(10.0, 1000.0); // fast refill
        // Exhaust
        for _ in 0..10 {
            bucket.try_acquire();
        }
        assert!(!bucket.try_acquire());
        // Simulate time passing by manipulating last_refill
        bucket.last_refill = Instant::now() - Duration::from_secs(1);
        assert!(bucket.try_acquire()); // refilled 1000 tokens
    }

    #[test]
    fn token_bucket_time_until_available_zero_when_tokens_available() {
        let bucket = TokenBucket::new(5.0, 1.0);
        assert_eq!(bucket.time_until_available(), Duration::ZERO);
    }

    #[test]
    fn token_bucket_time_until_available_nonzero_when_empty() {
        let mut bucket = TokenBucket::new(1.0, 1.0);
        bucket.try_acquire(); // exhaust
        let wait = bucket.time_until_available();
        assert!(wait > Duration::ZERO);
        assert!(wait <= Duration::from_secs(1));
    }

    #[test]
    fn token_bucket_does_not_exceed_max_tokens() {
        let mut bucket = TokenBucket::new(3.0, 1000.0);
        // Wait by manipulating time
        bucket.last_refill = Instant::now() - Duration::from_secs(10);
        bucket.try_acquire();
        // Tokens should be capped at max_tokens
        assert!(bucket.tokens <= bucket.max_tokens);
    }

    #[test]
    fn token_bucket_one_token_per_acquire() {
        let mut bucket = TokenBucket::new(2.0, 0.0);
        let before = bucket.tokens;
        bucket.try_acquire();
        let after = bucket.tokens;
        assert!((before - after - 1.0).abs() < f64::EPSILON);
    }

    // ── GitHubIssuePayload serialization ───────────────────────────────────────

    #[test]
    fn github_issue_payload_deserialize_roundtrip() {
        let payload = GitHubIssuePayload {
            title: "Feature: add dark mode".to_string(),
            body: "Implement dark mode toggle.".to_string(),
            labels: vec!["enhancement".to_string(), "ui".to_string()],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.title, payload.title);
        assert_eq!(restored.body, payload.body);
        assert_eq!(restored.labels, payload.labels);
    }

    #[test]
    fn github_issue_payload_labels_roundtrip() {
        let payload = GitHubIssuePayload {
            title: "Test".to_string(),
            body: "".to_string(),
            labels: vec![
                "bug".to_string(),
                "priority:high".to_string(),
                "agileplus".to_string(),
            ],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.labels.len(), 3);
        assert_eq!(restored.labels[0], "bug");
        assert_eq!(restored.labels[1], "priority:high");
        assert_eq!(restored.labels[2], "agileplus");
    }

    #[test]
    fn github_issue_payload_with_empty_body() {
        let payload = GitHubIssuePayload {
            title: "No description".to_string(),
            body: String::new(),
            labels: vec![],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"title\":\"No description\""));
        assert!(json.contains("\"body\":\"\""));
        let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
        assert!(restored.body.is_empty());
    }

    #[test]
    fn github_issue_payload_with_special_characters() {
        let payload = GitHubIssuePayload {
            title: "Bug: crash on \"start\"".to_string(),
            body: "Line1\nLine2\tTab".to_string(),
            labels: vec!["bug".to_string()],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: GitHubIssuePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.title, payload.title);
        assert_eq!(restored.body, payload.body);
    }

    // ── GitHubIssueResponse deserialization ────────────────────────────────────

    #[test]
    fn github_issue_response_deserialize() {
        let json = r#"{
            "number": 42,
            "title": "Fix crash",
            "body": "Steps to reproduce...",
            "state": "open",
            "labels": [{"name": "bug"}, {"name": "priority:high"}],
            "updated_at": "2025-01-15T10:30:00Z"
        }"#;
        let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.number, 42);
        assert_eq!(resp.title, "Fix crash");
        assert_eq!(resp.body.as_deref(), Some("Steps to reproduce..."));
        assert_eq!(resp.state, "open");
        assert_eq!(resp.labels.len(), 2);
        assert_eq!(resp.labels[0].name, "bug");
        assert_eq!(resp.labels[1].name, "priority:high");
    }

    #[test]
    fn github_issue_response_body_none() {
        let json = r#"{
            "number": 1,
            "title": "Simple issue",
            "body": null,
            "state": "open",
            "labels": [],
            "updated_at": "2025-01-15T10:30:00Z"
        }"#;
        let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
        assert!(resp.body.is_none());
    }

    #[test]
    fn github_issue_response_closed_state() {
        let json = r#"{
            "number": 99,
            "title": "Closed bug",
            "body": null,
            "state": "closed",
            "labels": [{"name": "wontfix"}],
            "updated_at": "2025-02-01T00:00:00Z"
        }"#;
        let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.state, "closed");
        assert_eq!(resp.labels[0].name, "wontfix");
    }

    #[test]
    fn github_issue_response_empty_labels() {
        let json = r#"{
            "number": 5,
            "title": "No labels",
            "body": null,
            "state": "open",
            "labels": [],
            "updated_at": "2025-01-15T10:30:00Z"
        }"#;
        let resp: GitHubIssueResponse = serde_json::from_str(json).unwrap();
        assert!(resp.labels.is_empty());
    }

    // ── GitHubLabel deserialization ────────────────────────────────────────────

    #[test]
    fn github_label_deserialize() {
        let json = r#"{"name": "bug"}"#;
        let label: GitHubLabel = serde_json::from_str(json).unwrap();
        assert_eq!(label.name, "bug");
    }

    #[test]
    fn github_label_deserialize_with_color() {
        // The struct only has `name`, extra fields are ignored by serde
        let json = r#"{"name": "enhancement", "color": "a2eeef"}"#;
        let label: GitHubLabel = serde_json::from_str(json).unwrap();
        assert_eq!(label.name, "enhancement");
    }

    // ── GitHubClient construction ──────────────────────────────────────────────

    #[test]
    fn github_client_construction() {
        let client = GitHubClient::new(
            "https://api.github.com".to_string(),
            "ghp_test_token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );
        assert_eq!(client.base_url, "https://api.github.com");
        assert_eq!(client.token, "ghp_test_token");
        assert_eq!(client.owner, "owner");
        assert_eq!(client.repo, "repo");
    }

    #[test]
    fn github_client_clone() {
        let client = GitHubClient::new(
            "https://api.github.com".to_string(),
            "token".to_string(),
            "owner".to_string(),
            "repo".to_string(),
        );
        let cloned = client.clone();
        assert_eq!(cloned.base_url, client.base_url);
        assert_eq!(cloned.token, client.token);
        assert_eq!(cloned.owner, client.owner);
        assert_eq!(cloned.repo, client.repo);
    }

    #[test]
    fn github_client_debug_format() {
        let client = GitHubClient::new(
            "https://api.github.com".to_string(),
            "secret".to_string(),
            "org".to_string(),
            "project".to_string(),
        );
        let debug = format!("{client:?}");
        assert!(debug.contains("GitHubClient"));
        assert!(debug.contains("org"));
        assert!(debug.contains("project"));
    }

    #[test]
    fn github_client_issues_url_format() {
        let client = GitHubClient::new(
            "https://api.github.com".to_string(),
            "token".to_string(),
            "my-org".to_string(),
            "my-repo".to_string(),
        );
        assert_eq!(
            client.issues_url(),
            "https://api.github.com/repos/my-org/my-repo/issues"
        );
    }

    #[test]
    fn github_client_issue_url_format() {
        let client = GitHubClient::new(
            "https://api.github.com".to_string(),
            "token".to_string(),
            "org".to_string(),
            "repo".to_string(),
        );
        assert_eq!(
            client.issue_url(42),
            "https://api.github.com/repos/org/repo/issues/42"
        );
    }

    #[test]
    fn github_client_issue_url_various_numbers() {
        let client = GitHubClient::new(
            "https://api.github.com".to_string(),
            "token".to_string(),
            "org".to_string(),
            "repo".to_string(),
        );
        assert!(client.issue_url(1).ends_with("/issues/1"));
        assert!(client.issue_url(999).ends_with("/issues/999"));
        assert!(client.issue_url(0).ends_with("/issues/0"));
    }
}
