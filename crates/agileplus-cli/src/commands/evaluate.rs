//! Independent evaluator receipt ingestion.
//!
//! This command does not decide product truth. It records a criterion-complete,
//! exact-candidate AgilePlus work Evaluation produced by an independent grader.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;

use agileplus_domain::domain::execution::{
    CriterionEvaluation, Evaluation, EvaluationResult, aggregate_evidence_refs,
    reduce_criterion_results, validate_criterion_receipt,
};
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::{ExecutionRecordPort, StoragePort};

#[derive(Debug, clap::Args)]
pub struct EvaluateArgs {
    /// Feature slug containing the work package.
    #[arg(long)]
    pub feature: String,

    /// Work package selector: numeric id or WP sequence such as WP01.
    #[arg(long)]
    pub wp: String,

    /// JSON evaluator receipt.
    #[arg(long)]
    pub receipt: PathBuf,
}

#[derive(Debug, Deserialize)]
struct EvaluationReceiptInput {
    candidate_ref: String,
    evaluator_id: String,
    evaluator_version: String,
    criterion_results: Vec<CriterionEvaluation>,
}

pub async fn run_evaluate<S>(args: EvaluateArgs, storage: &S) -> Result<()>
where
    S: StoragePort + ExecutionRecordPort,
{
    let feature = storage
        .get_feature_by_slug(&args.feature)
        .await
        .context("looking up feature")?
        .ok_or_else(|| anyhow::anyhow!("feature '{}' not found", args.feature))?;

    let work_packages = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("listing work packages")?;
    let wp = select_wp(&work_packages, &args.wp)
        .ok_or_else(|| anyhow::anyhow!("work package '{}' not found", args.wp))?;
    if wp.state != WpState::Review {
        anyhow::bail!(
            "WP{:02} is {:?}; independent evaluation requires Review",
            wp.sequence,
            wp.state
        );
    }

    let assignment = storage
        .get_active_assignment(wp.id)
        .await
        .context("loading active assignment")?
        .ok_or_else(|| anyhow::anyhow!("WP{:02} has no active Assignment", wp.sequence))?;
    let criteria = storage
        .list_assignment_criteria(&assignment.id)
        .await
        .context("loading assignment criteria")?;
    if criteria.is_empty() {
        anyhow::bail!(
            "Assignment {} has no frozen criteria; create a revised Assignment before evaluation",
            assignment.id
        );
    }

    let raw = fs::read_to_string(&args.receipt)
        .with_context(|| format!("reading evaluator receipt {}", args.receipt.display()))?;
    let receipt: EvaluationReceiptInput =
        serde_json::from_str(&raw).context("parsing evaluator receipt JSON")?;

    if !receipt.candidate_ref.starts_with("git:") {
        anyhow::bail!(
            "evaluator receipt candidate must be an exact git: reference, got {}",
            receipt.candidate_ref
        );
    }
    if receipt.evaluator_id.trim().is_empty() || receipt.evaluator_version.trim().is_empty() {
        anyhow::bail!("evaluator_id and evaluator_version must be non-empty");
    }

    validate_criterion_receipt(&criteria, &receipt.criterion_results)
        .map_err(anyhow::Error::msg)
        .context("validating criterion receipt")?;

    let attempts = storage
        .list_attempts(&assignment.id)
        .await
        .context("loading assignment attempts")?;
    let attempt = attempts
        .iter()
        .find(|attempt| {
            attempt.status == agileplus_domain::domain::execution::AttemptStatus::Completed
                && attempt.result_candidate_ref.as_deref() == Some(receipt.candidate_ref.as_str())
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no completed Attempt for Assignment {} produced candidate {}",
                assignment.id,
                receipt.candidate_ref
            )
        })?;

    if receipt.evaluator_id == attempt.worker_id {
        anyhow::bail!(
            "evaluator_id '{}' matches the producing worker identity; terminal grading must be independent",
            receipt.evaluator_id
        );
    }

    let result = reduce_criterion_results(&criteria, &receipt.criterion_results);
    let now = Utc::now();
    let evaluation = Evaluation {
        id: format!("evaluation:{}:{}", wp.id, now.timestamp_micros()),
        assignment_id: assignment.id.clone(),
        attempt_id: Some(attempt.id.clone()),
        candidate_ref: receipt.candidate_ref,
        evaluator_id: receipt.evaluator_id,
        evaluator_version: receipt.evaluator_version,
        result,
        evidence_refs: aggregate_evidence_refs(&receipt.criterion_results),
        started_at: now,
        finished_at: now,
    };
    storage
        .create_evaluation_receipt(&evaluation, &receipt.criterion_results)
        .await
        .context("persisting criterion-bound evaluation receipt")?;

    println!(
        "WP{:02} evaluation {} => {:?} for {}",
        wp.sequence, evaluation.id, evaluation.result, evaluation.candidate_ref
    );

    if evaluation.result != EvaluationResult::Satisfied {
        anyhow::bail!(
            "evaluation recorded as {:?}; WP remains in Review",
            evaluation.result
        );
    }
    Ok(())
}

fn select_wp<'a>(work_packages: &'a [WorkPackage], selector: &str) -> Option<&'a WorkPackage> {
    let selector = selector.trim();
    if let Ok(id) = selector.parse::<i64>() {
        return work_packages.iter().find(|wp| wp.id == id);
    }
    let sequence = selector
        .strip_prefix("WP")
        .or_else(|| selector.strip_prefix("wp"))
        .and_then(|value| value.parse::<i32>().ok())?;
    work_packages.iter().find(|wp| wp.sequence == sequence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wp_selector_supports_id_and_sequence() {
        let wp = WorkPackage::new(1, "Test", 3, "criterion");
        let mut wp = wp;
        wp.id = 42;
        let list = vec![wp];
        assert_eq!(select_wp(&list, "42").map(|wp| wp.id), Some(42));
        assert_eq!(select_wp(&list, "WP03").map(|wp| wp.id), Some(42));
        assert!(select_wp(&list, "WP04").is_none());
    }
}
