//! `agileplus ship` command implementation.
//!
//! Merges all WP branches to the target branch, cleans up worktrees,
//! archives the feature, and transitions to Shipped.
//! Traceability: FR-006 / WP13-T075, T077

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_application::use_cases::promotion::{prepare_promotion, verify_promotion_source};
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::state_machine::FeatureState;
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

    if args.skip_validate {
        anyhow::bail!(
            "--skip-validate is diagnostic-only under the mature contract and cannot ship a feature"
        );
    }

    let plan = prepare_promotion(storage, vcs, slug, args.target.as_deref())
        .await
        // The CLI's top-level message must reveal the concrete rejection
        // (missing feature, stale criterion receipt, branch drift, etc.).
        // A generic context-only error makes a fail-closed decision impossible
        // for a human operator to diagnose.
        .map_err(|error| anyhow::anyhow!("preparing exact-candidate promotion: {error}"))?;
    let target_branch = plan.target_branch.clone();

    // Dry-run: just show the merge plan
    if args.dry_run {
        println!("Dry-run: merge plan for feature '{slug}'");
        println!("  Target branch: {target_branch}");
        println!("  WPs to merge ({} total):", plan.entries.len());
        for entry in &plan.entries {
            println!(
                "    {} '{}' <- branch: {} @ {}",
                entry.wp_label, entry.title, entry.branch, entry.accepted_candidate_ref
            );
        }
        return Ok(());
    }

    // Perform merges in order. Recheck each source immediately before merge.
    let mut merged_branches: Vec<String> = Vec::new();
    for entry in &plan.entries {
        tracing::info!(
            wp_seq = entry.wp_sequence,
            branch = %entry.branch,
            target = %target_branch,
            accepted_candidate = %entry.accepted_candidate_ref,
            "merging accepted WP candidate"
        );

        let accepted_commit = verify_promotion_source(vcs, entry)
            .await
            .map_err(anyhow::Error::new)
            .with_context(|| format!("rechecking {}", entry.wp_label))?;
        let merge_result = vcs
            .merge_to_target(&accepted_commit, &target_branch)
            .await
            .with_context(|| {
                format!(
                    "merging {} accepted candidate '{}' (from branch '{}') into '{}'; shipping fails closed on merge errors",
                    entry.wp_label, accepted_commit, entry.branch, target_branch
                )
            })?;

        if !merge_result.success {
            let conflicts: Vec<String> = merge_result
                .conflicts
                .iter()
                .map(|conflict| conflict.path.clone())
                .collect();
            anyhow::bail!(
                "Merge conflict when merging {} accepted candidate '{}' from branch '{}' into '{}'.\n\
                Conflicting files:\n  {}\n\
                Resolve conflicts and re-evaluate the exact candidate before re-running `agileplus ship`.",
                entry.wp_label,
                accepted_commit,
                entry.branch,
                target_branch,
                conflicts.join("\n  ")
            );
        }

        merged_branches.push(entry.branch.clone());
        tracing::info!(branch = %entry.branch, commit = ?merge_result.merged_commit, "merged successfully");
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
        "accepted_candidates": plan.entries.iter().map(|entry| {
            serde_json::json!({
                "wp": entry.wp_label,
                "branch": entry.branch,
                "candidate": entry.accepted_candidate_ref
            })
        }).collect::<Vec<_>>(),
        "wp_count": plan.entries.len(),
    });
    let meta_json = serde_json::to_string_pretty(&meta).unwrap_or_default();
    vcs.write_artifact(slug, "meta.json", &meta_json)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(slug = %slug, error = %e, "failed to write meta.json artifact");
        });

    // Transition feature state to Shipped
    storage
        .update_feature_state(plan.feature_id, FeatureState::Shipped)
        .await
        .context("transitioning feature to Shipped")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, plan.feature_id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: plan.feature_id,
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

    append_feature_transition_event(storage, plan.feature_id, "Validated", "Shipped", "user")
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
