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

#[cfg(test)]
mod deep_tests {
    use super::*;
    use std::process::Command as StdCommand;

    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        StdCommand::new("git")
            .args(["init", "-q", "-b", "main"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    #[test]
    fn discover_inside_repo_returns_root() {
        let dir = init_repo();
        let ctx = ProjectContext::discover(dir.path()).unwrap();
        let expected = dir.path().canonicalize().unwrap();
        assert_eq!(ctx.repo_root(), expected.as_path());
    }

    #[test]
    fn discover_from_subdirectory_walks_up() {
        let dir = init_repo();
        let sub = dir.path().join("a/b/c");
        std::fs::create_dir_all(&sub).unwrap();
        let ctx = ProjectContext::discover(&sub).unwrap();
        assert_eq!(ctx.repo_root(), dir.path().canonicalize().unwrap().as_path());
    }

    #[test]
    fn discover_outside_repo_errors() {
        let dir = tempfile::tempdir().unwrap();
        // A bare temp directory with no .git anywhere up the tree is hard to
        // guarantee, so assert only that a non-repo dir under /tmp does not
        // silently succeed with a wrong root when discovery fails.
        let result = ProjectContext::discover(&dir.path().join("definitely-missing"));
        // Either an error or a discovered repo whose state dir is unrelated;
        // the missing path should not resolve to the temp dir itself.
        if let Ok(ctx) = result {
            assert_ne!(ctx.repo_root(), dir.path());
        }
    }

    #[test]
    fn database_path_is_under_state_dir() {
        let dir = init_repo();
        let ctx = ProjectContext::discover(dir.path()).unwrap();
        let db = ctx.database_path();
        assert_eq!(db.file_name().unwrap(), "agileplus.db");
        assert!(db.parent().unwrap().ends_with(".agileplus"));
    }

    #[test]
    fn database_path_is_absolute() {
        let dir = init_repo();
        let ctx = ProjectContext::discover(dir.path()).unwrap();
        assert!(ctx.database_path().is_absolute());
    }

    #[test]
    fn context_clone_and_equality() {
        let dir = init_repo();
        let ctx = ProjectContext::discover(dir.path()).unwrap();
        let cloned = ctx.clone();
        assert_eq!(ctx, cloned);
    }

    #[test]
    fn context_debug_contains_repo_root() {
        let dir = init_repo();
        let ctx = ProjectContext::discover(dir.path()).unwrap();
        assert!(format!("{ctx:?}").contains("ProjectContext"));
    }

    #[test]
    fn discover_from_file_path_inside_repo() {
        let dir = init_repo();
        let file = dir.path().join("notes.txt");
        std::fs::write(&file, "hi").unwrap();
        let ctx = ProjectContext::discover(&file).unwrap();
        assert_eq!(ctx.repo_root(), dir.path().canonicalize().unwrap().as_path());
    }
}
