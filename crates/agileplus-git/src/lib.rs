//! Git VCS adapter for AgilePlus.

use std::path::{Path, PathBuf};

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
};

/// Git-backed VCS adapter.
pub struct GitVcsAdapter {
    repo_root: PathBuf,
}

impl GitVcsAdapter {
    /// Create an adapter rooted at the current working directory.
    pub fn from_current_dir() -> anyhow::Result<Self> {
        Ok(Self {
            repo_root: std::env::current_dir()?,
        })
    }
}

#[async_trait::async_trait]
impl VcsPort for GitVcsAdapter {
    async fn create_worktree(&self, _feature_slug: &str, _wp_id: &str) -> Result<PathBuf, DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Ok(vec![])
    }
    async fn cleanup_worktree(&self, _path: &Path) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn create_branch(&self, _branch: &str, _base: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn list_branches(&self, _pattern: Option<&str>, _remote: bool) -> Result<Vec<BranchInfo>, DomainError> {
        Ok(vec![])
    }
    async fn delete_branch(&self, _branch: &str, _force: bool, _remote: Option<&str>) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn checkout_branch(&self, _branch: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn merge_to_target(&self, _source: &str, _target: &str) -> Result<MergeResult, DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn detect_conflicts(&self, _source: &str, _target: &str) -> Result<Vec<ConflictInfo>, DomainError> {
        Ok(vec![])
    }
    async fn read_artifact(&self, _feature_slug: &str, _relative_path: &str) -> Result<String, DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn write_artifact(&self, _feature_slug: &str, _relative_path: &str, _content: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }
    async fn artifact_exists(&self, _feature_slug: &str, _relative_path: &str) -> Result<bool, DomainError> {
        Ok(false)
    }
    async fn scan_feature_artifacts(&self, _feature_slug: &str) -> Result<FeatureArtifacts, DomainError> {
        Ok(FeatureArtifacts {
            spec: None,
            research: None,
            plan: None,
            other: vec![],
        })
    }
}
