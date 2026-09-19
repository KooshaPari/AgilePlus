//! `VcsPort` mock.
//!
//! Defaults match the behaviour every existing suite relies on (no branches,
//! no worktrees, `create_worktree` → `/tmp/worktree`, a merge that succeeds
//! without producing a commit). On top of that it exposes:
//!
//! - **Failure injection** (`fail_on`) so tests can drive the handlers' `Err`
//!   arms — no existing suite reaches them, because every port method here
//!   used to be infallible.
//! - **Call recording** (`calls`) so tests can prove the query/body values a
//!   handler received are the ones it actually forwarded to the port, instead
//!   of only observing the stub's canned reply.
//! - **Result overrides** (`with_branches`, `with_merge_result`,
//!   `with_worktree_path`) so the handler's mapping of a *populated* port
//!   result is observable over HTTP.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
};
use async_trait::async_trait;

/// One recorded call into the mock, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VcsCall {
    pub operation: &'static str,
    /// Arguments rendered as strings; `"<none>"` stands in for an absent option.
    pub args: Vec<String>,
}

impl VcsCall {
    fn new(operation: &'static str, args: &[String]) -> Self {
        Self { operation, args: args.to_vec() }
    }
}

/// Default worktree path the mock hands back when a test does not override it.
const DEFAULT_WORKTREE_PATH: &str = "/tmp/worktree";

#[derive(Default)]
struct MockVcsInner {
    failures: Mutex<HashMap<&'static str, String>>,
    calls: Mutex<Vec<VcsCall>>,
    branches: Mutex<Vec<BranchInfo>>,
    worktrees: Mutex<Vec<WorktreeInfo>>,
    worktree_path: Mutex<Option<PathBuf>>,
    merge_result: Mutex<Option<MergeResult>>,
}

/// In-memory `VcsPort` with failure injection and call recording.
#[derive(Clone, Default)]
pub(crate) struct MockVcs {
    inner: Arc<MockVcsInner>,
}

impl MockVcs {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Force `operation` (the `VcsPort` method name) to return
    /// `Err(DomainError::Storage(message))`.
    pub(crate) fn fail_on(&self, operation: &'static str, message: &str) {
        self.inner
            .failures
            .lock()
            .expect("vcs failures lock poisoned")
            .insert(operation, message.to_string());
    }

    /// The injected `DomainError` for `operation`, if any.
    fn injected_failure(&self, operation: &str) -> Option<DomainError> {
        self.inner
            .failures
            .lock()
            .expect("vcs failures lock poisoned")
            .get(operation)
            .map(|message| DomainError::Storage(message.clone()))
    }

    /// Every call recorded so far, oldest first.
    pub(crate) fn calls(&self) -> Vec<VcsCall> {
        self.inner
            .calls
            .lock()
            .expect("vcs calls lock poisoned")
            .clone()
    }

    /// The recorded calls for one operation.
    pub(crate) fn calls_of(&self, operation: &str) -> Vec<VcsCall> {
        self.calls()
            .into_iter()
            .filter(|c| c.operation == operation)
            .collect()
    }

    /// Branches `list_branches` reports.
    pub(crate) fn with_branches(&self, branches: Vec<BranchInfo>) {
        *self.inner.branches.lock().expect("vcs branches lock poisoned") = branches;
    }

    /// Worktrees `list_worktrees` reports.
    pub(crate) fn with_worktrees(&self, worktrees: Vec<WorktreeInfo>) {
        *self.inner.worktrees.lock().expect("vcs worktrees lock poisoned") = worktrees;
    }

    /// Path `create_worktree` reports.
    pub(crate) fn with_worktree_path(&self, path: impl Into<PathBuf>) {
        *self
            .inner
            .worktree_path
            .lock()
            .expect("vcs worktree path lock poisoned") = Some(path.into());
    }

    /// Result `merge_to_target` reports.
    pub(crate) fn with_merge_result(&self, result: MergeResult) {
        *self
            .inner
            .merge_result
            .lock()
            .expect("vcs merge result lock poisoned") = Some(result);
    }

    fn record(&self, operation: &'static str, args: &[String]) {
        self.inner
            .calls
            .lock()
            .expect("vcs calls lock poisoned")
            .push(VcsCall::new(operation, args));
    }

    fn check(&self, operation: &'static str) -> Result<(), DomainError> {
        match self.injected_failure(operation) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn create_worktree_path(&self) -> PathBuf {
        self.inner
            .worktree_path
            .lock()
            .expect("vcs worktree path lock poisoned")
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_WORKTREE_PATH))
    }

    fn merge(&self) -> MergeResult {
        self.inner
            .merge_result
            .lock()
            .expect("vcs merge result lock poisoned")
            .clone()
            .unwrap_or(MergeResult {
                success: true,
                conflicts: vec![],
                merged_commit: None,
                commit: None,
                message: None,
            })
    }
}

fn opt(value: Option<&str>) -> String {
    value.unwrap_or("<none>").to_string()
}

#[async_trait]
impl VcsPort for MockVcs {
    async fn create_worktree(&self, fs: &str, wp: &str) -> Result<PathBuf, DomainError> {
        self.record("create_worktree", &[fs.to_string(), wp.to_string()]);
        self.check("create_worktree")?;
        Ok(self.create_worktree_path())
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        self.record("list_worktrees", &[]);
        self.check("list_worktrees")?;
        Ok(self
            .inner
            .worktrees
            .lock()
            .expect("vcs worktrees lock poisoned")
            .clone())
    }

    async fn cleanup_worktree(&self, path: &Path) -> Result<(), DomainError> {
        self.record("cleanup_worktree", &[path.display().to_string()]);
        self.check("cleanup_worktree")?;
        Ok(())
    }

    async fn create_branch(&self, branch: &str, base: &str) -> Result<(), DomainError> {
        self.record("create_branch", &[branch.to_string(), base.to_string()]);
        self.check("create_branch")?;
        Ok(())
    }

    async fn list_branches(
        &self,
        pattern: Option<&str>,
        remote: bool,
    ) -> Result<Vec<BranchInfo>, DomainError> {
        self.record("list_branches", &[opt(pattern), remote.to_string()]);
        self.check("list_branches")?;
        Ok(self
            .inner
            .branches
            .lock()
            .expect("vcs branches lock poisoned")
            .clone())
    }

    async fn delete_branch(
        &self,
        branch_name: &str,
        force: bool,
        remote: Option<&str>,
    ) -> Result<(), DomainError> {
        self.record(
            "delete_branch",
            &[branch_name.to_string(), force.to_string(), opt(remote)],
        );
        self.check("delete_branch")?;
        Ok(())
    }

    async fn checkout_branch(&self, branch: &str) -> Result<(), DomainError> {
        self.record("checkout_branch", &[branch.to_string()]);
        self.check("checkout_branch")?;
        Ok(())
    }

    async fn merge_to_target(&self, source: &str, target: &str) -> Result<MergeResult, DomainError> {
        self.record("merge_to_target", &[source.to_string(), target.to_string()]);
        self.check("merge_to_target")?;
        Ok(self.merge())
    }

    async fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        self.record("detect_conflicts", &[source.to_string(), target.to_string()]);
        self.check("detect_conflicts")?;
        Ok(vec![])
    }

    async fn read_artifact(&self, fs: &str, path: &str) -> Result<String, DomainError> {
        self.record("read_artifact", &[fs.to_string(), path.to_string()]);
        self.check("read_artifact")?;
        Ok(String::new())
    }

    async fn write_artifact(&self, fs: &str, path: &str, content: &str) -> Result<(), DomainError> {
        self.record(
            "write_artifact",
            &[fs.to_string(), path.to_string(), content.to_string()],
        );
        self.check("write_artifact")?;
        Ok(())
    }

    async fn artifact_exists(&self, fs: &str, path: &str) -> Result<bool, DomainError> {
        self.record("artifact_exists", &[fs.to_string(), path.to_string()]);
        self.check("artifact_exists")?;
        Ok(false)
    }

    async fn scan_feature_artifacts(&self, fs: &str) -> Result<FeatureArtifacts, DomainError> {
        self.record("scan_feature_artifacts", &[fs.to_string()]);
        self.check("scan_feature_artifacts")?;
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
