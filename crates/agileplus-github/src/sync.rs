//! GitHub repository sync — pull issues and PRs, produce domain Stories.
//!
//! Traceability: WP19-T113
//!
//! # Design
//!
//! `sync_repository` is intentionally **pure / in-memory**: it accepts a
//! `GhDataSource` trait object so the function can be exercised in unit tests
//! without hitting the network.  A thin `LiveGhDataSource` wraps the real
//! `GitHubClient` for production use.
//!
//! # DB-persistence follow-up
//!
//! TODO(WP19-T114): wire `sync_repository` to `agileplus-sqlite` by having
//! callers iterate `SyncReport::stories` and upsert each story via the
//! `StoryRepository` port.  Keep this function free of I/O so it can stay
//! unit-tested without a DB fixture.

use async_trait::async_trait;

use agileplus_domain::domain::story::Story;
use agileplus_domain::error::DomainError;

use crate::map::{issue_to_story, pr_to_story, GhIssue, GhPullRequest};

// ── Data-source abstraction ───────────────────────────────────────────────────

/// Read-only access to GitHub issues and PRs for a single repository.
///
/// Implemented by `LiveGhDataSource` for production use and by test doubles
/// for unit testing.
#[async_trait]
pub trait GhDataSource: Send + Sync {
    /// Return all issues for the repository (both open and closed).
    async fn list_issues(&self) -> Result<Vec<GhIssue>, anyhow::Error>;
    /// Return all pull requests for the repository (both open and closed).
    async fn list_prs(&self) -> Result<Vec<GhPullRequest>, anyhow::Error>;
}

// ── Sync report ───────────────────────────────────────────────────────────────

/// Result of a `sync_repository` run.
///
/// Items that could not be mapped (e.g. empty title, unknown state) are
/// collected in `skipped` rather than aborting the entire sync.
#[derive(Debug, Default)]
pub struct SyncReport {
    /// Successfully mapped domain stories.
    pub stories: Vec<Story>,
    /// Items that could not be mapped: `(github_number, reason)`.
    pub skipped: Vec<(u64, String)>,
}

// ── Core sync function ────────────────────────────────────────────────────────

/// Pull all issues and PRs from a GitHub repository and map them to domain
/// [`Story`] values.
///
/// * `source`     — GitHub data source (network or test double).
/// * `project_id` — domain project to associate stories with.
/// * `epic_id`    — domain epic to associate stories with.
///
/// Invariant violations (empty title, unknown state) are recorded in
/// [`SyncReport::skipped`] rather than aborting the whole sync.
pub async fn sync_repository(
    source: &dyn GhDataSource,
    project_id: i64,
    epic_id: i64,
) -> Result<SyncReport, anyhow::Error> {
    let mut report = SyncReport::default();

    // --- issues ---------------------------------------------------------------
    let issues = source.list_issues().await?;
    for issue in &issues {
        match issue_to_story(issue, epic_id, project_id) {
            Ok(story) => report.stories.push(story),
            Err(DomainError::Validation(msg)) => {
                report.skipped.push((issue.number as u64, msg));
            }
            Err(other) => {
                return Err(anyhow::anyhow!(
                    "unexpected error mapping issue #{}: {other}",
                    issue.number
                ));
            }
        }
    }

    // --- pull requests --------------------------------------------------------
    let prs = source.list_prs().await?;
    for pr in &prs {
        match pr_to_story(pr, epic_id, project_id) {
            Ok(story) => report.stories.push(story),
            Err(DomainError::Validation(msg)) => {
                report.skipped.push((pr.number as u64, msg));
            }
            Err(other) => {
                return Err(anyhow::anyhow!(
                    "unexpected error mapping PR #{}: {other}",
                    pr.number
                ));
            }
        }
    }

    Ok(report)
}

// ── Production adapter ────────────────────────────────────────────────────────

/// Production implementation of [`GhDataSource`] backed by the REST API.
///
/// Uses [`reqwest`] directly so that `GitHubClient` in `client.rs` does not
/// need to be changed to support listing (which wasn't in the original CRUD
/// surface).
pub struct LiveGhDataSource {
    base_url: String,
    token: String,
    owner: String,
    repo: String,
    client: reqwest::Client,
}

impl LiveGhDataSource {
    /// Construct a production data source.
    ///
    /// `base_url` is typically `"https://api.github.com"`.
    pub fn new(
        base_url: impl Into<String>,
        token: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
            client: reqwest::Client::new(),
        }
    }
}

/// Wire shape for GitHub list-issues response items.
#[derive(serde::Deserialize)]
struct ApiIssue {
    number: i64,
    title: String,
    body: Option<String>,
    state: String,
    pull_request: Option<serde_json::Value>, // present iff this is a PR
    user: Option<ApiUser>,
}

/// Wire shape for GitHub list-pulls response items.
#[derive(serde::Deserialize)]
struct ApiPr {
    number: i64,
    title: String,
    body: Option<String>,
    state: String,
    merged: Option<bool>,
    user: Option<ApiUser>,
}

#[derive(serde::Deserialize)]
struct ApiUser {
    login: String,
    avatar_url: Option<String>,
}

#[async_trait]
impl GhDataSource for LiveGhDataSource {
    async fn list_issues(&self) -> Result<Vec<GhIssue>, anyhow::Error> {
        let url = format!(
            "{}/repos/{}/{}/issues?state=all&per_page=100",
            self.base_url, self.owner, self.repo
        );
        let items: Vec<ApiIssue> = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(items
            .into_iter()
            // GitHub includes PRs in the issues endpoint; filter them out.
            .filter(|i| i.pull_request.is_none())
            .map(|i| GhIssue {
                number: i.number,
                title: i.title,
                body: i.body,
                state: i.state,
                user_login: i.user.as_ref().map(|u| u.login.clone()),
                user_email: None,
                user_avatar_url: i.user.as_ref().and_then(|u| u.avatar_url.clone()),
            })
            .collect())
    }

    async fn list_prs(&self) -> Result<Vec<GhPullRequest>, anyhow::Error> {
        let url = format!(
            "{}/repos/{}/{}/pulls?state=all&per_page=100",
            self.base_url, self.owner, self.repo
        );
        let items: Vec<ApiPr> = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(items
            .into_iter()
            .map(|p| GhPullRequest {
                number: p.number,
                title: p.title,
                body: p.body,
                state: p.state,
                merged: p.merged.unwrap_or(false),
                user_login: p.user.as_ref().map(|u| u.login.clone()),
                user_email: None,
                user_avatar_url: p.user.as_ref().and_then(|u| u.avatar_url.clone()),
            })
            .collect())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fake data source ──────────────────────────────────────────────────────

    struct FakeSource {
        issues: Vec<GhIssue>,
        prs: Vec<GhPullRequest>,
    }

    impl FakeSource {
        fn new(issues: Vec<GhIssue>, prs: Vec<GhPullRequest>) -> Self {
            Self { issues, prs }
        }
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

    fn good_issue(n: i64, title: &str) -> GhIssue {
        GhIssue {
            number: n,
            title: title.to_string(),
            body: None,
            state: "open".to_string(),
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        }
    }

    fn good_pr(n: i64, title: &str) -> GhPullRequest {
        GhPullRequest {
            number: n,
            title: title.to_string(),
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        }
    }

    fn bad_issue(n: i64) -> GhIssue {
        // Empty title triggers DomainError::Validation
        GhIssue {
            number: n,
            title: "   ".to_string(),
            body: None,
            state: "open".to_string(),
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        }
    }

    // ── Test: all-mappable repo ───────────────────────────────────────────────

    /// All issues and PRs are valid → all stories mapped, nothing skipped.
    #[tokio::test]
    async fn all_mappable_produces_stories_and_no_skipped() {
        let source = FakeSource::new(
            vec![
                good_issue(1, "Fix login crash"),
                good_issue(2, "Add dark mode"),
                good_issue(3, "Improve performance"),
            ],
            vec![
                good_pr(10, "feat: dark mode impl"),
                good_pr(11, "fix: null deref"),
            ],
        );

        let report = sync_repository(&source, 42, 7).await.unwrap();

        // 3 issues + 2 PRs = 5 stories
        assert_eq!(report.stories.len(), 5, "expected 5 stories");
        assert_eq!(report.skipped.len(), 0, "expected 0 skipped");

        // All stories carry correct project_id / epic_id
        for story in &report.stories {
            assert_eq!(story.project_id, 42);
            assert_eq!(story.epic_id, 7);
        }
    }

    // ── Test: one bad issue doesn't abort sync ────────────────────────────────

    /// One issue with an empty title triggers a validation error that should
    /// be collected in `skipped`, not abort the sync.
    #[tokio::test]
    async fn bad_issue_is_skipped_not_aborted() {
        let source = FakeSource::new(
            vec![
                good_issue(1, "Valid issue"),
                bad_issue(2), // <── should be skipped
                good_issue(3, "Another valid issue"),
            ],
            vec![good_pr(10, "Valid PR")],
        );

        let report = sync_repository(&source, 1, 1).await.unwrap();

        // 2 good issues + 1 good PR = 3 stories; 1 bad issue = 1 skipped
        assert_eq!(report.stories.len(), 3, "expected 3 stories");
        assert_eq!(report.skipped.len(), 1, "expected 1 skipped");
        assert_eq!(report.skipped[0].0, 2u64, "skipped item should be issue #2");
    }

    // ── Test: bad PR is also gracefully skipped ───────────────────────────────

    #[tokio::test]
    async fn bad_pr_is_skipped_not_aborted() {
        let bad_pr = GhPullRequest {
            number: 99,
            title: "   ".to_string(), // empty title
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };

        let source = FakeSource::new(
            vec![good_issue(1, "Fine issue")],
            vec![bad_pr, good_pr(100, "Good PR")],
        );

        let report = sync_repository(&source, 5, 5).await.unwrap();

        assert_eq!(report.stories.len(), 2);
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].0, 99u64);
    }

    // ── Test: empty repo ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn empty_repo_produces_empty_report() {
        let source = FakeSource::new(vec![], vec![]);
        let report = sync_repository(&source, 1, 1).await.unwrap();
        assert_eq!(report.stories.len(), 0);
        assert_eq!(report.skipped.len(), 0);
    }
}
