//! AgilePlus SQLite adapter — persistence layer.
//!
//! Implements `StoragePort` using rusqlite with WAL mode and foreign keys.
//! Traceability: WP06

#[path = "lib/content_storage.rs"]
mod content_storage;
pub mod event_store;
pub mod migrations;
pub mod rebuild;
pub mod repository;
pub mod seed;
#[path = "lib/storage_port.rs"]
mod storage_port;
pub mod triage;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use agileplus_domain::error::DomainError;
#[allow(unused_imports)]
use agileplus_domain::ports::StoragePort;

use crate::migrations::MigrationRunner;

/// SQLite-backed storage adapter.
///
/// Uses a single write-serialized connection protected by a Mutex.
/// WAL mode is enabled to allow concurrent reads; all writes are serialized.
pub struct SqliteStorageAdapter {
    conn: Arc<Mutex<Connection>>,
}

pub use triage::SqliteTriageAdapter;

impl SqliteStorageAdapter {
    /// Open a file-backed database, enable WAL + FK pragma, and run all migrations.
    pub fn new(db_path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| DomainError::Storage(format!("failed to open db: {e}")))?;
        Self::configure_and_migrate(conn, true)
    }

    /// Open an in-memory database (for tests).
    pub fn in_memory() -> Result<Self, DomainError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DomainError::Storage(format!("failed to open in-memory db: {e}")))?;
        Self::configure_and_migrate(conn, false)
    }

    fn configure_and_migrate(conn: Connection, enable_wal: bool) -> Result<Self, DomainError> {
        // Enable WAL mode for file-backed databases (not applicable to in-memory).
        // WAL mode doesn't work reliably with in-memory databases, so skip it for tests.
        if enable_wal {
            conn.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| DomainError::Storage(format!("WAL pragma failed: {e}")))?;
        }

        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::Storage(format!("FK pragma failed: {e}")))?;

        // Run migrations
        let runner = MigrationRunner::new(&conn);
        runner.run_all()?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get a locked guard to the connection.
    pub(crate) fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.conn
            .lock()
            .map_err(|e| DomainError::Storage(format!("mutex poisoned: {e}")))
    }

    /// Expose a locked connection guard for benchmarks and test helpers.
    ///
    /// This method is intentionally public so that benchmark crates can access
    /// the underlying rusqlite `Connection` to call repository functions directly
    /// without going through the async `StoragePort` trait.
    pub fn conn_for_bench(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.lock()
    }
}



#[async_trait::async_trait]
impl agileplus_domain::ports::execution::AtomicAcceptancePort for SqliteStorageAdapter {
    async fn accept_feature_atomic(
        &self,
        command: &agileplus_domain::domain::acceptance::AcceptFeatureCommand,
    ) -> Result<agileplus_domain::domain::acceptance::AcceptanceOutcome, DomainError> {
        repository::acceptance::accept_feature_atomic(&mut self.lock()?, command)
    }
}

#[async_trait::async_trait]
impl agileplus_domain::ports::ExecutionRecordPort for SqliteStorageAdapter {
    async fn create_spec_revision(
        &self,
        r: &agileplus_domain::domain::execution::SpecRevision,
    ) -> Result<(), DomainError> {
        let c = self.lock()?;
        repository::execution::create_spec_revision(&c, r)
    }
    async fn create_assignment(
        &self,
        a: &agileplus_domain::domain::execution::Assignment,
    ) -> Result<(), DomainError> {
        let c = self.lock()?;
        repository::execution::create_assignment(&c, a)
    }
    async fn create_assignment_with_criteria(
        &self,
        a: &agileplus_domain::domain::execution::Assignment,
        criteria: &[agileplus_domain::domain::execution::AssignmentCriterion],
    ) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        repository::execution::create_assignment_with_criteria(&mut c, a, criteria)
    }
    async fn get_active_assignment(
        &self,
        wp_id: i64,
    ) -> Result<Option<agileplus_domain::domain::execution::Assignment>, DomainError> {
        let c = self.lock()?;
        repository::execution::get_active_assignment(&c, wp_id)
    }
    async fn supersede_assignment(
        &self,
        previous_assignment_id: &str,
        replacement: &agileplus_domain::domain::execution::Assignment,
    ) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        repository::execution::supersede_assignment(&mut c, previous_assignment_id, replacement)
    }
    async fn supersede_assignment_with_criteria(
        &self,
        previous_assignment_id: &str,
        replacement: &agileplus_domain::domain::execution::Assignment,
        criteria: &[agileplus_domain::domain::execution::AssignmentCriterion],
    ) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        repository::execution::supersede_assignment_with_criteria(
            &mut c,
            previous_assignment_id,
            replacement,
            criteria,
        )
    }
    async fn list_assignment_criteria(
        &self,
        assignment_id: &str,
    ) -> Result<Vec<agileplus_domain::domain::execution::AssignmentCriterion>, DomainError> {
        let c = self.lock()?;
        repository::execution::list_assignment_criteria(&c, assignment_id)
    }
    async fn create_attempt(
        &self,
        a: &agileplus_domain::domain::execution::Attempt,
    ) -> Result<(), DomainError> {
        let c = self.lock()?;
        repository::execution::create_attempt(&c, a)
    }
    async fn update_attempt_runtime(
        &self,
        attempt_id: &str,
        status: agileplus_domain::domain::execution::AttemptStatus,
        job_id: Option<&str>,
        result_candidate_ref: Option<&str>,
        failure_class: Option<&str>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DomainError> {
        let c = self.lock()?;
        repository::execution::update_attempt_runtime(
            &c,
            attempt_id,
            status,
            job_id,
            result_candidate_ref,
            failure_class,
            ended_at,
        )
    }
    async fn create_evaluation(
        &self,
        e: &agileplus_domain::domain::execution::Evaluation,
    ) -> Result<(), DomainError> {
        let c = self.lock()?;
        repository::execution::create_evaluation(&c, e)
    }
    async fn create_evaluation_receipt(
        &self,
        e: &agileplus_domain::domain::execution::Evaluation,
        criterion_results: &[agileplus_domain::domain::execution::CriterionEvaluation],
    ) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        repository::execution::create_evaluation_receipt(&mut c, e, criterion_results)
    }
    async fn list_attempts(
        &self,
        assignment_id: &str,
    ) -> Result<Vec<agileplus_domain::domain::execution::Attempt>, DomainError> {
        let c = self.lock()?;
        repository::execution::list_attempts(&c, assignment_id)
    }
    async fn list_evaluations(
        &self,
        assignment_id: &str,
    ) -> Result<Vec<agileplus_domain::domain::execution::Evaluation>, DomainError> {
        let c = self.lock()?;
        repository::execution::list_evaluations(&c, assignment_id)
    }
    async fn list_criterion_results(
        &self,
        evaluation_id: &str,
    ) -> Result<Vec<agileplus_domain::domain::execution::CriterionEvaluation>, DomainError> {
        let c = self.lock()?;
        repository::execution::list_criterion_results(&c, evaluation_id)
    }
}

#[cfg(test)]
#[path = "lib/tests_persistence.rs"]
mod tests_persistence;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use agileplus_domain::error::DomainError;

    use super::*;

    #[tokio::test]
    async fn poisoned_lock_surfaces_as_storage_error() {
        let adapter = Arc::new(SqliteStorageAdapter::in_memory().expect("in-memory adapter"));
        let worker = Arc::clone(&adapter);

        // Hold the connection guard and panic, poisoning the mutex.
        let joined = std::thread::spawn(move || {
            let _guard = worker.conn_for_bench().expect("guard");
            panic!("poison the connection mutex");
        })
        .join();
        assert!(joined.is_err(), "worker thread must have panicked");

        match adapter.conn_for_bench() {
            Ok(_) => panic!("expected a poisoned lock"),
            Err(err) => assert!(
                matches!(err, DomainError::Storage(_)),
                "expected Storage error, got {err:?}"
            ),
        }

        // The async port surface must report the same failure instead of panicking.
        let err = StoragePort::list_all_features(&*adapter)
            .await
            .expect_err("port call must fail on a poisoned lock");
        assert!(
            matches!(err, DomainError::Storage(_)),
            "expected Storage error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn replacement_attempt_preserves_history_and_exact_candidate_evaluation() {
        use agileplus_domain::{
            domain::{
                execution::{
                    Assignment, AssignmentStatus, Attempt, AttemptStatus, CriterionEvaluation,
                    Evaluation, EvaluationResult, SpecRevision, aggregate_evidence_refs,
                    reduce_criterion_results, snapshot_acceptance_criteria,
                },
                feature::Feature,
                work_package::WorkPackage,
            },
            ports::{ExecutionRecordPort, StoragePort},
        };
        use chrono::{Duration, Utc};

        let db = SqliteStorageAdapter::in_memory().expect("in-memory adapter");
        let feature_id = StoragePort::create_feature(
            &db,
            &Feature::new(
                "replacement-witness",
                "Replacement Witness",
                [7u8; 32],
                None,
            ),
        )
        .await
        .expect("feature");
        let wp_id = StoragePort::create_work_package(
            &db,
            &WorkPackage::new(feature_id, "Replacement WP", 1, "exact candidate"),
        )
        .await
        .expect("wp");

        let t0 = Utc::now();
        let spec = SpecRevision {
            id: "spec:replacement:1".into(),
            feature_id,
            content_hash: "sha256:spec-a".into(),
            parent_revision_id: None,
            accepted_at: t0,
            authority: "test-authority".into(),
        };
        ExecutionRecordPort::create_spec_revision(&db, &spec)
            .await
            .expect("spec revision");

        let assignment = Assignment {
            id: "assignment:replacement:1".into(),
            wp_id,
            spec_revision_id: spec.id.clone(),
            created_at: t0,
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        let criteria = snapshot_acceptance_criteria("exact candidate");
        ExecutionRecordPort::create_assignment_with_criteria(&db, &assignment, &criteria)
            .await
            .expect("assignment");

        let attempt_a = Attempt {
            id: "attempt:a".into(),
            assignment_id: assignment.id.clone(),
            worker_id: "worker-a".into(),
            backend: "test".into(),
            job_id: Some("job-a".into()),
            worktree_path: Some("/tmp/a".into()),
            base_candidate_ref: Some("git:base".into()),
            result_candidate_ref: None,
            status: AttemptStatus::Running,
            failure_class: None,
            started_at: t0,
            ended_at: None,
        };
        ExecutionRecordPort::create_attempt(&db, &attempt_a)
            .await
            .expect("attempt A");
        ExecutionRecordPort::update_attempt_runtime(
            &db,
            &attempt_a.id,
            AttemptStatus::Expired,
            None,
            None,
            Some("lease_expired"),
            Some(t0 + Duration::seconds(10)),
        )
        .await
        .expect("expire attempt A");

        let attempt_b = Attempt {
            id: "attempt:b".into(),
            assignment_id: assignment.id.clone(),
            worker_id: "worker-b".into(),
            backend: "test".into(),
            job_id: Some("job-b".into()),
            worktree_path: Some("/tmp/b".into()),
            base_candidate_ref: Some("git:base".into()),
            result_candidate_ref: None,
            status: AttemptStatus::Running,
            failure_class: None,
            started_at: t0 + Duration::seconds(11),
            ended_at: None,
        };
        ExecutionRecordPort::create_attempt(&db, &attempt_b)
            .await
            .expect("attempt B");
        ExecutionRecordPort::update_attempt_runtime(
            &db,
            &attempt_b.id,
            AttemptStatus::Completed,
            None,
            Some("git:candidate-b"),
            None,
            Some(t0 + Duration::seconds(20)),
        )
        .await
        .expect("complete attempt B");

        let criterion_results = vec![CriterionEvaluation {
            criterion_id: criteria[0].id.clone(),
            result: EvaluationResult::Satisfied,
            evidence_refs: vec!["evidence:test".into()],
            rationale: Some("replacement witness".into()),
        }];
        let evaluation = Evaluation {
            id: "evaluation:b".into(),
            assignment_id: assignment.id.clone(),
            attempt_id: Some(attempt_b.id.clone()),
            candidate_ref: "git:candidate-b".into(),
            evaluator_id: "test-evaluator".into(),
            evaluator_version: "1".into(),
            result: reduce_criterion_results(&criteria, &criterion_results),
            evidence_refs: aggregate_evidence_refs(&criterion_results),
            started_at: t0 + Duration::seconds(21),
            finished_at: t0 + Duration::seconds(22),
        };
        let direct_err = ExecutionRecordPort::create_evaluation(&db, &evaluation)
            .await
            .expect_err("Satisfied evaluation must require a criterion receipt");
        assert!(matches!(direct_err, DomainError::Validation(_)));

        ExecutionRecordPort::create_evaluation_receipt(&db, &evaluation, &criterion_results)
            .await
            .expect("evaluation");

        let attempts = ExecutionRecordPort::list_attempts(&db, &assignment.id)
            .await
            .expect("list attempts");
        assert_eq!(
            attempts.len(),
            2,
            "replacement must not erase prior attempt"
        );
        assert_eq!(attempts[0].id, "attempt:a");
        assert_eq!(attempts[0].status, AttemptStatus::Expired);
        assert_eq!(attempts[0].failure_class.as_deref(), Some("lease_expired"));
        assert_eq!(attempts[1].id, "attempt:b");
        assert_eq!(attempts[1].status, AttemptStatus::Completed);
        assert_eq!(
            attempts[1].result_candidate_ref.as_deref(),
            Some("git:candidate-b")
        );

        let evaluations = ExecutionRecordPort::list_evaluations(&db, &assignment.id)
            .await
            .expect("list evaluations");
        assert_eq!(evaluations.len(), 1);
        assert_eq!(evaluations[0].attempt_id.as_deref(), Some("attempt:b"));
        assert_eq!(evaluations[0].candidate_ref, "git:candidate-b");
        assert_eq!(evaluations[0].result, EvaluationResult::Satisfied);
    }

    #[tokio::test]
    async fn spec_revision_creation_is_idempotent_but_not_mutable() {
        use agileplus_domain::{
            domain::{execution::SpecRevision, feature::Feature},
            ports::{ExecutionRecordPort, StoragePort},
        };
        use chrono::Utc;

        let db = SqliteStorageAdapter::in_memory().expect("in-memory adapter");
        let feature_id = StoragePort::create_feature(
            &db,
            &Feature::new("spec-idempotent", "Spec Idempotent", [9u8; 32], None),
        )
        .await
        .expect("feature");
        let revision = SpecRevision {
            id: "spec:idempotent:1".into(),
            feature_id,
            content_hash: "sha256:stable".into(),
            parent_revision_id: None,
            accepted_at: Utc::now(),
            authority: "test".into(),
        };

        ExecutionRecordPort::create_spec_revision(&db, &revision)
            .await
            .expect("first insert");
        ExecutionRecordPort::create_spec_revision(&db, &revision)
            .await
            .expect("identical replay must be idempotent");

        let mut conflicting = revision.clone();
        conflicting.authority = "different-authority".into();
        let err = ExecutionRecordPort::create_spec_revision(&db, &conflicting)
            .await
            .expect_err("immutable revision mutation must fail");
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn assignment_supersession_is_atomic_and_only_one_active_remains() {
        use agileplus_domain::{
            domain::{
                execution::{Assignment, AssignmentStatus, SpecRevision},
                feature::Feature,
                work_package::WorkPackage,
            },
            ports::{ExecutionRecordPort, StoragePort},
        };
        use chrono::{Duration, Utc};

        let db = SqliteStorageAdapter::in_memory().expect("in-memory adapter");
        let feature_id = StoragePort::create_feature(
            &db,
            &Feature::new(
                "assignment-supersede",
                "Assignment Supersede",
                [10u8; 32],
                None,
            ),
        )
        .await
        .expect("feature");
        let wp_id = StoragePort::create_work_package(
            &db,
            &WorkPackage::new(feature_id, "WP", 1, "criterion"),
        )
        .await
        .expect("wp");
        let t0 = Utc::now();

        let spec_a = SpecRevision {
            id: "spec:a".into(),
            feature_id,
            content_hash: "sha256:a".into(),
            parent_revision_id: None,
            accepted_at: t0,
            authority: "test".into(),
        };
        let spec_b = SpecRevision {
            id: "spec:b".into(),
            feature_id,
            content_hash: "sha256:b".into(),
            parent_revision_id: Some(spec_a.id.clone()),
            accepted_at: t0 + Duration::seconds(1),
            authority: "test".into(),
        };
        ExecutionRecordPort::create_spec_revision(&db, &spec_a)
            .await
            .expect("spec A");
        ExecutionRecordPort::create_spec_revision(&db, &spec_b)
            .await
            .expect("spec B");

        let assignment_a = Assignment {
            id: "assignment:a".into(),
            wp_id,
            spec_revision_id: spec_a.id.clone(),
            created_at: t0,
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        ExecutionRecordPort::create_assignment(&db, &assignment_a)
            .await
            .expect("assignment A");
        assert_eq!(
            ExecutionRecordPort::get_active_assignment(&db, wp_id)
                .await
                .expect("active A")
                .expect("A exists")
                .id,
            assignment_a.id
        );

        let assignment_b = Assignment {
            id: "assignment:b".into(),
            wp_id,
            spec_revision_id: spec_b.id.clone(),
            created_at: t0 + Duration::seconds(2),
            supersedes_assignment_id: Some(assignment_a.id.clone()),
            status: AssignmentStatus::Active,
        };
        ExecutionRecordPort::supersede_assignment(&db, &assignment_a.id, &assignment_b)
            .await
            .expect("supersede A with B");

        let active = ExecutionRecordPort::get_active_assignment(&db, wp_id)
            .await
            .expect("active B")
            .expect("B exists");
        assert_eq!(active.id, assignment_b.id);
        assert_eq!(
            active.supersedes_assignment_id.as_deref(),
            Some("assignment:a")
        );

        let old_status: String = db
            .conn_for_bench()
            .expect("connection")
            .query_row(
                "SELECT status FROM assignments WHERE id='assignment:a'",
                [],
                |row| row.get(0),
            )
            .expect("old assignment status");
        assert_eq!(old_status, "superseded");

        let parallel = Assignment {
            id: "assignment:parallel".into(),
            wp_id,
            spec_revision_id: spec_b.id,
            created_at: t0 + Duration::seconds(3),
            supersedes_assignment_id: None,
            status: AssignmentStatus::Active,
        };
        let err = ExecutionRecordPort::create_assignment(&db, &parallel)
            .await
            .expect_err("partial unique index must reject parallel active assignment");
        assert!(matches!(err, DomainError::Storage(_)));
    }

    #[test]
    fn new_fails_when_parent_directory_is_missing() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "agileplus-sqlite-lib-{}-{nanos}",
            std::process::id()
        ));
        let path = root.join("absent").join("agileplus.db");

        let err = match SqliteStorageAdapter::new(&path) {
            Ok(_) => panic!("opening a database in a missing directory must fail"),
            Err(err) => err,
        };
        assert!(
            matches!(err, DomainError::Storage(_)),
            "expected Storage error, got {err:?}"
        );
    }
}
