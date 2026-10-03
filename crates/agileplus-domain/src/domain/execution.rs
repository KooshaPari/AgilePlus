//! Immutable execution records for MACE-style agent work.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecRevision {
    pub id: String,
    pub feature_id: i64,
    pub content_hash: String,
    pub parent_revision_id: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub authority: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    Active,
    Superseded,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub wp_id: i64,
    pub spec_revision_id: String,
    pub created_at: DateTime<Utc>,
    pub supersedes_assignment_id: Option<String>,
    pub status: AssignmentStatus,
}

/// Immutable acceptance criterion frozen with an Assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentCriterion {
    pub id: String,
    pub ordinal: u32,
    pub statement: String,
    pub source_ref: Option<String>,
    pub mandatory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    Pending,
    Running,
    Failed,
    Cancelled,
    Completed,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub assignment_id: String,
    pub worker_id: String,
    pub backend: String,
    pub job_id: Option<String>,
    pub worktree_path: Option<String>,
    pub base_candidate_ref: Option<String>,
    pub result_candidate_ref: Option<String>,
    pub status: AttemptStatus,
    pub failure_class: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationResult {
    Satisfied,
    Unsatisfied,
    Inconclusive,
    Unknown,
    NotConfigured,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriterionEvaluation {
    pub criterion_id: String,
    pub result: EvaluationResult,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub id: String,
    pub assignment_id: String,
    pub attempt_id: Option<String>,
    pub candidate_ref: String,
    pub evaluator_id: String,
    pub evaluator_version: String,
    pub result: EvaluationResult,
    pub evidence_refs: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

/// Convert the compatibility WorkPackage acceptance checklist into an immutable
/// criterion snapshot. Each non-empty checklist line becomes one criterion.
/// The exact text and ordering are frozen by the Assignment.
pub fn snapshot_acceptance_criteria(raw: &str) -> Vec<AssignmentCriterion> {
    raw.lines()
        .filter_map(|line| {
            let mut statement = line.trim();
            for prefix in ["- [ ] ", "- [x] ", "- [X] ", "- ", "* "] {
                if let Some(rest) = statement.strip_prefix(prefix) {
                    statement = rest.trim();
                    break;
                }
            }
            if statement.is_empty() {
                return None;
            }
            Some(statement.to_string())
        })
        .enumerate()
        .map(|(index, statement)| {
            let source_ref = statement
                .split_once(" -- ")
                .map(|(prefix, _)| prefix.trim())
                .filter(|prefix| {
                    prefix.starts_with("FR-")
                        || prefix.starts_with("NFR-")
                        || prefix.starts_with("AC-")
                })
                .map(ToString::to_string);
            AssignmentCriterion {
                id: format!("AC-{:03}", index + 1),
                ordinal: (index + 1) as u32,
                statement,
                source_ref,
                mandatory: true,
            }
        })
        .collect()
}

/// Validate an evaluator receipt against the exact frozen criterion set.
pub fn validate_criterion_receipt(
    criteria: &[AssignmentCriterion],
    results: &[CriterionEvaluation],
) -> Result<(), String> {
    if criteria.is_empty() {
        return Err("assignment has zero configured criteria".into());
    }

    let expected: BTreeSet<&str> = criteria.iter().map(|c| c.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    for result in results {
        if !expected.contains(result.criterion_id.as_str()) {
            return Err(format!(
                "unexpected criterion result {}",
                result.criterion_id
            ));
        }
        if !seen.insert(result.criterion_id.as_str()) {
            return Err(format!(
                "duplicate criterion result {}",
                result.criterion_id
            ));
        }
        if result.result == EvaluationResult::Satisfied && result.evidence_refs.is_empty() {
            return Err(format!(
                "satisfied criterion {} has no evidence references",
                result.criterion_id
            ));
        }
    }

    for criterion in criteria.iter().filter(|c| c.mandatory) {
        if !seen.contains(criterion.id.as_str()) {
            return Err(format!(
                "missing mandatory criterion result {}",
                criterion.id
            ));
        }
    }
    Ok(())
}

/// Reduce criterion-level results to the canonical aggregate result.
pub fn reduce_criterion_results(
    criteria: &[AssignmentCriterion],
    results: &[CriterionEvaluation],
) -> EvaluationResult {
    if criteria.is_empty() {
        return EvaluationResult::NotConfigured;
    }

    let mandatory: Vec<&AssignmentCriterion> = criteria.iter().filter(|c| c.mandatory).collect();
    if mandatory.is_empty() {
        return EvaluationResult::NotConfigured;
    }

    let mut values = Vec::with_capacity(mandatory.len());
    for criterion in mandatory {
        let Some(result) = results.iter().find(|r| r.criterion_id == criterion.id) else {
            return EvaluationResult::Unknown;
        };
        if result.result == EvaluationResult::Satisfied && result.evidence_refs.is_empty() {
            values.push(EvaluationResult::Inconclusive);
        } else {
            values.push(result.result);
        }
    }

    if values.contains(&EvaluationResult::Unsatisfied) {
        EvaluationResult::Unsatisfied
    } else if values.contains(&EvaluationResult::Inconclusive) {
        EvaluationResult::Inconclusive
    } else if values.contains(&EvaluationResult::Stale) {
        EvaluationResult::Stale
    } else if values.contains(&EvaluationResult::NotConfigured) {
        EvaluationResult::NotConfigured
    } else if values.contains(&EvaluationResult::Unknown) {
        EvaluationResult::Unknown
    } else {
        EvaluationResult::Satisfied
    }
}

pub fn aggregate_evidence_refs(results: &[CriterionEvaluation]) -> Vec<String> {
    results
        .iter()
        .flat_map(|result| result.evidence_refs.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_result_is_not_binary() {
        assert_ne!(EvaluationResult::NotConfigured, EvaluationResult::Satisfied);
        assert_ne!(EvaluationResult::Unknown, EvaluationResult::Satisfied);
    }

    #[test]
    fn snapshot_is_nonempty_and_deterministic() {
        let criteria = snapshot_acceptance_criteria(
            "- FR-001 -- login works\n- [ ] NFR-002 -- latency bounded",
        );
        assert_eq!(criteria.len(), 2);
        assert_eq!(criteria[0].id, "AC-001");
        assert_eq!(criteria[0].source_ref.as_deref(), Some("FR-001"));
        assert_eq!(criteria[1].source_ref.as_deref(), Some("NFR-002"));
        assert_eq!(
            criteria,
            snapshot_acceptance_criteria(
                "- FR-001 -- login works\n- [ ] NFR-002 -- latency bounded"
            )
        );
    }

    #[test]
    fn zero_criteria_never_reduce_to_satisfied() {
        assert_eq!(
            reduce_criterion_results(&[], &[]),
            EvaluationResult::NotConfigured
        );
    }

    #[test]
    fn satisfied_requires_evidence_and_every_mandatory_result() {
        let criteria = snapshot_acceptance_criteria("- one\n- two");
        let incomplete = vec![CriterionEvaluation {
            criterion_id: "AC-001".into(),
            result: EvaluationResult::Satisfied,
            evidence_refs: vec!["test:one".into()],
            rationale: None,
        }];
        assert!(validate_criterion_receipt(&criteria, &incomplete).is_err());

        let no_evidence = vec![
            CriterionEvaluation {
                criterion_id: "AC-001".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: vec![],
                rationale: None,
            },
            CriterionEvaluation {
                criterion_id: "AC-002".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: vec!["test:two".into()],
                rationale: None,
            },
        ];
        assert!(validate_criterion_receipt(&criteria, &no_evidence).is_err());

        let complete = vec![
            CriterionEvaluation {
                criterion_id: "AC-001".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: vec!["test:one".into()],
                rationale: None,
            },
            CriterionEvaluation {
                criterion_id: "AC-002".into(),
                result: EvaluationResult::Satisfied,
                evidence_refs: vec!["test:two".into()],
                rationale: None,
            },
        ];
        assert!(validate_criterion_receipt(&criteria, &complete).is_ok());
        assert_eq!(
            reduce_criterion_results(&criteria, &complete),
            EvaluationResult::Satisfied
        );
    }
}
