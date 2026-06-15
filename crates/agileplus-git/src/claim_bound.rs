//! Stub for claim-bound worktree creation.
//!
//! TODO(wtreen): implement full claim-bound worktree lifecycle.

use std::path::PathBuf;

use agileplus_domain::error::DomainError;
use agileplus_triage::claim::Claim;

/// Trait for stores that can look up and update claims.
pub trait ClaimStoreBound {
    fn update_claim_reason(&mut self, claim_id: &str, reason: String);
}

/// Worktree tied to a triage claim.
pub struct ClaimBoundWorktree;

impl ClaimBoundWorktree {
    pub fn create<S: ClaimStoreBound>(
        _repo_root: PathBuf,
        _feature_slug: &str,
        _wp_id: &str,
        _claim: &Claim,
        _claim_store: &mut S,
    ) -> Result<PathBuf, DomainError> {
        // Placeholder — full implementation pending.
        Err(DomainError::NotImplemented)
    }
}
