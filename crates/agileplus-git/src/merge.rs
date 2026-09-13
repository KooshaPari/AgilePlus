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
