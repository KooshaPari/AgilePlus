//! Event sourcing engine for AgilePlus.
//!
//! Provides:
//! - Append-only event storage with SHA-256 hash chain verification
//! - Snapshot management and aggregate replay
//! - Query filtering
//! - Typed domain-event enum + EventEnvelope + EventHandler port (hexagonal)
//!
//! Traceability: FR-008 / WP02

pub mod domain_event;
pub mod hash;
pub mod query;
pub mod replay;
pub mod snapshot;
pub mod store;

pub use domain_event::{
    AggregateId, AsyncEventBus, AsyncEventHandler, DomainEvent, EpicCreated, EpicStatusChanged,
    EventBus, EventEnvelope, EventHandler, EventHandlerError, FeatureCreated, FeatureShipped,
    FeatureStateAdvanced, ProjectArchived, ProjectCreated, ProjectRenamed, StoryAssigned,
    StoryCreated, StoryStatusChanged, UserAdded, UserRoleChanged, UserStatusChanged,
    WorkPackageCreated, WorkPackageStateChanged,
};
pub use hash::{HashError, compute_hash, verify_chain};
pub use query::{EventQuery, QueryError};
pub use replay::{Aggregate, ReplayError, replay_events, replay_events_since};
pub use snapshot::{
    InMemorySnapshotStore, LoadedState, SnapshotConfig, SnapshotError, SnapshotStore,
    should_snapshot,
};
pub use store::{EventError, EventStore, InMemoryEventStore};

#[derive(Debug, thiserror::Error)]
pub enum EventSourcingError {
    #[error("Store error: {0}")]
    Store(#[from] EventError),
    #[error("Hash error: {0}")]
    Hash(#[from] HashError),
    #[error("Replay error: {0}")]
    Replay(#[from] ReplayError),
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] SnapshotError),
    #[error("Query error: {0}")]
    Query(#[from] QueryError),
}
