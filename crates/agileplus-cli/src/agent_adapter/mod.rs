//! Real agent adapter for dispatching to configured backends.
//!
//! Supports multiple agent backends:
//! - `claude-cli`: Claude Code CLI (primary)
//! - `cheap-llm-mcp`: Cheap LLM via MCP (fallback)
//! - `stub`: No-op stub (testing only)
//!
//! Backend selection via environment variable `FOCAL_AGENT_BACKEND` (default: claude-cli).
//!
//! Traceability: FR-013, WP05-T027

mod backends;

use std::env;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentPort, AgentResult, AgentStatus, AgentTask};
use dashmap::DashMap;
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use uuid::Uuid;

/// How often the supervisor polls the child for exit while draining its pipes.
const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Real agent adapter with pluggable backend support.
pub struct RealAgentAdapter {
    backend: AgentBackend,
    jobs: Arc<DashMap<String, JobState>>,
}

#[derive(Debug, Clone)]
enum AgentBackend {
    ClaudeCli,
    CheapLlmMcp,
    Stub,
    /// Test-only backend that runs an arbitrary program, used by the
    /// supervision tests to exercise real child processes.
    #[cfg(test)]
    TestProcess {
        program: String,
        args: Vec<String>,
    },
}

impl AgentBackend {
    fn from_env() -> Self {
        Self::from_backend_name(env::var("FOCAL_AGENT_BACKEND").ok().as_deref())
    }

    fn from_backend_name(name: Option<&str>) -> Self {
        match name {
            Some("cheap-llm-mcp") => AgentBackend::CheapLlmMcp,
            Some("stub") => AgentBackend::Stub,
            _ => AgentBackend::ClaudeCli,
        }
    }

    fn binary_and_args(&self) -> Option<(String, Vec<String>)> {
        match self {
            AgentBackend::ClaudeCli => Some((
                "claude".to_string(),
                vec![
                    "--print".to_string(),
                    "--dangerously-skip-permissions".to_string(),
                ],
            )),
            AgentBackend::CheapLlmMcp => Some((
                "cheap-llm-mcp".to_string(),
                vec![
                    "dispatch".to_string(),
                    "--model".to_string(),
                    "minimax".to_string(),
                ],
            )),
            AgentBackend::Stub => None,
            #[cfg(test)]
            AgentBackend::TestProcess { program, args } => Some((program.clone(), args.clone())),
        }
    }
}

/// Shared handle to a supervised child process.
///
/// Both the supervisor and [`AgentPort::cancel`] reach the process through this
/// handle. The supervisor only borrows it for a non-blocking `try_wait`, so it
/// never takes exclusive ownership and cancellation can always take the live
/// child and kill it.
type SharedChild = Arc<Mutex<Option<Child>>>;

fn lock_child(shared: &SharedChild) -> std::sync::MutexGuard<'_, Option<Child>> {
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// State for a dispatched agent job.
#[derive(Debug)]
enum JobState {
    Running {
        child: SharedChild,
        stdin_tx: Option<tokio::sync::mpsc::Sender<Vec<u8>>>,
        pid: u32,
    },
    Finished(AgentStatus),
}

/// How the supervised wait for one child ended.
enum ExitOutcome {
    /// The child exited and was reaped by the supervisor.
    Exited(std::process::ExitStatus),
    /// Cancellation took the process handle first; nothing left to publish.
    Cancelled,
    /// Waiting on the child failed.
    Failed(String),
}

impl RealAgentAdapter {
    pub fn new() -> Self {
        let backend = AgentBackend::from_env();
        tracing::info!("Initializing agent adapter with backend: {:?}", backend);
        Self {
            backend,
            jobs: Arc::new(DashMap::new()),
        }
    }

    async fn spawn_backend(
        &self,
        task: &AgentTask,
        config: &AgentConfig,
    ) -> Result<(Child, tokio::sync::mpsc::Sender<Vec<u8>>), DomainError> {
        let (binary, args) = self
            .backend
            .binary_and_args()
            .ok_or_else(|| DomainError::Agent("spawn_backend called for stub".into()))?;
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        backends::spawn_process(&binary, &arg_refs, task, config).await
    }

    /// Publish a terminal status only while the job is still running.
    ///
    /// This is the sole gate that prevents a late completion (or a timeout that
    /// raced a cancellation) from overwriting an already-terminal state.
    fn publish_if_running(
        jobs: &DashMap<String, JobState>,
        job_id: &str,
        status: AgentStatus,
    ) -> bool {
        let Some(mut entry) = jobs.get_mut(job_id) else {
            return false;
        };
        if matches!(&*entry, JobState::Running { .. }) {
            *entry = JobState::Finished(status);
            true
        } else {
            false
        }
    }
}

impl Default for RealAgentAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for the supervised child to exit without holding the child handle.
///
/// The supervisor only borrows the process for `try_wait`, so `cancel` can take
/// the same handle at any moment and kill a live process.
async fn wait_for_exit(child: &SharedChild) -> ExitOutcome {
    loop {
        let mut orphaned: Option<Child> = None;
        let outcome = {
            let mut slot = lock_child(child);
            if slot.is_none() {
                Some(ExitOutcome::Cancelled)
            } else {
                let waited = slot.as_mut().expect("checked above").try_wait();
                match waited {
                    Ok(Some(status)) => {
                        slot.take();
                        Some(ExitOutcome::Exited(status))
                    }
                    Ok(None) => None,
                    Err(error) => {
                        orphaned = slot.take();
                        Some(ExitOutcome::Failed(format!("wait for agent: {error}")))
                    }
                }
            }
        };

        if let Some(mut process) = orphaned {
            let _ = process.start_kill();
            let _ = process.wait().await;
        }
        if let Some(outcome) = outcome {
            return outcome;
        }

        tokio::time::sleep(EXIT_POLL_INTERVAL).await;
    }
}

/// Kill and reap whichever child still lives in the shared handle.
async fn kill_and_reap(child: &SharedChild) {
    let taken = { lock_child(child).take() };
    if let Some(mut process) = taken {
        let _ = process.start_kill();
        let _ = process.wait().await;
    }
}

impl AgentPort for RealAgentAdapter {
    async fn dispatch(
        &self,
        task: AgentTask,
        config: &AgentConfig,
    ) -> Result<AgentResult, DomainError> {
        match self.backend {
            AgentBackend::Stub => backends::dispatch_stub(&task, config).await,
            _ => {
                let (child, _tx) = self.spawn_backend(&task, config).await?;
                let output = tokio::time::timeout(
                    std::time::Duration::from_secs(config.timeout_secs),
                    child.wait_with_output(),
                )
                .await
                .map_err(|_| DomainError::Timeout(config.timeout_secs))?
                .map_err(|e| DomainError::Agent(format!("wait for agent: {e}")))?;
                backends::build_result(output)
            }
        }
    }

    async fn dispatch_async(
        &self,
        task: AgentTask,
        config: &AgentConfig,
    ) -> Result<String, DomainError> {
        let job_id = Uuid::new_v4().to_string();

        match self.backend {
            AgentBackend::Stub => {
                let result = backends::dispatch_stub(&task, config).await?;
                let status = if result.success {
                    AgentStatus::Completed { result }
                } else {
                    AgentStatus::Failed {
                        error: result.stderr.clone(),
                    }
                };
                self.jobs.insert(job_id.clone(), JobState::Finished(status));
            }
            _ => {
                let (mut process, stdin_tx) = self.spawn_backend(&task, config).await?;
                let pid = process.id().unwrap_or(0);
                let backend_name = format!("{:?}", self.backend);

                // The supervisor owns the output pipes; the process handle
                // stays shared so cancellation can reach the live child.
                let stdout = process.stdout.take();
                let stderr = process.stderr.take();
                let child: SharedChild = Arc::new(Mutex::new(Some(process)));

                self.jobs.insert(
                    job_id.clone(),
                    JobState::Running {
                        child: child.clone(),
                        stdin_tx: Some(stdin_tx),
                        pid,
                    },
                );

                let jobs = self.jobs.clone();
                let watched_job_id = job_id.clone();
                let timeout = Duration::from_secs(config.timeout_secs);

                tokio::spawn(async move {
                    let mut stdout = stdout;
                    let mut stderr = stderr;
                    let mut stdout_buf = Vec::new();
                    let mut stderr_buf = Vec::new();

                    // Draining both pipes runs concurrently with the exit wait,
                    // and the outer timeout bounds both.
                    let supervised = async {
                        let drain_stdout = async {
                            if let Some(ref mut pipe) = stdout {
                                let _ = pipe.read_to_end(&mut stdout_buf).await;
                            }
                        };
                        let drain_stderr = async {
                            if let Some(ref mut pipe) = stderr {
                                let _ = pipe.read_to_end(&mut stderr_buf).await;
                            }
                        };
                        let ((), (), outcome) =
                            tokio::join!(drain_stdout, drain_stderr, wait_for_exit(&child));
                        outcome
                    };

                    let outcome = tokio::time::timeout(timeout, supervised).await;

                    let status = match outcome {
                        Ok(ExitOutcome::Exited(exit_status)) => {
                            let output = std::process::Output {
                                status: exit_status,
                                stdout: stdout_buf,
                                stderr: stderr_buf,
                            };
                            match backends::build_result(output) {
                                Ok(result) if result.success => AgentStatus::Completed { result },
                                Ok(result) => AgentStatus::Failed {
                                    error: if result.stderr.trim().is_empty() {
                                        format!("agent exited with code {}", result.exit_code)
                                    } else {
                                        result.stderr
                                    },
                                },
                                Err(error) => AgentStatus::Failed {
                                    error: error.to_string(),
                                },
                            }
                        }
                        Ok(ExitOutcome::Failed(error)) => {
                            kill_and_reap(&child).await;
                            AgentStatus::Failed { error }
                        }
                        Ok(ExitOutcome::Cancelled) => {
                            // `cancel` already published the terminal state.
                            kill_and_reap(&child).await;
                            tracing::info!(
                                job_id = %watched_job_id,
                                backend = %backend_name,
                                "async agent job cancelled"
                            );
                            return;
                        }
                        Err(_) => {
                            tracing::error!("agent timed out after {}s", timeout.as_secs());
                            kill_and_reap(&child).await;
                            AgentStatus::Failed {
                                error: format!("timeout after {}s", timeout.as_secs()),
                            }
                        }
                    };

                    if Self::publish_if_running(&jobs, &watched_job_id, status) {
                        tracing::info!(
                            job_id = %watched_job_id,
                            backend = %backend_name,
                            "async dispatch finished"
                        );
                    }
                });

                tracing::info!(job_id = %job_id, pid, "async dispatch started");
            }
        }

        Ok(job_id)
    }

    async fn query_status(&self, job_id: &str) -> Result<AgentStatus, DomainError> {
        self.jobs
            .get(job_id)
            .map(|entry| match &*entry {
                JobState::Running { pid, .. } => AgentStatus::Running { pid: *pid },
                JobState::Finished(status) => status.clone(),
            })
            .ok_or_else(|| DomainError::NotFound(format!("job {job_id} not found")))
    }

    async fn cancel(&self, job_id: &str) -> Result<(), DomainError> {
        let mut entry = self
            .jobs
            .get_mut(job_id)
            .ok_or_else(|| DomainError::NotFound(format!("job {job_id} not found")))?;

        let mut taken: Option<Child> = None;
        let was_running = match &mut *entry {
            JobState::Running { child, .. } => {
                taken = lock_child(child).take();
                true
            }
            JobState::Finished(_) => false,
        };

        if was_running {
            // Terminal state is published before the kill, so a completion that
            // lands after this point can never overwrite it.
            *entry = JobState::Finished(AgentStatus::Failed {
                error: "cancelled by user".to_string(),
            });
        } else {
            tracing::warn!(job_id, "cancel on already-finished job");
        }
        drop(entry);

        if let Some(mut process) = taken {
            let _ = process.start_kill();
            let _ = process.wait().await;
            tracing::info!(job_id, "killed agent process");
        }

        Ok(())
    }

    async fn send_instruction(&self, job_id: &str, instruction: &str) -> Result<(), DomainError> {
        let entry = self
            .jobs
            .get(job_id)
            .ok_or_else(|| DomainError::NotFound(format!("job {job_id} not found")))?;

        match &*entry {
            JobState::Running {
                stdin_tx: Some(tx), ..
            } => {
                let data = format!("{instruction}\n").into_bytes();
                tx.send(data).await.map_err(|_| {
                    DomainError::Agent(format!("stdin closed; cannot send instruction to {job_id}"))
                })?;
                tracing::info!(job_id, instruction = %instruction, "sent instruction");
                Ok(())
            }
            JobState::Running { stdin_tx: None, .. } => {
                Err(DomainError::Agent(format!("no stdin channel for {job_id}")))
            }
            JobState::Finished(_) => {
                Err(DomainError::Agent(format!("job {job_id} already finished")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_selection_from_env() {
        assert!(matches!(
            AgentBackend::from_backend_name(None),
            AgentBackend::ClaudeCli
        ));
        assert!(matches!(
            AgentBackend::from_backend_name(Some("cheap-llm-mcp")),
            AgentBackend::CheapLlmMcp
        ));
        assert!(matches!(
            AgentBackend::from_backend_name(Some("stub")),
            AgentBackend::Stub
        ));
    }

    #[tokio::test]
    async fn test_stub_dispatch() {
        let adapter = RealAgentAdapter {
            backend: AgentBackend::Stub,
            jobs: std::sync::Arc::new(DashMap::new()),
        };

        let task = AgentTask {
            wp_id: "WP08".to_string(),
            feature_slug: "test".to_string(),
            prompt_path: "/tmp/prompt.md".into(),
            worktree_path: "/tmp/test-wt".into(),
            context_files: vec![],
        };

        let config = AgentConfig {
            kind: agileplus_domain::ports::agent::AgentKind::ClaudeCode,
            max_review_cycles: 3,
            timeout_secs: 300,
            extra_args: vec![],
        };

        let result = adapter.dispatch(task, &config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_job_status_tracking() {
        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("prompt.md");
        std::fs::write(&prompt_path, "Test prompt").unwrap();

        let adapter = RealAgentAdapter {
            backend: AgentBackend::Stub,
            jobs: std::sync::Arc::new(DashMap::new()),
        };

        let task = AgentTask {
            wp_id: "WP01".to_string(),
            feature_slug: "test".to_string(),
            prompt_path,
            worktree_path: temp_dir.path().to_path_buf(),
            context_files: vec![],
        };

        let config = AgentConfig {
            kind: agileplus_domain::ports::agent::AgentKind::ClaudeCode,
            max_review_cycles: 3,
            timeout_secs: 300,
            extra_args: vec![],
        };

        let job_id = adapter.dispatch_async(task, &config).await.unwrap();
        let status = adapter.query_status(&job_id).await;
        assert!(status.is_ok());
        let cancel = adapter.cancel(&job_id).await;
        assert!(cancel.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_on_finished_job() {
        let adapter = RealAgentAdapter {
            backend: AgentBackend::Stub,
            jobs: std::sync::Arc::new(DashMap::new()),
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("prompt.md");
        std::fs::write(&prompt_path, "Test").unwrap();

        let task = AgentTask {
            wp_id: "WP99".to_string(),
            feature_slug: "test".to_string(),
            prompt_path,
            worktree_path: temp_dir.path().to_path_buf(),
            context_files: vec![],
        };

        let config = AgentConfig {
            kind: agileplus_domain::ports::agent::AgentKind::ClaudeCode,
            max_review_cycles: 1,
            timeout_secs: 300,
            extra_args: vec![],
        };

        let job_id = adapter.dispatch_async(task, &config).await.unwrap();
        let status = adapter.query_status(&job_id).await.unwrap();
        assert!(
            matches!(status, AgentStatus::Completed { .. }),
            "stub should complete, got: {status:?}"
        );
    }

    #[tokio::test]
    async fn test_send_instruction_on_finished_errors() {
        let adapter = RealAgentAdapter {
            backend: AgentBackend::Stub,
            jobs: std::sync::Arc::new(DashMap::new()),
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let prompt_path = temp_dir.path().join("prompt.md");
        std::fs::write(&prompt_path, "Test").unwrap();

        let task = AgentTask {
            wp_id: "WP98".to_string(),
            feature_slug: "test".to_string(),
            prompt_path,
            worktree_path: temp_dir.path().to_path_buf(),
            context_files: vec![],
        };

        let config = AgentConfig {
            kind: agileplus_domain::ports::agent::AgentKind::ClaudeCode,
            max_review_cycles: 1,
            timeout_secs: 300,
            extra_args: vec![],
        };

        let job_id = adapter.dispatch_async(task, &config).await.unwrap();
        let result = adapter.send_instruction(&job_id, "fix this").await;
        assert!(result.is_err());
    }
}

/// Process-level supervision tests. These spawn real shell children (no mocks)
/// and assert cancel/timeout/completion behavior end to end.
#[cfg(test)]
mod supervisor_tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    static MARKER_SEQ: AtomicU64 = AtomicU64::new(0);

    fn shell_script(script: &str) -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), script.to_string()],
            )
        } else {
            ("sh".to_string(), vec!["-c".to_string(), script.to_string()])
        }
    }

    fn process_adapter(script: &str) -> RealAgentAdapter {
        let (program, args) = shell_script(script);
        RealAgentAdapter {
            backend: AgentBackend::TestProcess { program, args },
            jobs: Arc::new(DashMap::new()),
        }
    }

    fn marker_path(name: &str) -> PathBuf {
        let seq = MARKER_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "agileplus-agent-supervisor-{name}-{}-{seq}.marker",
            std::process::id()
        ))
    }

    fn test_task(dir: &Path) -> AgentTask {
        let prompt_path = dir.join("prompt.md");
        std::fs::write(&prompt_path, "Test prompt").unwrap();
        AgentTask {
            wp_id: "WP-SUPERVISOR".to_string(),
            feature_slug: "test".to_string(),
            prompt_path,
            worktree_path: dir.to_path_buf(),
            context_files: vec![],
        }
    }

    fn test_config(timeout_secs: u64) -> AgentConfig {
        AgentConfig {
            kind: agileplus_domain::ports::agent::AgentKind::ClaudeCode,
            max_review_cycles: 1,
            timeout_secs,
            extra_args: vec![],
        }
    }

    async fn await_terminal(
        adapter: &RealAgentAdapter,
        job_id: &str,
        bound: Duration,
    ) -> AgentStatus {
        tokio::time::timeout(bound, async {
            loop {
                let status = adapter.query_status(job_id).await.unwrap();
                if !matches!(status, AgentStatus::Running { .. }) {
                    return status;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("job did not reach a terminal status within the test bound")
    }

    fn failed_error(status: AgentStatus) -> String {
        match status {
            AgentStatus::Failed { error } => error,
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    /// A child that sleeps ~2s and only then writes its marker must not survive
    /// cancellation, and the cancelled state must stay terminal.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancel_kills_real_child_and_blocks_late_completion() {
        let temp_dir = tempfile::tempdir().unwrap();
        let marker = marker_path("cancel");
        let script = if cfg!(windows) {
            format!(
                "ping -n 3 127.0.0.1 >NUL & echo completed> \"{}\"",
                marker.display()
            )
        } else {
            format!("sleep 2; printf completed > '{}'", marker.display())
        };
        let adapter = process_adapter(&script);
        let config = test_config(30);
        let job_id = adapter
            .dispatch_async(test_task(temp_dir.path()), &config)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;
        adapter.cancel(&job_id).await.unwrap();

        // Cancellation publishes the terminal state synchronously.
        let cancelled = failed_error(adapter.query_status(&job_id).await.unwrap());
        assert!(
            cancelled.contains("cancelled"),
            "unexpected terminal error: {cancelled:?}"
        );

        // The child would have written its marker ~2s in; it must never appear.
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert!(
            !marker.exists(),
            "cancelled child survived and wrote its completion marker"
        );

        // A late completion must not overwrite the cancelled state.
        for _ in 0..25 {
            let later = failed_error(adapter.query_status(&job_id).await.unwrap());
            assert!(
                later.contains("cancelled"),
                "late completion overwrote cancelled state: {later:?}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// A timeout while both pipes are full must kill and reap the child, and
    /// publish a terminal failure.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timeout_kills_real_child_with_full_pipes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let marker = marker_path("timeout");
        let script = if cfg!(windows) {
            format!(
                "for /L %i in (1,1,20000) do @echo stdout-filler & \
                 for /L %i in (1,1,20000) do @echo stderr-filler 1>&2 & \
                 ping -n 4 127.0.0.1 >NUL & echo completed> \"{}\"",
                marker.display()
            )
        } else {
            format!(
                "yes stdout-filler | head -n 20000 & \
                 yes stderr-filler 1>&2 | head -n 20000 & \
                 sleep 3; printf completed > '{}'",
                marker.display()
            )
        };
        let adapter = process_adapter(&script);
        let config = test_config(1);
        let started = Instant::now();
        let job_id = adapter
            .dispatch_async(test_task(temp_dir.path()), &config)
            .await
            .unwrap();

        let status = await_terminal(&adapter, &job_id, Duration::from_secs(8)).await;
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "timeout supervision took too long: {:?}",
            started.elapsed()
        );
        let error = failed_error(status);
        assert!(
            error.contains("timeout"),
            "expected timeout failure, got {error:?}"
        );

        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(
            !marker.exists(),
            "timed-out child survived and wrote its completion marker"
        );
    }

    /// A quick normal exit completes with captured stdout.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn normal_completion_captures_stdout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let script = if cfg!(windows) {
            "echo supervisor-ok & echo diagnostic 1>&2".to_string()
        } else {
            "printf 'supervisor-ok\\n'; printf 'diagnostic\\n' >&2".to_string()
        };
        let adapter = process_adapter(&script);
        let config = test_config(5);
        let job_id = adapter
            .dispatch_async(test_task(temp_dir.path()), &config)
            .await
            .unwrap();

        match await_terminal(&adapter, &job_id, Duration::from_secs(5)).await {
            AgentStatus::Completed { result } => {
                assert!(result.success, "result should be successful: {result:?}");
                assert_eq!(result.exit_code, 0);
                assert!(
                    result.stdout.contains("supervisor-ok"),
                    "stdout not captured: {:?}",
                    result.stdout
                );
                assert!(
                    result.stderr.contains("diagnostic"),
                    "stderr not captured: {:?}",
                    result.stderr
                );
            }
            other => panic!("expected Completed, got {other:?}"),
        }
    }
}
