// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mapping from GitHub REST API types → agileplus-domain entities.
//!
//! Traceability: WP19-T115

use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::domain::user::{User, UserRole};
use agileplus_domain::error::DomainError;

/// A minimal GitHub issue representation suitable for mapping.
/// Mirrors the fields returned by the GitHub REST API and our own
/// `GitHubIssueResponse`, but kept as a plain struct so tests can
/// construct it without hitting the network.
#[derive(Debug, Clone)]
pub struct GhIssue {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    /// "open" or "closed"
    pub state: String,
    pub user_login: Option<String>,
    pub user_email: Option<String>,
    pub user_avatar_url: Option<String>,
}

/// A minimal GitHub pull-request representation suitable for mapping.
#[derive(Debug, Clone)]
pub struct GhPullRequest {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    /// "open" or "closed" or "merged"
    pub state: String,
    pub merged: bool,
    pub user_login: Option<String>,
    pub user_email: Option<String>,
    pub user_avatar_url: Option<String>,
}

/// Map a GitHub issue state string to a domain `StoryStatus`.
///
/// * `"open"`   → `Todo`
/// * `"closed"` → `Done`
/// * anything else → `Err(DomainError::Validation(…))`
pub fn gh_state_to_story_status(state: &str) -> Result<StoryStatus, DomainError> {
    match state.to_ascii_lowercase().as_str() {
        "open" => Ok(StoryStatus::Todo),
        "closed" => Ok(StoryStatus::Done),
        other => Err(DomainError::Validation(format!(
            "unknown GitHub issue state: '{other}'"
        ))),
    }
}

/// Map a GitHub PR state to a domain `StoryStatus`.
///
/// * `"open"`   → `InProgress`
/// * `"merged"` / `"closed" + merged=true`  → `Done`
/// * `"closed"` (not merged) → `Cancelled`
pub fn gh_pr_state_to_story_status(state: &str, merged: bool) -> Result<StoryStatus, DomainError> {
    match state.to_ascii_lowercase().as_str() {
        "open" => Ok(StoryStatus::InProgress),
        "closed" | "merged" => {
            if merged {
                Ok(StoryStatus::Done)
            } else {
                Ok(StoryStatus::Cancelled)
            }
        }
        other => Err(DomainError::Validation(format!(
            "unknown GitHub PR state: '{other}'"
        ))),
    }
}

/// Convert a GitHub login into a best-effort `User`.
///
/// GitHub doesn't always expose a real email; when absent we synthesise
/// `<login>@github.invalid` so the domain invariant (must contain `@`)
/// is always satisfied.
///
/// The returned `User` has `id = 0` (not yet persisted) and
/// `github_login` set to the login string.
pub fn gh_user_to_domain(
    login: &str,
    email: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<User, DomainError> {
    let effective_email = match email {
        Some(e) if e.contains('@') => e.to_string(),
        _ => format!("{login}@github.invalid"),
    };

    let mut user = User::new(login, &effective_email, UserRole::Member)?;
    user.github_login = Some(login.to_string());
    user.avatar_url = avatar_url.map(str::to_string);
    Ok(user)
}

/// Map a `GhIssue` → domain `Story`.
///
/// * `epic_id` and `project_id` are caller-supplied context (GitHub has no concept of epics).
/// * Title must be non-empty; an empty title returns `Err(DomainError::Validation)`.
/// * Unknown `state` values return `Err(DomainError::Validation)`.
pub fn issue_to_story(
    issue: &GhIssue,
    epic_id: i64,
    project_id: i64,
) -> Result<Story, DomainError> {
    let status = gh_state_to_story_status(&issue.state)?;
    let mut story = Story::new(epic_id, project_id, &issue.title, None)?;
    story.description = issue.body.clone().filter(|b| !b.trim().is_empty());
    story.status = status;
    story.id = issue.number;
    // Stable external key for idempotent upsert (FR-AGP-013).
    story.requirement_id = Some(format!("gh:issue:{}", issue.number));
    Ok(story)
}

/// Map a `GhPullRequest` → domain `Story`.
///
/// PRs are treated as stories: open → InProgress, merged → Done, closed-without-merge → Cancelled.
pub fn pr_to_story(
    pr: &GhPullRequest,
    epic_id: i64,
    project_id: i64,
) -> Result<Story, DomainError> {
    let status = gh_pr_state_to_story_status(&pr.state, pr.merged)?;
    let mut story = Story::new(epic_id, project_id, &pr.title, None)?;
    story.description = pr.body.clone().filter(|b| !b.trim().is_empty());
    story.status = status;
    story.id = pr.number;
    // Stable external key for idempotent upsert (FR-AGP-013).
    story.requirement_id = Some(format!("gh:pr:{}", pr.number));
    Ok(story)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::story::StoryStatus;
    use agileplus_domain::error::DomainError;

    fn open_issue(title: &str) -> GhIssue {
        GhIssue {
            number: 42,
            title: title.to_string(),
            body: Some("Reproduces on startup.".to_string()),
            state: "open".to_string(),
            user_login: Some("octocat".to_string()),
            user_email: None,
            user_avatar_url: None,
        }
    }

    // ── issue_to_story ────────────────────────────────────────────────────────

    #[test]
    fn open_issue_maps_to_todo_story() {
        let issue = open_issue("Fix login crash");
        let story = issue_to_story(&issue, 1, 10).unwrap();
        assert_eq!(story.title, "Fix login crash");
        assert_eq!(story.status, StoryStatus::Todo);
        assert_eq!(story.id, 42);
        assert_eq!(story.epic_id, 1);
        assert_eq!(story.project_id, 10);
        assert_eq!(story.description.as_deref(), Some("Reproduces on startup."));
    }

    #[test]
    fn closed_issue_maps_to_done_story() {
        let mut issue = open_issue("Deploy hotfix");
        issue.state = "closed".to_string();
        let story = issue_to_story(&issue, 2, 20).unwrap();
        assert_eq!(story.status, StoryStatus::Done);
    }

    #[test]
    fn empty_title_returns_validation_error() {
        let issue = open_issue("   ");
        let err = issue_to_story(&issue, 1, 1).unwrap_err();
        assert!(
            matches!(err, DomainError::Validation(_)),
            "expected Validation, got {err:?}"
        );
    }

    #[test]
    fn unknown_state_returns_validation_error() {
        let mut issue = open_issue("Some issue");
        issue.state = "draft".to_string();
        let err = issue_to_story(&issue, 1, 1).unwrap_err();
        assert!(
            matches!(err, DomainError::Validation(_)),
            "expected Validation, got {err:?}"
        );
    }

    // ── pr_to_story ───────────────────────────────────────────────────────────

    #[test]
    fn open_pr_maps_to_in_progress() {
        let pr = GhPullRequest {
            number: 99,
            title: "feat: add dark mode".to_string(),
            body: Some("Implements dark mode toggle.".to_string()),
            state: "open".to_string(),
            merged: false,
            user_login: Some("dev".to_string()),
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 3, 30).unwrap();
        assert_eq!(story.status, StoryStatus::InProgress);
        assert_eq!(story.id, 99);
    }

    #[test]
    fn merged_pr_maps_to_done() {
        let pr = GhPullRequest {
            number: 100,
            title: "fix: null deref".to_string(),
            body: None,
            state: "closed".to_string(),
            merged: true,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 3, 30).unwrap();
        assert_eq!(story.status, StoryStatus::Done);
        assert!(story.description.is_none());
    }

    #[test]
    fn closed_unmerged_pr_maps_to_cancelled() {
        let pr = GhPullRequest {
            number: 101,
            title: "wip: experiment".to_string(),
            body: None,
            state: "closed".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 3, 30).unwrap();
        assert_eq!(story.status, StoryStatus::Cancelled);
    }

    // ── gh_user_to_domain ─────────────────────────────────────────────────────

    #[test]
    fn user_without_email_gets_synthetic_email() {
        let user = gh_user_to_domain("torvalds", None, None).unwrap();
        assert_eq!(user.email, "torvalds@github.invalid");
        assert_eq!(user.github_login.as_deref(), Some("torvalds"));
    }

    #[test]
    fn user_with_real_email_uses_it() {
        let user =
            gh_user_to_domain("alice", Some("alice@example.com"), Some("https://avatar")).unwrap();
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.avatar_url.as_deref(), Some("https://avatar"));
    }

    // ── gh_state_to_story_status ────────────────────────────────────────────────

    #[test]
    fn gh_state_open_uppercase() {
        assert_eq!(
            gh_state_to_story_status("OPEN").unwrap(),
            StoryStatus::Todo
        );
    }

    #[test]
    fn gh_state_open_mixed_case() {
        assert_eq!(
            gh_state_to_story_status("OpEn").unwrap(),
            StoryStatus::Todo
        );
    }

    #[test]
    fn gh_state_closed_uppercase() {
        assert_eq!(
            gh_state_to_story_status("CLOSED").unwrap(),
            StoryStatus::Done
        );
    }

    #[test]
    fn gh_state_closed_mixed_case() {
        assert_eq!(
            gh_state_to_story_status("ClOsEd").unwrap(),
            StoryStatus::Done
        );
    }

    #[test]
    fn gh_state_draft_returns_validation_error() {
        let err = gh_state_to_story_status("draft").unwrap_err();
        assert!(
            matches!(err, DomainError::Validation(_)),
            "expected Validation, got {err:?}"
        );
    }

    #[test]
    fn gh_state_empty_string_returns_validation_error() {
        let err = gh_state_to_story_status("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn gh_state_in_progress_returns_validation_error() {
        let err = gh_state_to_story_status("in_progress").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    // ── gh_pr_state_to_story_status ─────────────────────────────────────────────

    #[test]
    fn gh_pr_state_open_uppercase() {
        assert_eq!(
            gh_pr_state_to_story_status("OPEN", false).unwrap(),
            StoryStatus::InProgress
        );
    }

    #[test]
    fn gh_pr_state_open_mixed_case() {
        assert_eq!(
            gh_pr_state_to_story_status("OpEn", true).unwrap(),
            StoryStatus::InProgress
        );
    }

    #[test]
    fn gh_pr_state_merged_string_maps_to_done() {
        assert_eq!(
            gh_pr_state_to_story_status("merged", true).unwrap(),
            StoryStatus::Done
        );
    }

    #[test]
    fn gh_pr_state_merged_uppercase() {
        assert_eq!(
            gh_pr_state_to_story_status("MERGED", false).unwrap(),
            StoryStatus::Done
        );
    }

    #[test]
    fn gh_pr_state_closed_not_merged() {
        assert_eq!(
            gh_pr_state_to_story_status("closed", false).unwrap(),
            StoryStatus::Cancelled
        );
    }

    #[test]
    fn gh_pr_state_closed_merged_true() {
        assert_eq!(
            gh_pr_state_to_story_status("closed", true).unwrap(),
            StoryStatus::Done
        );
    }

    #[test]
    fn gh_pr_state_unknown_returns_validation_error() {
        let err = gh_pr_state_to_story_status("reverted", false).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn gh_pr_state_empty_returns_validation_error() {
        let err = gh_pr_state_to_story_status("", false).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    // ── gh_user_to_domain ───────────────────────────────────────────────────────

    #[test]
    fn user_with_empty_email_gets_synthetic_email() {
        let user = gh_user_to_domain("bob", Some(""), None).unwrap();
        assert_eq!(user.email, "bob@github.invalid");
    }

    #[test]
    fn user_with_no_at_email_gets_synthetic_email() {
        let user = gh_user_to_domain("carol", Some("noemail"), None).unwrap();
        assert_eq!(user.email, "carol@github.invalid");
    }

    #[test]
    fn user_with_email_without_domain_gets_synthetic() {
        let user = gh_user_to_domain("dave", Some("just@"), None).unwrap();
        assert_eq!(user.email, "just@"); // contains @ so it's kept
    }

    #[test]
    fn user_without_avatar_gets_none() {
        let user = gh_user_to_domain("eve", Some("eve@test.com"), None).unwrap();
        assert!(user.avatar_url.is_none());
    }

    #[test]
    fn user_with_avatar_sets_it() {
        let user = gh_user_to_domain("frank", None, Some("https://avatars.githubusercontent.com/u/1")).unwrap();
        assert_eq!(
            user.avatar_url.as_deref(),
            Some("https://avatars.githubusercontent.com/u/1")
        );
    }

    #[test]
    fn user_has_zero_id() {
        let user = gh_user_to_domain("test", None, None).unwrap();
        assert_eq!(user.id, 0);
    }

    #[test]
    fn user_is_active_by_default() {
        let user = gh_user_to_domain("test", None, None).unwrap();
        assert_eq!(user.status.to_string(), "active");
    }

    // ── issue_to_story ──────────────────────────────────────────────────────────

    #[test]
    fn issue_to_story_empty_body_filtered() {
        let mut issue = open_issue("Valid title");
        issue.body = Some("   ".to_string());
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert!(story.description.is_none());
    }

    #[test]
    fn issue_to_story_whitespace_only_body_filtered() {
        let mut issue = open_issue("Valid title");
        issue.body = Some("\t\n  \r\n".to_string());
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert!(story.description.is_none());
    }

    #[test]
    fn issue_to_story_none_body() {
        let mut issue = open_issue("Valid title");
        issue.body = None;
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert!(story.description.is_none());
    }

    #[test]
    fn issue_to_story_empty_string_body_filtered() {
        let mut issue = open_issue("Valid title");
        issue.body = Some(String::new());
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert!(story.description.is_none());
    }

    #[test]
    fn issue_to_story_requirement_id_format() {
        let mut issue = open_issue("Some issue");
        issue.number = 42;
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert_eq!(
            story.requirement_id.as_deref(),
            Some("gh:issue:42")
        );
    }

    #[test]
    fn issue_to_story_requirement_id_various_numbers() {
        for num in [1, 100, 9999] {
            let mut issue = open_issue("Issue");
            issue.number = num;
            let story = issue_to_story(&issue, 1, 1).unwrap();
            assert_eq!(
                story.requirement_id.as_deref(),
                Some(format!("gh:issue:{num}").as_str())
            );
        }
    }

    #[test]
    fn issue_to_story_preserves_epic_and_project_ids() {
        let issue = open_issue("Test");
        let story = issue_to_story(&issue, 42, 99).unwrap();
        assert_eq!(story.epic_id, 42);
        assert_eq!(story.project_id, 99);
    }

    #[test]
    fn issue_to_story_id_matches_github_number() {
        let mut issue = open_issue("Test");
        issue.number = 77;
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert_eq!(story.id, 77);
    }

    #[test]
    fn issue_to_story_with_actual_body_preserved() {
        let mut issue = open_issue("Test");
        issue.body = Some("This is a real description.".to_string());
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert_eq!(
            story.description.as_deref(),
            Some("This is a real description.")
        );
    }

    #[test]
    fn issue_to_story_status_todo_for_open() {
        let issue = open_issue("Open issue");
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert_eq!(story.status, StoryStatus::Todo);
    }

    #[test]
    fn issue_to_story_status_done_for_closed() {
        let mut issue = open_issue("Closed issue");
        issue.state = "closed".to_string();
        let story = issue_to_story(&issue, 1, 1).unwrap();
        assert_eq!(story.status, StoryStatus::Done);
    }

    // ── pr_to_story ─────────────────────────────────────────────────────────────

    #[test]
    fn pr_to_story_empty_body_filtered() {
        let mut pr = GhPullRequest {
            number: 10,
            title: "Some PR".to_string(),
            body: Some("   ".to_string()),
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert!(story.description.is_none());
    }

    #[test]
    fn pr_to_story_requirement_id_format() {
        let pr = GhPullRequest {
            number: 55,
            title: "Fix".to_string(),
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(story.requirement_id.as_deref(), Some("gh:pr:55"));
    }

    #[test]
    fn pr_to_story_id_matches_github_number() {
        let pr = GhPullRequest {
            number: 200,
            title: "PR".to_string(),
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(story.id, 200);
    }

    #[test]
    fn pr_to_story_preserves_epic_and_project_ids() {
        let pr = GhPullRequest {
            number: 1,
            title: "PR".to_string(),
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 7, 14).unwrap();
        assert_eq!(story.epic_id, 7);
        assert_eq!(story.project_id, 14);
    }

    #[test]
    fn pr_to_story_empty_title_returns_validation_error() {
        let pr = GhPullRequest {
            number: 1,
            title: "   ".to_string(),
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let err = pr_to_story(&pr, 1, 1).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn pr_to_story_with_actual_body_preserved() {
        let pr = GhPullRequest {
            number: 1,
            title: "Feature".to_string(),
            body: Some("Implements the new feature.".to_string()),
            state: "open".to_string(),
            merged: false,
            user_login: Some("dev".to_string()),
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(
            story.description.as_deref(),
            Some("Implements the new feature.")
        );
    }

    #[test]
    fn pr_to_story_state_in_progress_for_open() {
        let pr = GhPullRequest {
            number: 1,
            title: "Open PR".to_string(),
            body: None,
            state: "open".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(story.status, StoryStatus::InProgress);
    }

    #[test]
    fn pr_to_story_state_done_for_merged_via_closed() {
        let pr = GhPullRequest {
            number: 1,
            title: "Merged PR".to_string(),
            body: None,
            state: "closed".to_string(),
            merged: true,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(story.status, StoryStatus::Done);
    }

    #[test]
    fn pr_to_story_state_done_for_merged_via_merged_string() {
        let pr = GhPullRequest {
            number: 1,
            title: "Merged PR".to_string(),
            body: None,
            state: "merged".to_string(),
            merged: true,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(story.status, StoryStatus::Done);
    }

    #[test]
    fn pr_to_story_state_cancelled_for_closed_not_merged() {
        let pr = GhPullRequest {
            number: 1,
            title: "Closed PR".to_string(),
            body: None,
            state: "closed".to_string(),
            merged: false,
            user_login: None,
            user_email: None,
            user_avatar_url: None,
        };
        let story = pr_to_story(&pr, 1, 1).unwrap();
        assert_eq!(story.status, StoryStatus::Cancelled);
    }
}
