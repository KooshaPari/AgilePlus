use agileplus_domain::domain::governance::PolicyRule;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn create_policy_rule(
    _storage: &MockStorage,
    _rule: &PolicyRule,
) -> Result<i64, DomainError> {
    Ok(1)
}

pub(crate) async fn list_active_policies(
    _storage: &MockStorage,
) -> Result<Vec<PolicyRule>, DomainError> {
    Ok(vec![])
}
