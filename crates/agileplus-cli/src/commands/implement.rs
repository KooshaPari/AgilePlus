//! `agileplus implement` command implementation.
//!
//! Orchestrates work package implementation: creates worktrees, dispatches
//! agents, creates PRs, and manages the review-fix loop.
//! Traceability: FR-004, FR-010, FR-011, FR-012 / WP12-T069, T071, T072

#![cfg(feature = "full-deps")]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::agent::{AgentConfig, AgentKind, AgentPort, AgentTask};
use agileplus_domain::ports::{StoragePort, VcsPort};

use super::pr_builder::{build_pr_description, build_pr_title};
use super::review_loop::{ReviewOutcome, run_review_loop};
use super::scheduler::Scheduler;

/// Arguments for the `implement` subcommand.
#[derive(Debug, clap::Args)]
pub struct ImplementArgs {
    /// Feature slug to implement.
    #[arg(long)]
    pub feature: String,

    /// Implement a specific WP only (e.g. WP01 or by numeric ID).
    #[arg(long)]
    pub wp: Option<String>,

    /// Maximum parallel agents.
    #[arg(long, default_value = "3")]
    pub parallel: usize,

    /// Maximum review-fix cycles per WP.
    #[arg(long, default_value = "5")]
    pub max_review_cycles: u32,

    /// Resume from last checkpoint (re-attach to in-progress WPs).
    #[arg(long)]
    pub resume: bool,
}

/// Run the `implement` command.
pub async fn run_implement<S, V, A>(
    args: ImplementArgs,
    storage: &S,
    vcs: &V,
    agent: &A,
) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
    A: AgentPort,
{
    let start = std::time::Instant::now();
    let slug = &args.feature;

    // Look up feature
    let feature = storage
        .get_feature_by_slug(slug)
        .await
        .context("looking up feature")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Feature '{}' not found. Run `agileplus plan --feature {}` first.",
                slug,
                slug
            )
        })?;

    // Validate state: must be Planned or Implementing (resume)
    match feature.state {
        FeatureState::Planned | FeatureState::Implementing => {}
        _ => {
            anyhow::bail!(
                "Feature '{}' is in state '{}'. Expected 'Planned' or 'Implementing'.",
                slug,
                feature.state
            );
        }
    }

    // Transition to Implementing if not already
    if feature.state == FeatureState::Planned {
        storage
            .update_feature_state(feature.id, FeatureState::Implementing)
            .await
            .context("transitioning feature to Implementing")?;

        let prev_hash = get_latest_hash(storage, feature.id).await;
        let mut audit = AuditEntry {
            id: 0,
            feature_id: feature.id,
            wp_id: None,
            timestamp: Utc::now(),
            actor: "user".into(),
            transition: "Planned -> Implementing".into(),
            evidence_refs: vec![],
            prev_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        audit.hash = hash_entry(&audit);
        storage
            .append_audit_entry(&audit)
            .await
            .context("appending audit entry")?;
        println!("Feature '{slug}' transitioned to Implementing.");
    }

    // Load all WPs for this feature
    let all_wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("loading work packages")?;

    if all_wps.is_empty() {
        anyhow::bail!(
            "No work packages found for feature '{}'. Run `agileplus plan --feature {}` first.",
            slug,
            slug
        );
    }

    // Determine which WPs to process
    let target_wps: Vec<&WorkPackage> = if let Some(ref wp_ref) = args.wp {
        // Single WP mode: find by ID prefix or sequence
        let matched: Vec<&WorkPackage> = all_wps
            .iter()
            .filter(|wp| {
                let wp_str = wp_ref.to_uppercase();
                let by_seq = wp_str
                    .strip_prefix("WP")
                    .and_then(|s| s.parse::<i32>().ok())
                    == Some(wp.sequence);
                let by_id = wp_ref.parse::<i64>().ok() == Some(wp.id);
                by_seq || by_id
            })
            .collect();
        if matched.is_empty() {
            anyhow::bail!(
                "Work package '{}' not found for feature '{}'.",
                wp_ref,
                slug
            );
        }
        matched
    } else {
        all_wps.iter().collect()
    };

    // Build scheduler
    let mut wp_states: HashMap<i64, WpState> = HashMap::new();
    for wp in &all_wps {
        wp_states.insert(wp.id, wp.state);
    }

    let mut all_deps = Vec::new();
    for wp in &all_wps {
        let deps = storage
            .get_wp_dependencies(wp.id)
            .await
            .context("loading WP dependencies")?;
        all_deps.extend(deps);
    }

    let scheduler = Scheduler::new(wp_states.clone(), all_deps.clone());

    // Read spec content for PR descriptions
    let spec_content = vcs
        .read_artifact(slug, "spec.md")
        .await
        .unwrap_or_else(|_| String::new());

    // Agent config
    let agent_config = AgentConfig {
        kind: AgentKind::ClaudeCode,
        max_review_cycles: args.max_review_cycles,
        timeout_secs: 3600,
        extra_args: vec![],
    };

    // Process WPs
    let mut completed: HashSet<i64> = all_wps
        .iter()
        .filter(|wp| wp.state == WpState::Done)
        .map(|wp| wp.id)
        .collect();

    let target_ids: HashSet<i64> = target_wps.iter().map(|wp| wp.id).collect();

    if args.resume {
        println!("Resume mode: checking for in-progress WPs...");
    }

    for wp in &target_wps {
        // Skip already done
        if completed.contains(&wp.id) {
            println!(
                "  WP{:02} '{}' already done, skipping.",
                wp.sequence, wp.title
            );
            continue;
        }

        // Check dependencies
        if let Some(blockers) = scheduler.is_blocked(wp.id, &completed) {
            let blocker_ids: Vec<String> = blockers.iter().map(|id| id.to_string()).collect();
            println!(
                "  WP{:02} '{}' is blocked by: [{}]. Skipping.",
                wp.sequence,
                wp.title,
                blocker_ids.join(", ")
            );
            continue;
        }

        println!("Processing WP{:02}: '{}'...", wp.sequence, wp.title);

        // Resume mode: check if worktree already exists
        if args.resume && wp.state == WpState::Doing {
            println!(
                "  Resuming WP{:02} (already in 'doing' state)...",
                wp.sequence
            );
        }

        let wp_id_str = format!("WP{:02}", wp.sequence);
        let existing_worktree = if args.resume && wp.state == WpState::Doing {
            let worktrees = vcs
                .list_worktrees()
                .await
                .context("listing worktrees for resume")?;
            find_resume_worktree(&worktrees, slug, &wp_id_str)
        } else {
            None
        };
        let worktree_path = match existing_worktree {
            Some(path) => {
                println!("  Reusing worktree at: {}", path.display());
                path
            }
            None => {
                let path = vcs
                    .create_worktree(slug, &wp_id_str)
                    .await
                    .context(format!("creating worktree for {wp_id_str}"))?;
                println!("  Worktree created at: {}", path.display());
                path
            }
        };

        // Plan artifacts are stored in the canonical checkout. Materialize the
        // spec, plan, and WP prompt into the new worktree before dispatch so
        // agents have a self-contained, readable context.
        let prompt_relative = format!("tasks/WP{:02}-{}.md", wp.sequence, slugify(&wp.title));
        for relative in ["spec.md", "plan.md", prompt_relative.as_str()] {
            let content = vcs
                .read_artifact(slug, relative)
                .await
                .context(format!("reading artifact {relative}"))?;
            materialize_artifact(
                &worktree_path,
                &format!("kitty-specs/{slug}/{relative}"),
                &content,
            )
            .context(format!("materializing artifact {relative}"))?;
        }

        // Transition WP to Doing
        if wp.state != WpState::Doing {
            storage
                .update_wp_state(wp.id, WpState::Doing)
                .await
                .context("transitioning WP to Doing")?;
        }

        // Build prompt path
        let prompt_path = worktree_path.join(format!("kitty-specs/{slug}/{prompt_relative}"));

        // Build context files
        let context_files = vec![
            worktree_path.join(format!("kitty-specs/{slug}/spec.md")),
            worktree_path.join(format!("kitty-specs/{slug}/plan.md")),
            worktree_path.join(format!("kitty-specs/{slug}/research.md")),
        ];

        let task = AgentTask {
            wp_id: wp_id_str.clone(),
            feature_slug: slug.clone(),
            prompt_path,
            worktree_path: worktree_path.clone(),
            context_files,
        };

        // Dispatch agent asynchronously
        let job_id = agent
            .dispatch_async(task, &agent_config)
            .await
            .context(format!("dispatching agent for {wp_id_str}"))?;

        println!("  Agent dispatched (job: {job_id}).");

        // Build PR description
        let feature_ref = storage
            .get_feature_by_id(feature.id)
            .await?
            .unwrap_or(feature.clone());
        let pr_title = build_pr_title(wp);
        let pr_body = build_pr_description(wp, &feature_ref, &spec_content);

        println!("  PR title: {pr_title}");
        tracing::debug!(pr_body_len = pr_body.len(), "PR description generated");

        // Run review-fix loop
        let outcome = run_review_loop(
            wp,
            &job_id,
            agent,
            &agent_config,
            args.max_review_cycles,
            30,
        )
        .await;

        match outcome {
            ReviewOutcome::Approved => {
                println!("  WP{:02} approved!", wp.sequence);
                storage
                    .update_wp_state(wp.id, WpState::Review)
                    .await
                    .context("transitioning WP to Review")?;
                storage
                    .update_wp_state(wp.id, WpState::Done)
                    .await
                    .context("transitioning WP to Done")?;
                completed.insert(wp.id);

                // Audit
                let prev_hash = get_latest_hash(storage, feature.id).await;
                let mut audit = AuditEntry {
                    id: 0,
                    feature_id: feature.id,
                    wp_id: Some(wp.id),
                    timestamp: Utc::now(),
                    actor: "agent".into(),
                    transition: format!("WP{:02} Planned -> Done", wp.sequence),
                    evidence_refs: vec![],
                    prev_hash,
                    hash: [0u8; 32],
                    event_id: None,
                    archived_to: None,
                };
                audit.hash = hash_entry(&audit);
                storage
                    .append_audit_entry(&audit)
                    .await
                    .context("appending audit entry")?;

                // Cleanup worktree
                if let Err(e) = vcs.cleanup_worktree(&worktree_path).await {
                    tracing::warn!(error = %e, "worktree cleanup failed (non-fatal)");
                }
            }
            ReviewOutcome::MaxCyclesReached {
                cycles,
                last_feedback,
            } => {
                println!(
                    "  WP{:02} reached max review cycles ({cycles}). Marking blocked.",
                    wp.sequence
                );
                storage
                    .update_wp_state(wp.id, WpState::Blocked)
                    .await
                    .context("transitioning WP to Blocked")?;

                let prev_hash = get_latest_hash(storage, feature.id).await;
                let mut audit = AuditEntry {
                    id: 0,
                    feature_id: feature.id,
                    wp_id: Some(wp.id),
                    timestamp: Utc::now(),
                    actor: "system".into(),
                    transition: format!(
                        "WP{:02} Doing -> Blocked (max review cycles)",
                        wp.sequence
                    ),
                    evidence_refs: vec![],
                    prev_hash,
                    hash: [0u8; 32],
                    event_id: None,
                    archived_to: None,
                };
                audit.hash = hash_entry(&audit);
                storage.append_audit_entry(&audit).await.ok();

                eprintln!(
                    "WARNING: WP{:02} blocked after {cycles} review cycles.\nLast feedback: {}",
                    wp.sequence,
                    &last_feedback[..last_feedback.len().min(500)]
                );
            }
            ReviewOutcome::AgentFailed { error } => {
                storage.update_wp_state(wp.id, WpState::Blocked).await.ok();
                anyhow::bail!("Agent failed for WP{:02}: {}", wp.sequence, error);
            }
            ReviewOutcome::Cancelled => {
                storage.update_wp_state(wp.id, WpState::Blocked).await.ok();
                println!("  WP{:02} cancelled.", wp.sequence);
            }
        }
    }

    let elapsed_ms = start.elapsed().as_millis();
    let done_count = target_ids
        .iter()
        .filter(|id| completed.contains(id))
        .count();
    tracing::info!(command = "implement", slug = %slug, done = done_count, elapsed_ms = %elapsed_ms, "implement completed");

    println!();
    println!(
        "Implement complete: {done_count}/{} WPs done.",
        target_ids.len()
    );

    Ok(())
}

fn materialize_artifact(worktree_root: &Path, relative_path: &str, content: &str) -> Result<()> {
    let destination = worktree_root.join(relative_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).context("creating artifact directory")?;
    }
    fs::write(destination, content).context("writing artifact")?;
    Ok(())
}

fn find_resume_worktree(
    worktrees: &[agileplus_domain::ports::WorktreeInfo],
    feature_slug: &str,
    wp_id: &str,
) -> Option<PathBuf> {
    worktrees
        .iter()
        .find(|worktree| {
            (worktree.feature_slug == feature_slug && worktree.wp_id == wp_id)
                || worktree.branch == format!("feat/{feature_slug}/{wp_id}")
        })
        .map(|worktree| worktree.path.clone())
}

fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(40)
        .collect()
}

/// Get the prev_hash for audit chain (last entry's hash, or zeroes if none).
async fn get_latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> [u8; 32] {
    match storage.get_latest_audit_entry(feature_id).await {
        Ok(Some(entry)) => entry.hash,
        _ => [0u8; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // ── slugify ───────────────────────────────────────────────────────────

    #[test]
    fn slugify_basic() {
        assert_eq!(
            slugify("Implement Auth Module (WP01)"),
            "implement-auth-module-wp01"
        );
    }

    #[test]
    fn slugify_already_clean() {
        assert_eq!(slugify("hello"), "hello");
    }

    #[test]
    fn slugify_uppercase_lowered() {
        assert_eq!(slugify("My Feature"), "my-feature");
    }

    #[test]
    fn slugify_consecutive_non_alnum_collapsed() {
        assert_eq!(slugify("a  --  b!!c"), "a-b-c");
    }

    #[test]
    fn slugify_leading_trailing_non_alnum_trimmed() {
        assert_eq!(slugify("--hello--"), "hello");
        assert_eq!(slugify("  spaces  "), "spaces");
    }

    #[test]
    fn slugify_truncated_at_40_chars() {
        let long = "a".repeat(80);
        let result = slugify(&long);
        assert!(result.len() <= 40);
        assert_eq!(result, "a".repeat(40));
    }

    #[test]
    fn slugify_empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn slugify_only_non_alphanumeric() {
        assert_eq!(slugify("---___"), "");
    }

    #[test]
    fn slugify_numbers_preserved() {
        assert_eq!(slugify("WP01 is #1"), "wp01-is-1");
    }

    // ── materialize_artifact ──────────────────────────────────────────────

    #[test]
    fn materializes_artifact_into_worktree_layout() {
        let root = tempfile::tempdir().expect("tempdir");
        materialize_artifact(root.path(), "kitty-specs/demo/tasks/WP01-task.md", "prompt")
            .expect("artifact materialized");
        assert_eq!(
            fs::read_to_string(root.path().join("kitty-specs/demo/tasks/WP01-task.md"))
                .expect("prompt readable"),
            "prompt"
        );
    }

    #[test]
    fn materialize_artifact_creates_parent_dirs() {
        let root = tempfile::tempdir().expect("tempdir");
        materialize_artifact(root.path(), "a/b/c/deep.txt", "content").unwrap();
        assert_eq!(
            fs::read_to_string(root.path().join("a/b/c/deep.txt")).unwrap(),
            "content"
        );
    }

    #[test]
    fn materialize_artifact_overwrites_existing() {
        let root = tempfile::tempdir().expect("tempdir");
        materialize_artifact(root.path(), "file.txt", "old").unwrap();
        materialize_artifact(root.path(), "file.txt", "new").unwrap();
        assert_eq!(
            fs::read_to_string(root.path().join("file.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn materialize_artifact_empty_content() {
        let root = tempfile::tempdir().expect("tempdir");
        materialize_artifact(root.path(), "empty.txt", "").unwrap();
        assert_eq!(
            fs::read_to_string(root.path().join("empty.txt")).unwrap(),
            ""
        );
    }

    // ── find_resume_worktree ──────────────────────────────────────────────

    #[test]
    fn finds_existing_resume_worktree_for_wp() {
        let worktrees = vec![agileplus_domain::ports::WorktreeInfo {
            path: "/tmp/wp01".into(),
            commit: "abc".into(),
            branch: "feat/demo/WP01".into(),
            feature_slug: "demo".into(),
            wp_id: "WP01".into(),
        }];
        assert_eq!(
            find_resume_worktree(&worktrees, "demo", "WP01"),
            Some(PathBuf::from("/tmp/wp01"))
        );
    }

    #[test]
    fn finds_worktree_by_branch_convention() {
        let worktrees = vec![agileplus_domain::ports::WorktreeInfo {
            path: "/tmp/wp02".into(),
            commit: "def".into(),
            branch: "feat/my-feature/WP02".into(),
            feature_slug: "different-slug".into(),
            wp_id: "OTHER".into(),
        }];
        // Matches via branch convention even though slug/wp_id differ
        assert_eq!(
            find_resume_worktree(&worktrees, "my-feature", "WP02"),
            Some(PathBuf::from("/tmp/wp02"))
        );
    }

    #[test]
    fn find_resume_worktree_no_match_returns_none() {
        let worktrees = vec![agileplus_domain::ports::WorktreeInfo {
            path: "/tmp/wp01".into(),
            commit: "abc".into(),
            branch: "feat/demo/WP01".into(),
            feature_slug: "demo".into(),
            wp_id: "WP01".into(),
        }];
        assert_eq!(
            find_resume_worktree(&worktrees, "other-feature", "WP99"),
            None
        );
    }

    #[test]
    fn find_resume_worktree_empty_list_returns_none() {
        assert_eq!(find_resume_worktree(&[], "demo", "WP01"), None);
    }

    #[test]
    fn find_resume_worktree_multiple_pick_correct() {
        let worktrees = vec![
            agileplus_domain::ports::WorktreeInfo {
                path: "/tmp/wp01".into(),
                commit: "a".into(),
                branch: "feat/demo/WP01".into(),
                feature_slug: "demo".into(),
                wp_id: "WP01".into(),
            },
            agileplus_domain::ports::WorktreeInfo {
                path: "/tmp/wp02".into(),
                commit: "b".into(),
                branch: "feat/demo/WP02".into(),
                feature_slug: "demo".into(),
                wp_id: "WP02".into(),
            },
        ];
        assert_eq!(
            find_resume_worktree(&worktrees, "demo", "WP02"),
            Some(PathBuf::from("/tmp/wp02"))
        );
    }

    // ── ImplementArgs defaults ────────────────────────────────────────────

    #[test]
    fn implement_args_defaults() {
        // We can't easily construct ImplementArgs (it uses clap derive),
        // but we can verify the clap default values by parsing.
        use clap::Parser;

        #[derive(Parser)]
        struct Wrapper {
            #[command(subcommand)]
            cmd: Cmd,
        }
        #[derive(clap::Subcommand)]
        enum Cmd {
            Implement(ImplementArgs),
        }

        let args = Wrapper::try_parse_from([
            "test",
            "implement",
            "--feature",
            "auth",
        ])
        .unwrap();
        match args.cmd {
            Cmd::Implement(a) => {
                assert_eq!(a.feature, "auth");
                assert_eq!(a.parallel, 3);
                assert_eq!(a.max_review_cycles, 5);
                assert!(!a.resume);
                assert!(a.wp.is_none());
            }
        }
    }

    #[test]
    fn implement_args_custom_values() {
        use clap::Parser;

        #[derive(Parser)]
        struct Wrapper {
            #[command(subcommand)]
            cmd: Cmd,
        }
        #[derive(clap::Subcommand)]
        enum Cmd {
            Implement(ImplementArgs),
        }

        let args = Wrapper::try_parse_from([
            "test",
            "implement",
            "--feature",
            "payments",
            "--wp",
            "WP01",
            "--parallel",
            "6",
            "--max-review-cycles",
            "10",
            "--resume",
        ])
        .unwrap();
        match args.cmd {
            Cmd::Implement(a) => {
                assert_eq!(a.feature, "payments");
                assert_eq!(a.wp.as_deref(), Some("WP01"));
                assert_eq!(a.parallel, 6);
                assert_eq!(a.max_review_cycles, 10);
                assert!(a.resume);
            }
        }
    }

    // ── get_latest_hash ───────────────────────────────────────────────────

    #[tokio::test]
    async fn get_latest_hash_returns_zeroes_when_no_entries() {
        use agileplus_domain::error::DomainError;
        use async_trait::async_trait;

        struct StubStorage;

        #[async_trait]
        impl agileplus_domain::ports::StoragePort for StubStorage {
            // Minimal stub – only implement the method we need.
            async fn get_latest_audit_entry(
                &self,
                _feature_id: i64,
            ) -> Result<Option<agileplus_domain::domain::audit::AuditEntry>, DomainError> {
                Ok(None)
            }
            // -- required methods, all unimplemented stubs --
            async fn create_feature(&self, _: &agileplus_domain::domain::feature::Feature) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_feature_by_slug(&self, _: &str) -> Result<Option<agileplus_domain::domain::feature::Feature>, DomainError> { unimplemented!() }
            async fn get_feature_by_id(&self, _: i64) -> Result<Option<agileplus_domain::domain::feature::Feature>, DomainError> { unimplemented!() }
            async fn update_feature_state(&self, _: i64, _: agileplus_domain::domain::state_machine::FeatureState) -> Result<(), DomainError> { unimplemented!() }
            async fn list_features_by_state(&self, _: agileplus_domain::domain::state_machine::FeatureState) -> Result<Vec<agileplus_domain::domain::feature::Feature>, DomainError> { unimplemented!() }
            async fn list_all_features(&self) -> Result<Vec<agileplus_domain::domain::feature::Feature>, DomainError> { unimplemented!() }
            async fn create_work_package(&self, _: &agileplus_domain::domain::work_package::WorkPackage) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_work_package(&self, _: i64) -> Result<Option<agileplus_domain::domain::work_package::WorkPackage>, DomainError> { unimplemented!() }
            async fn update_wp_state(&self, _: i64, _: agileplus_domain::domain::work_package::WpState) -> Result<(), DomainError> { unimplemented!() }
            async fn list_wps_by_feature(&self, _: i64) -> Result<Vec<agileplus_domain::domain::work_package::WorkPackage>, DomainError> { unimplemented!() }
            async fn add_wp_dependency(&self, _: &agileplus_domain::domain::work_package::WpDependency) -> Result<(), DomainError> { unimplemented!() }
            async fn get_wp_dependencies(&self, _: i64) -> Result<Vec<agileplus_domain::domain::work_package::WpDependency>, DomainError> { unimplemented!() }
            async fn get_ready_wps(&self, _: i64) -> Result<Vec<agileplus_domain::domain::work_package::WorkPackage>, DomainError> { unimplemented!() }
            async fn append_audit_entry(&self, _: &agileplus_domain::domain::audit::AuditEntry) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_audit_trail(&self, _: i64) -> Result<Vec<agileplus_domain::domain::audit::AuditEntry>, DomainError> { unimplemented!() }
            async fn create_evidence(&self, _: &agileplus_domain::domain::governance::Evidence) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_evidence_by_wp(&self, _: i64) -> Result<Vec<agileplus_domain::domain::governance::Evidence>, DomainError> { unimplemented!() }
            async fn get_evidence_by_fr(&self, _: &str) -> Result<Vec<agileplus_domain::domain::governance::Evidence>, DomainError> { unimplemented!() }
            async fn create_policy_rule(&self, _: &agileplus_domain::domain::governance::PolicyRule) -> Result<i64, DomainError> { unimplemented!() }
            async fn list_active_policies(&self) -> Result<Vec<agileplus_domain::domain::governance::PolicyRule>, DomainError> { unimplemented!() }
            async fn record_metric(&self, _: &agileplus_domain::domain::metric::Metric) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_metrics_by_feature(&self, _: i64) -> Result<Vec<agileplus_domain::domain::metric::Metric>, DomainError> { unimplemented!() }
            async fn create_governance_contract(&self, _: &agileplus_domain::domain::governance::GovernanceContract) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_governance_contract(&self, _: i64, _: i32) -> Result<Option<agileplus_domain::domain::governance::GovernanceContract>, DomainError> { unimplemented!() }
            async fn get_latest_governance_contract(&self, _: i64) -> Result<Option<agileplus_domain::domain::governance::GovernanceContract>, DomainError> { unimplemented!() }
            async fn create_module(&self, _: &agileplus_domain::domain::module::Module) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_module(&self, _: i64) -> Result<Option<agileplus_domain::domain::module::Module>, DomainError> { unimplemented!() }
            async fn get_module_by_slug(&self, _: &str) -> Result<Option<agileplus_domain::domain::module::Module>, DomainError> { unimplemented!() }
            async fn update_module(&self, _: i64, _: &str, _: Option<&str>) -> Result<(), DomainError> { unimplemented!() }
            async fn delete_module(&self, _: i64) -> Result<(), DomainError> { unimplemented!() }
            async fn list_root_modules(&self) -> Result<Vec<agileplus_domain::domain::module::Module>, DomainError> { unimplemented!() }
            async fn list_child_modules(&self, _: i64) -> Result<Vec<agileplus_domain::domain::module::Module>, DomainError> { unimplemented!() }
            async fn get_module_with_features(&self, _: i64) -> Result<Option<agileplus_domain::domain::module::ModuleWithFeatures>, DomainError> { unimplemented!() }
            async fn tag_feature_to_module(&self, _: &agileplus_domain::domain::module::ModuleFeatureTag) -> Result<(), DomainError> { unimplemented!() }
            async fn untag_feature_from_module(&self, _: i64, _: i64) -> Result<(), DomainError> { unimplemented!() }
            async fn create_cycle(&self, _: &agileplus_domain::domain::cycle::Cycle) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_cycle(&self, _: i64) -> Result<Option<agileplus_domain::domain::cycle::Cycle>, DomainError> { unimplemented!() }
            async fn update_cycle_state(&self, _: i64, _: agileplus_domain::domain::cycle::CycleState) -> Result<(), DomainError> { unimplemented!() }
            async fn list_cycles_by_state(&self, _: agileplus_domain::domain::cycle::CycleState) -> Result<Vec<agileplus_domain::domain::cycle::Cycle>, DomainError> { unimplemented!() }
            async fn list_cycles_by_module(&self, _: i64) -> Result<Vec<agileplus_domain::domain::cycle::Cycle>, DomainError> { unimplemented!() }
            async fn list_all_cycles(&self) -> Result<Vec<agileplus_domain::domain::cycle::Cycle>, DomainError> { unimplemented!() }
            async fn get_cycle_with_features(&self, _: i64) -> Result<Option<agileplus_domain::domain::cycle::CycleWithFeatures>, DomainError> { unimplemented!() }
            async fn add_feature_to_cycle(&self, _: &agileplus_domain::domain::cycle::CycleFeature) -> Result<(), DomainError> { unimplemented!() }
            async fn remove_feature_from_cycle(&self, _: i64, _: i64) -> Result<(), DomainError> { unimplemented!() }
            async fn get_sync_mapping(&self, _: &str, _: i64) -> Result<Option<agileplus_domain::domain::sync_mapping::SyncMapping>, DomainError> { unimplemented!() }
            async fn upsert_sync_mapping(&self, _: &agileplus_domain::domain::sync_mapping::SyncMapping) -> Result<(), DomainError> { unimplemented!() }
            async fn get_sync_mapping_by_plane_id(&self, _: &str, _: &str) -> Result<Option<agileplus_domain::domain::sync_mapping::SyncMapping>, DomainError> { unimplemented!() }
            async fn delete_sync_mapping(&self, _: &str, _: i64) -> Result<(), DomainError> { unimplemented!() }
            async fn create_project(&self, _: &agileplus_domain::domain::project::Project) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_project_by_slug(&self, _: &str) -> Result<Option<agileplus_domain::domain::project::Project>, DomainError> { unimplemented!() }
            async fn list_all_projects(&self) -> Result<Vec<agileplus_domain::domain::project::Project>, DomainError> { unimplemented!() }
            async fn create_epic(&self, _: &agileplus_domain::domain::epic::Epic) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_epic(&self, _: i64) -> Result<Option<agileplus_domain::domain::epic::Epic>, DomainError> { unimplemented!() }
            async fn list_epics_by_project(&self, _: i64) -> Result<Vec<agileplus_domain::domain::epic::Epic>, DomainError> { unimplemented!() }
            async fn update_epic_status(&self, _: i64, _: agileplus_domain::domain::epic::EpicStatus) -> Result<(), DomainError> { unimplemented!() }
            async fn create_story(&self, _: &agileplus_domain::domain::story::Story) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_story(&self, _: i64) -> Result<Option<agileplus_domain::domain::story::Story>, DomainError> { unimplemented!() }
            async fn list_stories_by_epic(&self, _: i64) -> Result<Vec<agileplus_domain::domain::story::Story>, DomainError> { unimplemented!() }
            async fn update_story_status(&self, _: i64, _: agileplus_domain::domain::story::StoryStatus) -> Result<(), DomainError> { unimplemented!() }
            async fn create_user(&self, _: &agileplus_domain::domain::user::User) -> Result<i64, DomainError> { unimplemented!() }
            async fn get_user(&self, _: i64) -> Result<Option<agileplus_domain::domain::user::User>, DomainError> { unimplemented!() }
            async fn get_user_by_email(&self, _: &str) -> Result<Option<agileplus_domain::domain::user::User>, DomainError> { unimplemented!() }
            async fn list_all_users(&self) -> Result<Vec<agileplus_domain::domain::user::User>, DomainError> { unimplemented!() }
        }

        let hash = get_latest_hash(&StubStorage, 1).await;
        assert_eq!(hash, [0u8; 32]);
    }
}
