//! `agileplus ship` command implementation.
//!
//! Merges all WP branches to the target branch, cleans up worktrees,
//! archives the feature, and transitions to Shipped.
//! Traceability: FR-006 / WP13-T075, T077

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::execution::{AttemptStatus, EvaluationResult};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::{ExecutionRecordPort, StoragePort, VcsPort};
use agileplus_events::{EventStore, compute_hash};

/// Arguments for the `ship` subcommand.
#[derive(Debug, clap::Args)]
pub struct ShipArgs {
    /// Feature slug to ship.
    #[arg(long)]
    pub feature: String,

    /// Target branch override (default: feature.target_branch).
    #[arg(long)]
    pub target: Option<String>,

    /// Skip validation check (dangerous).
    #[arg(long)]
    pub skip_validate: bool,

    /// Dry run: show what would be merged without doing it.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `ship` command.
pub async fn run_ship<S, V>(args: ShipArgs, storage: &S, vcs: &V) -> Result<()>
where
    S: StoragePort + EventStore + ExecutionRecordPort,
    V: VcsPort,
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

    // State enforcement. `--skip-validate` is retained for CLI compatibility
    // but cannot authorize a terminal state transition under the mature MACE contract.
    if args.skip_validate {
        anyhow::bail!(
            "--skip-validate is diagnostic-only under the mature contract and cannot ship a feature"
        );
    }
    if feature.state != FeatureState::Validated {
        anyhow::bail!(
            "Feature '{}' is in state '{}'. Expected 'Validated'. \
            Run `agileplus validate --feature {}` first.",
            slug,
            feature.state,
            slug
        );
    }

    // Determine target branch
    let target_branch = args
        .target
        .clone()
        .unwrap_or_else(|| feature.target_branch.clone());
    tracing::debug!(target_branch = %target_branch, "shipping to target branch");

    // Load all WPs for the feature
    let all_wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("listing work packages")?;

    if all_wps.is_empty() && !args.dry_run {
        anyhow::bail!(
            "Feature '{}' has no work packages; shipping cannot be vacuously accepted",
            slug
        );
    }

    // Check all WPs are done
    let incomplete: Vec<_> = all_wps
        .iter()
        .filter(|wp| wp.state != WpState::Done)
        .collect();

    if !incomplete.is_empty() {
        let names: Vec<String> = incomplete
            .iter()
            .map(|wp| {
                format!(
                    "WP{:02} '{}' (state: {:?})",
                    wp.sequence, wp.title, wp.state
                )
            })
            .collect();
        anyhow::bail!(
            "Feature '{}' has incomplete work packages:\n  {}\nFinish all WPs before shipping.",
            slug,
            names.join("\n  ")
        );
    }

    // Collect WPs in sequence order and bind each merge source to the exact
    // independently accepted candidate before any target mutation occurs.
    let mut sorted_wps = all_wps.clone();
    sorted_wps.sort_by_key(|wp| wp.sequence);

    let active_worktrees = vcs.list_worktrees().await.unwrap_or_default();
    let mut wp_branches: Vec<(String, String, String)> = Vec::with_capacity(sorted_wps.len());
    for wp in &sorted_wps {
        let accepted = accepted_candidate_for_wp(storage, wp)
            .await
            .with_context(|| format!("checking accepted candidate for WP{:02}", wp.sequence))?;

        let matching_worktree = accepted
            .worktree_path
            .as_ref()
            .and_then(|path| active_worktrees.iter().find(|wt| &wt.path == path))
            .or_else(|| {
                active_worktrees.iter().find(|wt| {
                    wt.feature_slug == *slug && wt.wp_id == format!("WP{:02}", wp.sequence)
                })
            });

        let branch = matching_worktree
            .map(|wt| wt.branch.clone())
            // The Git adapter's canonical worktree branch is
            // feat/<feature-slug>/<WP-ID>. A filesystem worktree path is an
            // execution resource, not a branch identity, so never derive a
            // source ref from its directory name.
            .unwrap_or_else(|| format!("feat/{slug}/WP{:02}", wp.sequence));

        let current_commit = match matching_worktree {
            Some(worktree) => worktree.commit.clone(),
            None => resolve_branch_commit(vcs, &branch)
                .await
                .with_context(|| format!("resolving source branch {branch}"))?,
        };
        let current_candidate = format!("git:{current_commit}");
        if current_candidate != accepted.candidate_ref {
            anyhow::bail!(
                "WP{:02} source '{}' drifted from accepted candidate: current {}, accepted {}",
                wp.sequence,
                branch,
                current_candidate,
                accepted.candidate_ref
            );
        }

        wp_branches.push((
            format!("WP{:02}", wp.sequence),
            branch,
            accepted.candidate_ref,
        ));
    }

    // Dry-run: just show the merge plan
    if args.dry_run {
        println!("Dry-run: merge plan for feature '{slug}'");
        println!("  Target branch: {target_branch}");
        println!("  WPs to merge ({} total):", sorted_wps.len());
        for (wp, (wp_label, branch, candidate)) in sorted_wps.iter().zip(wp_branches.iter()) {
            println!(
                "    {} '{}' <- branch: {} @ {}",
                wp_label, wp.title, branch, candidate
            );
        }
        if sorted_wps.is_empty() {
            println!("    (no WPs to merge)");
        }
        return Ok(());
    }

    // Perform merges in order
    let mut merged_branches: Vec<String> = Vec::new();
    for (wp, (wp_label, branch, accepted_candidate)) in
        sorted_wps.iter().zip(wp_branches.iter())
    {
        tracing::info!(wp_seq = wp.sequence, branch = %branch, target = %target_branch, accepted_candidate = %accepted_candidate, "merging WP branch");

        // Re-read the source ref immediately before merge to reduce the
        // preflight-to-merge drift window.
        let source_commit = resolve_source_commit(vcs, branch)
            .await
            .with_context(|| format!("rechecking source branch {branch}"))?;
        let source_candidate = format!("git:{source_commit}");
        if &source_candidate != accepted_candidate {
            anyhow::bail!(
                "WP{:02} source '{}' changed after preflight: current {}, accepted {}",
                wp.sequence,
                branch,
                source_candidate,
                accepted_candidate
            );
        }

        let merge_result = vcs
            .merge_to_target(branch, &target_branch)
            .await
            .with_context(|| {
                format!(
                    "merging {} branch '{}' into '{}'; shipping fails closed on merge errors",
                    wp_label, branch, target_branch
                )
            })?;

        if !merge_result.success {
            let conflicts: Vec<String> = merge_result
                .conflicts
                .iter()
                .map(|c| c.path.clone())
                .collect();
            anyhow::bail!(
                "Merge conflict when merging {} branch '{}' into '{}'.\n\
                Conflicting files:\n  {}\n\
                Resolve conflicts manually and re-run `agileplus ship`.",
                wp_label,
                branch,
                target_branch,
                conflicts.join("\n  ")
            );
        }

        merged_branches.push(branch.clone());
        tracing::info!(branch = %branch, commit = ?merge_result.merged_commit, "merged successfully");
    }

    // Clean up worktrees
    let worktrees = vcs.list_worktrees().await.unwrap_or_default();
    for worktree in &worktrees {
        if worktree.feature_slug == *slug {
            match vcs.cleanup_worktree(&worktree.path).await {
                Ok(()) => tracing::info!(path = ?worktree.path, "cleaned up worktree"),
                Err(e) => {
                    tracing::warn!(path = ?worktree.path, error = %e, "failed to clean worktree (skipping)")
                }
            }
        }
    }

    // Write final meta artifact
    let meta = serde_json::json!({
        "feature_slug": slug,
        "state": "shipped",
        "target_branch": target_branch,
        "shipped_at": Utc::now().to_rfc3339(),
        "merged_branches": merged_branches,
        "accepted_candidates": wp_branches.iter().map(|(wp, branch, candidate)| {
            serde_json::json!({"wp": wp, "branch": branch, "candidate": candidate})
        }).collect::<Vec<_>>(),
        "wp_count": all_wps.len(),
    });
    let meta_json = serde_json::to_string_pretty(&meta).unwrap_or_default();
    vcs.write_artifact(slug, "meta.json", &meta_json)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(slug = %slug, error = %e, "failed to write meta.json artifact");
        });

    // Transition feature state to Shipped
    storage
        .update_feature_state(feature.id, FeatureState::Shipped)
        .await
        .context("transitioning feature to Shipped")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, feature.id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: "Validated -> Shipped".into(),
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

    append_feature_transition_event(storage, feature.id, "Validated", "Shipped", "user")
        .await
        .context("appending state transition event")?;

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(command = "ship", slug = %slug, merged = merged_branches.len(), elapsed_ms = %elapsed_ms, "ship completed");

    println!("Feature '{}' shipped.", slug);
    println!("  Target branch: {target_branch}");
    println!("  WPs merged: {}", merged_branches.len());
    println!("  State: Validated -> Shipped");

    Ok(())
}

struct AcceptedCandidate {
    candidate_ref: String,
    worktree_path: Option<std::path::PathBuf>,
}

async fn accepted_candidate_for_wp<S>(
    storage: &S,
    wp: &WorkPackage,
) -> Result<AcceptedCandidate>
where
    S: ExecutionRecordPort,
{
    let assignment = storage
        .get_active_assignment(wp.id)
        .await
        .with_context(|| format!("loading active assignment for WP{:02}", wp.sequence))?
        .ok_or_else(|| anyhow::anyhow!("WP{:02} has no active Assignment", wp.sequence))?;
    let evaluations = storage
        .list_evaluations(&assignment.id)
        .await
        .with_context(|| format!("loading evaluations for WP{:02}", wp.sequence))?;
    let evaluation = evaluations
        .last()
        .ok_or_else(|| anyhow::anyhow!("WP{:02} has no Evaluation", wp.sequence))?;
    if evaluation.result != EvaluationResult::Satisfied {
        anyhow::bail!(
            "WP{:02} latest Evaluation {} is {:?}, not Satisfied",
            wp.sequence,
            evaluation.id,
            evaluation.result
        );
    }
    if !evaluation.candidate_ref.starts_with("git:") {
        anyhow::bail!(
            "WP{:02} accepted Evaluation {} is not bound to an exact Git candidate: {}",
            wp.sequence,
            evaluation.id,
            evaluation.candidate_ref
        );
    }

    let attempt_id = evaluation.attempt_id.as_deref().ok_or_else(|| {
        anyhow::anyhow!(
            "WP{:02} accepted Evaluation {} is not bound to an Attempt",
            wp.sequence,
            evaluation.id
        )
    })?;
    let attempts = storage
        .list_attempts(&assignment.id)
        .await
        .with_context(|| format!("loading attempts for WP{:02}", wp.sequence))?;
    let attempt = attempts
        .iter()
        .find(|attempt| attempt.id == attempt_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "WP{:02} accepted Evaluation {} references missing Attempt {}",
                wp.sequence,
                evaluation.id,
                attempt_id
            )
        })?;
    if attempt.status != AttemptStatus::Completed {
        anyhow::bail!(
            "WP{:02} accepted Attempt {} is {:?}, not Completed",
            wp.sequence,
            attempt.id,
            attempt.status
        );
    }
    if attempt.result_candidate_ref.as_deref() != Some(evaluation.candidate_ref.as_str()) {
        anyhow::bail!(
            "WP{:02} accepted candidate mismatch between Attempt {:?} and Evaluation {}",
            wp.sequence,
            attempt.result_candidate_ref,
            evaluation.candidate_ref
        );
    }

    Ok(AcceptedCandidate {
        candidate_ref: evaluation.candidate_ref.clone(),
        worktree_path: attempt.worktree_path.clone().map(Into::into),
    })
}

async fn resolve_branch_commit<V: VcsPort>(vcs: &V, branch: &str) -> Result<String> {
    for remote in [false, true] {
        let branches = vcs
            .list_branches(Some(branch), remote)
            .await
            .with_context(|| format!("listing {}branches for {branch}", if remote { "remote " } else { "" }))?;
        if let Some(info) = branches
            .into_iter()
            .find(|info| info.name == branch || info.name.ends_with(&format!("/{branch}")))
        {
            return Ok(info.commit);
        }
    }
    anyhow::bail!("source branch '{branch}' could not be resolved to an exact commit")
}

async fn resolve_source_commit<V: VcsPort>(vcs: &V, branch: &str) -> Result<String> {
    if let Ok(worktrees) = vcs.list_worktrees().await {
        if let Some(worktree) = worktrees.iter().find(|worktree| worktree.branch == branch) {
            return Ok(worktree.commit.clone());
        }
    }
    resolve_branch_commit(vcs, branch).await
}

async fn get_latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> [u8; 32] {
    match storage.get_latest_audit_entry(feature_id).await {
        Ok(Some(entry)) => entry.hash,
        _ => [0u8; 32],
    }
}

async fn append_feature_transition_event<S: EventStore>(
    storage: &S,
    feature_id: i64,
    from: &str,
    to: &str,
    actor: &str,
) -> Result<()> {
    let prev_hash = storage
        .get_events("feature", feature_id)
        .await
        .map(|events| events.last().map(|event| event.hash).unwrap_or([0u8; 32]))
        .context("loading prior event chain")?;
    let sequence = storage
        .get_latest_sequence("feature", feature_id)
        .await
        .context("loading latest event sequence")?
        + 1;

    let payload = serde_json::json!({
        "from": from,
        "to": to,
    });

    let mut event = Event::new("feature", feature_id, "state_transitioned", payload, actor);
    event.sequence = sequence;
    event.prev_hash = prev_hash;
    event.hash = compute_hash(
        event.entity_id,
        &event.entity_type,
        &event.event_type,
        &event.payload,
        event.timestamp,
        &event.actor,
        &event.prev_hash,
    )
    .context("computing event hash")?;

    storage.append(&event).await.context("persisting event")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_args_defaults() {
        // Verify struct can be constructed
        let args = ShipArgs {
            feature: "my-feature".to_string(),
            target: None,
            skip_validate: false,
            dry_run: false,
        };
        assert_eq!(args.feature, "my-feature");
        assert!(!args.dry_run);
    }

    #[test]
    fn ship_args_with_target() {
        let args = ShipArgs {
            feature: "feat".to_string(),
            target: Some("release/v1.0".to_string()),
            skip_validate: false,
            dry_run: true,
        };
        assert_eq!(args.target, Some("release/v1.0".to_string()));
        assert!(args.dry_run);
    }

    #[test]
    fn audit_hash_is_nonzero() {
        let mut entry = AuditEntry {
            id: 0,
            feature_id: 42,
            wp_id: None,
            timestamp: Utc::now(),
            actor: "user".into(),
            transition: "Validated -> Shipped".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        entry.hash = hash_entry(&entry);
        assert_ne!(entry.hash, [0u8; 32]);
    }

    #[test]
    fn audit_hash_deterministic() {
        let entry1 = AuditEntry {
            id: 1,
            feature_id: 10,
            wp_id: None,
            timestamp: Utc::now(),
            actor: "user".into(),
            transition: "Created -> Specified".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        let entry2 = AuditEntry {
            id: 1,
            feature_id: 10,
            wp_id: None,
            timestamp: entry1.timestamp,
            actor: "user".into(),
            transition: "Created -> Specified".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        assert_eq!(hash_entry(&entry1), hash_entry(&entry2));
    }

    #[test]
    fn audit_hash_differs_for_different_actors() {
        let base = AuditEntry {
            id: 1,
            feature_id: 10,
            wp_id: None,
            timestamp: Utc::now(),
            actor: "user-a".into(),
            transition: "Created -> Specified".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        let mut other = base.clone();
        other.actor = "user-b".into();
        assert_ne!(hash_entry(&base), hash_entry(&other));
    }

    #[test]
    fn ship_args_dry_run_and_skip_validate() {
        let args = ShipArgs {
            feature: "feat".to_string(),
            target: Some("main".to_string()),
            skip_validate: true,
            dry_run: true,
        };
        assert!(args.skip_validate);
        assert!(args.dry_run);
    }
}
