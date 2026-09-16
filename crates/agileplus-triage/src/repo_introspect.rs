// SPDX-License-Identifier: MIT OR Apache-2.0
//! Repo introspection: detect git state, mangled directories, no-git
//! directories. Produces a `RepoInfo` snapshot suitable for triage
//! classification of the local working tree.
//!
//! Traceability: FR-AGP-020 (repo introspection)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// State of a directory as a candidate repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoState {
    /// Has a valid `.git/` directory with HEAD, branches, remotes.
    Git,
    /// Has `.git/` but it's corrupt or in an unexpected state (mangled).
    MangledGit,
    /// No `.git/` at all (plain directory, subproject of a parent repo, or fresh clone).
    NoGit,
}

/// One remote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

/// Snapshot of a repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub state: RepoState,
    pub current_branch: Option<String>,
    pub branches: Vec<String>,
    pub worktrees: Vec<String>,
    pub remotes: Vec<RemoteInfo>,
    /// 0..=100. `Git`=100, `MangledGit`=50, `NoGit`=30, `NoGit`+source markers=70
    /// (treated as a subproject of a parent repo).
    pub hygiene_score: u8,
}

/// Inspect a directory and produce a `RepoInfo` snapshot.
pub fn inspect_repo(path: &Path) -> RepoInfo {
    let path_str = path.to_string_lossy().into_owned();
    let git_entry = path.join(".git");
    if !git_entry.exists() {
        let hygiene = if has_source_files(path) { 70 } else { 30 };
        return RepoInfo {
            path: path_str,
            state: RepoState::NoGit,
            current_branch: None,
            branches: vec![],
            worktrees: vec![],
            remotes: vec![],
            hygiene_score: hygiene,
        };
    }
    let Some(git_dir) = resolve_git_dir(&git_entry) else {
        return RepoInfo {
            path: path_str,
            state: RepoState::MangledGit,
            hygiene_score: 50,
            current_branch: None,
            branches: vec![],
            worktrees: vec![],
            remotes: vec![],
        };
    };
    let head = git_dir.join("HEAD");
    if !head.exists() {
        return RepoInfo {
            path: path_str,
            state: RepoState::MangledGit,
            hygiene_score: 50,
            current_branch: None,
            branches: vec![],
            worktrees: vec![],
            remotes: vec![],
        };
    }
    let head_content = std::fs::read_to_string(&head).unwrap_or_default();
    let current_branch = head_content
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("ref: refs/heads/"))
        .map(String::from);
    let branches = read_branches(&git_dir);
    let worktrees = read_worktrees(&git_dir);
    let remotes = read_remotes(&git_dir);
    RepoInfo {
        path: path_str,
        state: RepoState::Git,
        hygiene_score: 100,
        current_branch,
        branches,
        worktrees,
        remotes,
    }
}

fn resolve_git_dir(git_entry: &Path) -> Option<PathBuf> {
    if git_entry.is_dir() {
        return Some(git_entry.to_path_buf());
    }

    let raw = std::fs::read_to_string(git_entry).ok()?;
    let target = raw.trim().strip_prefix("gitdir: ")?;
    let target = PathBuf::from(target);
    let target = if target.is_absolute() {
        target
    } else {
        git_entry.parent()?.join(target)
    };
    target.is_dir().then_some(target)
}

fn read_branches(git_dir: &Path) -> Vec<String> {
    let heads = git_dir.join("refs").join("heads");
    if !heads.exists() {
        return vec![];
    }
    walk_dir_files(&heads)
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(&heads)
                .ok()
                .map(|r| r.to_string_lossy().into_owned())
        })
        .collect()
}

fn read_worktrees(git_dir: &Path) -> Vec<String> {
    let wt = git_dir.join("worktrees");
    if !wt.exists() {
        return vec![];
    }
    std::fs::read_dir(&wt)
        .ok()
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn read_remotes(git_dir: &Path) -> Vec<RemoteInfo> {
    let cfg = git_dir.join("config");
    let raw = std::fs::read_to_string(&cfg).unwrap_or_default();
    let mut out = vec![];
    let mut current_name: Option<String> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.contains("remote") {
            // [remote "origin"]
            let name = trimmed.split('"').nth(1).unwrap_or("").to_string();
            if !name.is_empty() {
                current_name = Some(name);
            }
        } else if let Some(name) = &current_name
            && let Some(url) = trimmed.strip_prefix("url = ")
        {
            out.push(RemoteInfo {
                name: name.clone(),
                url: url.to_string(),
            });
            current_name = None;
        }
    }
    out
}

fn walk_dir_files(p: &Path) -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                out.extend(walk_dir_files(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

fn has_source_files(p: &Path) -> bool {
    for marker in &[
        "src", "lib", "pkg", "cmd", "crates", "backend", "frontend", "app",
    ] {
        if p.join(marker).is_dir() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn no_git_directory_no_source_files() {
        let tmp = TempDir::new().unwrap();
        let info = inspect_repo(tmp.path());
        assert_eq!(info.state, RepoState::NoGit);
        assert_eq!(info.hygiene_score, 30);
        assert!(info.current_branch.is_none());
        assert!(info.branches.is_empty());
        assert!(info.remotes.is_empty());
    }

    #[test]
    fn no_git_directory_with_source_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let info = inspect_repo(tmp.path());
        assert_eq!(info.state, RepoState::NoGit);
        assert_eq!(info.hygiene_score, 70);
    }

    #[test]
    fn has_source_files_various_markers() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_source_files(tmp.path()));
        for marker in &["src", "lib", "pkg", "cmd", "crates", "backend", "frontend", "app"] {
            let dir = tmp.path().join(marker);
            std::fs::create_dir_all(&dir).unwrap();
            assert!(has_source_files(tmp.path()), "{marker} should be detected");
            std::fs::remove_dir(&dir).unwrap();
        }
    }

    #[test]
    fn valid_git_repo() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        // Create HEAD
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        // Create branches
        let heads = git_dir.join("refs").join("heads");
        std::fs::create_dir_all(&heads).unwrap();
        std::fs::write(heads.join("main"), "abc123\n").unwrap();
        std::fs::write(heads.join("feat-x"), "def456\n").unwrap();
        // Create config with a remote
        std::fs::write(
            git_dir.join("config"),
            "[remote \"origin\"]\n\turl = https://github.com/test/repo.git\n",
        )
        .unwrap();

        let info = inspect_repo(tmp.path());
        assert_eq!(info.state, RepoState::Git);
        assert_eq!(info.hygiene_score, 100);
        assert_eq!(info.current_branch.as_deref(), Some("main"));
        assert!(info.branches.contains(&"main".to_string()));
        assert!(info.branches.contains(&"feat-x".to_string()));
        assert_eq!(info.remotes.len(), 1);
        assert_eq!(info.remotes[0].name, "origin");
        assert_eq!(info.remotes[0].url, "https://github.com/test/repo.git");
    }

    #[test]
    fn mangled_git_missing_head() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        // No HEAD file
        let info = inspect_repo(tmp.path());
        assert_eq!(info.state, RepoState::MangledGit);
        assert_eq!(info.hygiene_score, 50);
    }

    #[test]
    fn linked_worktree_gitdir_file() {
        let tmp = TempDir::new().unwrap();
        let worktree_dir = tmp.path().join("worktree");
        std::fs::create_dir_all(&worktree_dir).unwrap();

        // Create a real git dir elsewhere
        let real_git = tmp.path().join("real_git");
        std::fs::create_dir_all(&real_git).unwrap();
        std::fs::write(real_git.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        // Write a .git file pointing to the real git dir
        std::fs::write(
            worktree_dir.join(".git"),
            format!("gitdir: {}\n", real_git.display()),
        )
        .unwrap();

        let info = inspect_repo(&worktree_dir);
        assert_eq!(info.state, RepoState::Git);
        assert_eq!(info.hygiene_score, 100);
    }

    #[test]
    fn linked_worktree_broken_gitdir() {
        let tmp = TempDir::new().unwrap();
        let worktree_dir = tmp.path().join("worktree");
        std::fs::create_dir_all(&worktree_dir).unwrap();
        std::fs::write(
            worktree_dir.join(".git"),
            "gitdir: /nonexistent/path\n",
        )
        .unwrap();

        let info = inspect_repo(&worktree_dir);
        assert_eq!(info.state, RepoState::MangledGit);
        assert_eq!(info.hygiene_score, 50);
    }

    #[test]
    fn read_remotes_multiple() {
        let tmp = TempDir::new().unwrap();
        let git_dir = tmp.path();
        let config_content = r#"
[remote "origin"]
	url = https://github.com/org/repo.git
[remote "upstream"]
	url = https://github.com/upstream/repo.git
"#;
        std::fs::write(git_dir.join("config"), config_content).unwrap();
        let remotes = read_remotes(git_dir);
        assert_eq!(remotes.len(), 2);
        assert_eq!(remotes[0].name, "origin");
        assert_eq!(remotes[0].url, "https://github.com/org/repo.git");
        assert_eq!(remotes[1].name, "upstream");
    }

    #[test]
    fn read_remotes_empty_config() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("config"), "").unwrap();
        let remotes = read_remotes(tmp.path());
        assert!(remotes.is_empty());
    }

    #[test]
    fn read_branches_empty() {
        let tmp = TempDir::new().unwrap();
        let heads = tmp.path().join("refs").join("heads");
        std::fs::create_dir_all(&heads).unwrap();
        let branches = read_branches(tmp.path());
        assert!(branches.is_empty());
    }

    #[test]
    fn read_branches_with_files() {
        let tmp = TempDir::new().unwrap();
        let heads = tmp.path().join("refs").join("heads");
        std::fs::create_dir_all(&heads.join("sub")).unwrap();
        std::fs::write(heads.join("main"), "abc").unwrap();
        std::fs::write(heads.join("sub").join("nested"), "def").unwrap();
        let branches = read_branches(tmp.path());
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"sub/nested".to_string()));
    }

    #[test]
    fn read_worktrees_empty() {
        let tmp = TempDir::new().unwrap();
        let worktrees = read_worktrees(tmp.path());
        assert!(worktrees.is_empty());
    }

    #[test]
    fn read_worktrees_with_entries() {
        let tmp = TempDir::new().unwrap();
        let wt = tmp.path().join("worktrees");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::create_dir(wt.join("wt-1")).unwrap();
        std::fs::create_dir(wt.join("wt-2")).unwrap();
        let worktrees = read_worktrees(tmp.path());
        assert_eq!(worktrees.len(), 2);
        assert!(worktrees.contains(&"wt-1".to_string()));
        assert!(worktrees.contains(&"wt-2".to_string()));
    }

    #[test]
    fn repo_info_serialization_roundtrip() {
        let info = RepoInfo {
            path: "/tmp/test".to_string(),
            state: RepoState::Git,
            current_branch: Some("main".to_string()),
            branches: vec!["main".to_string()],
            worktrees: vec![],
            remotes: vec![RemoteInfo {
                name: "origin".to_string(),
                url: "https://example.com".to_string(),
            }],
            hygiene_score: 100,
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: RepoInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.state, RepoState::Git);
        assert_eq!(back.hygiene_score, 100);
    }

    #[test]
    fn repo_state_serialization() {
        for state in [RepoState::Git, RepoState::MangledGit, RepoState::NoGit] {
            let json = serde_json::to_string(&state).unwrap();
            let back: RepoState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, state);
        }
    }
}
