//! Schema-first PM items. Workflow progress never grants AcceptedWork.
use crate::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemKind {
    Task,
    Bug,
    Story,
    Feature,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemState {
    Intake,
    Triaged,
    Planned,
    InProgress,
    InReview,
}
impl WorkItemState {
    pub fn transition(self, target: Self) -> Result<(), DomainError> {
        use WorkItemState::*;
        if matches!(
            (self, target),
            (Intake, Triaged)
                | (Triaged, Planned)
                | (Planned, InProgress)
                | (InProgress, InReview)
                | (InReview, InProgress)
        ) {
            Ok(())
        } else {
            Err(DomainError::Conflict(format!(
                "invalid work-item transition {self:?} -> {target:?}"
            )))
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: i64,
    pub kind: WorkItemKind,
    pub title: String,
    pub state: WorkItemState,
    pub cycle_id: Option<i64>,
    pub feature_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewWorkItem {
    pub kind: WorkItemKind,
    pub title: String,
    pub cycle_id: Option<i64>,
    pub feature_id: Option<i64>,
}
impl NewWorkItem {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() || self.title.len() > 4096 {
            return Err(DomainError::Validation(
                "title must contain 1..4096 bytes".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: i64,
    pub work_item_id: i64,
    pub actor: String,
    pub action: String,
    pub payload_json: String,
    pub prev_hash: String,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}
impl AuditEvent {
    /// Versioned JSON tuple binds every event field without delimiter ambiguity.
    pub fn compute_hash(&self) -> String {
        let bytes = serde_json::to_vec(&(
            "agileplus.work-item.audit.v1",
            self.id,
            self.work_item_id,
            &self.actor,
            &self.action,
            &self.payload_json,
            &self.prev_hash,
            self.created_at,
        ))
        .expect("serializable audit tuple");
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
pub fn validate_actor(actor: &str) -> Result<(), DomainError> {
    if actor.trim().is_empty() || actor.len() > 256 {
        return Err(DomainError::Validation(
            "actor must contain 1..256 bytes".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn work_item_audit_hash_binds_every_field() {
        let e = AuditEvent {
            id: 1,
            work_item_id: 2,
            actor: "operator".into(),
            action: "created".into(),
            payload_json: "{}".into(),
            prev_hash: String::new(),
            hash: String::new(),
            created_at: Utc::now(),
        };
        let expected = e.compute_hash();
        let variants = [
            AuditEvent { id: 3, ..e.clone() },
            AuditEvent {
                work_item_id: 3,
                ..e.clone()
            },
            AuditEvent {
                actor: "other".into(),
                ..e.clone()
            },
            AuditEvent {
                action: "other".into(),
                ..e.clone()
            },
            AuditEvent {
                payload_json: "[]".into(),
                ..e.clone()
            },
            AuditEvent {
                prev_hash: "other".into(),
                ..e.clone()
            },
            AuditEvent {
                created_at: e.created_at + chrono::Duration::seconds(1),
                ..e.clone()
            },
        ];
        assert_eq!(expected.len(), 64);
        for v in variants {
            assert_ne!(v.compute_hash(), expected);
        }
    }
}
