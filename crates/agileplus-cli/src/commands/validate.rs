//! `agileplus validate` command implementation.
//!
//! Checks governance compliance for a feature in Implementing state.
//! Transitions to Validated on success.
//! Traceability: FR-005, FR-018, FR-019 / WP13-T073, T074, T077

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::execution::{
    AttemptStatus, EvaluationResult, aggregate_evidence_refs, reduce_criterion_results,
    validate_criterion_receipt,
};
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
    let accepted_wps = require_exact_candidate_acceptance(storage, feature.id)
        .await
        .context("checking exact-candidate work acceptance")?;

    for wp_id in accepted_wps {
        let wp = storage
            .get_work_package(wp_id)
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
                .with_context(|| format!("recording terminal acceptance for WP{:02}", wp.sequence))?;
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

async fn require_exact_candidate_acceptance<S>(storage: &S, feature_id: i64) -> Result<Vec<i64>>
where
    S: StoragePort + ExecutionRecordPort,
{
    let work_packages = storage
        .list_wps_by_feature(feature_id)
        .await
        .context("listing work packages for acceptance")?;
    if work_packages.is_empty() {
        anyhow::bail!(
            "Feature {feature_id} has no work packages; terminal acceptance cannot be vacuous"
        );
    }

    let mut accepted = Vec::with_capacity(work_packages.len());
    for wp in work_packages {
        if !matches!(wp.state, WpState::Review | WpState::Done) {
            anyhow::bail!(
                "WP{:02} '{}' is in state {:?}; exact-candidate acceptance requires Review or Done",
                wp.sequence,
                wp.title,
                wp.state
            );
        }

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

        let criteria = storage
            .list_assignment_criteria(&assignment.id)
            .await
            .with_context(|| format!("loading frozen criteria for WP{:02}", wp.sequence))?;
        if criteria.is_empty() {
            anyhow::bail!(
                "WP{:02} Satisfied Evaluation {} has no frozen Assignment criteria; legacy naked grades are not authoritative",
                wp.sequence,
                evaluation.id
            );
        }
        let criterion_results = storage
            .list_criterion_results(&evaluation.id)
            .await
            .with_context(|| format!("loading criterion results for WP{:02}", wp.sequence))?;
        validate_criterion_receipt(&criteria, &criterion_results)
            .map_err(anyhow::Error::msg)
            .with_context(|| {
                format!(
                    "WP{:02} Evaluation {} has an invalid criterion receipt",
                    wp.sequence, evaluation.id
                )
            })?;
        if reduce_criterion_results(&criteria, &criterion_results) != EvaluationResult::Satisfied {
            anyhow::bail!(
                "WP{:02} Evaluation {} aggregate is inconsistent with its criterion receipt",
                wp.sequence,
                evaluation.id
            );
        }
        if aggregate_evidence_refs(&criterion_results) != evaluation.evidence_refs {
            anyhow::bail!(
                "WP{:02} Evaluation {} evidence union does not match its criterion receipt",
                wp.sequence,
                evaluation.id
            );
        }

        if !evaluation.candidate_ref.starts_with("git:") {
            anyhow::bail!(
                "WP{:02} Satisfied Evaluation {} is not bound to an exact Git candidate: {}",
                wp.sequence,
                evaluation.id,
                evaluation.candidate_ref
            );
        }

        let attempt_id = evaluation.attempt_id.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "WP{:02} Satisfied Evaluation {} is not bound to an Attempt",
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
                    "WP{:02} Evaluation {} references missing Attempt {}",
                    wp.sequence,
                    evaluation.id,
                    attempt_id
                )
            })?;
        if attempt.status != AttemptStatus::Completed {
            anyhow::bail!(
                "WP{:02} evaluated Attempt {} is {:?}, not Completed",
                wp.sequence,
                attempt.id,
                attempt.status
            );
        }
        if attempt.result_candidate_ref.as_deref() != Some(evaluation.candidate_ref.as_str()) {
            anyhow::bail!(
                "WP{:02} candidate mismatch: Attempt {:?}, Evaluation {}",
                wp.sequence,
                attempt.result_candidate_ref,
                evaluation.candidate_ref
            );
        }

        accepted.push(wp.id);
    }

    Ok(accepted)
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
