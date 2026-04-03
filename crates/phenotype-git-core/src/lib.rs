//! # Phenotype Git Core
//!
//! Core git operations for the Phenotype ecosystem.
//! Provides abstractions for git repository operations, diffing, and merging.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::path::{Path, PathBuf};
use std::process::Command;
use phenotype_error_core::StorageError as Error;
use std::io;

/// Custom result type for git operations
pub type Result<T> = std::result::Result<T, Error>;

/// Represents a git repository at a specific path
#[derive(Debug, Clone)]
pub struct GitRepository {
    root: PathBuf,
}

impl GitRepository {
    /// Opens an existing git repository
    pub fn open<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        if !root.join(".git").exists() {
            return Err(Error::NotFound(format!("Git repository not found at {}", root.display())));
        }
        Ok(Self { root })
    }

    /// Runs a git command in the repository context
    pub fn run(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(Error::Other(err));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Checks if the path is inside a work tree
    pub fn is_worktree(&self) -> Result<bool> {
        Ok(self.run(&["rev-parse", "--is-inside-work-tree"])? == "true")
    }

    /// Checks if the repository has uncommitted changes
    pub fn is_clean(&self) -> Result<bool> {
        let output = self.run(&["status", "--porcelain"])?;
        Ok(output.is_empty())
    }

    /// Checks if the current branch has an upstream configured
    pub fn has_upstream(&self) -> bool {
        self.run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).is_ok()
    }

    /// Gets ahead/behind counts for the upstream branch
    pub fn get_upstream_counts(&self) -> Result<(i64, i64)> {
        let counts = self.run(&["rev-list", "--left-right", "--count", "@{u}...HEAD"])?;
        let mut parts = counts.split_whitespace();
        let behind = parts.next().ok_or_else(|| Error::Other("Missing behind count".into()))?;
        let ahead = parts.next().ok_or_else(|| Error::Other("Missing ahead count".into()))?;
        
        let behind = behind.parse::<i64>().map_err(|e| Error::Other(e.to_string()))?;
        let ahead = ahead.parse::<i64>().map_err(|e| Error::Other(e.to_string()))?;
        
        Ok((behind, ahead))
    }

    /// Gets the current branch name
    pub fn current_branch(&self) -> Result<String> {
        self.run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    /// Gets the SHA of the current HEAD
    pub fn head_sha(&self) -> Result<String> {
        self.run(&["rev-parse", "HEAD"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_git(dir: &Path) {
        Command::new("git").arg("init").current_dir(dir).status().unwrap();
        Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(dir).status().unwrap();
        Command::new("git").args(["config", "user.name", "Test User"]).current_dir(dir).status().unwrap();
        // Set default branch to main
        Command::new("git").args(["config", "init.defaultBranch", "main"]).current_dir(dir).status().unwrap();
    }

    #[test]
    fn test_git_repo_basics() {
        let dir = tempdir().unwrap();
        setup_git(dir.path());
        
        let repo = GitRepository::open(dir.path()).unwrap();
        assert!(repo.is_worktree().unwrap());
        assert!(repo.is_clean().unwrap());
        
        // Add a file
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
        assert!(!repo.is_clean().unwrap());
        
        // Commit it
        repo.run(&["add", "."]).unwrap();
        repo.run(&["commit", "-m", "initial commit"]).unwrap();
        assert!(repo.is_clean().unwrap());
        
        assert_eq!(repo.current_branch().unwrap(), "main");
        assert!(repo.head_sha().unwrap().len() == 40);
    }
}

