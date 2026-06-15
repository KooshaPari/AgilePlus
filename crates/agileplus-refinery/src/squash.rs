//! Squash source branch into target branch.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;

/// Squash helper.
#[derive(Debug, Clone)]
pub struct Squash {
    adapter: GitVcsAdapter,
    repo_root: PathBuf,
}

impl Squash {
    pub fn new(adapter: GitVcsAdapter) -> Self {
        // We clone the path via GitVcsAdapter::new because the field is
        // pub(crate) inside the git crate.  Since GitVcsAdapter::new
        // takes a PathBuf and stores it as-is, we can reconstruct it.
        let repo_root = PathBuf::from("."); // placeholder — we will pass explicitly in real usage
        Self { adapter, repo_root }
    }

    /// Create a squash runner rooted at an explicit path.
    pub fn with_root(adapter: GitVcsAdapter, repo_root: PathBuf) -> Self {
        Self { adapter, repo_root }
    }

    /// Squash `source_branch` into `target_branch`, producing a single
    /// commit with `message`.  Returns the resulting commit SHA.
    ///
    /// Steps:
    /// 1. `git checkout target_branch`
    /// 2. `git merge --squash source_branch`
    /// 3. If conflicts are detected, abort and report.
    /// 4. `git commit -m "squash: <message>"`
    pub async fn run(
        &self,
        source_branch: &str,
        target_branch: &str,
        message: &str,
    ) -> Result<String> {
        // 1. Checkout target.
        self.run_git(&["checkout", target_branch])
            .with_context(|| format!("checkout {target_branch}"))?;

        // 2. Detect conflicts before touching the index via merge --squash.
        //    We use the adapter's conflict detector (read-only).
        let conflicts = self
            .adapter
            .detect_conflicts(source_branch, target_branch)
            .await
            .with_context(|| "conflict detection failed")?;
        if !conflicts.is_empty() {
            let files: Vec<String> = conflicts.iter().map(|c| c.file_path.clone()).collect();
            anyhow::bail!("merge conflicts detected: {}", files.join(", "));
        }

        // 3. Merge --squash (does not create a commit yet).
        let repo_root = self.repo_root.clone();
        let source = source_branch.to_string();
        let output = tokio::task::spawn_blocking(move || {
            Command::new("git")
                .args(["merge", "--squash", &source])
                .current_dir(&repo_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .with_context(|| format!("git merge --squash {source}"))
        })
        .await
        .context("spawn_blocking failed")??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Abort the merge attempt so the tree is clean.
            let _ = self.run_git(&["merge", "--abort"]);
            anyhow::bail!("git merge --squash failed: {}", stderr.trim());
        }

        // 4. Commit the squashed changes.
        let commit_msg = format!("squash: {message}");
        self.run_git(&["commit", "-m", &commit_msg])
            .with_context(|| "commit squashed changes")?;

        // 5. Return the new HEAD.
        let head = self.run_git(&["rev-parse", "HEAD"])?;
        Ok(head)
    }

    fn run_git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("git {} failed to spawn", args.join(" ")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git {} failed: {}", args.join(" "), stderr.trim());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
