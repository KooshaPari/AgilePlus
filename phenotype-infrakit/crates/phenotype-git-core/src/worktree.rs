//! Worktree management utilities

/// Worktree specification
pub struct WorktreeSpec {
    pub name: String,
    pub path: std::path::PathBuf,
}

/// Worktree manager
pub struct Worktree {
    pub root: std::path::PathBuf,
}

impl Worktree {
    /// Create a new worktree manager
    pub fn new(root: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
        Ok(Self {
            root: root.into(),
        })
    }

    /// List worktrees
    pub fn list_worktrees(&self) -> std::io::Result<Vec<WorktreeSpec>> {
        Ok(Vec::new())
    }
}
