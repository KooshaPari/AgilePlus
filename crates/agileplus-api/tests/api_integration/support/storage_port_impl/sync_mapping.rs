use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn get_sync_mapping(
    storage: &MockStorage,
    entity_type: &str,
    entity_id: i64,
) -> Result<Option<SyncMapping>, DomainError> {
    Ok(storage
        .sync_mappings
        .lock()
        .expect("sync mappings lock poisoned")
        .iter()
        .find(|mapping| mapping.entity_type == entity_type && mapping.entity_id == entity_id)
        .cloned())
}

pub(crate) async fn upsert_sync_mapping(
    storage: &MockStorage,
    mapping: &SyncMapping,
) -> Result<(), DomainError> {
    let mut mappings = storage
        .sync_mappings
        .lock()
        .expect("sync mappings lock poisoned");
    if mappings.iter().any(|existing| {
        existing.plane_issue_id == mapping.plane_issue_id
            && (existing.entity_type != mapping.entity_type
                || existing.entity_id != mapping.entity_id)
    }) {
        return Err(DomainError::Conflict(format!(
            "plane issue {} is already mapped",
            mapping.plane_issue_id
        )));
    }
    if let Some(existing) = mappings.iter_mut().find(|existing| {
        existing.entity_type == mapping.entity_type && existing.entity_id == mapping.entity_id
    }) {
        *existing = mapping.clone();
    } else {
        mappings.push(mapping.clone());
    }
    Ok(())
}

pub(crate) async fn get_sync_mapping_by_plane_id(
    storage: &MockStorage,
    entity_type: &str,
    plane_issue_id: &str,
) -> Result<Option<SyncMapping>, DomainError> {
    Ok(storage
        .sync_mappings
        .lock()
        .expect("sync mappings lock poisoned")
        .iter()
        .find(|mapping| {
            mapping.entity_type == entity_type && mapping.plane_issue_id == plane_issue_id
        })
        .cloned())
}

pub(crate) async fn delete_sync_mapping(
    storage: &MockStorage,
    entity_type: &str,
    entity_id: i64,
) -> Result<(), DomainError> {
    storage
        .sync_mappings
        .lock()
        .expect("sync mappings lock poisoned")
        .retain(|mapping| mapping.entity_type != entity_type || mapping.entity_id != entity_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use agileplus_domain::domain::sync_mapping::SyncMapping;

    use super::*;

    #[tokio::test]
    async fn sync_mapping_upsert_lookup_and_delete_observe_writes() {
        let storage = MockStorage::default();
        let mapping = SyncMapping::new("feature", 7, "plane-77", "hash-1");

        upsert_sync_mapping(&storage, &mapping).await.unwrap();
        assert_eq!(
            get_sync_mapping(&storage, "feature", 7)
                .await
                .unwrap()
                .unwrap()
                .plane_issue_id,
            "plane-77"
        );
        assert_eq!(
            get_sync_mapping_by_plane_id(&storage, "feature", "plane-77")
                .await
                .unwrap()
                .unwrap()
                .entity_id,
            7
        );

        delete_sync_mapping(&storage, "feature", 7).await.unwrap();
        assert!(
            get_sync_mapping(&storage, "feature", 7)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn sync_mapping_rejects_duplicate_plane_issue_id() {
        let storage = MockStorage::default();
        upsert_sync_mapping(
            &storage,
            &SyncMapping::new("feature", 7, "plane-77", "hash-1"),
        )
        .await
        .unwrap();

        let error = upsert_sync_mapping(
            &storage,
            &SyncMapping::new("work-package", 8, "plane-77", "hash-2"),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, DomainError::Conflict(_)));
        assert!(
            get_sync_mapping(&storage, "work-package", 8)
                .await
                .unwrap()
                .is_none()
        );
    }
}
