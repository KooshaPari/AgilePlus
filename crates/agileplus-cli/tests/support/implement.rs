// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared fixtures for the `agileplus implement` command integration tests.
//!
//! `run_implement` is the real entry point that drives `run_review_loop`. The
//! unit-level `review_loop_flow` tests exercise the loop directly; these
//! fixtures exist so the implement tests can drive the full orchestration with
//! a real SQLite storage adapter and a recording VCS/agent double.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;

use agileplus_cli::commands::implement::ImplementArgs;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentPort, AgentResult, AgentStatus, AgentTask};
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo,
};
use agileplus_domain::ports::{StoragePort, VcsPort};
use agileplus_sqlite::SqliteStorageAdapter;

/// `tokio` macros are not enabled for this crate, so drive futures by hand.
/// Never call this from inside an existing runtime.
pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio_test::block_on(fut)
}

/// A feature row in one of the states the implement tests need.
pub fn feature(slug: &str, state: FeatureState) -> Feature {
    Feature {
        id: 0,
        slug: slug.to_string(),
        friendly_name: "Implement Test Feature".to_string(),
        state,
        spec_hash: [7u8; 32],
        target_branch: "main".to_string(),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at_commit: None,
        last_modified_commit: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Default `ImplementArgs`: all WPs, default parallelism, five review cycles.
pub fn args(feature: &str) -> ImplementArgs {
    ImplementArgs {
        feature: feature.to_string(),
        wp: None,
        parallel: 1,
        max_review_cycles: 2,
        resume: false,
    }
}

/// Plan artifacts `run_implement` reads before dispatching an agent.
pub const SPEC: &str = "# Spec\nAcceptance: the thing works.\n";
pub const PLAN: &str = "# Plan\nStep one.\n";

/// A VCS double that materializes real files under a temp worktree root so the
/// artifact-materialization step in `run_implement` performs genuine filesystem
/// writes, and records the worktrees it hands out.
pub struct TempVcs {
    root: PathBuf,
    created: Mutex<Vec<PathBuf>>,
    cleaned: Mutex<Vec<PathBuf>>,
    pub missing_prompt: bool,
}

impl TempVcs {
    /// Create a VCS double rooted at a fresh, collision-free temp directory.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "agileplus-implement-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).expect("creating vcs root");
        Self {
            root,
            created: Mutex::new(vec![]),
            cleaned: Mutex::new(vec![]),
            missing_prompt: false,
        }
    }

    pub fn with_missing_prompt(mut self) -> Self {
        self.missing_prompt = true;
        self
    }

    /// Worktree paths handed out by `create_worktree`, in call order.
    pub fn created_worktrees(&self) -> Vec<PathBuf> {
        self.created.lock().unwrap().clone()
    }

    /// Worktree paths passed to `cleanup_worktree`.
    pub fn cleaned_worktrees(&self) -> Vec<PathBuf> {
        self.cleaned.lock().unwrap().clone()
    }

    /// Drop the temp tree. Tests call this at the end so repeated runs do not
    /// accumulate directories.
    pub fn cleanup(self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn unique() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

/// Drive a future on a tokio runtime whose clock is paused, so the review
/// loop's `tokio::time::sleep` advances instantly instead of burning real
/// wall-clock seconds. `run_implement` hardcodes a 30s poll interval, so a
/// real-clock run of the max-cycles test would otherwise take a minute.
///
/// `start_paused` only advances the clock when the runtime has no ready task,
/// so a single `block_on` would stall on the first sleep. Instead the future
/// runs on its own current-thread runtime on a blocking thread while the outer
/// runtime repeatedly advances its own paused clock; the inner runtime also
/// auto-advances whenever it goes idle, which is what actually unblocks the
/// loop's sleeps.
pub fn block_on_paused<F>(fut: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let outer = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .start_paused(true)
        .build()
        .expect("building outer paused runtime");
    outer.block_on(async {
        let handle = tokio::task::spawn_blocking(move || {
            let inner = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .start_paused(true)
                .build()
                .expect("building inner paused runtime");
            inner.block_on(fut)
        });
        handle.await.expect("joining paused future")
    })
}

#[async_trait]
impl VcsPort for TempVcs {
    async fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        let path = self.root.join(feature_slug).join(wp_id);
        std::fs::create_dir_all(&path).map_err(|e| DomainError::Storage(e.to_string()))?;
        self.created.lock().unwrap().push(path.clone());
        Ok(path)
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Ok(vec![])
    }

    async fn cleanup_worktree(&self, worktree_path: &Path) -> Result<(), DomainError> {
        self.cleaned
            .lock()
            .unwrap()
            .push(worktree_path.to_path_buf());
        Ok(())
    }

    async fn create_branch(&self, _branch_name: &str, _base: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_branches(
        &self,
        _pattern: Option<&str>,
        _remote: bool,
    ) -> Result<Vec<BranchInfo>, DomainError> {
        Ok(vec![])
    }

    async fn delete_branch(
        &self,
        _branch_name: &str,
        _force: bool,
        _remote: Option<&str>,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn checkout_branch(&self, _branch_name: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn merge_to_target(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<MergeResult, DomainError> {
        Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: None,
            commit: None,
            message: None,
        })
    }

    async fn detect_conflicts(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        Ok(vec![])
    }

    /// `run_implement` reads the plan artifacts from here before dispatching.
    /// A missing prompt is an error so tests can assert the failure propagates.
    async fn read_artifact(
        &self,
        _feature_slug: &str,
        relative_path: &str,
    ) -> Result<String, DomainError> {
        match relative_path {
            "spec.md" => Ok(SPEC.to_string()),
            "plan.md" => Ok(PLAN.to_string()),
            other if other.starts_with("tasks/") => {
                if self.missing_prompt {
                    Err(DomainError::NotFound(format!("missing {other}")))
                } else {
                    Ok(format!("# Prompt\n{other}\n"))
                }
            }
            _ => Ok(String::new()),
        }
    }

    async fn write_artifact(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
        _content: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn artifact_exists(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
    ) -> Result<bool, DomainError> {
        Ok(false)
    }

    async fn scan_feature_artifacts(
        &self,
        _feature_slug: &str,
    ) -> Result<FeatureArtifacts, DomainError> {
        Ok(FeatureArtifacts {
            spec: None,
            research: None,
            plan: None,
            other: vec![],
            meta_json: None,
            audit_chain: None,
            evidence_paths: vec![],
        })
    }
}

/// How the fake agent should report the dispatched job.
#[derive(Debug, Clone)]
pub enum AgentBehavior {
    /// Report success on the first poll.
    Success,
    /// Report the given failure stderr on every poll, exhausting the cycles.
    AlwaysFail(String),
    /// Report a hard failure on the first poll.
    HardFail(String),
}

/// An `AgentPort` double recording the dispatch task and driving the review
/// loop to a chosen outcome. `run_implement` calls `dispatch_async` then polls
/// via `query_status`; recording the task lets tests assert the real prompt and
/// context paths that were built.
pub struct ScriptedAgent {
    behavior: AgentBehavior,
    /// The `AgentTask` from the most recent `dispatch_async`.
    pub dispatched: Mutex<Option<AgentTask>>,
    /// Prompt path from the dispatch, in call order.
    pub instructions: Mutex<Vec<String>>,
    poll_count: Mutex<u32>,
}

impl ScriptedAgent {
    pub fn new(behavior: AgentBehavior) -> Self {
        Self {
            behavior,
            dispatched: Mutex::new(None),
            instructions: Mutex::new(vec![]),
            poll_count: Mutex::new(0),
        }
    }

    pub fn success() -> Self {
        Self::new(AgentBehavior::Success)
    }

    /// How many times the review loop polled the agent.
    pub fn polls(&self) -> u32 {
        *self.poll_count.lock().unwrap()
    }
}

impl AgentPort for ScriptedAgent {
    fn dispatch(
        &self,
        _task: AgentTask,
        _config: &AgentConfig,
    ) -> impl std::future::Future<Output = Result<AgentResult, DomainError>> + Send {
        async { Err(DomainError::NotFound("not used by implement".into())) }
    }

    fn dispatch_async(
        &self,
        task: AgentTask,
        _config: &AgentConfig,
    ) -> impl std::future::Future<Output = Result<String, DomainError>> + Send {
        let recorded = task;
        async move {
            *self.dispatched.lock().unwrap() = Some(recorded);
            Ok("job-1".to_string())
        }
    }

    fn query_status(
        &self,
        _job_id: &str,
    ) -> impl std::future::Future<Output = Result<AgentStatus, DomainError>> + Send {
        let status = {
            *self.poll_count.lock().unwrap() += 1;
            match &self.behavior {
                AgentBehavior::Success => AgentStatus::Completed {
                    result: AgentResult {
                        success: true,
                        pr_url: Some("https://example.invalid/pr/1".to_string()),
                        commits: vec!["abc123".to_string()],
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                },
                AgentBehavior::AlwaysFail(stderr) => AgentStatus::Completed {
                    result: AgentResult {
                        success: false,
                        pr_url: None,
                        commits: vec![],
                        stdout: String::new(),
                        stderr: stderr.clone(),
                        exit_code: 1,
                    },
                },
                AgentBehavior::HardFail(error) => AgentStatus::Failed {
                    error: error.clone(),
                },
            }
        };
        async move { Ok(status) }
    }

    fn cancel(
        &self,
        _job_id: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        async { Ok(()) }
    }

    fn send_instruction(
        &self,
        _job_id: &str,
        instruction: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let recorded = instruction.to_string();
        async move {
            self.instructions.lock().unwrap().push(recorded);
            Ok(())
        }
    }
}

/// Create a feature in `Planned` plus one WP, returning `(feature_id, wp_id)`.
pub async fn seed(storage: &SqliteStorageAdapter, slug: &str, fstate: FeatureState) -> (i64, i64) {
    let fid = StoragePort::create_feature(storage, &feature(slug, fstate))
        .await
        .unwrap();
    let mut wp = WorkPackage::new(fid, "Build the thing", 1, "works end to end");
    wp.id = 0;
    let wp_id = StoragePort::create_work_package(storage, &wp)
        .await
        .unwrap();
    (fid, wp_id)
}
