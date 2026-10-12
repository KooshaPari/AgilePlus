//! `agileplus validate`: diagnostic governance report and atomic work acceptance.
//! Canonical state, audit, event and receipt are committed by the application port.

use agileplus_application::use_cases::accept_feature::{AcceptFeatureCommand, accept_feature};
use agileplus_domain::{
    domain::{
        governance_evaluator::{GovernanceEvaluationOptions, evaluate_governance_with_options},
        state_machine::FeatureState,
    },
    ports::{StoragePort, VcsPort, execution::AtomicAcceptancePort},
};
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::PathBuf;

<<<<<<< HEAD
// Legacy test submodules import these names from their parent. They are not
// production acceptance dependencies and must not restore piecemeal writes.
#[cfg(test)]
#[allow(unused_imports)]
use agileplus_domain::domain::{audit::hash_entry, work_package::WpState};
#[cfg(test)]
=======
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::{StoragePort, VcsPort};
use agileplus_events::{EventStore, compute_hash};

>>>>>>> origin/fix/journey-acceptance-20261008
mod evidence;
mod report;
pub use report::{EvidenceCheck, PolicyEvalResult, ValidationReport};

#[derive(Debug, clap::Args)]
pub struct ValidateArgs {
    #[arg(long)]
    pub feature: String,
    #[arg(long, default_value = "markdown")]
    pub format: String,
    /// Diagnostic evidence-only check; cannot grant terminal acceptance.
    #[arg(long)]
    pub skip_policies: bool,
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Diagnostic state override; cannot grant terminal acceptance.
    #[arg(long)]
    pub force: bool,
}

pub async fn run_validate<S, V>(args: ValidateArgs, storage: &S, vcs: &V) -> Result<()>
where
    S: StoragePort + AtomicAcceptancePort,
    V: VcsPort,
{
    let start = std::time::Instant::now();
    let slug = &args.feature;
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
    let mut governance_exceptions = Vec::new();
    if feature.state != FeatureState::Implementing {
        if args.force {
            let exception = format!(
                "Force flag used: expected state 'Implementing', got '{}' for feature '{}'",
                feature.state, slug
            );
            eprintln!("Warning: {exception}");
            governance_exceptions.push(exception);
        } else {
            anyhow::bail!(
                "Feature '{}' is in state '{}'. Expected 'Implementing'. Run `agileplus implement --feature {}` first, or use --force.",
                slug,
                feature.state,
                slug
            );
        }
    }
<<<<<<< HEAD
    let contract = storage.get_latest_governance_contract(feature.id).await.context("loading governance contract")?
        .ok_or_else(|| anyhow::anyhow!("No governance contract found for feature '{}'. Run `agileplus plan --feature {}` first.", slug, slug))?;
    let options = if args.skip_policies {
        GovernanceEvaluationOptions::evidence_only()
=======

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

    // Evaluate evidence
    let (evidence_results, missing_evidence) =
        evaluate_evidence(storage, &contract, feature.id).await?;

    // Evaluate policies unless explicitly running a diagnostic-only evidence check.
    // Skipping policy evaluation must never authorize a Validated transition.
    let mut policy_results = if args.skip_policies {
        Vec::new()
>>>>>>> origin/fix/journey-acceptance-20261008
    } else {
        GovernanceEvaluationOptions::default()
    };
<<<<<<< HEAD
    let evaluation = evaluate_governance_with_options(storage, &contract, feature.id, options)
        .await
        .context("evaluating governance")?;
    let overall_pass = evaluation.passed(&contract);
=======
    let wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("loading work package states")?;
    let incomplete: Vec<String> = wps
        .iter()
        .filter(|wp| wp.state != WpState::Done)
        .map(|wp| format!("WP{:02}={:?}", wp.sequence, wp.state))
        .collect();
    policy_results.push(PolicyEvalResult {
        policy_id: 0,
        domain: "work_packages".into(),
        passed: !wps.is_empty() && incomplete.is_empty(),
        message: if wps.is_empty() {
            "No work packages found".into()
        } else if incomplete.is_empty() {
            format!("All {} work packages are Done", wps.len())
        } else {
            format!("Incomplete work packages: {}", incomplete.join(", "))
        },
    });

    // Compute overall pass
    let evidence_pass =
        missing_evidence.is_empty() && evidence_results.iter().all(|e| e.found && e.threshold_met);
    let policy_pass = policy_results.iter().all(|p| p.passed);
    let overall_pass = evidence_pass && policy_pass;
    let authoritative_transition = !args.skip_policies && !args.force;

>>>>>>> origin/fix/journey-acceptance-20261008
    let report = ValidationReport {
        feature_slug: slug.clone(),
        timestamp: Utc::now(),
        overall_pass,
        evidence_results: evaluation
            .evidence_results
            .into_iter()
            .map(EvidenceCheck::from)
            .collect(),
        policy_results: evaluation
            .policy_results
            .into_iter()
            .map(PolicyEvalResult::from)
            .collect(),
        missing_evidence: evaluation.missing_evidence,
        governance_exceptions,
    };
    let summary = report.summary();
    println!("Governance preflight: {summary}. This report is not terminal acceptance.");
    let report_content = if args.format == "json" {
        report.to_json()
    } else {
        report.to_markdown()
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
    if args.force || args.skip_policies {
        anyhow::bail!(
            "Validation checks completed, but --force/--skip-policies is diagnostic-only and cannot transition feature '{}' to Validated",
            slug
        );
    }
<<<<<<< HEAD
    let outcome = accept_feature(
        storage,
        &AcceptFeatureCommand {
            request_id: format!("cli:{}", uuid::Uuid::new_v4()),
            feature_id: feature.id,
            actor: "user".into(),
            expected_governance_version: Some(contract.version),
        },
    )
    .await
    .map_err(|error| anyhow::anyhow!("exact-candidate work acceptance failed: {error}"))?;
=======

    // Transition to Validated
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
        transition: format!("{:?} -> Validated", feature.state),
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

    append_feature_transition_event(
        storage,
        feature.id,
        &format!("{:?}", feature.state),
        "Validated",
        "user",
    )
    .await
    .context("appending state transition event")?;

    // Also write report as artifact
>>>>>>> origin/fix/journey-acceptance-20261008
    let report_md = if args.format == "json" {
        report.to_markdown()
    } else {
        report_content
    };
    if let Err(error) = vcs
        .write_artifact(slug, "validation-report.md", &report_md)
        .await
    {
        tracing::warn!(%error, "acceptance committed; report artifact delivery failed");
    }
    let receipt = serde_json::to_string_pretty(&outcome.receipt)?;
    if let Err(error) = vcs
        .write_artifact(slug, "acceptance-receipt.json", &receipt)
        .await
    {
        tracing::warn!(%error, "acceptance committed; receipt artifact delivery failed");
    }
    tracing::info!(command="validate", slug=%slug, elapsed_ms=%start.elapsed().as_millis(), request_id=%outcome.receipt.request_id, "acceptance committed");
    println!("Feature '{}' validated successfully.", slug);
<<<<<<< HEAD
    println!("  State: Implementing -> Validated");
    println!("  Acceptance receipt: {}", outcome.receipt.request_id);
=======
    println!("  State: {:?} -> Validated", feature.state);
    println!("  Report: docs/agileplus/{slug}/validation-report.md");

>>>>>>> origin/fix/journey-acceptance-20261008
    Ok(())
}

// Compatibility for audit-reader unit fixtures only. Read errors fail the test
// rather than inventing a genesis hash; production uses the transactional port.
#[cfg(test)]
#[allow(dead_code)]
async fn get_latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> [u8; 32] {
    storage
        .get_latest_audit_entry(feature_id)
        .await
        .expect("audit fixture read failed")
        .map(|entry| entry.hash)
        .unwrap_or([0; 32])
}

#[cfg(test)]
mod tests;
