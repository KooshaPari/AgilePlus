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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionPlanEntry {
    pub wp_id: i64,
    pub wp_sequence: i32,
    pub wp_label: String,
    pub title: String,
    pub branch: String,
    pub accepted_candidate_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionPlan {
    pub feature_id: i64,
    pub feature_slug: String,
    pub target_branch: String,
    pub entries: Vec<PromotionPlanEntry>,
}

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
