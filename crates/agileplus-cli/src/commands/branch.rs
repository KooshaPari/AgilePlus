//! `agileplus branch` command group.
//!
//! Provides branch create, checkout, delete, list, and sync operations.

use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

use agileplus_domain::ports::VcsPort;

#[derive(Debug, Clone, Serialize)]
struct BranchInfo {
    name: String,
    is_remote: bool,
}

#[derive(Debug, clap::Args)]
pub struct BranchArgs {
    #[command(subcommand)]
    pub command: BranchCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum BranchCommand {
    /// Create a new branch from a base ref.
    Create {
        /// Branch name to create.
        #[arg(long)]
        name: String,
        /// Base ref to branch from.
        #[arg(long, default_value = "main")]
        base: String,
    },
    /// Check out an existing local branch.
    Checkout {
        /// Branch name to check out.
        #[arg(long)]
        name: String,
    },
    /// Delete a branch locally or remotely.
    Delete {
        /// Branch name to delete.
        #[arg(long)]
        name: String,
        /// Force deletion even if not merged.
        #[arg(long)]
        force: bool,
        /// Remote name to delete from (for example, origin).
        #[arg(long)]
        remote: Option<String>,
    },
    /// List branches, optionally filtering by pattern.
    List {
        /// Shell-style pattern (for example, feat/*).
        #[arg(long)]
        pattern: Option<String>,
        /// List remote branches instead of local branches.
        #[arg(long)]
        remote: bool,
        /// Output format: table (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
    /// Sync one branch into another using the normal merge engine.
    Sync {
        /// Source branch to merge from.
        #[arg(long, default_value = "main")]
        source: String,
        /// Target branch to merge into.
        #[arg(long, default_value = "canary")]
        target: String,
        /// Output format: table (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
}

pub async fn run<V: VcsPort>(args: BranchArgs, vcs: &V) -> Result<()> {
    match args.command {
        BranchCommand::Create { name, base } => {
            vcs.create_branch(&name, &base)
                .await
                .with_context(|| format!("creating branch '{name}' from '{base}'"))?;
            println!("Created branch {name} from {base}");
        }
        BranchCommand::Checkout { name } => {
            vcs.checkout_branch(&name)
                .await
                .with_context(|| format!("checking out branch '{name}'"))?;
            println!("Checked out branch {name}");
        }
        BranchCommand::Delete {
            name,
            force,
            remote,
        } => {
            let remote_str = remote.as_deref().unwrap_or("origin");
            let flag = if force { "-D" } else { "-d" };
            let output = Command::new("git")
                .args(["push", remote_str, &format!(":{flag}"), &name])
                .output()
                .with_context(|| format!("pushing branch deletion to {remote_str}"))?;
            if !output.status.success() {
                anyhow::bail!(
                    "git push {remote_str} :{flag} {name} failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            let local_output = Command::new("git")
                .args(["branch", flag, &name])
                .output()
                .with_context(|| format!("deleting local branch {name}"))?;
            if !local_output.status.success() {
                anyhow::bail!(
                    "git branch {flag} {name} failed: {}",
                    String::from_utf8_lossy(&local_output.stderr)
                );
            }
            println!("Deleted branch {name} (force={force})");
        }
        BranchCommand::List {
            pattern,
            remote,
            output,
        } => {
            let args = if remote {
                vec!["branch", "-r"]
            } else {
                vec!["branch", "-l"]
            };
            let git_out = Command::new("git")
                .args(&args)
                .output()
                .context("running git branch")?;
            if !git_out.status.success() {
                anyhow::bail!(
                    "git branch failed: {}",
                    String::from_utf8_lossy(&git_out.stderr)
                );
            }
            let raw = String::from_utf8_lossy(&git_out.stdout);
            let branches: Vec<BranchInfo> = raw
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|line| {
                    let name = line.trim_start_matches('*').trim().to_string();
                    BranchInfo {
                        name,
                        is_remote: remote,
                    }
                })
                .filter(|b| pattern.as_deref().is_none_or(|p| b.name.contains(p)))
                .collect();
            print_branches(&branches, &output)?;
        }
        BranchCommand::Sync {
            source,
            target,
            output,
        } => {
            let result = vcs
                .merge_to_target(&source, &target)
                .await
                .with_context(|| format!("syncing '{source}' into '{target}'"))?;
            print_sync_result(
                &source,
                &target,
                result.success,
                &output,
                result.merged_commit,
            )?;
        }
    }

    Ok(())
}

fn print_branches(branches: &[BranchInfo], output: &str) -> Result<()> {
    if output == "json" {
        println!("{}", serde_json::to_string_pretty(branches)?);
        return Ok(());
    }

    if branches.is_empty() {
        println!("No branches found");
        return Ok(());
    }

    for branch in branches {
        let remote = if branch.is_remote { "remote" } else { "local" };
        println!("{:<8} {}", remote, branch.name);
    }

    Ok(())
}

fn print_sync_result(
    source: &str,
    target: &str,
    success: bool,
    output: &str,
    merged_commit: Option<String>,
) -> Result<()> {
    if output == "json" {
        let payload = serde_json::json!({
            "source": source,
            "target": target,
            "success": success,
            "merged_commit": merged_commit,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if success {
        if let Some(commit) = merged_commit {
            println!("Synced {source} -> {target} at {commit}");
        } else {
            println!("Synced {source} -> {target}");
        }
    } else {
        println!("Sync {source} -> {target} reported conflicts");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::error::DomainError;
    use agileplus_domain::ports::vcs::{BranchInfo as VcsBranchInfo, FeatureArtifacts, MergeResult};

    // ── Mock VcsPort ────────────────────────────────────────────────────

    struct MockVcs {
        create_should_fail: bool,
        checkout_should_fail: bool,
        merge_conflicts: bool,
    }

    impl Default for MockVcs {
        fn default() -> Self {
            Self {
                create_should_fail: false,
                checkout_should_fail: false,
                merge_conflicts: false,
            }
        }
    }

    #[async_trait::async_trait]
    impl VcsPort for MockVcs {
        async fn create_worktree(
            &self,
            _: &str,
            _: &str,
        ) -> Result<std::path::PathBuf, DomainError> {
            unimplemented!()
        }
        async fn list_worktrees(&self) -> Result<Vec<agileplus_domain::ports::vcs::WorktreeInfo>, DomainError> {
            unimplemented!()
        }
        async fn cleanup_worktree(&self, _: &std::path::Path) -> Result<(), DomainError> {
            unimplemented!()
        }
        async fn create_branch(&self, _: &str, _: &str) -> Result<(), DomainError> {
            if self.create_should_fail {
                Err(DomainError::NotFound("branch failed".into()))
            } else {
                Ok(())
            }
        }
        async fn list_branches(
            &self,
            _: Option<&str>,
            _: bool,
        ) -> Result<Vec<VcsBranchInfo>, DomainError> {
            unimplemented!()
        }
        async fn delete_branch(&self, _: &str, _: bool, _: Option<&str>) -> Result<(), DomainError> {
            unimplemented!()
        }
        async fn checkout_branch(&self, _: &str) -> Result<(), DomainError> {
            if self.checkout_should_fail {
                Err(DomainError::NotFound("checkout failed".into()))
            } else {
                Ok(())
            }
        }
        async fn merge_to_target(&self, _: &str, _: &str) -> Result<MergeResult, DomainError> {
            if self.merge_conflicts {
                Ok(MergeResult {
                    success: false,
                    conflicts: vec![],
                    merged_commit: None,
                    commit: None,
                    message: Some("conflict".into()),
                })
            } else {
                Ok(MergeResult {
                    success: true,
                    conflicts: vec![],
                    merged_commit: Some("abc123".into()),
                    commit: Some("abc123".into()),
                    message: None,
                })
            }
        }
        async fn detect_conflicts(
            &self,
            _: &str,
            _: &str,
        ) -> Result<Vec<agileplus_domain::ports::vcs::ConflictInfo>, DomainError> {
            unimplemented!()
        }
        async fn read_artifact(&self, _: &str, _: &str) -> Result<String, DomainError> {
            unimplemented!()
        }
        async fn write_artifact(&self, _: &str, _: &str, _: &str) -> Result<(), DomainError> {
            unimplemented!()
        }
        async fn artifact_exists(&self, _: &str, _: &str) -> Result<bool, DomainError> {
            unimplemented!()
        }
        async fn scan_feature_artifacts(&self, _: &str) -> Result<FeatureArtifacts, DomainError> {
            unimplemented!()
        }
    }

    // ── print_branches ──────────────────────────────────────────────────

    #[test]
    fn print_branches_json_format() {
        let branches = vec![
            BranchInfo {
                name: "main".into(),
                is_remote: false,
            },
            BranchInfo {
                name: "feat/x".into(),
                is_remote: true,
            },
        ];
        let result = print_branches(&branches, "json");
        assert!(result.is_ok());
    }

    #[test]
    fn print_branches_empty() {
        let branches: Vec<BranchInfo> = vec![];
        let result = print_branches(&branches, "table");
        assert!(result.is_ok());
    }

    #[test]
    fn print_branches_table_with_entries() {
        let branches = vec![BranchInfo {
            name: "main".into(),
            is_remote: false,
        }];
        let result = print_branches(&branches, "table");
        assert!(result.is_ok());
    }

    #[test]
    fn print_branches_json_empty() {
        let branches: Vec<BranchInfo> = vec![];
        let result = print_branches(&branches, "json");
        assert!(result.is_ok());
    }

    // ── print_sync_result ───────────────────────────────────────────────

    #[test]
    fn print_sync_result_success_with_commit() {
        let result = print_sync_result("main", "canary", true, "table", Some("abc123".into()));
        assert!(result.is_ok());
    }

    #[test]
    fn print_sync_result_success_without_commit() {
        let result = print_sync_result("main", "canary", true, "table", None);
        assert!(result.is_ok());
    }

    #[test]
    fn print_sync_result_conflicts() {
        let result = print_sync_result("main", "canary", false, "table", None);
        assert!(result.is_ok());
    }

    #[test]
    fn print_sync_result_json_format() {
        let result = print_sync_result(
            "main",
            "canary",
            true,
            "json",
            Some("abc123".into()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn print_sync_result_json_conflicts() {
        let result = print_sync_result("main", "canary", false, "json", None);
        assert!(result.is_ok());
    }

    // ── BranchInfo serialization ────────────────────────────────────────

    #[test]
    fn branch_info_serialize() {
        let info = BranchInfo {
            name: "main".into(),
            is_remote: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("main"));
        assert!(json.contains("false"));
    }

    #[test]
    fn branch_info_remote_serialize() {
        let info = BranchInfo {
            name: "origin/feat".into(),
            is_remote: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("origin/feat"));
        assert!(json.contains("true"));
    }

    // ── run with mock VcsPort ───────────────────────────────────────────

    #[tokio::test]
    async fn run_create_branch_success() {
        let mock = MockVcs::default();
        let args = BranchArgs {
            command: BranchCommand::Create {
                name: "feat/test".into(),
                base: "main".into(),
            },
        };
        assert!(run(args, &mock).await.is_ok());
    }

    #[tokio::test]
    async fn run_create_branch_failure() {
        let mock = MockVcs {
            create_should_fail: true,
            ..Default::default()
        };
        let args = BranchArgs {
            command: BranchCommand::Create {
                name: "feat/test".into(),
                base: "main".into(),
            },
        };
        assert!(run(args, &mock).await.is_err());
    }

    #[tokio::test]
    async fn run_checkout_branch_success() {
        let mock = MockVcs::default();
        let args = BranchArgs {
            command: BranchCommand::Checkout {
                name: "feat/test".into(),
            },
        };
        assert!(run(args, &mock).await.is_ok());
    }

    #[tokio::test]
    async fn run_checkout_branch_failure() {
        let mock = MockVcs {
            checkout_should_fail: true,
            ..Default::default()
        };
        let args = BranchArgs {
            command: BranchCommand::Checkout {
                name: "feat/test".into(),
            },
        };
        assert!(run(args, &mock).await.is_err());
    }

    #[tokio::test]
    async fn run_sync_success() {
        let mock = MockVcs::default();
        let args = BranchArgs {
            command: BranchCommand::Sync {
                source: "main".into(),
                target: "canary".into(),
                output: "table".into(),
            },
        };
        assert!(run(args, &mock).await.is_ok());
    }

    #[tokio::test]
    async fn run_sync_conflicts() {
        let mock = MockVcs {
            merge_conflicts: true,
            ..Default::default()
        };
        let args = BranchArgs {
            command: BranchCommand::Sync {
                source: "main".into(),
                target: "canary".into(),
                output: "table".into(),
            },
        };
        assert!(run(args, &mock).await.is_ok());
    }

    #[tokio::test]
    async fn run_sync_json_output() {
        let mock = MockVcs::default();
        let args = BranchArgs {
            command: BranchCommand::Sync {
                source: "main".into(),
                target: "canary".into(),
                output: "json".into(),
            },
        };
        assert!(run(args, &mock).await.is_ok());
    }
}
