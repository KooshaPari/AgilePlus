use agileplus_domain::domain::governance::Evidence;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn create_evidence(
    _storage: &MockStorage,
    _evidence: &Evidence,
) -> Result<i64, DomainError> {
    Ok(1)
}

/// Evidence attached to `wp_id`.
///
/// Reads the seeded [`MockStorage::evidence`] store and filters by work-package
/// id, so a test can exercise the governance evaluator's per-work-package
/// evidence lookup. The store is empty by default, which is what every
/// pre-existing suite relies on.
pub(crate) async fn get_evidence_by_wp(
    storage: &MockStorage,
    wp_id: i64,
) -> Result<Vec<Evidence>, DomainError> {
    Ok(storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        .iter()
        .filter(|e| e.wp_id == wp_id)
        .cloned()
        .collect())
}

/// Evidence carrying the functional-requirement id `fr_id`.
///
/// Mirrors [`get_evidence_by_wp`] for the by-FR lookup the governance evaluator
/// performs first. Empty by default.
pub(crate) async fn get_evidence_by_fr(
    storage: &MockStorage,
    fr_id: &str,
) -> Result<Vec<Evidence>, DomainError> {
    Ok(storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        .iter()
        .filter(|e| e.fr_id == fr_id)
        .cloned()
        .collect())
}
