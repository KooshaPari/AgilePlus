// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared integration-test doubles for the `agileplus-cli` command layer.
//!
//! These are intentionally test-side helpers only (nothing here is compiled
//! into the production binary). The `MockVcs` keeps artifacts in memory so
//! command entry points that read/write specs and revision diffs can be
//! exercised without touching the real filesystem or git.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo,
};
use agileplus_domain::ports::VcsPort;

/// In-memory `VcsPort` double.
///
/// Artifacts are keyed by `(feature_slug, relative_path)`. Read/write can be
/// made to fail on demand so error-handling branches can be driven.
#[derive(Default)]
pub struct MockVcs {
    artifacts: Mutex<HashMap<(String, String), String>>,
    /// When set, every `read_artifact` returns `Err(NotFound)`.
    pub read_fails: bool,
    /// When set, every `write_artifact` returns `Err(Storage)`.
    pub write_fails: bool,
}

impl MockVcs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed an artifact before handing the mock to a command.
    pub fn with_artifact(self, slug: &str, path: &str, content: &str) -> Self {
        self.artifacts
            .lock()
            .unwrap()
            .insert((slug.to_string(), path.to_string()), content.to_string());
        self
    }

    /// Read an artifact back without going through the trait (test assertion).
    pub fn get(&self, slug: &str, path: &str) -> Option<String> {
        self.artifacts
            .lock()
            .unwrap()
            .get(&(slug.to_string(), path.to_string()))
            .cloned()
    }

    /// Number of artifacts currently stored.
    pub fn artifact_count(&self) -> usize {
        self.artifacts.lock().unwrap().len()
    }
}

#[async_trait]
impl VcsPort for MockVcs {
    async fn create_worktree(
        &self,
        _feature_slug: &str,
        _wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        unimplemented!("MockVcs::create_worktree")
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Ok(Vec::new())
    }

    async fn cleanup_worktree(&self, _worktree_path: &Path) -> Result<(), DomainError> {
        unimplemented!("MockVcs::cleanup_worktree")
    }

    async fn create_branch(&self, _branch_name: &str, _base: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_branches(
        &self,
        _pattern: Option<&str>,
        _remote: bool,
    ) -> Result<Vec<BranchInfo>, DomainError> {
        Ok(Vec::new())
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
            conflicts: Vec::new(),
            merged_commit: Some("mock".into()),
            commit: Some("mock".into()),
            message: None,
        })
    }

    async fn detect_conflicts(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        Ok(Vec::new())
    }

    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<String, DomainError> {
        if self.read_fails {
            return Err(DomainError::NotFound(format!(
                "mock read disabled: {feature_slug}/{relative_path}"
            )));
        }
        self.artifacts
            .lock()
            .unwrap()
            .get(&(feature_slug.to_string(), relative_path.to_string()))
            .cloned()
            .ok_or_else(|| {
                DomainError::NotFound(format!("no artifact {feature_slug}/{relative_path}"))
            })
    }

    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> Result<(), DomainError> {
        if self.write_fails {
            return Err(DomainError::Storage("mock write disabled".into()));
        }
        self.artifacts.lock().unwrap().insert(
            (feature_slug.to_string(), relative_path.to_string()),
            content.to_string(),
        );
        Ok(())
    }

    async fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<bool, DomainError> {
        Ok(self
            .artifacts
            .lock()
            .unwrap()
            .contains_key(&(feature_slug.to_string(), relative_path.to_string())))
    }

    async fn scan_feature_artifacts(
        &self,
        _feature_slug: &str,
    ) -> Result<FeatureArtifacts, DomainError> {
        Ok(FeatureArtifacts {
            spec: None,
            research: None,
            plan: None,
            other: Vec::new(),
            meta_json: None,
            audit_chain: None,
            evidence_paths: Vec::new(),
        })
    }
}
