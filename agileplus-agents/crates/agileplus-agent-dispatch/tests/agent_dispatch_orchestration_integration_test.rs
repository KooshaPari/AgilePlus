//! Integration tests for dispatch orchestration
//! (`agileplus_agent_dispatch::{dispatch, adapter, pr_loop::create_pr}`).
//!
//! Covers the wiring that the orchestrator and gRPC service depend on:
//! worktree creation, context/prompt staging, harness selection, the parallel
//! fan-out clamp, the `AgentPort` job registry (sync + async dispatch, status,
//! cancellation, instruction delivery), and `gh pr create` argument
//! construction including its failure modes.
//!
//! Tests that need a subprocess put a stub `claude` / `gh` first on `PATH`,
//! which is process-global, so they hold [`PATH_LOCK`] for the whole window.
//!
//! Traceability: T044b, T047, T048 / FR-011

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use agileplus_agent_dispatch::adapter::{AgentDispatchAdapter, AgentPort};
use agileplus_agent_dispatch::dispatch::{dispatch_wp, dispatch_wp_parallel};
use agileplus_agent_dispatch::ports::VcsPort;
use agileplus_agent_dispatch::pr_loop::{PrDescription, create_pr};
use agileplus_agent_dispatch::types::{AgentConfig, AgentKind, AgentTask, DomainError, JobState};
use async_trait::async_trait;
use std::ffi::OsString;

// ---------------------------------------------------------------------------
// PATH scoping
// ---------------------------------------------------------------------------

static PATH_LOCK: Mutex<()> = Mutex::new(());

struct PathGuard {
    previous: OsString,
    _lock: MutexGuard<'static, ()>,
}

impl PathGuard {
    fn prepend(directory: &Path) -> Self {
        let lock = PATH_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os("PATH").unwrap_or_default();
        let mut entries = vec![directory.to_path_buf()];
        entries.extend(std::env::split_paths(&previous));
        let joined = std::env::join_paths(entries).expect("valid PATH entries");
        unsafe { std::env::set_var("PATH", joined) };
        Self {
            previous,
            _lock: lock,
        }
    }

    fn only(directory: &Path) -> Self {
        let lock = PATH_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var_os("PATH").unwrap_or_default();
        unsafe { std::env::set_var("PATH", directory) };
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for PathGuard {
    fn drop(&mut self) {
        unsafe { std::env::set_var("PATH", &self.previous) };
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Records the worktree requests and optionally fails them.
struct FakeVcs {
    worktree: PathBuf,
    created: AtomicU32,
    fail_create: bool,
}

impl FakeVcs {
    fn returning(worktree: &Path) -> Self {
        Self {
            worktree: worktree.to_path_buf(),
            created: AtomicU32::new(0),
            fail_create: false,
        }
    }
}

#[async_trait]
impl VcsPort for FakeVcs {
    async fn create_worktree(
        &self,
        _feature_slug: &str,
        _wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        self.created.fetch_add(1, Ordering::SeqCst);
        if self.fail_create {
            return Err(DomainError::VcsError("worktree already exists".to_string()));
        }
        Ok(self.worktree.clone())
    }

    async fn remove_worktree(&self, _worktree_path: &PathBuf) -> Result<(), DomainError> {
        Ok(())
    }

    async fn new_commits_since(
        &self,
        _worktree_path: &PathBuf,
        _since_sha: &str,
    ) -> Result<Vec<String>, DomainError> {
        Ok(vec!["cafebabe".to_string()])
    }
}

fn write_stub(directory: &Path, name: &str, body: &str) -> PathBuf {
    let path = directory.join(name);
    std::fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

/// A stub that records args/cwd and prints a PR URL.
///
/// Args are written separated by ASCII record-separator (`0x1e`) so a
/// multi-line argument such as a PR body survives intact; use [`read_args`].
fn capturing_stub(capture_dir: &Path, url: &str) -> String {
    format!(
        "#!/bin/sh\n\
         out=\"{}\"\n\
         printf '%s\\036' \"$@\" > \"$out/args.txt\"\n\
         /bin/pwd > \"$out/cwd.txt\"\n\
         echo \"{}\"\n\
         exit 0\n",
        capture_dir.display(),
        url
    )
}

/// Read the record-separated argv written by [`capturing_stub`].
fn read_args(capture_dir: &Path) -> Vec<String> {
    let raw = std::fs::read_to_string(capture_dir.join("args.txt")).unwrap();
    raw.split('\u{1e}')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn make_task(worktree: &Path, prompt: PathBuf, context: Vec<PathBuf>) -> AgentTask {
    AgentTask {
        job_id: String::new(),
        feature_slug: "001-feature".to_string(),
        wp_sequence: 8,
        wp_id: "WP08".to_string(),
        prompt_path: prompt,
        context_paths: context,
        worktree_path: worktree.to_path_buf(),
    }
}

fn claude_config(timeout_secs: u64, num_agents: usize) -> AgentConfig {
    AgentConfig {
        kind: AgentKind::ClaudeCode,
        timeout_secs,
        num_agents,
        ..Default::default()
    }
}

fn sample_description() -> PrDescription {
    PrDescription {
        wp_id: "WP08".to_string(),
        wp_title: "Agent Dispatch".to_string(),
        goal: "Implement agent dispatch".to_string(),
        fr_references: vec!["FR-011".to_string()],
        acceptance_criteria: "- [ ] PR created".to_string(),
        context_summary: "spec.md §3".to_string(),
    }
}

/// Poll until the job leaves its in-flight state (stub agents finish instantly).
async fn await_terminal(adapter: &AgentDispatchAdapter, job_id: &str) -> JobState {
    for _ in 0..200 {
        let state = adapter.query_status(job_id).await.unwrap();
        if state != JobState::Pending && state != JobState::Running {
            return state;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("job {job_id} never reached a terminal state");
}

// ---------------------------------------------------------------------------
// dispatch_wp
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_wp_stages_the_prompt_and_context_into_the_new_worktree() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "claude",
        &capturing_stub(capture.path(), "https://github.com/o/r/pull/9"),
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# WP08 real prompt").unwrap();
    let context = source.path().join("spec.md");
    std::fs::write(&context, "# Spec body").unwrap();

    let vcs = FakeVcs::returning(worktree.path());
    let task = make_task(source.path(), prompt, vec![context]);

    let result = dispatch_wp(&vcs, task, &claude_config(60, 1))
        .await
        .unwrap();

    assert_eq!(result.job_id, "");
    assert_eq!(
        result.pr_url,
        Some("https://github.com/o/r/pull/9".to_string())
    );
    assert_eq!(vcs.created.load(Ordering::SeqCst), 1);

    // The prompt is staged as WP-PROMPT.md and the context file is copied.
    let staged = std::fs::read_to_string(worktree.path().join("WP-PROMPT.md")).unwrap();
    assert_eq!(staged, "# WP08 real prompt");
    let staged_context = std::fs::read_to_string(worktree.path().join("spec.md")).unwrap();
    assert_eq!(staged_context, "# Spec body");
}

#[tokio::test]
async fn dispatch_wp_propagates_a_worktree_creation_failure() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let vcs = FakeVcs {
        worktree: worktree.path().to_path_buf(),
        created: AtomicU32::new(0),
        fail_create: true,
    };
    let task = make_task(source.path(), prompt, vec![]);

    let error = dispatch_wp(&vcs, task, &claude_config(60, 1))
        .await
        .unwrap_err();
    assert!(
        matches!(error, DomainError::VcsError(ref message) if message == "worktree already exists"),
        "{error:?}"
    );
}

#[tokio::test]
async fn dispatch_wp_parallel_runs_one_result_per_clamped_agent() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "claude",
        &capturing_stub(capture.path(), "https://github.com/o/r/pull/10"),
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let vcs = FakeVcs::returning(worktree.path());
    let mut task = make_task(source.path(), prompt, vec![]);
    task.job_id = "parallel".to_string();

    // 5 requested agents clamp to 3.
    let results = dispatch_wp_parallel(&vcs, task, &claude_config(60, 5))
        .await
        .unwrap();
    assert_eq!(results.len(), 3);
    let mut ids: Vec<&str> = results.iter().map(|r| r.job_id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["parallel-sub0", "parallel-sub1", "parallel-sub2"]);
    assert_eq!(vcs.created.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn dispatch_wp_parallel_with_zero_agents_still_runs_once() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "claude",
        &capturing_stub(capture.path(), "https://github.com/o/r/pull/11"),
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let vcs = FakeVcs::returning(worktree.path());
    let task = make_task(source.path(), prompt, vec![]);
    let results = dispatch_wp_parallel(&vcs, task, &claude_config(60, 0))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn dispatch_wp_parallel_reports_the_first_harness_failure() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let _path = PathGuard::only(empty.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let vcs = FakeVcs::returning(worktree.path());
    let task = make_task(source.path(), prompt, vec![]);
    let error = dispatch_wp_parallel(&vcs, task, &claude_config(60, 1))
        .await
        .unwrap_err();
    assert!(matches!(error, DomainError::ProcessError(_)), "{error:?}");
}

// ---------------------------------------------------------------------------
// AgentDispatchAdapter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sync_dispatch_records_a_completed_job_with_its_result() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "claude",
        &capturing_stub(capture.path(), "https://github.com/o/r/pull/12"),
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(worktree.path())));
    let task = make_task(source.path(), prompt, vec![]);

    let result = adapter.dispatch(task, claude_config(60, 1)).await.unwrap();
    assert_eq!(
        result.pr_url,
        Some("https://github.com/o/r/pull/12".to_string())
    );
    // The adapter assigns a fresh job id.
    assert!(!result.job_id.is_empty());

    let state = adapter.query_status(&result.job_id).await.unwrap();
    assert_eq!(state, JobState::Completed);
}

#[tokio::test]
async fn sync_dispatch_records_a_failed_job_when_the_harness_cannot_start() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let _path = PathGuard::only(empty.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(worktree.path())));
    let task = make_task(source.path(), prompt, vec![]);

    // There is no way to learn the job id from a failed sync dispatch, but the
    // adapter must still return the underlying error.
    let error = adapter
        .dispatch(task, claude_config(60, 1))
        .await
        .unwrap_err();
    assert!(matches!(error, DomainError::ProcessError(_)), "{error:?}");
}

#[tokio::test]
async fn async_dispatch_returns_immediately_and_reaches_completed() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "claude",
        &capturing_stub(capture.path(), "https://github.com/o/r/pull/13"),
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let adapter = Arc::new(AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(
        worktree.path(),
    ))));

    let first = adapter
        .dispatch_async(
            make_task(source.path(), prompt.clone(), vec![]),
            claude_config(60, 1),
        )
        .await
        .unwrap();
    let second = adapter
        .dispatch_async(
            make_task(source.path(), prompt, vec![]),
            claude_config(60, 1),
        )
        .await
        .unwrap();
    assert_ne!(first, second, "job ids must be unique");

    assert_eq!(await_terminal(&adapter, &first).await, JobState::Completed);
    assert_eq!(await_terminal(&adapter, &second).await, JobState::Completed);
    // A completed job id is removed from the handle table but stays queryable.
    assert_eq!(
        adapter.query_status(&first).await.unwrap(),
        JobState::Completed
    );
}

#[tokio::test]
async fn query_status_for_an_unknown_job_is_a_job_not_found_error() {
    let worktree = tempfile::tempdir().unwrap();
    let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(worktree.path())));

    let error = adapter.query_status("does-not-exist").await.unwrap_err();
    assert!(
        matches!(error, DomainError::JobNotFound(ref id) if id == "does-not-exist"),
        "{error:?}"
    );
}

#[tokio::test]
async fn cancel_marks_a_running_job_cancelled_and_rejects_unknown_ids() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    // A slow stub keeps the job in flight long enough to cancel it.
    write_stub(bin.path(), "claude", "#!/bin/sh\nsleep 5\n");
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(worktree.path())));
    let job_id = adapter
        .dispatch_async(
            make_task(source.path(), prompt, vec![]),
            claude_config(120, 1),
        )
        .await
        .unwrap();

    adapter.cancel(&job_id, "operator abort").await.unwrap();
    assert_eq!(
        adapter.query_status(&job_id).await.unwrap(),
        JobState::Cancelled
    );

    let error = adapter.cancel("no-such-job", "test").await.unwrap_err();
    assert!(matches!(error, DomainError::JobNotFound(_)), "{error:?}");
}

#[tokio::test]
async fn send_instruction_writes_into_the_jobs_recorded_worktree() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "claude", "#!/bin/sh\nsleep 5\n");
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(worktree.path())));
    // The job registry stores the task exactly as dispatched, so the recorded
    // worktree is the one the caller supplied.
    let task = make_task(worktree.path(), prompt, vec![]);
    let job_id = adapter
        .dispatch_async(task, claude_config(120, 1))
        .await
        .unwrap();

    adapter
        .send_instruction(&job_id, "## Fix\n\nPlease retry.")
        .await
        .unwrap();
    let written =
        std::fs::read_to_string(worktree.path().join(".agileplus-instruction.md")).unwrap();
    assert!(written.contains("Please retry."));

    let error = adapter.send_instruction("missing", "x").await.unwrap_err();
    assert!(matches!(error, DomainError::JobNotFound(_)), "{error:?}");
}

#[tokio::test]
async fn send_instruction_rejects_a_job_that_has_no_worktree_path() {
    let source = tempfile::tempdir().unwrap();
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "claude", "#!/bin/sh\nsleep 5\n");
    let _path = PathGuard::prepend(bin.path());

    let prompt = source.path().join("WP08.md");
    std::fs::write(&prompt, "# prompt").unwrap();

    let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs::returning(worktree.path())));
    let mut task = make_task(source.path(), prompt, vec![]);
    // A caller that has not resolved a worktree yet passes an empty path.
    task.worktree_path = PathBuf::new();
    let job_id = adapter
        .dispatch_async(task, claude_config(120, 1))
        .await
        .unwrap();

    let error = adapter
        .send_instruction(&job_id, "hello")
        .await
        .unwrap_err();
    assert!(
        matches!(error, DomainError::Other(ref message) if message == "worktree path not yet set for this job"),
        "{error:?}"
    );
}

// ---------------------------------------------------------------------------
// create_pr
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_pr_passes_the_title_body_and_base_to_gh() {
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "gh",
        &capturing_stub(
            capture.path(),
            "https://github.com/phenotype/agileplus/pull/88",
        ),
    );
    let _path = PathGuard::prepend(bin.path());

    let url = create_pr(
        worktree.path(),
        "WP08",
        "Agent Dispatch",
        &sample_description(),
        "main",
    )
    .await
    .unwrap();

    assert_eq!(url, "https://github.com/phenotype/agileplus/pull/88");

    let args = read_args(capture.path());
    assert_eq!(args[0], "pr");
    assert_eq!(args[1], "create");
    let title_index = args.iter().position(|a| a == "--title").unwrap();
    assert_eq!(args[title_index + 1], "WP08: Agent Dispatch");
    let body_index = args.iter().position(|a| a == "--body").unwrap();
    let body = &args[body_index + 1];
    assert!(body.contains("## WP Goal"));
    assert!(body.contains("Implement agent dispatch"));
    assert!(body.contains("- FR-011"));
    let base_index = args.iter().position(|a| a == "--base").unwrap();
    assert_eq!(args[base_index + 1], "main");

    // `gh` must run inside the worktree so it picks up the right remote.
    let cwd = std::fs::read_to_string(capture.path().join("cwd.txt")).unwrap();
    let cwd = std::fs::canonicalize(cwd.trim()).unwrap();
    assert_eq!(cwd, std::fs::canonicalize(worktree.path()).unwrap());
}

#[tokio::test]
async fn create_pr_last_url_line_wins_when_gh_prints_noise() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "gh",
        "#!/bin/sh\n\
         echo 'Warning: some deprecation notice'\n\
         echo 'https://github.com/o/r/pull/1'\n\
         echo 'https://github.com/o/r/pull/2'\n",
    );
    let _path = PathGuard::prepend(bin.path());

    let url = create_pr(
        worktree.path(),
        "WP01",
        "Title",
        &sample_description(),
        "main",
    )
    .await
    .unwrap();
    assert_eq!(url, "https://github.com/o/r/pull/2");
}

#[tokio::test]
async fn create_pr_reports_gh_failure_with_its_stderr() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "gh",
        "#!/bin/sh\nprintf 'no commits between branches\\n' >&2\nexit 1\n",
    );
    let _path = PathGuard::prepend(bin.path());

    let error = create_pr(
        worktree.path(),
        "WP01",
        "Title",
        &sample_description(),
        "main",
    )
    .await
    .unwrap_err();

    match error {
        DomainError::PrCreationError(ref message) => {
            assert!(
                message.contains("gh pr create failed (exit 1)"),
                "{message}"
            );
            assert!(message.contains("no commits between branches"), "{message}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn create_pr_requires_a_parseable_url_in_gh_output() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "gh", "#!/bin/sh\necho 'all done'\n");
    let _path = PathGuard::prepend(bin.path());

    let error = create_pr(
        worktree.path(),
        "WP01",
        "Title",
        &sample_description(),
        "main",
    )
    .await
    .unwrap_err();
    match error {
        DomainError::PrCreationError(ref message) => {
            assert!(message.contains("could not parse PR URL"), "{message}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn create_pr_missing_gh_binary_is_a_creation_error() {
    let worktree = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let _path = PathGuard::only(empty.path());

    let error = create_pr(
        worktree.path(),
        "WP01",
        "Title",
        &sample_description(),
        "main",
    )
    .await
    .unwrap_err();
    match error {
        DomainError::PrCreationError(ref message) => {
            assert!(message.contains("gh spawn failed"), "{message}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
