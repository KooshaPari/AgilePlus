use agileplus_domain::domain::governance::Evidence;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) async fn create_evidence(
    storage: &MockStorage,
    evidence: &Evidence,
) -> Result<i64, DomainError> {
    let mut records = storage.evidence.lock().expect("evidence lock poisoned");
    let id = records.iter().map(|record| record.id).max().unwrap_or(0) + 1;
    let mut created = evidence.clone();
    created.id = id;
    records.push(created);
    Ok(id)
}

pub(crate) async fn get_evidence_by_wp(
    storage: &MockStorage,
    wp_id: i64,
) -> Result<Vec<Evidence>, DomainError> {
    Ok(storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        .iter()
        .filter(|record| record.wp_id == wp_id)
        .cloned()
        .collect())
}

pub(crate) async fn get_evidence_by_fr(
    storage: &MockStorage,
    fr_id: &str,
) -> Result<Vec<Evidence>, DomainError> {
    Ok(storage
        .evidence
        .lock()
        .expect("evidence lock poisoned")
        .iter()
        .filter(|record| record.fr_id == fr_id)
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use agileplus_domain::domain::governance::{Evidence, EvidenceType};
    use chrono::Utc;

    use super::*;

    #[tokio::test]
    async fn evidence_writes_are_observable_by_wp_and_fr() {
        let storage = MockStorage::default();
        let evidence = Evidence {
            id: 0,
            wp_id: 7,
            fr_id: "FR-007".to_string(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: "target/test.xml".to_string(),
            metadata: None,
            created_at: Utc::now(),
        };

        let id = create_evidence(&storage, &evidence).await.unwrap();

        assert!(id > 0);
        assert_eq!(get_evidence_by_wp(&storage, 7).await.unwrap()[0].id, id);
        assert_eq!(
            get_evidence_by_fr(&storage, "FR-007").await.unwrap()[0].artifact_path,
            "target/test.xml"
        );
    }
}
