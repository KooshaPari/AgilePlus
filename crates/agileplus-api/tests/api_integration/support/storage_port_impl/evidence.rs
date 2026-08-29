use agileplus_domain::domain::governance::Evidence;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn create_evidence(
    _storage: &MockStorage,
    _evidence: &Evidence,
) -> Result<i64, DomainError> {
    Ok(1)
}

pub(crate) async fn get_evidence_by_wp(
    _storage: &MockStorage,
    _wp_id: i64,
) -> Result<Vec<Evidence>, DomainError> {
    Ok(vec![])
}

pub(crate) async fn get_evidence_by_fr(
    _storage: &MockStorage,
    _fr_id: &str,
) -> Result<Vec<Evidence>, DomainError> {
    Ok(vec![])
}
