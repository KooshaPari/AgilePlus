//! `agileplus ship` command implementation.
//!
//! Merges all WP branches to the target branch, cleans up worktrees,
//! archives the feature, and transitions to Shipped.
//! Traceability: FR-006 / WP13-T075, T077

use anyhow::{Context, Result};
#[cfg(test)]
use chrono::Utc;

use agileplus_application::use_cases::promotion::prepare_promotion;
#[cfg(test)]
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::ports::{ExecutionRecordPort, StoragePort, VcsPort};

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
    S: StoragePort
        + ExecutionRecordPort
        + agileplus_domain::ports::execution::AtomicAcceptancePort
        + agileplus_domain::ports::promotion::PromotionPort,
    V: VcsPort + agileplus_domain::ports::promotion::PromotionVcsPort,
{
    let start = std::time::Instant::now();
    let slug = &args.feature;

    if args.skip_validate {
        anyhow::bail!(
            "--skip-validate is diagnostic-only under the mature contract and cannot ship a feature"
        );
    }

    // Dry-run: just show the merge plan
    if args.dry_run {
        let plan = prepare_promotion(storage, vcs, slug, args.target.as_deref())
            .await
            // The CLI's top-level message must reveal the concrete rejection
            // (missing feature, stale criterion receipt, branch drift, etc.).
            // A generic context-only error makes a fail-closed decision impossible
            // for a human operator to diagnose.
            .map_err(|error| anyhow::anyhow!("preparing exact-candidate promotion: {error}"))?;
        let target_branch = plan.target_branch.clone();

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

    let journal = agileplus_application::use_cases::promotion::promote_feature(
        storage,
        vcs,
        slug,
        args.target.as_deref(),
    )
    .await
    .map_err(|error| anyhow::anyhow!("durable exact-candidate promotion: {error}"))?;
    let receipt = journal
        .receipt
        .context("promotion completed without a receipt")?;
    tracing::info!(command="ship",slug=%slug,elapsed_ms=%start.elapsed().as_millis(),"ship committed");
    println!("Feature '{slug}' shipped.");
    println!(
        "  Target branch: {} @ {}",
        receipt.target_branch, receipt.resulting_commit
    );
    println!("  State: Validated -> Shipped");
    println!(
        "  Cleanup: {}",
        if journal.cleanup_complete {
            "complete"
        } else {
            "pending; rerun ship to retry"
        }
    );
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
