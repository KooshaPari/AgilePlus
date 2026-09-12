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

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentPort, AgentResult, AgentStatus, AgentTask};
use dashmap::DashMap;
use tokio::process::Child;
use uuid::Uuid;

/// Real agent adapter with pluggable backend support.
pub struct RealAgentAdapter {
    backend: AgentBackend,
    jobs: std::sync::Arc<DashMap<String, JobState>>,
}

#[derive(Debug, Clone)]
enum AgentBackend {
    ClaudeCli,
    CheapLlmMcp,
    Stub,
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

    fn binary_and_args(&self) -> Option<(&'static str, Vec<&'static str>)> {
        match self {
            AgentBackend::ClaudeCli => Some((
                "claude",
                vec!["--print", "--dangerously-skip-permissions"],
            )),
            AgentBackend::CheapLlmMcp => Some((
                "cheap-llm-mcp",
                vec!["dispatch", "--model", "minimax"],
            )),
            AgentBackend::Stub => None,
        }
    }
}

/// State for a dispatched agent job.
#[derive(Debug)]
enum JobState {
    Running {
        child: std::sync::Mutex<Option<Child>>,
        stdin_tx: Option<tokio::sync::mpsc::Sender<Vec<u8>>>,
    },
    Finished(AgentStatus),
}

impl RealAgentAdapter {
    pub fn new() -> Self {
        let backend = AgentBackend::from_env();
        tracing::info!("Initializing agent adapter with backend: {:?}", backend);
        Self {
            backend,
            jobs: std::sync::Arc::new(DashMap::new()),
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
        backends::spawn_process(binary, &args, task, config).await
    }
}

impl Default for RealAgentAdapter {
    fn default() -> Self {
        Self::new()
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
                let (mut child, _tx) = self.spawn_backend(&task, config).await?;
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
                self.jobs
                    .insert(job_id.clone(), JobState::Finished(status));
            }
            _ => {
                let (child, stdin_tx) = self.spawn_backend(&task, config).await?;
                let pid = child.id().unwrap_or(0);

                self.jobs.insert(
                    job_id.clone(),
                    JobState::Running {
                        child: std::sync::Mutex::new(Some(child)),
                        stdin_tx: Some(stdin_tx),
                    },
                );

                let jobs = self.jobs.clone();
                let bj_id = job_id.clone();
                let timeout = config.timeout_secs;
                let backend_name = format!("{:?}", self.backend);

                tokio::spawn(async move {
                    let child_handle = jobs.get(&bj_id).and_then(|s| match &*s {
                        JobState::Running { child, .. } => child.lock().unwrap().take(),
                        _ => None,
                    });

                    let Some(mut child) = child_handle else {
                        return;
                    };

                    let output = match tokio::time::timeout(
                        std::time::Duration::from_secs(timeout),
                        child.wait_with_output(),
                    )
                    .await
                    {
                        Ok(Ok(o)) => o,
                        Ok(Err(e)) => {
                            tracing::error!("agent process error: {e}");
                            if let Some(mut s) = jobs.get_mut(&bj_id) {
                                *s = JobState::Finished(AgentStatus::Failed {
                                    error: format!("process error: {e}"),
                                });
                            }
                            return;
                        }
                        Err(_) => {
                            tracing::error!("agent timed out after {timeout}s");
                            let _ = child.kill().await;
                            if let Some(mut s) = jobs.get_mut(&bj_id) {
                                *s = JobState::Finished(AgentStatus::Failed {
                                    error: format!("timeout after {timeout}s"),
                                });
                            }
                            return;
                        }
                    };

                    let result = match backends::build_result(output) {
                        Ok(r) => r,
                        Err(e) => {
                            if let Some(mut s) = jobs.get_mut(&bj_id) {
                                *s = JobState::Finished(AgentStatus::Failed {
                                    error: e.to_string(),
                                });
                            }
                            return;
                        }
                    };

                    let status = if result.success {
                        AgentStatus::Completed { result }
                    } else {
                        AgentStatus::Failed {
                            error: result.stderr.clone(),
                        }
                    };

                    if let Some(mut s) = jobs.get_mut(&bj_id) {
                        *s = JobState::Finished(status);
                    }
                    tracing::info!(job_id = %bj_id, backend = %backend_name, "async dispatch completed");
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
                JobState::Running { .. } => AgentStatus::Running { pid: 0 },
                JobState::Finished(status) => status.clone(),
            })
            .ok_or_else(|| DomainError::NotFound(format!("job {job_id} not found")))
    }

    async fn cancel(&self, job_id: &str) -> Result<(), DomainError> {
        let mut entry = self
            .jobs
            .get_mut(job_id)
            .ok_or_else(|| DomainError::NotFound(format!("job {job_id} not found")))?;

        match &mut *entry {
            JobState::Running { child, .. } => {
                if let Some(mut cp) = child.lock().unwrap().take() {
                    let _ = cp.kill().await;
                    tracing::info!(job_id, "killed agent process");
                }
                *entry = JobState::Finished(AgentStatus::Failed {
                    error: "cancelled by user".to_string(),
                });
            }
            JobState::Finished(_) => {
                tracing::warn!(job_id, "cancel on already-finished job");
            }
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
                    DomainError::Agent(format!(
                        "stdin closed; cannot send instruction to {job_id}"
                    ))
                })?;
                tracing::info!(job_id, instruction = %instruction, "sent instruction");
                Ok(())
            }
            JobState::Running {
                stdin_tx: None, ..
            } => Err(DomainError::Agent(format!(
                "no stdin channel for {job_id}"
            ))),
            JobState::Finished(_) => Err(DomainError::Agent(format!(
                "job {job_id} already finished"
            ))),
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
