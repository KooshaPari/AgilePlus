//! Event sourcing engine for AgilePlus.
//!
//! Provides append-only event storage with SHA-256 hash chain verification,
//! snapshot management, aggregate replay, and query filtering.
//! Traceability: FR-008 / WP02

pub mod bus;
pub mod domain_event;
pub mod hash;
pub mod query;
pub mod replay;
pub mod snapshot;
pub mod store;

pub use bus::{DomainEvent, EventBus, EventSubscriber};

pub use domain_event::{
    AggregateId, AsyncEventBus, AsyncEventHandler, EpicCreated, EpicStatusChanged, EventEnvelope,
    EventHandler, EventHandlerError, FeatureCreated, FeatureShipped, FeatureStateAdvanced,
    ProjectArchived, ProjectCreated, ProjectRenamed, StoryAssigned, StoryCreated,
    StoryStatusChanged, UserAdded, UserRoleChanged, UserStatusChanged, WorkPackageCreated,
    WorkPackageStateChanged,
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

#[cfg(test)]
mod error_conversion_tests {
    use super::*;
    use agileplus_domain::domain::event::Event;
    use chrono::Utc;

    fn sample_event() -> Event {
        Event::new(
            "Feature",
            1,
            "created",
            serde_json::json!({"payload": true}),
            "tester",
        )
    }

    /// Store whose every operation fails, so `?` has a real error to convert.
    struct NeverStore;

    #[async_trait::async_trait]
    impl EventStore for NeverStore {
        async fn append(&self, _event: &Event) -> Result<i64, EventError> {
            Err(EventError::StorageError("append always fails".into()))
        }

        async fn get_events(&self, _e: &str, _i: i64) -> Result<Vec<Event>, EventError> {
            Err(EventError::StorageError("read always fails".into()))
        }

        async fn get_events_since(
            &self,
            _e: &str,
            _i: i64,
            _s: i64,
        ) -> Result<Vec<Event>, EventError> {
            Err(EventError::StorageError("read-since always fails".into()))
        }

        async fn get_events_by_range(
            &self,
            _e: &str,
            _i: i64,
            _f: chrono::DateTime<Utc>,
            _t: chrono::DateTime<Utc>,
        ) -> Result<Vec<Event>, EventError> {
            Err(EventError::StorageError("range read always fails".into()))
        }

        async fn get_latest_sequence(&self, _e: &str, _i: i64) -> Result<i64, EventError> {
            Err(EventError::StorageError("latest always fails".into()))
        }
    }

    #[tokio::test]
    async fn question_mark_converts_store_error_into_event_sourcing_error() {
        async fn append_one(store: &dyn EventStore) -> Result<i64, EventSourcingError> {
            Ok(store.append(&sample_event()).await?)
        }

        let err = append_one(&NeverStore).await.unwrap_err();
        assert!(matches!(err, EventSourcingError::Store(_)));
        assert_eq!(
            err.to_string(),
            "Store error: Storage error: append always fails"
        );
    }

    #[test]
    fn from_conversions_select_the_matching_variant() {
        assert!(matches!(
            EventSourcingError::from(EventError::NotFound("e9".into())),
            EventSourcingError::Store(_)
        ));
        assert!(matches!(
            EventSourcingError::from(HashError::ChainBroken { sequence: 1 }),
            EventSourcingError::Hash(_)
        ));
        assert!(matches!(
            EventSourcingError::from(ReplayError::AggregateError("bad".into())),
            EventSourcingError::Replay(_)
        ));
        assert!(matches!(
            EventSourcingError::from(SnapshotError::Invalid("bad".into())),
            EventSourcingError::Snapshot(_)
        ));
        assert!(matches!(
            EventSourcingError::from(QueryError::Error("bad".into())),
            EventSourcingError::Query(_)
        ));
    }

    #[test]
    fn event_sourcing_error_display_keeps_source_message() {
        let cases: Vec<(EventSourcingError, &str)> = vec![
            (
                EventError::NotFound("e9".into()).into(),
                "Store error: Event not found: e9",
            ),
            (
                HashError::ChainBroken { sequence: 4 }.into(),
                "Hash error: Hash chain broken at sequence 4",
            ),
            (
                ReplayError::InvalidState("mixed".into()).into(),
                "Replay error: Invalid state: mixed",
            ),
            (
                SnapshotError::NotFound {
                    entity_type: "Feature".into(),
                    entity_id: 3,
                }
                .into(),
                "Snapshot error: Snapshot not found for Feature:3",
            ),
            (
                QueryError::Error("bad filter".into()).into(),
                "Query error: Query error: bad filter",
            ),
        ];

        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }
}
