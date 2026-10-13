// SPDX-License-Identifier: MIT OR Apache-2.0
//! Persistence boundary for immutable cross-device aggregate identities.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::stable_identity::{StableEntityIdentity, StableEntityKind};
use crate::error::DomainError;

/// Dedicated identity port kept separate from broad aggregate storage.
///
/// Implementations must preserve one immutable UID per `(kind, local_id)` and
/// one logical entity per UID. Deletion may tombstone a mapping but must not
/// recycle it for another entity.
#[async_trait]
pub trait StableIdentityPort: Send + Sync {
    /// Look up the stable identity assigned to one local aggregate row.
    async fn get_stable_identity(
        &self,
        entity_kind: StableEntityKind,
        entity_id: i64,
    ) -> Result<Option<StableEntityIdentity>, DomainError>;

    /// Resolve one stable UID without assuming its local integer key.
    async fn get_stable_identity_by_uid(
        &self,
        uid: Uuid,
    ) -> Result<Option<StableEntityIdentity>, DomainError>;

    /// Assign an identity to an existing legacy row if one is missing.
    ///
    /// Repeated calls for the same entity are idempotent and return the original
    /// identity. The implementation must fail if the local entity does not
    /// exist, and must not replace a tombstoned or conflicting mapping.
    async fn ensure_stable_identity(
        &self,
        entity_kind: StableEntityKind,
        entity_id: i64,
    ) -> Result<StableEntityIdentity, DomainError>;
}
