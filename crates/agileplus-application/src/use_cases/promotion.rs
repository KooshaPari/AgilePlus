// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canonical read-only promotion preflight.
//!
//! Promotion has external VCS side effects and cannot be modeled as one SQL
//! transaction. This module centralizes the authority checks that must succeed
//! before any merge and provides an immediate pre-merge drift recheck.

use std::path::PathBuf;

use agileplus_domain::{
    domain::{state_machine::FeatureState, work_package::WpState},
    ports::{ExecutionRecordPort, StoragePort, VcsPort},
};

use crate::{error::AppError, use_cases::acceptance::accepted_candidate_for_wp};

pub use agileplus_domain::domain::promotion::{PromotionPlan, PromotionPlanEntry};

fn validation(message: impl Into<String>) -> AppError {
    agileplus_domain::error::DomainError::Validation(message.into()).into()
}

async fn resolve_branch_commit<V: VcsPort>(vcs: &V, branch: &str) -> Result<String, AppError> {
    for remote in [false, true] {
        let branches = vcs.list_branches(Some(branch), remote).await?;
        if let Some(info) = branches
            .into_iter()
            .find(|info| info.name == branch || info.name.ends_with(&format!("/{branch}")))
        {
            return Ok(info.commit);
        }
    }
    Err(validation(format!(
        "source branch '{branch}' could not be resolved to an exact commit"
    )))
}

/// Build a complete promotion plan without mutating Git or persistence.
pub async fn prepare_promotion<S, V>(
    storage: &S,
    vcs: &V,
    feature_slug: &str,
    target_override: Option<&str>,
) -> Result<PromotionPlan, AppError>
where
    S: StoragePort + ExecutionRecordPort,
    V: VcsPort,
{
    let feature = storage
        .get_feature_by_slug(feature_slug)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("feature {feature_slug}")))?;

    if feature.state != FeatureState::Validated {
        return Err(validation(format!(
            "feature '{}' is {:?}; promotion requires Validated",
            feature.slug, feature.state
        )));
    }

    let mut work_packages = storage.list_wps_by_feature(feature.id).await?;
    if work_packages.is_empty() {
        return Err(validation(
            "promotion cannot be vacuous: feature has no work packages",
        ));
    }
    work_packages.sort_by_key(|wp| wp.sequence);
    if let Some(wp) = work_packages.iter().find(|wp| wp.state != WpState::Done) {
        return Err(validation(format!(
            "WP{:02} '{}' is {:?}; promotion requires every work package Done",
            wp.sequence, wp.title, wp.state
        )));
    }

    let active_worktrees = vcs.list_worktrees().await.unwrap_or_default();
    let mut entries = Vec::with_capacity(work_packages.len());
    for wp in &work_packages {
        let accepted = accepted_candidate_for_wp(storage, wp).await?;
        let accepted_path = accepted.worktree_path.as_ref().map(PathBuf::from);
        let matching_worktree = accepted_path
            .as_ref()
            .and_then(|path| {
                active_worktrees
                    .iter()
                    .find(|worktree| &worktree.path == path)
            })
            .or_else(|| {
                active_worktrees.iter().find(|worktree| {
                    worktree.feature_slug == feature_slug
                        && worktree.wp_id == format!("WP{:02}", wp.sequence)
                })
            });
        let branch = matching_worktree
            .map(|worktree| worktree.branch.clone())
            .unwrap_or_else(|| format!("feat/{feature_slug}/WP{:02}", wp.sequence));
        let current_commit = match matching_worktree {
            Some(worktree) => worktree.commit.clone(),
            None => resolve_branch_commit(vcs, &branch).await?,
        };
        let current_candidate = format!("git:{current_commit}");
        if current_candidate != accepted.candidate_ref {
            return Err(validation(format!(
                "WP{:02} source '{}' drifted from accepted candidate: current {}, accepted {}",
                wp.sequence, branch, current_candidate, accepted.candidate_ref
            )));
        }
        entries.push(PromotionPlanEntry {
            wp_id: wp.id,
            wp_sequence: wp.sequence,
            wp_label: format!("WP{:02}", wp.sequence),
            title: wp.title.clone(),
            branch,
            accepted_candidate_ref: accepted.candidate_ref,
        });
    }

    Ok(PromotionPlan {
        feature_id: feature.id,
        feature_slug: feature.slug,
        target_branch: target_override
            .map(ToOwned::to_owned)
            .unwrap_or(feature.target_branch),
        entries,
    })
}

/// Re-resolve the exact source immediately before merge.
///
/// Returns the bare Git commit only when the source still matches the accepted
/// candidate. This narrows the preflight-to-merge drift window.
pub async fn verify_promotion_source<V: VcsPort>(
    vcs: &V,
    entry: &PromotionPlanEntry,
) -> Result<String, AppError> {
    let current_commit = if let Ok(worktrees) = vcs.list_worktrees().await {
        if let Some(worktree) = worktrees
            .iter()
            .find(|worktree| worktree.branch == entry.branch)
        {
            worktree.commit.clone()
        } else {
            resolve_branch_commit(vcs, &entry.branch).await?
        }
    } else {
        resolve_branch_commit(vcs, &entry.branch).await?
    };
    let current_candidate = format!("git:{current_commit}");
    if current_candidate != entry.accepted_candidate_ref {
        return Err(validation(format!(
            "{} source '{}' changed after preflight: current {}, accepted {}",
            entry.wp_label, entry.branch, current_candidate, entry.accepted_candidate_ref
        )));
    }
    entry
        .accepted_candidate_ref
        .strip_prefix("git:")
        .filter(|commit| !commit.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            validation(format!(
                "{} accepted candidate is not an exact Git commit",
                entry.wp_label
            ))
        })
}

/// Execute or resume one feature's durable promotion. Git and SQLite never
/// pretend to share a transaction: each immutable Git result is journaled before
/// publication, and terminal persistence is a separate atomic commit.
pub async fn promote_feature<S, V>(
    storage: &S,
    vcs: &V,
    slug: &str,
    target: Option<&str>,
) -> Result<agileplus_domain::domain::promotion::PromotionJournal, AppError>
where
    S: StoragePort
        + ExecutionRecordPort
        + agileplus_domain::ports::execution::AtomicAcceptancePort
        + agileplus_domain::ports::promotion::PromotionPort,
    V: VcsPort + agileplus_domain::ports::promotion::PromotionVcsPort,
{
    use agileplus_domain::domain::promotion::{PromotionJournal, PromotionStep};
    let feature = storage
        .get_feature_by_slug(slug)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("feature {slug}")))?;
    let mut journal = match storage.get_promotion(feature.id).await? {
        Some(prior) => {
            if target.is_some_and(|t| t != prior.plan.target_branch) {
                return Err(validation(
                    "target override differs from durable promotion plan",
                ));
            }
            prior
        }
        None => {
            let plan = prepare_promotion(storage, vcs, slug, target).await?;
            let receipt = storage
                .get_feature_acceptance_receipt(feature.id)
                .await?
                .ok_or_else(|| {
                    validation("durable acceptance receipt is required for promotion")
                })?;
            let initial_target = vcs.promotion_target(&plan.target_branch).await?;
            storage
                .begin_promotion(&PromotionJournal {
                    plan,
                    acceptance_request_id: receipt.request_id,
                    initial_target,
                    steps: Vec::new(),
                    receipt: None,
                    cleanup_complete: false,
                })
                .await?
        }
    };
    if journal.receipt.is_none() {
        if feature.state != FeatureState::Validated {
            return Err(validation("promotion requires Validated"));
        }
        for (index, entry) in journal.plan.entries.iter().enumerate() {
            if let Some(step) = journal.steps.get(index) {
                if step.confirmed {
                    continue;
                }
            } else {
                let expected = journal
                    .steps
                    .last()
                    .map(|s| s.resulting_commit.clone())
                    .unwrap_or_else(|| journal.initial_target.clone());
                if vcs.promotion_target(&journal.plan.target_branch).await? != expected {
                    return Err(validation(
                        "promotion target changed after the recorded plan",
                    ));
                }
                let source = verify_promotion_source(vcs, entry).await?;
                let result = vcs.prepare_promotion_merge(&source, &expected).await?;
                let step = PromotionStep {
                    expected_target: expected,
                    resulting_commit: result,
                    confirmed: false,
                };
                storage
                    .prepare_promotion_step(feature.id, index, &step)
                    .await?;
                journal.steps.push(step);
            }
            let step = &journal.steps[index];
            // A lost reply after publication can be recovered even if the source
            // resource was subsequently removed. Otherwise recheck source drift.
            if vcs.promotion_target(&journal.plan.target_branch).await? != step.resulting_commit {
                verify_promotion_source(vcs, entry).await?;
            }
            vcs.publish_promotion_merge(
                &journal.plan.target_branch,
                &step.expected_target,
                &step.resulting_commit,
            )
            .await?;
            storage.confirm_promotion_step(feature.id, index).await?;
            journal.steps[index].confirmed = true;
        }
        let expected = &journal
            .steps
            .last()
            .ok_or_else(|| validation("empty promotion"))?
            .resulting_commit;
        if vcs.promotion_target(&journal.plan.target_branch).await? != *expected {
            return Err(validation(
                "promotion target changed before final persistence",
            ));
        }
        journal.receipt = Some(storage.finalize_promotion(feature.id).await?);
    }
    // Cleanup is recoverable follow-up, never a prerequisite for the terminal
    // transaction, and never performed before it has committed.
    if !journal.cleanup_complete {
        let artifact = serde_json::to_string_pretty(&serde_json::json!({"feature_slug":slug,"state":"shipped","shipped_at":journal.receipt.as_ref().map(|r|r.committed_at.to_rfc3339()),"target_branch":journal.plan.target_branch,"merged_branches":journal.plan.entries.iter().map(|e|e.branch.clone()).collect::<Vec<_>>(),"accepted_candidates":journal.plan.entries.iter().map(|e|serde_json::json!({"wp":e.wp_label,"branch":e.branch,"candidate":e.accepted_candidate_ref})).collect::<Vec<_>>(),"wp_count":journal.plan.entries.len(),"promotion_receipt":journal.receipt})).map_err(|e|validation(e.to_string()))?;
        let mut complete = vcs
            .write_artifact(slug, "meta.json", &artifact)
            .await
            .is_ok();
        match vcs.list_worktrees().await {
            Ok(worktrees) => {
                for wt in worktrees.into_iter().filter(|wt| wt.feature_slug == slug) {
                    if let Err(error) = vcs.cleanup_worktree(&wt.path).await {
                        tracing::warn!(%error,path=?wt.path,"promotion committed; cleanup remains pending");
                        complete = false;
                    }
                }
            }
            Err(error) => {
                tracing::warn!(%error,"promotion committed; worktree cleanup remains pending");
                complete = false;
            }
        }
        if complete {
            storage.complete_promotion_cleanup(feature.id).await?;
            journal.cleanup_complete = true;
        }
    }
    Ok(journal)
}
