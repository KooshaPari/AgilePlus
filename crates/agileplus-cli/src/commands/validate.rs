//! `agileplus validate` command implementation.
//!
//! Checks governance compliance for a feature in Implementing state.
//! Transitions to Validated on success.
//! Traceability: FR-005, FR-018, FR-019 / WP13-T073, T074, T077

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_application::use_cases::acceptance::require_feature_acceptance;
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::governance_evaluator::{
    GovernanceEvaluationOptions, evaluate_governance_with_options,
};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::{ExecutionRecordPort, StoragePort, VcsPort};
use agileplus_events::{EventStore, compute_hash};

#[cfg(test)]
mod evidence;
mod report;

pub use report::{EvidenceCheck, PolicyEvalResult, ValidationReport};

/// Arguments for the `validate` subcommand.
#[derive(Debug, clap::Args)]
pub struct ValidateArgs {
    /// Feature slug to validate.
    #[arg(long)]
    pub feature: String,

    /// Output format for validation report (markdown or json).
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Skip policy rule evaluation (evidence-only check).
    #[arg(long)]
    pub skip_policies: bool,

    /// Write report to file instead of stdout.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Force validation even if not in Implementing state (logs governance exception).
    #[arg(long)]
    pub force: bool,
}

/// Run the `validate` command.
pub async fn run_validate<S, V>(args: ValidateArgs, storage: &S, vcs: &V) -> Result<()>
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

    // State enforcement
    let mut governance_exceptions: Vec<String> = Vec::new();
    if feature.state != FeatureState::Implementing {
        if args.force {
            let exc = format!(
                "Force flag used: expected state 'Implementing', got '{}' for feature '{}'",
                feature.state, slug
            );
            eprintln!("Warning: {exc}");
            governance_exceptions.push(exc);
        } else {
            anyhow::bail!(
                "Feature '{}' is in state '{}'. Expected 'Implementing'. \
                Run `agileplus implement --feature {}` first, or use --force.",
                slug,
                feature.state,
                slug
            );
        }
    }

    // Load governance contract
    let contract = storage
        .get_latest_governance_contract(feature.id)
        .await
        .context("loading governance contract")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No governance contract found for feature '{}'. Run `agileplus plan --feature {}` first.",
                slug, slug
            )
        })?;

    // All production transports must share one governance evaluator.
    // --skip-policies remains diagnostic-only and must never authorize a transition.
    let evaluation_options = if args.skip_policies {
        GovernanceEvaluationOptions::evidence_only()
    } else {
        GovernanceEvaluationOptions::default()
    };
    let evaluation =
        evaluate_governance_with_options(storage, &contract, feature.id, evaluation_options)
            .await
            .context("evaluating governance")?;
    let overall_pass = evaluation.passed(&contract);
    let evidence_results = evaluation
        .evidence_results
        .into_iter()
        .map(EvidenceCheck::from)
        .collect();
    let policy_results = evaluation
        .policy_results
        .into_iter()
        .map(PolicyEvalResult::from)
        .collect();
    let missing_evidence = evaluation.missing_evidence;
    let authoritative_transition = !args.skip_policies && !args.force;

    let report = ValidationReport {
        feature_slug: slug.clone(),
        timestamp: Utc::now(),
        overall_pass,
        evidence_results,
        policy_results,
        missing_evidence,
        governance_exceptions,
    };

    let summary = report.summary();
    println!("Validation summary: {summary}");

    // Format and output the report
    let report_content = match args.format.as_str() {
        "json" => report.to_json(),
        _ => report.to_markdown(),
    };

    if let Some(ref output_path) = args.output {
        std::fs::write(output_path, &report_content)
            .with_context(|| format!("writing report to {}", output_path.display()))?;
        println!("Validation report written to: {}", output_path.display());
    } else {
        print!("{report_content}");
    }

    if !overall_pass {
        anyhow::bail!(
            "Validation FAILED for feature '{}'. Fix the issues above and re-run validate.",
            slug
        );
    }

    if !authoritative_transition {
        anyhow::bail!(
            "Validation checks completed, but --force/--skip-policies is diagnostic-only and cannot transition feature '{}' to Validated",
            slug
        );
    }

    // Correctness and governance are separate acceptance boundaries.
    // Governance passed above; now require each WP to have an independently
    // Satisfied Evaluation bound to the exact completed candidate.
    let accepted = require_feature_acceptance(storage, feature.id)
        .await
        .map_err(anyhow::Error::new)
        .context("checking exact-candidate work acceptance")?;

    for accepted_candidate in accepted {
        let wp = storage
            .get_work_package(accepted_candidate.wp_id)
            .await
            .context("loading accepted work package")?
            .ok_or_else(|| anyhow::anyhow!("accepted work package {wp_id} disappeared"))?;
        if wp.state == WpState::Review {
            storage
                .update_wp_state(wp.id, WpState::Done)
                .await
                .with_context(|| format!("transitioning WP{} from Review to Done", wp.sequence))?;

            let prev_hash = get_latest_hash(storage, feature.id).await;
            let mut wp_audit = AuditEntry {
                id: 0,
                feature_id: feature.id,
                wp_id: Some(wp.id),
                timestamp: Utc::now(),
                actor: "validator".into(),
                transition: format!(
                    "WP{:02} Review -> Done (exact-candidate correctness + governance accepted)",
                    wp.sequence
                ),
                evidence_refs: vec![],
                prev_hash,
                hash: [0u8; 32],
                event_id: None,
                archived_to: None,
            };
            wp_audit.hash = hash_entry(&wp_audit);
            storage
                .append_audit_entry(&wp_audit)
                .await
                .with_context(|| {
                    format!("recording terminal acceptance for WP{:02}", wp.sequence)
                })?;
        }
    }

    // Transition to Validated only after both correctness and governance pass.
    storage
        .update_feature_state(feature.id, FeatureState::Validated)
        .await
        .context("transitioning feature to Validated")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, feature.id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: "Implementing -> Validated".into(),
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

    append_feature_transition_event(storage, feature.id, "Implementing", "Validated", "user")
        .await
        .context("appending state transition event")?;

    // Also write report as artifact
    let report_md = if args.format == "json" {
        report.to_markdown()
    } else {
        report_content.clone()
    };
    vcs.write_artifact(slug, "validation-report.md", &report_md)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(slug = %slug, error = %e, "failed to write validation-report.md artifact");
        });

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(
        command = "validate",
        slug = %slug,
        summary = %summary,
        elapsed_ms = %elapsed_ms,
        "validate completed"
    );

    println!("Feature '{}' validated successfully.", slug);
    println!("  State: Implementing -> Validated");
    println!("  Report: kitty-specs/{slug}/validation-report.md");

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

    tracing::info!(
        entity = "feature",
        feature_id,
        from,
        to,
        sequence,
        hash = ?event.hash,
        "persisted state transition event"
    );

    Ok(())
}

#[cfg(test)]
mod tests;
