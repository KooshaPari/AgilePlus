//! Integration contract tests for the PR review-fix loop
//! (`agileplus_agent_dispatch::pr_loop`).
//!
//! `run_review_fix_loop` is the governance enforcement point of FR-012: it must
//! retry while changes are requested, feed authentic review comments (and CI
//! failure notices) back to the agent through the worktree instruction file,
//! tolerate transient review/CI errors, and fail loudly with
//! `ReviewLoopExhausted` once the cycle budget is spent.
//!
//! The loop sleeps for real between cycles (30s, 60s, …), so these tests run on
//! a paused tokio clock: `start_paused` auto-advances idle timers, which keeps
//! the retry/backoff logic exact without waiting minutes of wall-clock time.
//! No test here spawns a subprocess or touches `PATH`.
//!
//! Traceability: FR-011, FR-012 / T048, T049

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use agileplus_agent_dispatch::ports::ReviewPort;
use agileplus_agent_dispatch::pr_loop::{
    PrDescription, ReviewLoopResult, format_review_comments_as_instruction, run_review_fix_loop,
};
use agileplus_agent_dispatch::types::{
    AgentConfig, AgentTask, CiStatus, CommentSeverity, DomainError, ReviewComment, ReviewOutcome,
};
use async_trait::async_trait;

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

/// A scripted review system: replays `outcomes` in order (repeating the last
/// one), and can be told to fail any of its three calls.
struct ScriptedReview {
    outcomes: Vec<ReviewOutcome>,
    comments: Vec<ReviewComment>,
    ci: CiStatus,
    review_calls: Arc<AtomicU32>,
    comment_calls: Arc<AtomicU32>,
    ci_calls: Arc<AtomicU32>,
    fail_review: bool,
    fail_comments: bool,
    fail_ci: bool,
}

impl ScriptedReview {
    fn new(outcomes: Vec<ReviewOutcome>) -> Self {
        Self {
            outcomes,
            comments: vec![ReviewComment {
                file_path: "src/lib.rs".to_string(),
                line: Some(42),
                severity: CommentSeverity::Major,
                body: "Needs error handling".to_string(),
            }],
            ci: CiStatus::Passing,
            review_calls: Arc::new(AtomicU32::new(0)),
            comment_calls: Arc::new(AtomicU32::new(0)),
            ci_calls: Arc::new(AtomicU32::new(0)),
            fail_review: false,
            fail_comments: false,
            fail_ci: false,
        }
    }
}

#[async_trait]
impl ReviewPort for ScriptedReview {
    async fn await_review(
        &self,
        _pr_url: &str,
        _timeout: Duration,
    ) -> Result<ReviewOutcome, DomainError> {
        let call = self.review_calls.fetch_add(1, Ordering::SeqCst) as usize;
        if self.fail_review {
            return Err(DomainError::Other("review backend unavailable".to_string()));
        }
        Ok(self
            .outcomes
            .get(call)
            .or_else(|| self.outcomes.last())
            .cloned()
            .unwrap_or(ReviewOutcome::Pending))
    }

    async fn get_actionable_comments(
        &self,
        _pr_url: &str,
    ) -> Result<Vec<ReviewComment>, DomainError> {
        self.comment_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_comments {
            return Err(DomainError::Other("comments unavailable".to_string()));
        }
        Ok(self.comments.clone())
    }

    async fn await_ci(&self, _pr_url: &str, _timeout: Duration) -> Result<CiStatus, DomainError> {
        self.ci_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_ci {
            return Err(DomainError::Other("ci unavailable".to_string()));
        }
        Ok(self.ci.clone())
    }
}

fn task_in(worktree: &std::path::Path) -> AgentTask {
    AgentTask {
        job_id: "job-1".to_string(),
        feature_slug: "001-feature".to_string(),
        wp_sequence: 8,
        wp_id: "WP08".to_string(),
        prompt_path: worktree.join("prompt.md"),
        context_paths: vec![],
        worktree_path: worktree.to_path_buf(),
    }
}

fn instruction_text(worktree: &std::path::Path) -> String {
    std::fs::read_to_string(worktree.join(".agileplus-review-instruction.md")).unwrap_or_default()
}

fn pr_url() -> &'static str {
    "https://github.com/phenotype/agileplus/pull/42"
}

// ---------------------------------------------------------------------------
// PrDescription
// ---------------------------------------------------------------------------

#[test]
fn pr_description_renders_every_required_section() {
    let description = PrDescription {
        wp_id: "WP08".to_string(),
        wp_title: "Agent Dispatch".to_string(),
        goal: "Implement agent dispatch".to_string(),
        fr_references: vec!["FR-011".to_string(), "FR-012".to_string()],
        acceptance_criteria: "- [ ] PR created\n- [ ] Review loop runs".to_string(),
        context_summary: "See spec.md §3".to_string(),
    };
    let markdown = description.to_markdown();

    for heading in [
        "## WP Goal",
        "## Functional Requirements",
        "## Acceptance Criteria",
        "## Context",
    ] {
        assert!(markdown.contains(heading), "missing {heading}");
    }
    assert!(markdown.contains("- FR-011"));
    assert!(markdown.contains("- FR-012"));
    assert!(markdown.contains("- [ ] PR created"));
    assert!(markdown.contains("See spec.md §3"));
    assert!(markdown.contains("AgilePlus spec-driven development engine"));
    // The wp_id/wp_title are used for the PR title, not the body.
    assert!(!markdown.contains("WP08: Agent Dispatch"));
}

#[test]
fn pr_description_without_fr_references_says_none_explicitly() {
    let description = PrDescription {
        wp_id: "WP01".to_string(),
        wp_title: "Bootstrap".to_string(),
        goal: "Set things up".to_string(),
        fr_references: vec![],
        acceptance_criteria: "Works".to_string(),
        context_summary: "n/a".to_string(),
    };
    let markdown = description.to_markdown();
    assert!(markdown.contains("- (none)"));
    assert!(!markdown.contains("- FR-"));
}

// ---------------------------------------------------------------------------
// Comment formatting
// ---------------------------------------------------------------------------

#[test]
fn comment_instruction_carries_location_and_severity_for_every_level() {
    let comments = vec![
        ReviewComment {
            file_path: "src/a.rs".to_string(),
            line: Some(7),
            severity: CommentSeverity::Critical,
            body: "Security hole".to_string(),
        },
        ReviewComment {
            file_path: "src/b.rs".to_string(),
            line: None,
            severity: CommentSeverity::Major,
            body: "Logic bug".to_string(),
        },
        ReviewComment {
            file_path: "src/c.rs".to_string(),
            line: Some(99),
            severity: CommentSeverity::Minor,
            body: "Style".to_string(),
        },
        ReviewComment {
            file_path: "README.md".to_string(),
            line: None,
            severity: CommentSeverity::Info,
            body: "FYI".to_string(),
        },
    ];
    let instruction = format_review_comments_as_instruction(&comments);

    assert!(instruction.contains("## Review Feedback - Fix Required"));
    assert!(instruction.contains("src/a.rs, Line 7 (Critical)"));
    assert!(instruction.contains("Security hole"));
    assert!(instruction.contains("### File: src/b.rs (Major)"));
    assert!(instruction.contains("src/c.rs, Line 99 (Minor)"));
    assert!(instruction.contains("README.md (Info)"));
    // A version without a line still renders as `path (severity)`.
    assert!(!instruction.contains("README.md, Line"));
}

#[test]
fn an_empty_comment_list_produces_an_explicit_no_actionable_comments_instruction() {
    let instruction = format_review_comments_as_instruction(&[]);
    assert!(instruction.contains("## Review Feedback"));
    assert!(instruction.contains("No actionable comments found."));
    assert!(!instruction.contains("Fix Required"));
}

// ---------------------------------------------------------------------------
// Review loop
// ---------------------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn approval_on_the_first_cycle_returns_immediately_without_touching_the_agent() {
    let directory = tempfile::tempdir().unwrap();
    let review = ScriptedReview::new(vec![ReviewOutcome::Approved]);
    let task = task_in(directory.path());

    let result = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 5)
        .await
        .unwrap();

    assert!(result.approved);
    assert_eq!(result.cycles_used, 1);
    assert_eq!(result.final_review_outcome, ReviewOutcome::Approved);
    assert_eq!(result.final_ci_status, CiStatus::Passing);
    assert!(result.comment_history.is_empty());
    // No review comments were requested and the agent was never notified.
    assert_eq!(review.comment_calls.load(Ordering::SeqCst), 0);
    assert!(
        !directory
            .path()
            .join(".agileplus-review-instruction.md")
            .exists()
    );
}

#[tokio::test(start_paused = true)]
async fn changes_requested_feed_the_comments_to_the_agent_before_approval() {
    let directory = tempfile::tempdir().unwrap();
    let review = ScriptedReview::new(vec![
        ReviewOutcome::ChangesRequested,
        ReviewOutcome::ChangesRequested,
        ReviewOutcome::Approved,
    ]);
    let task = task_in(directory.path());

    let result = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 5)
        .await
        .unwrap();

    assert_eq!(result.cycles_used, 3);
    assert_eq!(result.comment_history.len(), 2);
    for comments in &result.comment_history {
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].severity, CommentSeverity::Major);
    }
    // The last instruction written to the worktree is the review feedback.
    let instruction = instruction_text(directory.path());
    assert!(instruction.contains("src/lib.rs, Line 42 (Major)"));
    assert!(instruction.contains("Needs error handling"));
}

#[tokio::test(start_paused = true)]
async fn dismissed_and_pending_reviews_do_not_notify_the_agent() {
    let directory = tempfile::tempdir().unwrap();
    let review = ScriptedReview::new(vec![
        ReviewOutcome::Pending,
        ReviewOutcome::Dismissed,
        ReviewOutcome::Approved,
    ]);
    let task = task_in(directory.path());

    let result = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 4)
        .await
        .unwrap();

    assert_eq!(result.cycles_used, 3);
    assert!(result.comment_history.is_empty());
    assert_eq!(review.comment_calls.load(Ordering::SeqCst), 0);
    assert!(
        !directory
            .path()
            .join(".agileplus-review-instruction.md")
            .exists()
    );
}

#[tokio::test(start_paused = true)]
async fn a_failing_ci_pipeline_notifies_the_agent_even_when_review_is_pending() {
    let directory = tempfile::tempdir().unwrap();
    let mut review = ScriptedReview::new(vec![ReviewOutcome::Pending, ReviewOutcome::Approved]);
    review.ci = CiStatus::Failing;
    let task = task_in(directory.path());

    let result = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 3)
        .await
        .unwrap();

    // First cycle: Pending + failing CI → CI notice. Second: Approved.
    assert_eq!(result.cycles_used, 2);
    let instruction = instruction_text(directory.path());
    assert!(instruction.contains("## CI Failure"));
    assert!(instruction.contains(pr_url()));
}

#[tokio::test(start_paused = true)]
async fn an_empty_comment_payload_still_notifies_the_agent_it_has_nothing_to_fix() {
    let directory = tempfile::tempdir().unwrap();
    let mut review = ScriptedReview::new(vec![
        ReviewOutcome::ChangesRequested,
        ReviewOutcome::Approved,
    ]);
    review.comments = vec![];
    let task = task_in(directory.path());

    let result = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 3)
        .await
        .unwrap();

    assert_eq!(result.cycles_used, 2);
    // The history records the (empty) comment batch for the changes-requested cycle.
    assert_eq!(result.comment_history.len(), 1);
    assert!(result.comment_history[0].is_empty());
}

#[tokio::test(start_paused = true)]
async fn transient_review_and_ci_errors_are_tolerated_rather_than_aborting_the_loop() {
    let directory = tempfile::tempdir().unwrap();
    let mut review = ScriptedReview::new(vec![ReviewOutcome::Approved]);
    review.fail_review = true;
    review.fail_comments = true;
    review.fail_ci = true;
    let task = task_in(directory.path());

    // A backend that errors on every call behaves like "still pending": the
    // loop exhausts its budget instead of panicking or returning a partial Ok.
    let error = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 2)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        DomainError::ReviewLoopExhausted { cycles: 2 }
    ));
    assert_eq!(review.ci_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test(start_paused = true)]
async fn a_changes_requested_review_whose_comment_fetch_fails_still_notifies_the_agent() {
    let directory = tempfile::tempdir().unwrap();
    let mut review = ScriptedReview::new(vec![
        ReviewOutcome::ChangesRequested,
        ReviewOutcome::Approved,
    ]);
    review.fail_comments = true;
    let task = task_in(directory.path());

    let result = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 3)
        .await
        .unwrap();

    assert_eq!(result.cycles_used, 2);
    assert_eq!(result.comment_history.len(), 1);
    assert!(result.comment_history[0].is_empty());
    let instruction = instruction_text(directory.path());
    assert!(instruction.contains("No actionable comments found."));
}

#[tokio::test(start_paused = true)]
async fn exhausting_the_cycle_budget_reports_the_governance_exception() {
    let directory = tempfile::tempdir().unwrap();
    let review = ScriptedReview::new(vec![ReviewOutcome::ChangesRequested]);
    let task = task_in(directory.path());

    let error = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 3)
        .await
        .unwrap_err();

    match error {
        DomainError::ReviewLoopExhausted { cycles } => assert_eq!(cycles, 3),
        other => panic!("unexpected error: {other}"),
    }
    // Every cycle polled the review system.
    assert_eq!(review.review_calls.load(Ordering::SeqCst), 3);
    assert_eq!(review.ci_calls.load(Ordering::SeqCst), 3);
}

#[tokio::test(start_paused = true)]
async fn a_zero_cycle_budget_exhausts_immediately() {
    let directory = tempfile::tempdir().unwrap();
    let review = ScriptedReview::new(vec![ReviewOutcome::Approved]);
    let task = task_in(directory.path());

    let error = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 0)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        DomainError::ReviewLoopExhausted { cycles: 0 }
    ));
    assert_eq!(review.review_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn the_loop_result_surfaces_the_final_ci_status_when_approved() {
    let directory = tempfile::tempdir().unwrap();
    let mut review = ScriptedReview::new(vec![ReviewOutcome::Approved]);
    review.ci = CiStatus::Failing;
    let task = task_in(directory.path());

    let ReviewLoopResult {
        approved,
        final_ci_status,
        ..
    } = run_review_fix_loop(pr_url(), &review, &AgentConfig::default(), &task, 2)
        .await
        .unwrap();

    assert!(approved);
    // An approval short-circuits before the CI poll, so CI is reported as passing.
    assert_eq!(final_ci_status, CiStatus::Passing);
}

#[test]
fn agent_task_worktree_is_the_instruction_destination() {
    // Guards the contract that instructions land next to the agent's worktree.
    let directory = tempfile::tempdir().unwrap();
    let task = task_in(directory.path());
    assert_eq!(task.worktree_path, PathBuf::from(directory.path()));
    assert!(task.context_paths.is_empty());
}
