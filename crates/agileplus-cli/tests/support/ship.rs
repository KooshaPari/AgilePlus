// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared fixtures for the `agileplus ship` command integration tests.
//!
//! Provides a recording `VcsPort` double that captures merge calls, artifact
//! writes, and worktree cleanups, plus the feature/WP seeding helpers the
//! ship tests use to drive `run_ship` to each branch of its control flow.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;

use agileplus_cli::commands::ship::ShipArgs;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::error::DomainError;
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

/// A feature row in one of the states the ship tests need.
pub fn feature(slug: &str, state: FeatureState) -> Feature {
    Feature {
        id: 0,
        slug: slug.to_string(),
        friendly_name: "Ship Test Feature".to_string(),
        state,
        spec_hash: [9u8; 32],
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

/// Default `ShipArgs` for a slug: no target override, no skip, no dry run.
pub fn args(feature: &str) -> ShipArgs {
    ShipArgs {
        feature: feature.to_string(),
        target: None,
        skip_validate: false,
        dry_run: false,
    }
}

/// How `merge_to_target` should behave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeOutcome {
    Ok,
    Error,
    Conflicts(Vec<String>),
}

/// Recording `VcsPort` double. Captures merge calls, artifact writes, and
/// cleanup paths so tests can assert the observable side effects of `run_ship`.
pub struct RecordingVcs {
    merge_outcome: MergeOutcome,
    worktrees: Vec<WorktreeInfo>,
    cleanup_fails: bool,
    list_worktrees_fails: bool,
    write_artifact_fails: bool,
    /// `(source_branch, target_branch)` per merge attempt, in call order.
    pub merges: Mutex<Vec<(String, String)>>,
    /// Worktree paths passed to a successful `cleanup_worktree`.
    pub cleaned: Mutex<Vec<PathBuf>>,
    /// `(slug, relative_path, content)` per successful `write_artifact`.
    pub artifacts: Mutex<Vec<(String, String, String)>>,
}

impl Default for RecordingVcs {
    fn default() -> Self {
        Self {
            merge_outcome: MergeOutcome::Ok,
            worktrees: vec![],
            cleanup_fails: false,
            list_worktrees_fails: false,
            write_artifact_fails: false,
            merges: Mutex::new(vec![]),
            cleaned: Mutex::new(vec![]),
            artifacts: Mutex::new(vec![]),
        }
    }
}

impl RecordingVcs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_merge_outcome(mut self, outcome: MergeOutcome) -> Self {
        self.merge_outcome = outcome;
        self
    }

    pub fn with_worktrees(mut self, worktrees: Vec<WorktreeInfo>) -> Self {
        self.worktrees = worktrees;
        self
    }

    pub fn with_cleanup_failure(mut self) -> Self {
        self.cleanup_fails = true;
        self
    }

    pub fn with_worktree_listing_failure(mut self) -> Self {
        self.list_worktrees_fails = true;
        self
    }

    pub fn with_artifact_write_failure(mut self) -> Self {
        self.write_artifact_fails = true;
        self
    }

    /// A worktree belonging to `slug`, for cleanup-filtering tests.
    pub fn worktree(path: &str, slug: &str) -> WorktreeInfo {
        WorktreeInfo {
            path: PathBuf::from(path),
            commit: "a".into(),
            branch: format!("{slug}/wp01"),
            feature_slug: slug.to_string(),
            wp_id: "WP01".into(),
        }
    }
}

#[async_trait]
impl VcsPort for RecordingVcs {
    async fn create_worktree(
        &self,
        _feature_slug: &str,
        _wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        Err(DomainError::NotFound("not used by ship".into()))
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        if self.list_worktrees_fails {
            Err(DomainError::NotFound("worktree listing failed".into()))
        } else {
            Ok(self.worktrees.clone())
        }
    }

    async fn cleanup_worktree(&self, worktree_path: &Path) -> Result<(), DomainError> {
        if self.cleanup_fails {
            return Err(DomainError::NotFound("cleanup failed".into()));
        }
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
        source: &str,
        target: &str,
    ) -> Result<MergeResult, DomainError> {
        self.merges
            .lock()
            .unwrap()
            .push((source.to_string(), target.to_string()));
        match &self.merge_outcome {
            MergeOutcome::Ok => Ok(MergeResult {
                success: true,
                conflicts: vec![],
                merged_commit: Some("abc123".into()),
                commit: Some("abc123".into()),
                message: Some("merged".into()),
            }),
            MergeOutcome::Error => Err(DomainError::NotFound("branch missing".into())),
            MergeOutcome::Conflicts(paths) => Ok(MergeResult {
                success: false,
                conflicts: paths
                    .iter()
                    .map(|p| ConflictInfo {
                        path: p.clone(),
                        file_path: p.clone(),
                        conflict_type: "content".into(),
                        ours: None,
                        theirs: None,
                    })
                    .collect(),
                merged_commit: None,
                commit: None,
                message: Some("conflicts".into()),
            }),
        }
    }

    async fn detect_conflicts(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        Ok(vec![])
    }

    async fn read_artifact(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
    ) -> Result<String, DomainError> {
        Err(DomainError::NotFound("no artifact".into()))
    }

    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> Result<(), DomainError> {
        if self.write_artifact_fails {
            return Err(DomainError::NotFound("artifact write failed".into()));
        }
        self.artifacts.lock().unwrap().push((
            feature_slug.to_string(),
            relative_path.to_string(),
            content.to_string(),
        ));
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

/// Create a feature plus one work package per `(sequence, state)` pair,
/// persisting each WP's requested state. Returns the feature id.
pub async fn seed(
    storage: &SqliteStorageAdapter,
    slug: &str,
    fstate: FeatureState,
    states: &[(i32, WpState)],
) -> i64 {
    let id = StoragePort::create_feature(storage, &feature(slug, fstate))
        .await
        .unwrap();
    for (seq, state) in states {
        let mut wp = WorkPackage::new(id, &format!("WP {seq}"), *seq, "done");
        wp.id = 0;
        let wp_id = StoragePort::create_work_package(storage, &wp)
            .await
            .unwrap();
        if *state != WpState::Planned {
            StoragePort::update_wp_state(storage, wp_id, *state)
                .await
                .unwrap();
        }
    }
    id
}

/// Create a Done work package whose `worktree_path` is set, so ship derives
/// its branch from the worktree name rather than the `feature/{slug}/wpNN`
/// convention.
pub async fn seed_with_worktree_wp(
    storage: &SqliteStorageAdapter,
    slug: &str,
    fstate: FeatureState,
    sequence: i32,
    worktree_path: &str,
) -> (i64, i64) {
    let id = StoragePort::create_feature(storage, &feature(slug, fstate))
        .await
        .unwrap();
    let mut wp = WorkPackage::new(id, &format!("WP {sequence}"), sequence, "done");
    wp.id = 0;
    wp.worktree_path = Some(worktree_path.to_string());
    let wp_id = StoragePort::create_work_package(storage, &wp)
        .await
        .unwrap();
    StoragePort::update_wp_state(storage, wp_id, WpState::Done)
        .await
        .unwrap();
    (id, wp_id)
}
