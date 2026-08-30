use agileplus_domain::domain::metric::Metric;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn record_metric(
    _storage: &MockStorage,
    _metric: &Metric,
) -> Result<i64, DomainError> {
    Ok(1)
}

pub(crate) async fn get_metrics_by_feature(
    _storage: &MockStorage,
    _feature_id: i64,
) -> Result<Vec<Metric>, DomainError> {
    Ok(vec![])
}
