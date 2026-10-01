//! SQLite CRUD for immutable execution records.

use agileplus_domain::{
    domain::execution::{
        Assignment, AssignmentCriterion, AssignmentStatus, Attempt, AttemptStatus,
        CriterionEvaluation, Evaluation, EvaluationResult, SpecRevision, aggregate_evidence_refs,
        reduce_criterion_results, validate_criterion_receipt,
    },
    error::DomainError,
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

fn err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}
fn dt(s: String) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(&s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DomainError::Storage(e.to_string()))
}
fn asg(s: &str) -> Result<AssignmentStatus, DomainError> {
    match s {
        "active" => Ok(AssignmentStatus::Active),
        "superseded" => Ok(AssignmentStatus::Superseded),
        "cancelled" => Ok(AssignmentStatus::Cancelled),
        x => Err(DomainError::Storage(format!(
            "unknown assignment status {x}"
        ))),
    }
}
fn att(s: &str) -> Result<AttemptStatus, DomainError> {
    match s {
        "pending" => Ok(AttemptStatus::Pending),
        "running" => Ok(AttemptStatus::Running),
        "failed" => Ok(AttemptStatus::Failed),
        "cancelled" => Ok(AttemptStatus::Cancelled),
        "completed" => Ok(AttemptStatus::Completed),
        "expired" => Ok(AttemptStatus::Expired),
        x => Err(DomainError::Storage(format!("unknown attempt status {x}"))),
    }
}
fn ev(s: &str) -> Result<EvaluationResult, DomainError> {
    match s {
        "satisfied" => Ok(EvaluationResult::Satisfied),
        "unsatisfied" => Ok(EvaluationResult::Unsatisfied),
        "inconclusive" => Ok(EvaluationResult::Inconclusive),
        "unknown" => Ok(EvaluationResult::Unknown),
        "not_configured" => Ok(EvaluationResult::NotConfigured),
        "stale" => Ok(EvaluationResult::Stale),
        x => Err(DomainError::Storage(format!(
            "unknown evaluation result {x}"
        ))),
    }
}

pub fn create_spec_revision(c: &Connection, r: &SpecRevision) -> Result<(), DomainError> {
    let changed = c
        .execute(
            "INSERT OR IGNORE INTO spec_revisions(id,feature_id,content_hash,parent_revision_id,accepted_at,authority)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                r.id,
                r.feature_id,
                r.content_hash,
                r.parent_revision_id,
                r.accepted_at.to_rfc3339(),
                r.authority
            ],
        )
        .map_err(err)?;
    if changed == 1 {
        return Ok(());
    }

    let existing: Option<(i64, String, Option<String>, String)> = c
        .query_row(
            "SELECT feature_id,content_hash,parent_revision_id,authority
             FROM spec_revisions WHERE id=?1",
            [&r.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(err)?;

    match existing {
        Some((feature_id, content_hash, parent_revision_id, authority))
            if feature_id == r.feature_id
                && content_hash == r.content_hash
                && parent_revision_id == r.parent_revision_id
                && authority == r.authority =>
        {
            Ok(())
        }
        Some(_) => Err(DomainError::Conflict(format!(
            "spec revision {} already exists with different immutable content",
            r.id
        ))),
        None => Err(DomainError::Conflict(format!(
            "spec revision {} conflicts with an existing feature/content revision",
            r.id
        ))),
    }
}
pub fn create_assignment(c: &Connection, a: &Assignment) -> Result<(), DomainError> {
    let st = match a.status {
        AssignmentStatus::Active => "active",
        AssignmentStatus::Superseded => "superseded",
        AssignmentStatus::Cancelled => "cancelled",
    };
    c.execute("INSERT INTO assignments(id,wp_id,spec_revision_id,created_at,supersedes_assignment_id,status) VALUES (?1,?2,?3,?4,?5,?6)",params![a.id,a.wp_id,a.spec_revision_id,a.created_at.to_rfc3339(),a.supersedes_assignment_id,st]).map_err(err)?;
    Ok(())
}
pub fn create_assignment_with_criteria(
    c: &mut Connection,
    assignment: &Assignment,
    criteria: &[AssignmentCriterion],
) -> Result<(), DomainError> {
    if criteria.is_empty() {
        return Err(DomainError::Validation(
            "assignment must freeze at least one acceptance criterion".into(),
        ));
    }
    let tx = c.transaction().map_err(err)?;
    create_assignment(&tx, assignment)?;
    insert_assignment_criteria(&tx, &assignment.id, criteria)?;
    tx.commit().map_err(err)?;
    Ok(())
}

fn insert_assignment_criteria(
    c: &Connection,
    assignment_id: &str,
    criteria: &[AssignmentCriterion],
) -> Result<(), DomainError> {
    for criterion in criteria {
        c.execute(
            "INSERT INTO assignment_criteria
             (assignment_id,criterion_id,ordinal,statement,source_ref,mandatory)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                assignment_id,
                criterion.id,
                i64::from(criterion.ordinal),
                criterion.statement,
                criterion.source_ref,
                i64::from(criterion.mandatory)
            ],
        )
        .map_err(err)?;
    }
    Ok(())
}

pub fn list_assignment_criteria(
    c: &Connection,
    assignment_id: &str,
) -> Result<Vec<AssignmentCriterion>, DomainError> {
    let mut query = c
        .prepare(
            "SELECT criterion_id,ordinal,statement,source_ref,mandatory
             FROM assignment_criteria
             WHERE assignment_id=?1
             ORDER BY ordinal,criterion_id",
        )
        .map_err(err)?;
    let rows = query
        .query_map([assignment_id], |row| {
            Ok(AssignmentCriterion {
                id: row.get(0)?,
                ordinal: row.get::<_, i64>(1)? as u32,
                statement: row.get(2)?,
                source_ref: row.get(3)?,
                mandatory: row.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_active_assignment(
    c: &Connection,
    wp_id: i64,
) -> Result<Option<Assignment>, DomainError> {
    c.query_row(
        "SELECT id,wp_id,spec_revision_id,created_at,supersedes_assignment_id,status
         FROM assignments WHERE wp_id=?1 AND status='active'
         ORDER BY created_at DESC,id DESC LIMIT 1",
        [wp_id],
        |row| {
            let created_at: String = row.get(3)?;
            let status: String = row.get(5)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                created_at,
                row.get::<_, Option<String>>(4)?,
                status,
            ))
        },
    )
    .optional()
    .map_err(err)?
    .map(
        |(id, wp_id, spec_revision_id, created_at, supersedes_assignment_id, status)| {
            Ok(Assignment {
                id,
                wp_id,
                spec_revision_id,
                created_at: dt(created_at)?,
                supersedes_assignment_id,
                status: asg(&status)?,
            })
        },
    )
    .transpose()
}

pub fn supersede_assignment(
    c: &mut Connection,
    previous_assignment_id: &str,
    replacement: &Assignment,
) -> Result<(), DomainError> {
    if replacement.status != AssignmentStatus::Active {
        return Err(DomainError::Validation(
            "replacement assignment must be active".into(),
        ));
    }
    if replacement.supersedes_assignment_id.as_deref() != Some(previous_assignment_id) {
        return Err(DomainError::Validation(format!(
            "replacement assignment {} must explicitly supersede {}",
            replacement.id, previous_assignment_id
        )));
    }

    let tx = c.transaction().map_err(err)?;
    let previous: Option<(i64, String)> = tx
        .query_row(
            "SELECT wp_id,status FROM assignments WHERE id=?1",
            [previous_assignment_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((previous_wp_id, previous_status)) = previous else {
        return Err(DomainError::NotFound(format!(
            "assignment {previous_assignment_id}"
        )));
    };
    if previous_wp_id != replacement.wp_id || previous_status != "active" {
        return Err(DomainError::Conflict(format!(
            "assignment {previous_assignment_id} is not the active assignment for WP {}",
            replacement.wp_id
        )));
    }

    tx.execute(
        "UPDATE assignments SET status='superseded' WHERE id=?1 AND status='active'",
        [previous_assignment_id],
    )
    .map_err(err)?;

    create_assignment(&tx, replacement)?;
    tx.commit().map_err(err)?;
    Ok(())
}

pub fn supersede_assignment_with_criteria(
    c: &mut Connection,
    previous_assignment_id: &str,
    replacement: &Assignment,
    criteria: &[AssignmentCriterion],
) -> Result<(), DomainError> {
    if criteria.is_empty() {
        return Err(DomainError::Validation(
            "replacement assignment must freeze at least one acceptance criterion".into(),
        ));
    }
    if replacement.status != AssignmentStatus::Active
        || replacement.supersedes_assignment_id.as_deref() != Some(previous_assignment_id)
    {
        return Err(DomainError::Validation(
            "replacement assignment has invalid supersession metadata".into(),
        ));
    }

    let tx = c.transaction().map_err(err)?;
    let previous: Option<(i64, String)> = tx
        .query_row(
            "SELECT wp_id,status FROM assignments WHERE id=?1",
            [previous_assignment_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let Some((previous_wp_id, previous_status)) = previous else {
        return Err(DomainError::NotFound(format!(
            "assignment {previous_assignment_id}"
        )));
    };
    if previous_wp_id != replacement.wp_id || previous_status != "active" {
        return Err(DomainError::Conflict(format!(
            "assignment {previous_assignment_id} is not the active assignment for WP {}",
            replacement.wp_id
        )));
    }

    tx.execute(
        "UPDATE assignments SET status='superseded' WHERE id=?1 AND status='active'",
        [previous_assignment_id],
    )
    .map_err(err)?;
    create_assignment(&tx, replacement)?;
    insert_assignment_criteria(&tx, &replacement.id, criteria)?;
    tx.commit().map_err(err)?;
    Ok(())
}

pub fn create_attempt(c: &Connection, a: &Attempt) -> Result<(), DomainError> {
    let st = match a.status {
        AttemptStatus::Pending => "pending",
        AttemptStatus::Running => "running",
        AttemptStatus::Failed => "failed",
        AttemptStatus::Cancelled => "cancelled",
        AttemptStatus::Completed => "completed",
        AttemptStatus::Expired => "expired",
    };
    c.execute("INSERT INTO attempts(id,assignment_id,worker_id,backend,job_id,worktree_path,base_candidate_ref,result_candidate_ref,status,failure_class,started_at,ended_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![a.id,a.assignment_id,a.worker_id,a.backend,a.job_id,a.worktree_path,a.base_candidate_ref,a.result_candidate_ref,st,a.failure_class,a.started_at.to_rfc3339(),a.ended_at.map(|d|d.to_rfc3339())]).map_err(err)?;
    Ok(())
}
fn evaluation_result_str(result: EvaluationResult) -> &'static str {
    match result {
        EvaluationResult::Satisfied => "satisfied",
        EvaluationResult::Unsatisfied => "unsatisfied",
        EvaluationResult::Inconclusive => "inconclusive",
        EvaluationResult::Unknown => "unknown",
        EvaluationResult::NotConfigured => "not_configured",
        EvaluationResult::Stale => "stale",
    }
}

fn insert_evaluation(c: &Connection, e: &Evaluation) -> Result<(), DomainError> {
    let refs =
        serde_json::to_string(&e.evidence_refs).map_err(|x| DomainError::Storage(x.to_string()))?;
    c.execute(
        "INSERT INTO evaluations
         (id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result,evidence_refs,started_at,finished_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            e.id,
            e.assignment_id,
            e.attempt_id,
            e.candidate_ref,
            e.evaluator_id,
            e.evaluator_version,
            evaluation_result_str(e.result),
            refs,
            e.started_at.to_rfc3339(),
            e.finished_at.to_rfc3339()
        ],
    )
    .map_err(err)?;
    Ok(())
}

pub fn create_evaluation(c: &Connection, e: &Evaluation) -> Result<(), DomainError> {
    if e.result == EvaluationResult::Satisfied {
        return Err(DomainError::Validation(
            "Satisfied evaluations require a criterion-bound evaluation receipt".into(),
        ));
    }
    insert_evaluation(c, e)
}

pub fn create_evaluation_receipt(
    c: &mut Connection,
    e: &Evaluation,
    criterion_results: &[CriterionEvaluation],
) -> Result<(), DomainError> {
    let criteria = list_assignment_criteria(c, &e.assignment_id)?;
    validate_criterion_receipt(&criteria, criterion_results)
        .map_err(DomainError::Validation)?;
    let expected_result = reduce_criterion_results(&criteria, criterion_results);
    if e.result != expected_result {
        return Err(DomainError::Validation(format!(
            "evaluation aggregate {:?} does not match criterion result {:?}",
            e.result, expected_result
        )));
    }
    let expected_refs = aggregate_evidence_refs(criterion_results);
    if e.evidence_refs != expected_refs {
        return Err(DomainError::Validation(
            "evaluation evidence_refs do not equal the criterion evidence union".into(),
        ));
    }

    if e.result == EvaluationResult::Satisfied {
        if !e.candidate_ref.starts_with("git:") {
            return Err(DomainError::Validation(
                "Satisfied evaluation requires an exact git: candidate".into(),
            ));
        }
        let attempt_id = e.attempt_id.as_deref().ok_or_else(|| {
            DomainError::Validation("Satisfied evaluation requires an Attempt".into())
        })?;
        let attempt: Option<(String, String, Option<String>, String)> = c
            .query_row(
                "SELECT assignment_id,status,result_candidate_ref,worker_id
                 FROM attempts WHERE id=?1",
                [attempt_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(err)?;
        let Some((assignment_id, status, candidate_ref, worker_id)) = attempt else {
            return Err(DomainError::NotFound(format!("attempt {attempt_id}")));
        };
        if assignment_id != e.assignment_id
            || status != "completed"
            || candidate_ref.as_deref() != Some(e.candidate_ref.as_str())
        {
            return Err(DomainError::Validation(
                "Satisfied evaluation does not match a completed Attempt candidate".into(),
            ));
        }
        if worker_id == e.evaluator_id {
            return Err(DomainError::Validation(
                "producing worker cannot self-award terminal evaluation".into(),
            ));
        }
    }

    let tx = c.transaction().map_err(err)?;
    insert_evaluation(&tx, e)?;
    for result in criterion_results {
        let refs = serde_json::to_string(&result.evidence_refs)
            .map_err(|error| DomainError::Storage(error.to_string()))?;
        tx.execute(
            "INSERT INTO evaluation_criterion_results
             (evaluation_id,assignment_id,criterion_id,result,evidence_refs,rationale)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                e.id,
                e.assignment_id,
                result.criterion_id,
                evaluation_result_str(result.result),
                refs,
                result.rationale
            ],
        )
        .map_err(err)?;
    }
    tx.commit().map_err(err)?;
    Ok(())
}

pub fn list_attempts(c: &Connection, assignment_id: &str) -> Result<Vec<Attempt>, DomainError> {
    let mut q=c.prepare("SELECT id,assignment_id,worker_id,backend,job_id,worktree_path,base_candidate_ref,result_candidate_ref,status,failure_class,started_at,ended_at FROM attempts WHERE assignment_id=?1 ORDER BY started_at,id").map_err(err)?;
    let rows = q
        .query_map([assignment_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, String>(10)?,
                r.get::<_, Option<String>>(11)?,
            ))
        })
        .map_err(err)?;
    rows.map(|x| {
        let (
            id,
            assignment_id,
            worker_id,
            backend,
            job_id,
            worktree_path,
            base_candidate_ref,
            result_candidate_ref,
            status,
            failure_class,
            started_at,
            ended_at,
        ) = x.map_err(err)?;
        Ok(Attempt {
            id,
            assignment_id,
            worker_id,
            backend,
            job_id,
            worktree_path,
            base_candidate_ref,
            result_candidate_ref,
            status: att(&status)?,
            failure_class,
            started_at: dt(started_at)?,
            ended_at: ended_at.map(dt).transpose()?,
        })
    })
    .collect()
}

pub fn list_evaluations(
    c: &Connection,
    assignment_id: &str,
) -> Result<Vec<Evaluation>, DomainError> {
    let mut q=c.prepare("SELECT id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result,evidence_refs,started_at,finished_at FROM evaluations WHERE assignment_id=?1 ORDER BY finished_at,id").map_err(err)?;
    let rows = q
        .query_map([assignment_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, String>(9)?,
            ))
        })
        .map_err(err)?;
    rows.map(|x| {
        let (
            id,
            assignment_id,
            attempt_id,
            candidate_ref,
            evaluator_id,
            evaluator_version,
            result,
            refs,
            started,
            finished,
        ) = x.map_err(err)?;
        Ok(Evaluation {
            id,
            assignment_id,
            attempt_id,
            candidate_ref,
            evaluator_id,
            evaluator_version,
            result: ev(&result)?,
            evidence_refs: serde_json::from_str(&refs)
                .map_err(|e| DomainError::Storage(e.to_string()))?,
            started_at: dt(started)?,
            finished_at: dt(finished)?,
        })
    })
    .collect()
}

pub fn list_criterion_results(
    c: &Connection,
    evaluation_id: &str,
) -> Result<Vec<CriterionEvaluation>, DomainError> {
    let mut query = c
        .prepare(
            "SELECT criterion_id,result,evidence_refs,rationale
             FROM evaluation_criterion_results
             WHERE evaluation_id=?1
             ORDER BY criterion_id",
        )
        .map_err(err)?;
    let rows = query
        .query_map([evaluation_id], |row| {
            let result: String = row.get(1)?;
            let refs: String = row.get(2)?;
            Ok((row.get::<_, String>(0)?, result, refs, row.get::<_, Option<String>>(3)?))
        })
        .map_err(err)?;
    rows.map(|row| {
        let (criterion_id, result, refs, rationale) = row.map_err(err)?;
        Ok(CriterionEvaluation {
            criterion_id,
            result: ev(&result)?,
            evidence_refs: serde_json::from_str(&refs)
                .map_err(|error| DomainError::Storage(error.to_string()))?,
            rationale,
        })
    })
    .collect()
}

pub fn update_attempt_runtime(
    c: &Connection,
    id: &str,
    status: AttemptStatus,
    job_id: Option<&str>,
    result_candidate_ref: Option<&str>,
    failure_class: Option<&str>,
    ended_at: Option<DateTime<Utc>>,
) -> Result<(), DomainError> {
    let st = match status {
        AttemptStatus::Pending => "pending",
        AttemptStatus::Running => "running",
        AttemptStatus::Failed => "failed",
        AttemptStatus::Cancelled => "cancelled",
        AttemptStatus::Completed => "completed",
        AttemptStatus::Expired => "expired",
    };
    let changed=c.execute("UPDATE attempts SET status=?2, job_id=COALESCE(?3,job_id), result_candidate_ref=COALESCE(?4,result_candidate_ref), failure_class=COALESCE(?5,failure_class), ended_at=COALESCE(?6,ended_at) WHERE id=?1",
 params![id,st,job_id,result_candidate_ref,failure_class,ended_at.map(|d|d.to_rfc3339())]).map_err(err)?;
    if changed != 1 {
        return Err(DomainError::Storage(format!("attempt {id} not found")));
    }
    Ok(())
}
