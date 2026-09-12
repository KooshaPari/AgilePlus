use std::path::{Path, PathBuf};

use git2::Repository;

use agileplus_domain::error::DomainError;

/// Canonical repository-local paths used by AgilePlus stateful operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    repo_root: PathBuf,
    state_dir: PathBuf,
}

impl ProjectContext {
    /// Discover the non-bare Git worktree that contains `start`.
    pub fn discover(start: &Path) -> Result<Self, DomainError> {
        let repository = Repository::discover(start).map_err(|error| {
            DomainError::Storage(format!(
                "failed to discover git repository from {}: {error}",
                start.display()
            ))
        })?;
        let repo_root = repository
            .workdir()
            .ok_or_else(|| {
                DomainError::Storage("bare repositories cannot own AgilePlus state".into())
            })?
            .canonicalize()
            .map_err(|error| {
                DomainError::Storage(format!("canonicalize repository root: {error}"))
            })?;
        let state_dir = repo_root.join(".agileplus");

        Ok(Self {
            repo_root,
            state_dir,
        })
    }

    /// Return the canonical Git worktree root.
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Return the canonical repository-local SQLite database path.
    pub fn database_path(&self) -> PathBuf {
        self.state_dir.join("agileplus.db")
    }
}
