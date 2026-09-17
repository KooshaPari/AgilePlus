//! StoragePort contract: evidence artifacts and feature metrics.
//!
//! Every test uses an isolated in-memory SQLite adapter through the public
//! `StoragePort` trait — no file I/O and no network required.
//!
//! Traceability: WP19-T108 (governance evidence + telemetry surface)

use agileplus_domain::{
    domain::{
        feature::Feature,
        governance::{Evidence, EvidenceType},
        metric::Metric,
        work_package::WorkPackage,
    },
    error::DomainError,
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

fn storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter should initialise")
}

async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str) -> i64 {
    storage
        .create_feature(&Feature::new(slug, slug, [0u8; 32], None))
        .await
        .expect("create_feature should succeed")
}

async fn seed_wp(storage: &SqliteStorageAdapter, slug: &str) -> i64 {
    let feature_id = seed_feature(storage, slug).await;
    storage
        .create_work_package(&WorkPackage::new(feature_id, "evidence-wp", 1, "ac"))
        .await
        .expect("create_work_package should succeed")
}

fn evidence(wp_id: i64, fr_id: &str, evidence_type: EvidenceType) -> Evidence {
    Evidence {
        id: 0,
        wp_id,
        fr_id: fr_id.to_string(),
        evidence_type,
        artifact_path: format!("{fr_id}.log"),
        metadata: None,
        created_at: Utc::now(),
    }
}

fn metric(feature_id: Option<i64>, command: &str, metadata: Option<serde_json::Value>) -> Metric {
    Metric {
        id: 0,
        feature_id,
        command: command.to_string(),
        duration_ms: 1200,
        agent_runs: 2,
        review_cycles: 1,
        metadata,
        timestamp: Utc::now(),
    }
}

// --- Evidence -------------------------------------------------------------

#[tokio::test]
async fn evidence_create_and_get_by_wp() {
    let storage = storage();
    let wp_id = seed_wp(&storage, "evidence-wp").await;

    let id = storage
        .create_evidence(&evidence(wp_id, "FR-1", EvidenceType::TestResult))
        .await
        .expect("create_evidence should succeed");

    let records = storage
        .get_evidence_by_wp(wp_id)
        .await
        .expect("get_evidence_by_wp should succeed");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, id);
    assert_eq!(records[0].evidence_type, EvidenceType::TestResult);
}

#[tokio::test]
async fn evidence_get_by_wp_empty_for_unknown_wp() {
    let storage = storage();
    let records = storage
        .get_evidence_by_wp(98765)
        .await
        .expect("get_evidence_by_wp should succeed");
    assert!(records.is_empty());
}

#[tokio::test]
async fn evidence_get_by_fr_filters_correctly() {
    let storage = storage();
    let wp_id = seed_wp(&storage, "evidence-fr").await;
    for fr in ["FR-A", "FR-B"] {
        storage
            .create_evidence(&evidence(wp_id, fr, EvidenceType::CiOutput))
            .await
            .expect("create_evidence should succeed");
    }

    let matched = storage
        .get_evidence_by_fr("FR-B")
        .await
        .expect("get_evidence_by_fr should succeed");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].artifact_path, "FR-B.log");
}

#[tokio::test]
async fn evidence_metadata_roundtrips() {
    let storage = storage();
    let wp_id = seed_wp(&storage, "evidence-meta").await;
    let mut record = evidence(wp_id, "FR-META", EvidenceType::SecurityScan);
    record.metadata = Some(serde_json::json!({ "scanner": "semgrep", "findings": 0 }));

    storage
        .create_evidence(&record)
        .await
        .expect("create_evidence should succeed");

    let records = storage
        .get_evidence_by_wp(wp_id)
        .await
        .expect("get_evidence_by_wp should succeed");
    assert_eq!(records[0].metadata.as_ref().unwrap()["scanner"], "semgrep");
}

#[tokio::test]
async fn evidence_requires_existing_work_package() {
    let storage = storage();
    let result = storage
        .create_evidence(&evidence(31337, "FR-ORPHAN", EvidenceType::LintResult))
        .await;
    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "evidence must reference a real work package"
    );
}

#[tokio::test]
async fn evidence_types_all_roundtrip() {
    let storage = storage();
    let wp_id = seed_wp(&storage, "evidence-types").await;
    let types = [
        EvidenceType::TestResult,
        EvidenceType::CiOutput,
        EvidenceType::ReviewApproval,
        EvidenceType::SecurityScan,
        EvidenceType::LintResult,
        EvidenceType::ManualAttestation,
    ];
    for (index, evidence_type) in types.into_iter().enumerate() {
        storage
            .create_evidence(&evidence(wp_id, &format!("FR-{index}"), evidence_type))
            .await
            .expect("create_evidence should succeed");
    }

    let records = storage
        .get_evidence_by_wp(wp_id)
        .await
        .expect("get_evidence_by_wp should succeed");
    assert_eq!(records.len(), types.len());
    for evidence_type in types {
        assert!(
            records.iter().any(|r| r.evidence_type == evidence_type),
            "missing evidence type {evidence_type:?}"
        );
    }
}

// --- Metrics --------------------------------------------------------------

#[tokio::test]
async fn metric_record_and_get_by_feature() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "metric-feature").await;

    let id = storage
        .record_metric(&metric(Some(feature_id), "cargo test", None))
        .await
        .expect("record_metric should succeed");
    assert!(id > 0);

    let metrics = storage
        .get_metrics_by_feature(feature_id)
        .await
        .expect("get_metrics_by_feature should succeed");
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].command, "cargo test");
    assert_eq!(metrics[0].duration_ms, 1200);
}

#[tokio::test]
async fn metric_metadata_roundtrips() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "metric-meta").await;
    storage
        .record_metric(&metric(
            Some(feature_id),
            "cargo clippy",
            Some(serde_json::json!({ "warnings": 0 })),
        ))
        .await
        .expect("record_metric should succeed");

    let metrics = storage
        .get_metrics_by_feature(feature_id)
        .await
        .expect("get_metrics_by_feature should succeed");
    assert_eq!(metrics[0].metadata.as_ref().unwrap()["warnings"], 0);
}

#[tokio::test]
async fn metric_without_feature_is_not_returned_for_any_feature() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "metric-orphan").await;
    storage
        .record_metric(&metric(None, "global", None))
        .await
        .expect("record_metric should succeed");

    let metrics = storage
        .get_metrics_by_feature(feature_id)
        .await
        .expect("get_metrics_by_feature should succeed");
    assert!(metrics.is_empty());
}

#[tokio::test]
async fn metric_records_are_scoped_to_feature() {
    let storage = storage();
    let feature_a = seed_feature(&storage, "metric-scope-a").await;
    let feature_b = seed_feature(&storage, "metric-scope-b").await;
    for command in ["a", "b", "c"] {
        storage
            .record_metric(&metric(Some(feature_a), command, None))
            .await
            .expect("record_metric should succeed");
    }
    storage
        .record_metric(&metric(Some(feature_b), "b-only", None))
        .await
        .expect("record_metric should succeed");

    let metrics = storage
        .get_metrics_by_feature(feature_a)
        .await
        .expect("get_metrics_by_feature should succeed");
    assert_eq!(metrics.len(), 3);
    assert!(metrics.iter().all(|m| m.command != "b-only"));
}

#[tokio::test]
async fn metric_empty_for_unknown_feature() {
    let storage = storage();
    let metrics = storage
        .get_metrics_by_feature(55555)
        .await
        .expect("get_metrics_by_feature should succeed");
    assert!(metrics.is_empty());
}
