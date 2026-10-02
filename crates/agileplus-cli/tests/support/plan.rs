// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared fixtures for the `agileplus plan` command integration tests.
//!
//! `run_plan` reads `spec.md`/`research.md` through the `VcsPort` and writes
//! `plan.md`, one `tasks/WPNN-<slug>.md` prompt per work package, and
//! `contracts/governance-v1.json`. `PlanVcs` implements that port against real
//! files under a temp root, so those outputs are asserted on disk rather than
//! out of a memory map. Storage is the real SQLite adapter.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use agileplus_cli::commands::plan::PlanArgs;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo,
};
use agileplus_domain::ports::{StoragePort, VcsPort};
use agileplus_sqlite::SqliteStorageAdapter;

pub use super::implement::{block_on, feature};

/// A `VcsPort` backed by real files under `root`, laid out as
/// `<root>/<feature-slug>/<relative-path>`.
///
/// Only the artifact methods are reachable from `run_plan`; the git-lifecycle
/// methods a planning run never calls are left `unimplemented!` so an
/// unexpected call fails loudly instead of silently succeeding.
pub struct PlanVcs {
    root: PathBuf,
}

impl PlanVcs {
    /// Create the VCS rooted at `root` (a temp directory owned by the test).
    pub fn new(root: &Path) -> Self {
        std::fs::create_dir_all(root).expect("creating plan vcs root");
        Self {
            root: root.to_path_buf(),
        }
    }

    /// Absolute path a given artifact write lands on.
    pub fn path_for(&self, slug: &str, relative_path: &str) -> PathBuf {
        self.root.join(slug).join(relative_path)
    }

    /// Read an artifact back from disk, if it was written.
    pub fn read_file(&self, slug: &str, relative_path: &str) -> Option<String> {
        std::fs::read_to_string(self.path_for(slug, relative_path)).ok()
    }

    /// Artifacts actually present under a feature's directory.
    pub fn written_files(&self, slug: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut stack = vec![self.root.join(slug)];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Ok(rel) = path.strip_prefix(self.root.join(slug)) {
                    found.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        found.sort();
        found
    }
}

#[async_trait]
impl VcsPort for PlanVcs {
    async fn create_worktree(
        &self,
        _feature_slug: &str,
        _wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        unimplemented!("PlanVcs::create_worktree")
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Ok(Vec::new())
    }

    async fn cleanup_worktree(&self, _worktree_path: &Path) -> Result<(), DomainError> {
        unimplemented!("PlanVcs::cleanup_worktree")
    }

    async fn create_branch(&self, _branch_name: &str, _base: &str) -> Result<(), DomainError> {
        unimplemented!("PlanVcs::create_branch")
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
        unimplemented!("PlanVcs::delete_branch")
    }

    async fn checkout_branch(&self, _branch_name: &str) -> Result<(), DomainError> {
        unimplemented!("PlanVcs::checkout_branch")
    }

    async fn merge_to_target(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<MergeResult, DomainError> {
        unimplemented!("PlanVcs::merge_to_target")
    }

    async fn detect_conflicts(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        unimplemented!("PlanVcs::detect_conflicts")
    }

    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<String, DomainError> {
        let path = self.path_for(feature_slug, relative_path);
        std::fs::read_to_string(&path)
            .map_err(|e| DomainError::NotFound(format!("{}: {e}", path.display())))
    }

    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> Result<(), DomainError> {
        let path = self.path_for(feature_slug, relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DomainError::Storage(format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(&path, content)
            .map_err(|e| DomainError::Storage(format!("{}: {e}", path.display())))
    }

    async fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<bool, DomainError> {
        Ok(self.path_for(feature_slug, relative_path).exists())
    }

    async fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> Result<FeatureArtifacts, DomainError> {
        Ok(FeatureArtifacts {
            spec: self.read_file(feature_slug, "spec.md"),
            research: self.read_file(feature_slug, "research.md"),
            plan: self.read_file(feature_slug, "plan.md"),
            other: Vec::new(),
            meta_json: None,
            audit_chain: None,
            evidence_paths: Vec::new(),
        })
    }
}

// ── Seeding ──────────────────────────────────────────────────────────────────

/// Create a feature row in `state` and return its id.
pub async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str, state: FeatureState) -> i64 {
    StoragePort::create_feature(storage, &feature(slug, state))
        .await
        .expect("seeding feature")
}

/// Default `PlanArgs`: the named feature, library max_wps, one agent per WP.
pub fn args(feature_slug: &str) -> PlanArgs {
    PlanArgs {
        feature: feature_slug.to_string(),
        max_wps: 20,
        agents_per_wp: 1,
    }
}

/// `PlanArgs` with an explicit `max_wps` cap, for grouping tests.
pub fn args_with_max_wps(feature_slug: &str, max_wps: usize) -> PlanArgs {
    PlanArgs {
        feature: feature_slug.to_string(),
        max_wps,
        agents_per_wp: 1,
    }
}

/// Build `spec.md` body from bare FR descriptions, numbered `FR-001`.. up.
///
/// Produces the `- **FR-NNN**: <description>` shape that
/// `run_plan`'s parser accepts.
pub fn spec_with_frs(descriptions: &[&str]) -> String {
    let mut lines = vec![
        "# Specification".to_string(),
        String::new(),
        "## Functional Requirements".to_string(),
    ];
    for (i, description) in descriptions.iter().enumerate() {
        lines.push(format!("- **FR-{:03}**: {description}", i + 1));
    }
    lines.join("\n")
}

/// A `spec.md` with no recognisable FR lines at all.
pub fn spec_without_frs() -> String {
    "# Specification\n\nProse only; no numbered requirements.\n".to_string()
}

/// Seed `spec.md` for a feature through the real VCS write path, so the file
/// exists on disk exactly where `run_plan` will read it from.
pub fn write_spec(vcs: &PlanVcs, slug: &str, content: &str) {
    block_on(vcs.write_artifact(slug, "spec.md", content)).expect("writing spec.md");
}
