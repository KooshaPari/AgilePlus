use agileplus_domain::{
    domain::{
        acceptance::AcceptFeatureCommand,
        execution::{
            Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation, Evaluation,
            EvaluationResult, SpecRevision, snapshot_acceptance_criteria,
        },
        feature::Feature,
        governance::{Evidence, EvidenceType, GovernanceContract, GovernanceRule},
        state_machine::FeatureState,
        work_package::{WorkPackage, WpState},
    },
    ports::{ExecutionRecordPort, StoragePort},
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

pub struct Fixture {
    pub db: SqliteStorageAdapter,
    pub feature_id: i64,
    pub wp_ids: Vec<i64>,
}

pub async fn seed(db: SqliteStorageAdapter) -> Fixture {
    let mut feature = Feature::new("atomic", "Atomic acceptance", [7; 32], None);
    feature.state = FeatureState::Implementing;
    let feature_id = StoragePort::create_feature(&db, &feature).await.unwrap();
    let now = Utc::now();
    let revision = SpecRevision {
        id: "spec:1".into(),
        feature_id,
        content_hash: "sha256:accepted".into(),
        parent_revision_id: None,
        accepted_at: now,
        authority: "owner".into(),
    };
    db.create_spec_revision(&revision).await.unwrap();
    let mut wp_ids = Vec::new();
    for seq in [1, 2] {
        let mut wp = WorkPackage::new(feature_id, "Atomic WP", seq, "result meets the contract");
        wp.state = WpState::Review;
        let wp_id = StoragePort::create_work_package(&db, &wp).await.unwrap();
        wp_ids.push(wp_id);
        StoragePort::create_evidence(
            &db,
            &Evidence {
                id: 0,
                wp_id,
                fr_id: format!("FR-{seq}"),
                evidence_type: EvidenceType::TestResult,
                artifact_path: "artifact:fixture".into(),
                metadata: None,
                created_at: now,
            },
        )
        .await
        .unwrap();
        let assignment = Assignment {
            id: format!("assignment:{seq}"),
            wp_id,
            spec_revision_id: revision.id.clone(),
            created_at: now,
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        let criteria = snapshot_acceptance_criteria("result meets the contract");
        db.create_assignment_with_criteria(&assignment, &criteria)
            .await
            .unwrap();
        let candidate_ref = format!("git:{:040x}", seq);
        let attempt = Attempt {
            id: format!("attempt:{seq}"),
            assignment_id: assignment.id.clone(),
            worker_id: "worker".into(),
            backend: "test".into(),
            job_id: None,
            worktree_path: None,
            base_candidate_ref: Some(format!("git:{:040x}", 0)),
            result_candidate_ref: Some(candidate_ref.clone()),
            status: AttemptStatus::Completed,
            failure_class: None,
            started_at: now,
            ended_at: Some(now),
        };
        db.create_attempt(&attempt).await.unwrap();
        let result = CriterionEvaluation {
            criterion_id: criteria[0].id.clone(),
            result: EvaluationResult::Satisfied,
            evidence_refs: vec![format!("artifact:proof-{seq}")],
            rationale: Some("fixture".into()),
        };
        db.create_evaluation_receipt(
            &Evaluation {
                id: format!("evaluation:{seq}"),
                assignment_id: assignment.id,
                attempt_id: Some(attempt.id),
                candidate_ref,
                evaluator_id: "independent-grader".into(),
                evaluator_version: "1".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: result.evidence_refs.clone(),
                started_at: now,
                finished_at: now,
            },
            &[result],
        )
        .await
        .unwrap();
    }
    StoragePort::create_governance_contract(
        &db,
        &GovernanceContract {
            id: 0,
            feature_id,
            version: 1,
            bound_at: now,
            rules: vec![GovernanceRule {
                transition: "Implementing -> Validated".into(),
                required_evidence: vec!["FR-1:test_result".into(), "FR-2:test_result".into()],
                policy_refs: Vec::new(),
            }],
        },
    )
    .await
    .unwrap();
    Fixture {
        db,
        feature_id,
        wp_ids,
    }
}

impl Fixture {
    pub fn command(&self) -> AcceptFeatureCommand {
        AcceptFeatureCommand {
            request_id: "request:atomic".into(),
            feature_id: self.feature_id,
            actor: "owner".into(),
            expected_governance_version: Some(1),
        }
    }
    pub fn sql(&self, sql: &str) {
        self.db
            .conn_for_bench()
            .unwrap()
            .execute_batch(sql)
            .unwrap();
    }
    pub fn count(&self, table: &str) -> i64 {
        self.db
            .conn_for_bench()
            .unwrap()
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }
    pub async fn assert_unchanged(&self) {
        assert_eq!(
            StoragePort::get_feature_by_id(&self.db, self.feature_id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Implementing
        );
        for wp_id in &self.wp_ids {
            assert_eq!(
                StoragePort::get_work_package(&self.db, *wp_id)
                    .await
                    .unwrap()
                    .unwrap()
                    .state,
                WpState::Review
            );
        }
        for table in ["audit_log", "events", "feature_acceptance_receipts"] {
            assert_eq!(self.count(table), 0, "{table}");
        }
    }
}
