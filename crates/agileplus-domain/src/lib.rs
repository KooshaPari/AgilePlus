// SPDX-License-Identifier: MIT OR Apache-2.0
//! `agileplus-domain` — core domain types, error, and port traits.

pub use error::DomainError;
pub use error::ErrorCode;
pub type DomainResult<T> = std::result::Result<T, DomainError>;

pub mod adapters;
pub mod builder;
pub mod config;
pub mod credentials;
pub mod domain;
pub mod error;
pub mod ids;
pub mod intent_graph;
pub mod ports;
pub mod traceability;

// Shared PM/traceability spine (phenotype-pm-core). AgilePlus-local aggregates
// remain in `domain::*`; lifecycle, governance, and intent graph are canonical
// in `traceability-core` and re-exported here for backward-compatible paths.
pub use traceability_core::governance::{
    BuiltinPolicy, Evidence, EvidenceRequirement, EvidenceType, GovernanceContract, GovernanceRule,
    PolicyCheck, PolicyDefinition, PolicyDomain, PolicyRule,
};
pub use traceability_core::intent_graph::{
    CanonicalLinkType, CanonicalMap, DagStage, Edge, GraphMetadata, IntentGraph, Meta, Node,
    NodeType, RelationshipType, Status as NodeStatus, ValidationError,
};
pub use traceability_core::lifecycle::{FeatureState, Transition, TransitionResult};

/// Test-only helpers shared by the crate's unit tests.
#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    /// Serialize tests that mutate process-wide environment variables.
    ///
    /// `set_var`/`remove_var` are process-global, so tests that repoint `HOME`
    /// or set `AGILEPLUS_*` variables must hold this lock for the whole window
    /// in which the value matters. A poisoned lock is recovered rather than
    /// propagated so one panicking test cannot fail every later one.
    pub(crate) fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
