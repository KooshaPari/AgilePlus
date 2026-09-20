//! Integration tests for the Claude Code and Codex harnesses in
//! `agileplus_agent_dispatch::{claude_code, codex}`.
//!
//! The harnesses are thin wrappers around a subprocess contract: the WP prompt
//! (plus context files) must be piped to the agent on stdin, the documented
//! CLI flags must be passed, stdout must be parsed for a PR URL and commit
//! SHAs, a non-zero exit must be reported rather than swallowed, and the whole
//! run must be bounded by `AgentConfig::timeout_secs`.
//!
//! To test that contract without invoking a real model, each test puts a
//! purpose-built stub named `claude` / `codex` first on `PATH`. `PATH` is
//! process-global, so every test that rewrites it holds [`PATH_LOCK`] for the
//! whole window.
//!
//! Traceability: T045, T046 (agent harnesses)

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use agileplus_agent_dispatch::claude_code::{
    extract_commits_from_output, extract_pr_url, spawn_claude_code,
};
use agileplus_agent_dispatch::codex::spawn_codex;
use agileplus_agent_dispatch::types::{AgentConfig, AgentKind, AgentTask, DomainError};

// ---------------------------------------------------------------------------
// PATH scoping
// ---------------------------------------------------------------------------

static PATH_LOCK: Mutex<()> = Mutex::new(());

/// Holds the PATH lock and restores the previous PATH on drop.
struct PathGuard {
    previous: OsString,
    _lock: MutexGuard<'static, ()>,
}

impl PathGuard {
    /// Prepend `directory` to PATH so the stubs there shadow any real binary.
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

    /// Replace PATH with exactly `directory`, so no stub is discoverable.
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
// Stub binaries
// ---------------------------------------------------------------------------

/// Write an executable `/bin/sh` stub named `name` into `directory`.
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

/// A stub that records its stdin and args into `capture_dir` and exits 0,
/// printing a PR URL and two commit SHAs.
fn claude_stub(capture_dir: &Path) -> String {
    format!(
        "#!/bin/sh\n\
         out=\"{}\"\n\
         cat > \"$out/stdin.txt\"\n\
         printf '%s\\n' \"$@\" > \"$out/args.txt\"\n\
         printf 'Created PR: https://github.com/phenotype/agileplus/pull/77\\n'\n\
         printf 'Committed abc1234 and def5678901234567890123456789012345678901\\n'\n\
         exit 0\n",
        capture_dir.display()
    )
}

fn task_in(worktree: &Path, prompt: PathBuf, context: Vec<PathBuf>) -> AgentTask {
    AgentTask {
        job_id: "job-1".to_string(),
        feature_slug: "001-feature".to_string(),
        wp_sequence: 8,
        wp_id: "WP08".to_string(),
        prompt_path: prompt,
        context_paths: context,
        worktree_path: worktree.to_path_buf(),
    }
}

fn config(kind: AgentKind, timeout_secs: u64) -> AgentConfig {
    AgentConfig {
        kind,
        timeout_secs,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Output parsing (pure, no subprocess)
// ---------------------------------------------------------------------------

#[test]
fn pr_url_extraction_finds_the_first_github_pull_request_url() {
    let output = "Agent created PR: https://github.com/phenotype/agileplus/pull/42 and finished.";
    assert_eq!(
        extract_pr_url(output),
        Some("https://github.com/phenotype/agileplus/pull/42".to_string())
    );
    // A link to an issue is not a PR.
    assert_eq!(extract_pr_url("https://github.com/o/r/issues/42"), None);
    assert_eq!(extract_pr_url("no url here"), None);
    assert_eq!(extract_pr_url(""), None);
}

#[test]
fn commit_extraction_finds_short_and_full_shas_in_order() {
    let output = "Committed abc1234 then 1234567890abcdef1234567890abcdef12345678 then deadbee.";
    let commits = extract_commits_from_output(output);
    assert_eq!(
        commits,
        vec![
            "abc1234".to_string(),
            "1234567890abcdef1234567890abcdef12345678".to_string(),
            "deadbee".to_string(),
        ]
    );
    // Fewer than 7 hex characters is not a SHA.
    assert!(extract_commits_from_output("short abcdef").is_empty());
    assert!(extract_commits_from_output("").is_empty());
}

// ---------------------------------------------------------------------------
// Claude Code
// ---------------------------------------------------------------------------

#[tokio::test]
async fn claude_code_receives_the_prompt_on_stdin_and_is_parsed_from_stdout() {
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "claude", &claude_stub(capture.path()));
    let _path = PathGuard::prepend(bin.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "# WP08 Prompt\n\nDo the thing.").unwrap();
    let context = worktree.path().join("spec.md");
    std::fs::write(&context, "# Spec\n\nDetails.").unwrap();

    let task = task_in(worktree.path(), prompt, vec![context]);
    let result = spawn_claude_code(&task, &config(AgentKind::ClaudeCode, 60))
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.job_id, "job-1");
    assert_eq!(
        result.pr_url,
        Some("https://github.com/phenotype/agileplus/pull/77".to_string())
    );
    assert!(result.commits.contains(&"abc1234".to_string()));
    assert!(
        result
            .commits
            .contains(&"def5678901234567890123456789012345678901".to_string()),
        "commits: {:?}",
        result.commits
    );
    assert!(result.stdout.contains("Created PR"));

    let stdin = std::fs::read_to_string(capture.path().join("stdin.txt")).unwrap();
    assert!(stdin.contains("# WP08 Prompt"));
    assert!(stdin.contains("Do the thing."));
    assert!(stdin.contains("## Reference Context Files"));
    assert!(stdin.contains("### spec.md"));
    assert!(stdin.contains("# Spec"));

    let args = std::fs::read_to_string(capture.path().join("args.txt")).unwrap();
    let args: Vec<&str> = args.lines().collect();
    assert_eq!(args, vec!["--print", "--dangerously-skip-permissions"]);
}

#[tokio::test]
async fn claude_code_without_context_files_sends_only_the_prompt() {
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "claude", &claude_stub(capture.path()));
    let _path = PathGuard::prepend(bin.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "ONLY-PROMPT").unwrap();

    let task = task_in(worktree.path(), prompt, vec![]);
    spawn_claude_code(&task, &config(AgentKind::ClaudeCode, 60))
        .await
        .unwrap();

    let stdin = std::fs::read_to_string(capture.path().join("stdin.txt")).unwrap();
    assert_eq!(stdin, "ONLY-PROMPT");
    assert!(!stdin.contains("Reference Context Files"));
}

#[tokio::test]
async fn a_non_zero_agent_exit_is_reported_as_an_unsuccessful_result() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "claude",
        "#!/bin/sh\nprintf 'partial output\\n'\nprintf 'boom\\n' >&2\nexit 7\n",
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "prompt").unwrap();

    let task = task_in(worktree.path(), prompt, vec![]);
    let result = spawn_claude_code(&task, &config(AgentKind::ClaudeCode, 60))
        .await
        .unwrap();

    assert!(!result.success);
    assert_eq!(result.exit_code, 7);
    assert!(result.stderr.contains("boom"));
    assert!(result.stdout.contains("partial output"));
    assert_eq!(result.pr_url, None);
    assert!(result.commits.is_empty());
}

#[tokio::test]
async fn a_missing_prompt_file_fails_before_any_process_is_spawned() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "claude", &claude_stub(worktree.path()));
    let _path = PathGuard::prepend(bin.path());

    let task = task_in(worktree.path(), worktree.path().join("missing.md"), vec![]);
    let error = spawn_claude_code(&task, &config(AgentKind::ClaudeCode, 60))
        .await
        .unwrap_err();

    assert!(
        matches!(error, DomainError::ProcessError(ref message) if message.contains("failed to read prompt file")),
        "{error:?}"
    );
}

#[tokio::test]
async fn an_absent_agent_binary_surfaces_as_a_process_error() {
    let worktree = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let _path = PathGuard::only(empty.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "prompt").unwrap();

    let task = task_in(worktree.path(), prompt, vec![]);
    let error = spawn_claude_code(&task, &config(AgentKind::ClaudeCode, 60))
        .await
        .unwrap_err();

    assert!(
        matches!(error, DomainError::ProcessError(ref message) if message.contains("failed to spawn claude")),
        "{error:?}"
    );
}

#[tokio::test]
async fn a_slow_agent_is_killed_by_the_configured_timeout() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(bin.path(), "claude", "#!/bin/sh\nsleep 30\nexit 0\n");
    let _path = PathGuard::prepend(bin.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "prompt").unwrap();

    let task = task_in(worktree.path(), prompt, vec![]);
    let error = spawn_claude_code(&task, &config(AgentKind::ClaudeCode, 1))
        .await
        .unwrap_err();

    assert!(matches!(error, DomainError::Timeout(1)), "{error:?}");
}

// ---------------------------------------------------------------------------
// Codex
// ---------------------------------------------------------------------------

#[tokio::test]
async fn codex_receives_the_prompt_on_stdin_and_cleans_up_its_temp_prompt_file() {
    let worktree = tempfile::tempdir().unwrap();
    let capture = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "codex",
        &format!(
            "#!/bin/sh\n\
             out=\"{}\"\n\
             cat > \"$out/codex-stdin.txt\"\n\
             printf '%s\\n' \"$@\" > \"$out/codex-args.txt\"\n\
             exit 0\n",
            capture.path().display()
        ),
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "# Codex Prompt").unwrap();
    let context = worktree.path().join("plan.md");
    std::fs::write(&context, "# Plan").unwrap();

    let task = task_in(worktree.path(), prompt, vec![context]);
    let result = spawn_codex(&task, &config(AgentKind::Codex, 60))
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.pr_url, None);

    let stdin = std::fs::read_to_string(capture.path().join("codex-stdin.txt")).unwrap();
    assert!(stdin.contains("# Codex Prompt"));
    assert!(stdin.contains("### plan.md"));
    assert!(stdin.contains("# Plan"));

    let args = std::fs::read_to_string(capture.path().join("codex-args.txt")).unwrap();
    let args: Vec<&str> = args.lines().collect();
    assert_eq!(args, vec!["--quiet", "--approval-mode=full-auto"]);

    // The scratch prompt file Codex reads must not survive the run.
    assert!(!worktree.path().join(".agileplus-codex-prompt.md").exists());
}

#[tokio::test]
async fn codex_missing_binary_is_a_process_error() {
    let worktree = tempfile::tempdir().unwrap();
    let empty = tempfile::tempdir().unwrap();
    let _path = PathGuard::only(empty.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "prompt").unwrap();

    let task = task_in(worktree.path(), prompt, vec![]);
    let error = spawn_codex(&task, &config(AgentKind::Codex, 60))
        .await
        .unwrap_err();

    assert!(
        matches!(error, DomainError::ProcessError(ref message) if message.contains("failed to spawn codex")),
        "{error:?}"
    );
}

#[tokio::test]
async fn codex_failure_exit_is_reported_with_its_stderr() {
    let worktree = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    write_stub(
        bin.path(),
        "codex",
        "#!/bin/sh\nprintf 'no auth\\n' >&2\nexit 2\n",
    );
    let _path = PathGuard::prepend(bin.path());

    let prompt = worktree.path().join("prompt.md");
    std::fs::write(&prompt, "prompt").unwrap();

    let task = task_in(worktree.path(), prompt, vec![]);
    let result = spawn_codex(&task, &config(AgentKind::Codex, 60))
        .await
        .unwrap();

    assert!(!result.success);
    assert_eq!(result.exit_code, 2);
    assert!(result.stderr.contains("no auth"));
}
