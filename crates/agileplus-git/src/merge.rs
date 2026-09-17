// SPDX-License-Identifier: MIT OR Apache-2.0
//! Merge operations for `GitVcsAdapter`.
//!
//! Extracted from `lib.rs` to keep the main module under 350 lines.
//!
//! Traceability: recs #10, #11 from `AUDIT_BLOC_VS_2026_SOTA.md`.

use std::path::Path;
use std::process::{Command, Stdio};

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::MergeResult;

use super::GitVcsAdapter;

impl GitVcsAdapter {
    /// Run `git merge --no-ff` of `source` into the current HEAD of `dir`.
    pub(crate) fn merge_in_dir(
        &self,
        dir: &Path,
        source: &str,
        target: &str,
    ) -> Result<MergeResult, DomainError> {
        let result = Command::new("git")
            .args([
                "merge",
                "--no-ff",
                "--autostash",
                "-m",
                &format!("Merge branch '{source}' into {target}"),
                source,
            ])
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| DomainError::Storage(format!("failed to spawn git merge: {e}")))?;
        let stdout = String::from_utf8_lossy(&result.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
        if result.status.success() {
            let head_out = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .ok();
            let head = head_out
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            Ok(MergeResult {
                success: true,
                conflicts: vec![],
                merged_commit: Some(head.clone()),
                commit: Some(head),
                message: Some(if stdout.is_empty() { stderr } else { stdout }),
            })
        } else {
            let conflicts = Self::unresolved_conflicts_in(dir);
            let conflicts = if conflicts.is_empty() {
                Self::parse_conflicts(&format!("{stdout}\n{stderr}"))
            } else {
                conflicts
            };
            Ok(MergeResult {
                success: false,
                conflicts,
                merged_commit: None,
                commit: None,
                message: Some(if stderr.is_empty() { stdout } else { stderr }),
            })
        }
    }

    /// Merge `source` into `target` inside a throwaway worktree, so we
    /// never need to check out `target` in the caller's worktree.
    pub(crate) fn merge_via_temp_worktree(
        &self,
        source: &str,
        target: &str,
    ) -> Result<MergeResult, DomainError> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let tmp = std::env::temp_dir().join(format!(
            "agileplus-ship-{}-{}-{}",
            target.replace('/', "_"),
            std::process::id(),
            stamp
        ));
        if tmp.exists() {
            let _ = std::fs::remove_dir_all(&tmp);
        }
        let tmp_str = tmp.to_string_lossy().into_owned();
        self.run_git_status(&["worktree", "add", "--force", &tmp_str, target])?;
        let merge_result = self.merge_in_dir(&tmp, source, target);
        if let Err(e) = self.run_git(&["worktree", "remove", "--force", &tmp_str]) {
            tracing::warn!(
                path = %tmp_str,
                error = %e,
                "failed to remove temporary ship worktree; attempting filesystem cleanup"
            );
            let _ = std::fs::remove_dir_all(&tmp);
        }
        merge_result
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command as StdCommand;
    use tempfile::tempdir;

    fn git(dir: &Path, args: &[&str]) {
        let out = StdCommand::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn make_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        git(&path, &["init", "-q", "-b", "main"]);
        git(&path, &["config", "user.email", "t@example.com"]);
        git(&path, &["config", "user.name", "tester"]);
        std::fs::write(path.join("f.txt"), "base\n").unwrap();
        git(&path, &["add", "."]);
        git(&path, &["commit", "-q", "-m", "init"]);
        (dir, path)
    }

    fn commit_file(dir: &Path, name: &str, content: &str, msg: &str) {
        std::fs::write(dir.join(name), content).unwrap();
        git(dir, &["add", name]);
        git(dir, &["commit", "-q", "-m", msg]);
    }

    #[test]
    fn unresolved_conflicts_in_non_repo_is_empty() {
        let tmp = tempdir().unwrap();
        let out = GitVcsAdapter::unresolved_conflicts_in(tmp.path());
        assert!(out.is_empty());
    }

    #[test]
    fn unresolved_conflicts_in_clean_repo_is_empty() {
        let (_d, path) = make_repo();
        assert!(GitVcsAdapter::unresolved_conflicts_in(&path).is_empty());
    }

    #[test]
    fn merge_in_dir_success_sets_commit_and_no_conflicts() {
        let (_d, path) = make_repo();
        git(&path, &["checkout", "-q", "-b", "feature"]);
        commit_file(&path, "feature.txt", "feat\n", "add feature");
        git(&path, &["checkout", "-q", "main"]);

        let adapter = GitVcsAdapter::new(path.clone());
        let result = adapter.merge_in_dir(&path, "feature", "main").unwrap();
        assert!(result.success);
        assert!(result.conflicts.is_empty());
        assert!(result.commit.is_some());
        assert!(result.merged_commit.is_some());
        assert!(result.message.is_some());
    }

    #[test]
    fn merge_in_dir_conflict_reports_paths() {
        let (_d, path) = make_repo();
        git(&path, &["checkout", "-q", "-b", "feature"]);
        commit_file(&path, "f.txt", "feature side\n", "feature edit");
        git(&path, &["checkout", "-q", "main"]);
        commit_file(&path, "f.txt", "main side\n", "main edit");

        let adapter = GitVcsAdapter::new(path.clone());
        let result = adapter.merge_in_dir(&path, "feature", "main").unwrap();
        assert!(!result.success);
        assert!(!result.conflicts.is_empty());
        assert_eq!(result.conflicts[0].path, "f.txt");
        assert!(result.commit.is_none());
        // Clean up the conflicted merge state so the temp dir can be removed.
        let _ = StdCommand::new("git")
            .args(["merge", "--abort"])
            .current_dir(&path)
            .output();
    }

    #[test]
    fn merge_in_dir_up_to_date_still_succeeds() {
        let (_d, path) = make_repo();
        git(&path, &["branch", "feature"]);
        let adapter = GitVcsAdapter::new(path.clone());
        // feature is an ancestor of main, so merge is a no-op success.
        let result = adapter.merge_in_dir(&path, "feature", "main").unwrap();
        assert!(result.success);
    }

    #[test]
    fn merge_in_dir_missing_source_returns_failure_result() {
        let (_d, path) = make_repo();
        let adapter = GitVcsAdapter::new(path.clone());
        let result = adapter
            .merge_in_dir(&path, "does-not-exist", "main")
            .unwrap();
        assert!(!result.success);
    }

    #[test]
    fn merge_via_temp_worktree_succeeds_for_free_branch() {
        let (_d, path) = make_repo();
        git(&path, &["checkout", "-q", "-b", "feature"]);
        commit_file(&path, "x.txt", "x\n", "feature work");
        git(&path, &["checkout", "-q", "main"]);
        git(&path, &["branch", "release"]);

        let adapter = GitVcsAdapter::new(path);
        let result = adapter.merge_via_temp_worktree("feature", "release").unwrap();
        assert!(result.success);
    }

    #[test]
    fn parse_conflicts_via_merge_module_works() {
        // Sanity: the shared parser is reachable from merge.rs.
        let out = GitVcsAdapter::parse_conflicts("CONFLICT (content): Merge conflict in a.rs\n");
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn merge_via_temp_worktree_cleans_up_on_success() {
        let (_d, path) = make_repo();
        git(&path, &["checkout", "-q", "-b", "feature"]);
        commit_file(&path, "y.txt", "y\n", "work");
        git(&path, &["checkout", "-q", "main"]);
        git(&path, &["branch", "release2"]);

        let adapter = GitVcsAdapter::new(path.clone());
        adapter.merge_via_temp_worktree("feature", "release2").unwrap();
        let raw = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&path)
            .output()
            .unwrap();
        let list = String::from_utf8_lossy(&raw.stdout);
        assert!(
            !list.contains("agileplus-ship-"),
            "temporary ship worktree should be removed: {list}"
        );
    }
}
