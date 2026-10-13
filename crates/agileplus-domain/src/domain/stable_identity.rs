// SPDX-License-Identifier: MIT OR Apache-2.0
//! Stable cross-device identities layered over local SQLite integer keys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// AgilePlus aggregate kinds that receive globally stable identities first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StableEntityKind {
    /// Top-level code project.
    Project,
    /// Governed feature/specification aggregate.
    Feature,
    /// Governed development work package.
    WorkPackage,
}

impl StableEntityKind {
    /// Stable storage representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Feature => "feature",
            Self::WorkPackage => "work_package",
        }
    }

    /// Parse the stable storage representation.
    ///
    /// # Errors
    ///
    /// Returns a validation error for unknown entity kinds.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "project" => Ok(Self::Project),
            "feature" => Ok(Self::Feature),
            "work_package" => Ok(Self::WorkPackage),
            other => Err(DomainError::Validation(format!(
                "unknown stable entity kind {other:?}"
            ))),
        }
    }
}

/// Immutable global identity for one local aggregate row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StableEntityIdentity {
    /// Aggregate kind.
    pub entity_kind: StableEntityKind,
    /// Local SQLite primary key; valid only within this project database.
    pub entity_id: i64,
    /// Globally stable UUIDv7 identity.
    pub uid: Uuid,
    /// Time the stable identity was first assigned.
    pub created_at: DateTime<Utc>,
    /// Tombstone time when the local row was deleted, if applicable.
    pub tombstoned_at: Option<DateTime<Utc>>,
}

impl StableEntityIdentity {
    /// Construct a live stable identity and validate its invariants.
    ///
    /// # Errors
    ///
    /// Returns a validation error for non-positive local IDs or non-v7 UUIDs.
    pub fn new(
        entity_kind: StableEntityKind,
        entity_id: i64,
        uid: Uuid,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if entity_id <= 0 {
            return Err(DomainError::Validation(
                "stable identity requires a positive local entity ID".into(),
            ));
        }
        if uid.as_bytes()[6] >> 4 != 7 {
            return Err(DomainError::Validation(
                "stable identity requires a UUIDv7 value".into(),
            ));
        }
        Ok(Self {
            entity_kind,
            entity_id,
            uid,
            created_at,
            tombstoned_at: None,
        })
    }

    /// Return a canonical AgilePlus reference for the entity.
    #[must_use]
    pub fn canonical_uri(&self, project_uid: Uuid) -> String {
        match self.entity_kind {
            StableEntityKind::Project => format!("agileplus://{project_uid}"),
            StableEntityKind::Feature => {
                format!("agileplus://{project_uid}/features/{}", self.uid)
            }
            StableEntityKind::WorkPackage => {
                format!("agileplus://{project_uid}/work-packages/{}", self.uid)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_uuid_v7_and_formats_work_package_reference() {
        let project_uid = Uuid::now_v7();
        let identity = StableEntityIdentity::new(
            StableEntityKind::WorkPackage,
            7,
            Uuid::now_v7(),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            identity.canonical_uri(project_uid),
            format!(
                "agileplus://{project_uid}/work-packages/{}",
                identity.uid
            )
        );
    }

    #[test]
    fn rejects_uuid_v4_and_non_positive_local_id() {
        assert!(StableEntityIdentity::new(
            StableEntityKind::Feature,
            1,
            Uuid::new_v4(),
            Utc::now(),
        )
        .is_err());
        assert!(StableEntityIdentity::new(
            StableEntityKind::Feature,
            0,
            Uuid::now_v7(),
            Utc::now(),
        )
        .is_err());
    }
}
