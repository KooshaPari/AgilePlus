// Integration tests for map.rs — extended mapping logic.
// Supplements inline tests in src/map.rs (open/closed/unknown state, basic user mapping).

use agileplus_domain::domain::story::StoryStatus;
use agileplus_domain::domain::user::UserRole;
use agileplus_domain::error::DomainError;
use agileplus_github::map::*;

// ── gh_state_to_story_status edge cases ────────────────────────────────────

#[test]
fn issue_state_with_whitespace_is_unknown() {
    let err = gh_state_to_story_status(" open ").unwrap_err();
    assert!(matches!(err, DomainError::Validation(_)));
}

#[test]
fn issue_state_numeric_is_unknown() {
    let err = gh_state_to_story_status("42").unwrap_err();
    assert!(matches!(err, DomainError::Validation(_)));
}

#[test]
fn issue_state_closed_lowercase_exact_match() {
    assert_eq!(
        gh_state_to_story_status("closed").unwrap(),
        StoryStatus::Done
    );
}

// ── gh_pr_state_to_story_status edge cases ─────────────────────────────────

#[test]
fn pr_state_closed_not_merged_is_cancelled() {
    assert_eq!(
        gh_pr_state_to_story_status("closed", false).unwrap(),
        StoryStatus::Cancelled
    );
}

#[test]
fn pr_state_merged_string_merged_flag_true() {
    assert_eq!(
        gh_pr_state_to_story_status("merged", true).unwrap(),
        StoryStatus::Done
    );
}

#[test]
fn pr_state_unknown_string_returns_error() {
    let err = gh_pr_state_to_story_status("review", false).unwrap_err();
    assert!(matches!(err, DomainError::Validation(_)));
}

#[test]
fn pr_state_whitespace_is_unknown() {
    let err = gh_pr_state_to_story_status(" open ", false).unwrap_err();
    assert!(matches!(err, DomainError::Validation(_)));
}

#[test]
fn pr_state_numeric_string_is_unknown() {
    let err = gh_pr_state_to_story_status("0", false).unwrap_err();
    assert!(matches!(err, DomainError::Validation(_)));
}

#[test]
fn pr_closed_with_merged_true_overrides_state_to_done() {
    // Even though state is "closed", merged=true means Done
    assert_eq!(
        gh_pr_state_to_story_status("closed", true).unwrap(),
        StoryStatus::Done
    );
}

// ── gh_user_to_domain edge cases ──────────────────────────────────────────

#[test]
fn user_with_empty_email_gets_synthetic() {
    let user = gh_user_to_domain("dev", Some(""), None).unwrap();
    assert_eq!(user.email, "dev@github.invalid");
}

#[test]
fn user_email_without_at_gets_synthetic() {
    let user = gh_user_to_domain("dev", Some("not-an-email"), None).unwrap();
    assert_eq!(user.email, "dev@github.invalid");
}

#[test]
fn user_all_fields_populated() {
    let user = gh_user_to_domain(
        "octocat",
        Some("octo@github.com"),
        Some("https://avatars.example.com/octo.png"),
    )
    .unwrap();
    assert_eq!(user.display_name, "octocat");
    assert_eq!(user.email, "octo@github.com");
    assert_eq!(
        user.avatar_url.as_deref(),
        Some("https://avatars.example.com/octo.png")
    );
    assert_eq!(user.github_login.as_deref(), Some("octocat"));
    assert_eq!(user.role, UserRole::Member);
    assert_eq!(user.id, 0);
}

#[test]
fn user_with_none_avatar_has_none_field() {
    let user = gh_user_to_domain("dev", Some("dev@example.com"), None).unwrap();
    assert!(user.avatar_url.is_none());
}

// ── issue_to_story extended ────────────────────────────────────────────────

#[test]
fn issue_empty_body_is_none_in_story() {
    let issue = GhIssue {
        number: 1,
        title: "Clean title".to_string(),
        body: Some("".to_string()),
        state: "open".to_string(),
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = issue_to_story(&issue, 1, 1).unwrap();
    // Empty body should be filtered to None
    assert!(story.description.is_none());
}

#[test]
fn issue_whitespace_only_body_is_none() {
    let issue = GhIssue {
        number: 2,
        title: "Has whitespace body".to_string(),
        body: Some("   \n  \t ".to_string()),
        state: "open".to_string(),
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = issue_to_story(&issue, 1, 1).unwrap();
    assert!(story.description.is_none());
}

#[test]
fn issue_none_body_is_none_in_story() {
    let issue = GhIssue {
        number: 3,
        title: "No body".to_string(),
        body: None,
        state: "closed".to_string(),
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = issue_to_story(&issue, 5, 10).unwrap();
    assert!(story.description.is_none());
    assert_eq!(story.status, StoryStatus::Done);
}

#[test]
fn issue_requirement_id_format() {
    let issue = GhIssue {
        number: 77,
        title: "Req test".to_string(),
        body: None,
        state: "open".to_string(),
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = issue_to_story(&issue, 1, 1).unwrap();
    assert_eq!(
        story.requirement_id.as_deref(),
        Some("gh:issue:77")
    );
    assert_eq!(story.id, 77);
}

#[test]
fn issue_large_number() {
    let issue = GhIssue {
        number: i64::MAX,
        title: "Max number".to_string(),
        body: None,
        state: "open".to_string(),
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = issue_to_story(&issue, 1, 1).unwrap();
    assert_eq!(story.id, i64::MAX);
}

// ── pr_to_story extended ──────────────────────────────────────────────────

#[test]
fn pr_empty_body_is_none() {
    let pr = GhPullRequest {
        number: 50,
        title: "Clean PR".to_string(),
        body: Some("".to_string()),
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
fn pr_requirement_id_format() {
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
    assert_eq!(story.requirement_id.as_deref(), Some("gh:pr:200"));
}

#[test]
fn pr_epic_id_and_project_id_propagated() {
    let pr = GhPullRequest {
        number: 10,
        title: "Test".to_string(),
        body: Some("body".to_string()),
        state: "open".to_string(),
        merged: false,
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = pr_to_story(&pr, 42, 99).unwrap();
    assert_eq!(story.epic_id, 42);
    assert_eq!(story.project_id, 99);
}

#[test]
fn pr_whitespace_body_is_none() {
    let pr = GhPullRequest {
        number: 11,
        title: "Whitespace".to_string(),
        body: Some("  \n ".to_string()),
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
fn pr_real_body_preserved() {
    let pr = GhPullRequest {
        number: 12,
        title: "Real body".to_string(),
        body: Some("Fixes #42".to_string()),
        state: "closed".to_string(),
        merged: true,
        user_login: None,
        user_email: None,
        user_avatar_url: None,
    };
    let story = pr_to_story(&pr, 1, 1).unwrap();
    assert_eq!(story.description.as_deref(), Some("Fixes #42"));
    assert_eq!(story.status, StoryStatus::Done);
}