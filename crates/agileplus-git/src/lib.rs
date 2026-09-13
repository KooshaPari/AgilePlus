// SPDX-License-Identifier: MIT OR Apache-2.0
//! Git VCS adapter for AgilePlus.
//!
//! Backed by [`git2`](https://docs.rs/git2) (libgit2 FFI) for read /
//! write operations that have a clean `git2` API (worktree add, branch
//! create / delete, ref resolution, branch listing), and by the
//! `git(1)` CLI for operations that libgit2 either does not expose or
//! exposes in a way that is brittle across versions â€” namely
//!
//! - `git worktree remove --force <path>` (more reliable than
//!   `Worktree::prune`, which silently no-ops on dirty worktrees),
//! - `git push <remote> :<branch>` to delete a remote branch,
//! - `git checkout` for fast branch switching,
//! - `git merge --no-ff <source> --autostash` to merge a feature branch
//!   with the standard "no fast-forward" / autostash combination the
//!   domain expects, and
//! - `git merge-tree <base> <branch>` for fast file-level conflict
//!   detection without touching the working tree.
//!
//! The adapter is rooted at a single `repo_root: PathBuf` and re-opens
//! the libgit2 repository on each call. This is cheap (libgit2 caches
//! the on-disk parse) and avoids the lifetime pain of holding an open
//! `git2::Repository` across await points.
//!
//! Audit traceability: recs #10, #11 from
//! `AUDIT_BLOC_VS_2026_SOTA.md`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use git2::Repository;

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    ConflictInfo, MergeResult,
};

use crate::claim_bound::ClaimBoundWorktree;

pub mod claim_bound;
pub mod materialize;
pub mod project_context;
#[path = "lib/vcs_port_impl.rs"]
mod vcs_port_impl;

pub use project_context::ProjectContext;

/// Git-backed VCS adapter. Cheap to clone (the heavy state lives in
/// libgit2's on-disk cache).
#[derive(Debug, Clone)]
pub struct GitVcsAdapter {
    pub(crate) repo_root: PathBuf,
}

impl GitVcsAdapter {
    /// Create an adapter rooted at the current working directory.
    pub fn from_current_dir() -> anyhow::Result<Self> {
        Ok(Self {
            repo_root: std::env::current_dir()?,
        })
    }

    /// Create an adapter rooted at an explicit path. The path does not
    /// need to be the repo root â€” it may be a subdirectory; we
    /// `discover()` the actual root on each call.
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Return the adapter's repository root.
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Open the underlying libgit2 repository, walking up from
    /// `repo_root` if necessary. Maps libgit2 errors to
    /// [`DomainError::Storage`].
    pub(crate) fn open(&self) -> Result<Repository, DomainError> {
        Repository::discover(&self.repo_root).map_err(|e| {
            DomainError::Storage(format!(
                "failed to open git repository at {}: {e}",
                self.repo_root.display()
            ))
        })
    }

    /// Resolve a base ref (branch, tag, commit-ish) to its
    /// [`git2::Commit`].
    fn resolve_commit<'a>(
        repo: &'a Repository,
        base: &str,
    ) -> Result<git2::Commit<'a>, DomainError> {
        let object = repo
            .revparse_single(base)
            .map_err(|e| DomainError::Storage(format!("revparse({base}): {e}")))?;
        object
            .peel_to_commit()
            .map_err(|e| DomainError::Storage(format!("peel to commit({base}): {e}")))
    }

    /// Build the canonical worktree name and branch name for a
    /// `(feature_slug, wp_id)` pair.
    pub(crate) fn worktree_branch(feature_slug: &str, wp_id: &str) -> String {
        format!("feat/{feature_slug}/{wp_id}")
    }

    /// Build the canonical worktree directory name (sibling of
    /// `repo_root`).
    fn worktree_dirname(feature_slug: &str, wp_id: &str) -> String {
        format!("{feature_slug}-{wp_id}")
    }

    /// Compute the absolute worktree path for a `(feature_slug,
    /// wp_id)` pair: `<repo_root>/../<feature_slug>-<wp_id>`.
    pub fn worktree_path(&self, feature_slug: &str, wp_id: &str) -> PathBuf {
        let parent = self
            .repo_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        parent.join(Self::worktree_dirname(feature_slug, wp_id))
    }

    /// Run `git(1)` with the given args inside `repo_root` and return
    /// the trimmed stdout on success, or a [`DomainError::Storage`]
    /// with stderr on failure.
    fn run_git(&self, args: &[&str]) -> Result<String, DomainError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| DomainError::Storage(format!("failed to spawn git: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            return Err(DomainError::Storage(format!(
                "git {} failed: {}",
                args.join(" "),
                if stderr.trim().is_empty() {
                    stdout.trim()
                } else {
                    stderr.trim()
                }
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Run `git(1)` and return stdout even on non-zero exit (for
    /// commands like `merge-tree` that output conflict info on stdout
    /// and exit with non-zero).
    fn run_git_allow_failure(&self, args: &[&str]) -> Result<String, DomainError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| DomainError::Storage(format!("failed to spawn git: {e}")))?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !stdout.trim().is_empty() {
                return Ok(stdout);
            }
            return Err(DomainError::Storage(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }
        Ok(stdout.trim().to_string())
    }

    /// Run `git(1)` and ignore stdout; only the exit status matters.
    pub(crate) fn run_git_status(&self, args: &[&str]) -> Result<(), DomainError> {
        let _ = self.run_git(args)?;
        Ok(())
    }

    // -----------------------------------------------------------------
    // Claim-bound worktree creation (rec #11)
    // -----------------------------------------------------------------

    /// Create a worktree bound to a [`agileplus_triage::claim::Claim`].
    /// Validates that the claim is `kind=Worktree` + `state=Active` and
    /// records the resulting worktree path in the claim's
    /// [`agileplus_triage::claim::ClaimReason::Branch`] so a later
    /// `from_claim()` lookup can recover it.
    ///
    /// `claim_store` is the in-memory or SQLite-backed store that
    /// contains the claim. After worktree creation we look the claim
    /// back up to update its `reason`.
    pub fn create_claim_bound_worktree<S: claim_bound::ClaimStoreBound>(
        repo_root: PathBuf,
        feature_slug: &str,
        wp_id: &str,
        claim: &agileplus_triage::claim::Claim,
        claim_store: &mut S,
    ) -> Result<PathBuf, DomainError> {
        ClaimBoundWorktree::create(repo_root, feature_slug, wp_id, claim, claim_store)
    }

    /// True when `git checkout` failed because the branch is already
    /// checked out in a different worktree.
    pub(crate) fn checkout_blocked_by_other_worktree(err: &DomainError) -> bool {
        let msg = err.to_string().to_lowercase();
        msg.contains("already used by worktree")
            || msg.contains("is already checked out")
            || (msg.contains("checkout") && msg.contains("worktree"))
    }

    /// Extract conflicts from the human-readable output emitted by Git's
    /// merge machinery.  `merge-tree` differs across Git versions: Apple
    /// Git 2.54 writes unmerged index rows (`mode oid stage<TAB>path`) plus
    /// `CONFLICT (...)` diagnostics, while older versions can emit a patch.
    fn parse_conflicts(raw: &str) -> Vec<ConflictInfo> {
        let mut paths = BTreeMap::<String, String>::new();
        let mut current_diff_path: Option<String> = None;
        let mut legacy_conflict_heading = false;

        for line in raw.lines() {
            if line.trim_end() == "changed in both" {
                legacy_conflict_heading = true;
                continue;
            }

            if legacy_conflict_heading {
                if let Some(path) = Self::legacy_merge_tree_path(line) {
                    paths
                        .entry(path.to_string())
                        .or_insert_with(|| "content".to_string());
                    legacy_conflict_heading = false;
                    continue;
                }
                if !line.starts_with(char::is_whitespace) {
                    legacy_conflict_heading = false;
                }
            }

            if let Some((metadata, path)) = line.split_once('\t') {
                let fields: Vec<_> = metadata.split_whitespace().collect();
                if fields.len() == 3
                    && fields[0].chars().all(|c| c.is_ascii_digit())
                    && fields[2].chars().all(|c| c.is_ascii_digit())
                    && !path.is_empty()
                {
                    paths
                        .entry(path.to_string())
                        .or_insert_with(|| "content".to_string());
                    continue;
                }
            }

            if let Some(rest) = line.strip_prefix("CONFLICT (") {
                if let Some((kind, description)) = rest.split_once("): ")
                    && let Some(path) = Self::conflict_diagnostic_path(description)
                {
                    paths.insert(path.to_string(), kind.to_string());
                }
                continue;
            }

            if let Some(path) = line.strip_prefix("changed in both ") {
                if !path.is_empty() {
                    paths
                        .entry(path.to_string())
                        .or_insert_with(|| "content".to_string());
                }
                continue;
            }

            if line.starts_with("diff --git ") {
                current_diff_path = Self::diff_header_destination_path(line);
            } else if (line.starts_with("<<<<<<<")
                || line.starts_with("=======")
                || line.starts_with(">>>>>>>"))
                && current_diff_path.is_some()
            {
                let path = current_diff_path.take().expect("checked above");
                paths.entry(path).or_insert_with(|| "content".to_string());
            }
        }

        paths
            .into_iter()
            .map(|(path, conflict_type)| ConflictInfo {
                path: path.clone(),
                file_path: path,
                conflict_type,
                ours: None,
                theirs: None,
            })
            .collect()
    }

    /// Extract the path from the `base <mode> <oid> <path>` row following a
    /// legacy `changed in both` merge-tree heading without losing spaces in
    /// the path itself.
    fn legacy_merge_tree_path(line: &str) -> Option<&str> {
        let mut remainder = line.trim_start();
        for _ in 0..3 {
            let delimiter = remainder.find(char::is_whitespace)?;
            remainder = remainder[delimiter..].trim_start();
        }
        (!remainder.is_empty()).then_some(remainder)
    }

    /// Return the destination path from a `diff --git` header. Git quotes
    /// paths containing whitespace, so splitting the header on whitespace
    /// would truncate a valid conflicted path.
    fn diff_header_destination_path(line: &str) -> Option<String> {
        let input = line.strip_prefix("diff --git ")?;
        let (_, remainder) = Self::git_path_token(input)?;
        let (destination, _) = Self::git_path_token(remainder.trim_start())?;
        Some(
            destination
                .strip_prefix("b/")
                .unwrap_or(&destination)
                .to_string(),
        )
    }

    /// Parse one Git diff header path token, including Git's C-style quoting
    /// for whitespace and special characters.
    fn git_path_token(input: &str) -> Option<(String, &str)> {
        if !input.starts_with('"') {
            let end = input.find(char::is_whitespace).unwrap_or(input.len());
            return (!input[..end].is_empty()).then(|| (input[..end].to_string(), &input[end..]));
        }

        let mut decoded = Vec::new();
        let mut chars = input[1..].chars();
        while let Some(character) = chars.next() {
            match character {
                '"' => return Some((String::from_utf8(decoded).ok()?, chars.as_str())),
                '\\' => {
                    let escaped = chars.next()?;
                    match escaped {
                        'n' => decoded.push(b'\n'),
                        'r' => decoded.push(b'\r'),
                        't' => decoded.push(b'\t'),
                        '"' | '\\' => decoded.push(escaped as u8),
                        '0'..='7' => {
                            let second = chars.next()?;
                            let third = chars.next()?;
                            let octal = [escaped, second, third].iter().collect::<String>();
                            let byte = u8::from_str_radix(&octal, 8).ok()?;
                            decoded.push(byte);
                        }
                        other => {
                            let mut buffer = [0; 4];
                            decoded.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
                        }
                    }
                }
                other => {
                    let mut buffer = [0; 4];
                    decoded.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
                }
            }
        }
        None
    }

    /// Extract a path only from merge-tree diagnostic forms whose path
    /// position is defined by Git.  In particular, do not split arbitrary
    /// prose on ` in `: branch names and diagnostic prose can contain that
    /// phrase and are not file paths.
    fn conflict_diagnostic_path(description: &str) -> Option<&str> {
        if let Some(path) = description.strip_prefix("Merge conflict in ") {
            return (!path.is_empty()).then_some(path);
        }

        // e.g. "foo.rs deleted in HEAD and modified in topic".
        if let Some((path, _)) = description.split_once(" deleted in ") {
            return (!path.is_empty()).then_some(path);
        }

        // e.g. "foo.rs renamed to bar.rs in HEAD and renamed to baz.rs in topic".
        // The path follows the fixed ` renamed to ` token and ends at the
        // immediately following fixed ` in ` token.
        let (_, renamed) = description.split_once(" renamed to ")?;
        let (path, _) = renamed.split_once(" in ")?;
        (!path.is_empty()).then_some(path)
    }

    /// Return unresolved paths from a merge worktree.  This is authoritative
    /// after `git merge` fails because it reads Git's unmerged index directly.
    fn unresolved_conflicts_in(dir: &Path) -> Vec<ConflictInfo> {
        let output = Command::new("git")
            .args(["diff", "--name-only", "--diff-filter=U", "-z"])
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        let Ok(output) = output else {
            return vec![];
        };
        if !output.status.success() {
            return vec![];
        }
        output
            .stdout
            .split(|byte| *byte == b'\0')
            .filter(|path| !path.is_empty())
            .map(|path| {
                let path = String::from_utf8_lossy(path).into_owned();
                ConflictInfo {
                    path: path.clone(),
                    file_path: path,
                    conflict_type: "content".to_string(),
                    ours: None,
                    theirs: None,
                }
            })
            .collect()
    }

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

impl GitVcsAdapter {
    /// Resolve the absolute path of a feature artifact on disk.
    /// Path: `<repo_root>/docs/agileplus/<feature_slug>/<relative_path>`.
    fn artifact_path(&self, feature_slug: &str, relative_path: &str) -> PathBuf {
        self.repo_root
            .join("docs")
            .join("agileplus")
            .join(feature_slug)
            .join(relative_path)
    }
}


/// Collect file paths recursively from a directory.
pub(crate) fn collect_evidence_paths(dir: &Path, paths: &mut Vec<String>) -> Result<(), DomainError> {
    for entry in std::fs::read_dir(dir)
        .map_err(|e| DomainError::Storage(format!("scan evidence {}: {e}", dir.display())))?
    {
        let entry = entry
            .map_err(|e| DomainError::Storage(format!("scan evidence {}: {e}", dir.display())))?;
        let path = entry.path();
        if path.is_dir() {
            collect_evidence_paths(&path, paths)?;
        } else {
            paths.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

/// Tiny glob matcher that supports `*` and a literal suffix. Used for
/// the `list_branches` `pattern` argument.
pub(crate) fn glob_match(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == name;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut idx = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !name.starts_with(part) {
                return false;
            }
            idx = part.len();
        } else if i == parts.len() - 1 {
            if !name.ends_with(part) {
                return false;
            }
            if idx > name.len() - part.len() {
                return false;
            }
        } else if let Some(found) = name[idx..].find(part) {
            idx += found + part.len();
        } else {
            return false;
        }
    }
    true
}

/// Scan the docs/agileplus directory for all feature slugs.
pub fn scan_all_features(adapter: &GitVcsAdapter) -> Result<Vec<String>, DomainError> {
    let specs_dir = adapter.repo_root.join("docs").join("agileplus");
    if !specs_dir.exists() {
        return Ok(vec![]);
    }
    let mut slugs: Vec<String> = std::fs::read_dir(&specs_dir)
        .map_err(|e| DomainError::Storage(format!("failed to read docs/agileplus dir: {e}")))?
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| e.path().join("meta.json").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    slugs.sort();
    Ok(slugs)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
