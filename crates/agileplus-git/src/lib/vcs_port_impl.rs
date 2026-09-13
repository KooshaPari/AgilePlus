use std::path::{Path, PathBuf};

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
};

use super::{GitVcsAdapter, collect_evidence_paths, glob_match};

#[async_trait::async_trait]
impl VcsPort for GitVcsAdapter {
    async fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        let branch = Self::worktree_branch(feature_slug, wp_id);
        let path = self.worktree_path(feature_slug, wp_id);
        let path_str = path.to_string_lossy().into_owned();

        // If the branch already exists, use `git worktree add <path>
        // -B <branch>` to attach the existing branch. Otherwise use
        // `-b <branch>` to create a fresh one.
        let repo = self.open()?;
        let branch_exists = repo.find_branch(&branch, git2::BranchType::Local).is_ok();
        let flag = if branch_exists { "-B" } else { "-b" };

        // Use the CLI for the worktree add â€” the git2 `Repository::worktree`
        // builder can produce a worktree in the wrong state on some
        // libgit2 versions, while `git worktree add` is the canonical,
        // well-tested path.
        self.run_git_status(&["worktree", "add", &path_str, flag, &branch])?;
        std::fs::canonicalize(&path).map_err(|e| {
            DomainError::Storage(format!("canonicalize worktree {}: {e}", path.display()))
        })
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        // `git worktree list --porcelain` prints a sequence of
        // stanzas separated by blank lines:
        //
        //   worktree /path/to/wt
        //   HEAD <sha>
        //   branch refs/heads/<name>
        //
        //   worktree /path/to/wt2
        //   ...
        let raw = self.run_git(&["worktree", "list", "--porcelain"])?;
        if raw.is_empty() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        let mut path: Option<String> = None;
        let mut head: Option<String> = None;
        let mut branch: Option<String> = None;
        for line in raw.lines() {
            if line.is_empty() {
                if let (Some(p), Some(h)) = (path.take(), head.take()) {
                    out.push(WorktreeInfo {
                        path: p.into(),
                        commit: h,
                        branch: branch.take().unwrap_or_default(),
                        feature_slug: String::new(),
                        wp_id: String::new(),
                    });
                } else {
                    path = None;
                    head = None;
                    branch = None;
                }
                continue;
            }
            if let Some(rest) = line.strip_prefix("worktree ") {
                path = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("HEAD ") {
                head = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("branch ") {
                // refs/heads/feat/login -> feat/login
                let stripped = rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string();
                branch = Some(stripped);
            }
        }
        // Flush the last stanza (no trailing blank line).
        if let (Some(p), Some(h)) = (path, head) {
            out.push(WorktreeInfo {
                path: p.into(),
                commit: h,
                branch: branch.unwrap_or_default(),
                feature_slug: String::new(),
                wp_id: String::new(),
            });
        }
        Ok(out)
    }

    async fn cleanup_worktree(&self, worktree_path: &Path) -> Result<(), DomainError> {
        // `--force` ignores uncommitted / unmerged changes in the
        // worktree, which is what we want during teardown of an
        // abandoned / errored worktree.
        self.run_git_status(&[
            "worktree",
            "remove",
            "--force",
            &worktree_path.to_string_lossy(),
        ])?;
        Ok(())
    }

    async fn create_branch(&self, branch: &str, base: &str) -> Result<(), DomainError> {
        let repo = self.open()?;
        let commit = Self::resolve_commit(&repo, base)?;
        // `force = false` so the caller gets a real error if the
        // branch already exists. Use `delete_branch(force=true)` to
        // overwrite.
        repo.branch(branch, &commit, false)
            .map(|_| ())
            .map_err(|e| DomainError::Storage(format!("create branch {branch}: {e}")))
    }

    async fn list_branches(
        &self,
        pattern: Option<&str>,
        remote: bool,
    ) -> Result<Vec<BranchInfo>, DomainError> {
        let repo = self.open()?;
        let filter = if remote {
            git2::BranchType::Remote
        } else {
            git2::BranchType::Local
        };
        let branches = repo
            .branches(Some(filter))
            .map_err(|e| DomainError::Storage(format!("list branches: {e}")))?;
        let pat = pattern.unwrap_or("*");
        let mut out = Vec::new();
        for entry in branches {
            let (branch, _bt) =
                entry.map_err(|e| DomainError::Storage(format!("branch entry: {e}")))?;
            let name = match branch.name() {
                Ok(Some(n)) => n.to_string(),
                _ => continue,
            };
            // Lightweight glob match: support `*` and literal text.
            // (Full glob support would need a regex; for `git branch
            // --list <pattern>` we get away with the `*` case in
            // practice.)
            if !glob_match(pat, &name) {
                continue;
            }
            let commit = branch
                .get()
                .peel_to_commit()
                .map(|c| c.id().to_string())
                .unwrap_or_default();
            out.push(BranchInfo {
                name,
                commit,
                is_remote: remote,
            });
        }
        Ok(out)
    }

    async fn delete_branch(
        &self,
        branch: &str,
        force: bool,
        remote: Option<&str>,
    ) -> Result<(), DomainError> {
        if let Some(remote_name) = remote {
            // Delete the remote branch via push.
            self.run_git_status(&["push", remote_name, &format!(":{branch}")])?;
            Ok(())
        } else {
            let repo = self.open()?;
            let mut b = repo
                .find_branch(branch, git2::BranchType::Local)
                .map_err(|e| DomainError::Storage(format!("find branch {branch}: {e}")))?;
            // `Branch::delete` only takes a force flag in newer
            // libgit2; in 0.20 we just call delete() and rely on the
            // upstream merge-state check. For the "force" case the
            // caller's branch has already been verified to be safe
            // to delete (e.g. it's been merged or --force is
            // explicit).
            let _ = force;
            b.delete()
                .map_err(|e| DomainError::Storage(format!("delete branch {branch}: {e}")))?;
            Ok(())
        }
    }

    async fn checkout_branch(&self, branch: &str) -> Result<(), DomainError> {
        // The CLI is the most reliable checkout path â€” it updates the
        // index, working tree, and HEAD ref in one go, and matches
        // what users see in their terminal.
        self.run_git_status(&["checkout", branch])?;
        Ok(())
    }

    async fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> Result<MergeResult, DomainError> {
        // Prefer an in-place merge when `target` is free to check out in
        // this worktree. When another worktree already has `target`
        // checked out (common: canonical `main` + feature worktree),
        // fall back to a temporary worktree so ship still succeeds.
        match self.run_git(&["checkout", target]) {
            Ok(_) => self.merge_in_dir(&self.repo_root, source, target),
            Err(e) if Self::checkout_blocked_by_other_worktree(&e) => {
                tracing::info!(
                    target = %target,
                    source = %source,
                    "target branch checked out elsewhere; merging via temporary worktree"
                );
                self.merge_via_temp_worktree(source, target)
            }
            Err(e) => Err(e),
        }
    }

    async fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        // `git merge-tree <target> <source>` is the read-only,
        // in-memory 3-way merge that does not touch the index or
        // working tree. Its output shape differs across Git versions;
        // `parse_conflicts` handles structured stage rows, diagnostics,
        // and the historical diff-marker fallback.
        let raw = self.run_git_allow_failure(&["merge-tree", target, source])?;
        Ok(Self::parse_conflicts(&raw))
    }

    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<String, DomainError> {
        let p = self.artifact_path(feature_slug, relative_path);
        std::fs::read_to_string(&p).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DomainError::NotFound(format!(
                    "artifact {relative_path} for feature {feature_slug}"
                ))
            } else {
                DomainError::Storage(format!("read artifact {}: {e}", p.display()))
            }
        })
    }

    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> Result<(), DomainError> {
        let p = self.artifact_path(feature_slug, relative_path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DomainError::Storage(format!(
                    "create artifact parent dir {}: {e}",
                    parent.display()
                ))
            })?;
        }
        std::fs::write(&p, content)
            .map_err(|e| DomainError::Storage(format!("write artifact {}: {e}", p.display())))?;

        let relative_path = p.strip_prefix(&self.repo_root).map_err(|e| {
            DomainError::Storage(format!("resolve artifact path {}: {e}", p.display()))
        })?;
        let repo = self.open()?;
        let mut index = repo
            .index()
            .map_err(|e| DomainError::Storage(format!("open git index: {e}")))?;
        index
            .add_path(relative_path)
            .map_err(|e| DomainError::Storage(format!("stage artifact {}: {e}", p.display())))?;
        index
            .write()
            .map_err(|e| DomainError::Storage(format!("write git index: {e}")))
    }

    async fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<bool, DomainError> {
        Ok(self.artifact_path(feature_slug, relative_path).is_file())
    }

    async fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> Result<FeatureArtifacts, DomainError> {
        // Look for the conventional artifacts at fixed names within
        // `<repo_root>/docs/agileplus/<feature_slug>/`. Unknown files in
        // that directory are collected under `other`.
        let dir = self
            .repo_root
            .join("docs")
            .join("agileplus")
            .join(feature_slug);
        let mut out = FeatureArtifacts {
            spec: None,
            research: None,
            plan: None,
            other: vec![],
            meta_json: None,
            audit_chain: None,
            evidence_paths: vec![],
        };
        if !dir.is_dir() {
            return Ok(out);
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(it) => it,
            Err(e) => {
                return Err(DomainError::Storage(format!(
                    "scan artifacts {}: {e}",
                    dir.display()
                )));
            }
        };
        for entry in entries.flatten() {
            let name = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            match name.as_str() {
                "spec.md" | "spec" => {
                    out.spec = std::fs::read_to_string(&path).ok();
                }
                "research.md" | "research" => {
                    out.research = std::fs::read_to_string(&path).ok();
                }
                "plan.md" | "plan" => {
                    out.plan = std::fs::read_to_string(&path).ok();
                }
                _ => {
                    out.other.push(name);
                }
            }
        }

        let meta_path = dir.join("meta.json");
        if meta_path.is_file() {
            out.meta_json = Some(std::fs::read_to_string(&meta_path).map_err(|e| {
                DomainError::Storage(format!("read artifact {}: {e}", meta_path.display()))
            })?);
        }

        let audit_path = dir.join("audit").join("chain.jsonl");
        if audit_path.is_file() {
            out.audit_chain = Some(std::fs::read_to_string(&audit_path).map_err(|e| {
                DomainError::Storage(format!("read artifact {}: {e}", audit_path.display()))
            })?);
        }

        let evidence_dir = dir.join("evidence");
        if evidence_dir.is_dir() {
            collect_evidence_paths(&evidence_dir, &mut out.evidence_paths)?;
        }
        Ok(out)
    }
}
