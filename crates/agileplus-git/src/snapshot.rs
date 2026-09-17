//! Point-in-time snapshot of git repository state using git2.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshot {
    pub head_commit: String,
    pub branch: Option<String>,
    pub is_detached: bool,
    pub dirty_files: Vec<DirtyFile>,
    pub worktrees: Vec<WorktreeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirtyFile {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head_commit: String,
    pub is_main: bool,
}

impl GitSnapshot {
    /// Take a snapshot using git2. Falls back gracefully if repo is not available.
    pub fn capture(
        repo_root: &std::path::Path,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let repo = git2::Repository::open(repo_root)?;

        // Get HEAD
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?.id().to_string();
        let is_detached = repo.head_detached()?;
        let branch = if is_detached {
            None
        } else {
            head.shorthand().map(|s| s.to_string())
        };

        // Get dirty files
        let mut dirty_files = Vec::new();
        let statuses = repo.statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(false),
        ))?;

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();
            let file_status = if status.contains(git2::Status::WT_NEW)
                || status.contains(git2::Status::INDEX_NEW)
            {
                FileStatus::Added
            } else if status.contains(git2::Status::WT_DELETED)
                || status.contains(git2::Status::INDEX_DELETED)
            {
                FileStatus::Deleted
            } else if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::INDEX_MODIFIED)
            {
                FileStatus::Modified
            } else {
                FileStatus::Untracked
            };
            dirty_files.push(DirtyFile {
                path,
                status: file_status,
            });
        }

        // Build worktrees list
        let mut worktrees = Vec::new();

        // Main worktree
        worktrees.push(WorktreeInfo {
            path: repo_root.to_path_buf(),
            branch: branch.clone(),
            head_commit: head_commit.clone(),
            is_main: true,
        });

        // Linked worktrees
        if let Ok(wt_names) = repo.worktrees() {
            for name in wt_names.iter().flatten() {
                if let Ok(wt) = repo.find_worktree(name) {
                    let wt_path = wt.path().to_path_buf();
                    if let Ok(wt_repo) = git2::Repository::open(&wt_path) {
                        let wt_head = wt_repo.head().ok();
                        let wt_commit = wt_head
                            .as_ref()
                            .and_then(|h| h.peel_to_commit().ok())
                            .map(|c| c.id().to_string())
                            .unwrap_or_default();
                        let wt_branch = wt_head
                            .as_ref()
                            .and_then(|h| h.shorthand().map(|s| s.to_string()));
                        worktrees.push(WorktreeInfo {
                            path: wt_path,
                            branch: wt_branch,
                            head_commit: wt_commit,
                            is_main: false,
                        });
                    }
                }
            }
        }

        Ok(Self {
            head_commit,
            branch,
            is_detached,
            dirty_files,
            worktrees,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "t@example.com"]);
        git(dir.path(), &["config", "user.name", "tester"]);
        std::fs::write(dir.path().join("a.txt"), "hello\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", "init"]);
        dir
    }

    #[test]
    fn capture_on_non_repo_errors() {
        let dir = tempfile::tempdir().unwrap();
        assert!(GitSnapshot::capture(dir.path()).is_err());
    }

    #[test]
    fn capture_reports_head_commit_and_branch() {
        let dir = init_repo();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert_eq!(snap.branch.as_deref(), Some("main"));
        assert!(!snap.is_detached);
        assert_eq!(snap.head_commit.len(), 40);
        assert!(snap.head_commit.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn capture_clean_repo_has_no_dirty_files() {
        let dir = init_repo();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert!(snap.dirty_files.is_empty(), "got {:?}", snap.dirty_files);
    }

    #[test]
    fn capture_detects_modified_file() {
        let dir = init_repo();
        std::fs::write(dir.path().join("a.txt"), "changed\n").unwrap();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        let modified: Vec<_> = snap
            .dirty_files
            .iter()
            .filter(|f| matches!(f.status, FileStatus::Modified))
            .collect();
        assert_eq!(modified.len(), 1);
        assert_eq!(modified[0].path, "a.txt");
    }

    #[test]
    fn capture_detects_untracked_file() {
        let dir = init_repo();
        std::fs::write(dir.path().join("new.txt"), "x\n").unwrap();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert!(snap
            .dirty_files
            .iter()
            .any(|f| f.path == "new.txt" && matches!(f.status, FileStatus::Untracked)));
    }

    #[test]
    fn capture_detects_added_file() {
        let dir = init_repo();
        std::fs::write(dir.path().join("new.txt"), "x\n").unwrap();
        git(dir.path(), &["add", "new.txt"]);
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert!(snap
            .dirty_files
            .iter()
            .any(|f| f.path == "new.txt" && matches!(f.status, FileStatus::Added)));
    }

    #[test]
    fn capture_detects_deleted_file() {
        let dir = init_repo();
        std::fs::remove_file(dir.path().join("a.txt")).unwrap();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert!(snap
            .dirty_files
            .iter()
            .any(|f| f.path == "a.txt" && matches!(f.status, FileStatus::Deleted)));
    }

    #[test]
    fn capture_detached_head_has_no_branch() {
        let dir = init_repo();
        let head = {
            let out = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(dir.path())
                .output()
                .unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(dir.path(), &["checkout", "-q", &head]);
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert!(snap.is_detached);
        assert!(snap.branch.is_none());
    }

    #[test]
    fn capture_includes_main_worktree_entry() {
        let dir = init_repo();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        assert_eq!(snap.worktrees.len(), 1);
        assert!(snap.worktrees[0].is_main);
        assert_eq!(snap.worktrees[0].branch.as_deref(), Some("main"));
    }

    #[test]
    fn snapshot_serde_roundtrip() {
        let dir = init_repo();
        let snap = GitSnapshot::capture(dir.path()).unwrap();
        let json = serde_json::to_string(&snap).unwrap();
        let back: GitSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.head_commit, snap.head_commit);
        assert_eq!(back.branch, snap.branch);
    }

    #[test]
    fn file_status_serde_roundtrip() {
        for status in [
            FileStatus::Modified,
            FileStatus::Added,
            FileStatus::Deleted,
            FileStatus::Untracked,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: FileStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_string(&back).unwrap(),
                json,
                "status roundtrip"
            );
        }
    }

    #[test]
    fn dirty_file_serde_roundtrip() {
        let f = DirtyFile {
            path: "src/lib.rs".into(),
            status: FileStatus::Modified,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: DirtyFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, "src/lib.rs");
        assert!(matches!(back.status, FileStatus::Modified));
    }

    #[test]
    fn worktree_info_serde_roundtrip() {
        let w = WorktreeInfo {
            path: PathBuf::from("/tmp/wt"),
            branch: Some("feature/x".into()),
            head_commit: "deadbeef".into(),
            is_main: false,
        };
        let json = serde_json::to_string(&w).unwrap();
        let back: WorktreeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.branch.as_deref(), Some("feature/x"));
        assert!(!back.is_main);
    }
}
