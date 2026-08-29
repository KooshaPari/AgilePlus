use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn get_sync_mapping(
    _storage: &MockStorage,
    _entity_type: &str,
    _entity_id: i64,
) -> Result<Option<SyncMapping>, DomainError> {
    Ok(None)
}

pub(crate) async fn upsert_sync_mapping(
    _storage: &MockStorage,
    _mapping: &SyncMapping,
) -> Result<(), DomainError> {
    Ok(())
}

pub(crate) async fn get_sync_mapping_by_plane_id(
    _storage: &MockStorage,
    _entity_type: &str,
    _plane_issue_id: &str,
) -> Result<Option<SyncMapping>, DomainError> {
    Ok(None)
}

pub(crate) async fn delete_sync_mapping(
    _storage: &MockStorage,
    _entity_type: &str,
    _entity_id: i64,
) -> Result<(), DomainError> {
    Ok(())
}
