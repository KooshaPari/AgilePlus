// SPDX-License-Identifier: MIT OR Apache-2.0
//! Typed domain-event enum — every aggregate transition becomes a concrete variant.
//!
//! Design: the events crate defines *what happened* in the domain. It depends on
//! `agileplus-domain` for status/state types but introduces no concrete bus or
//! persistence.  Consumers (bus adapters, projections) implement the `EventHandler`
//! port defined in this module.
//!
//! Traceability: FR-008 / WP02 (domain-event layer)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use agileplus_domain::domain::epic::EpicStatus;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::StoryStatus;
use agileplus_domain::domain::user::{UserRole, UserStatus};
use agileplus_domain::domain::work_package::WpState;

// ──────────────────────────────────────────────────────────────────────────────
// Aggregate-id newtype (keeps IDs from different aggregates from mixing)
// ──────────────────────────────────────────────────────────────────────────────

/// Strongly-typed identifier for a domain aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AggregateId(pub i64);

impl From<i64> for AggregateId {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Payload types (carry only the data needed by projections / subscribers)
// ──────────────────────────────────────────────────────────────────────────────

/// Project created.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectCreated {
    pub project_id: AggregateId,
    pub slug: String,
    pub name: String,
}

/// Project renamed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectRenamed {
    pub project_id: AggregateId,
    pub old_name: String,
    pub new_name: String,
}

/// Project archived (soft-delete).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectArchived {
    pub project_id: AggregateId,
}

/// Epic created under a project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpicCreated {
    pub epic_id: AggregateId,
    pub project_id: AggregateId,
    pub title: String,
}

/// Epic status changed (e.g. Backlog → Active).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpicStatusChanged {
    pub epic_id: AggregateId,
    pub project_id: AggregateId,
    pub from: EpicStatus,
    pub to: EpicStatus,
}

/// Story created under an epic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryCreated {
    pub story_id: AggregateId,
    pub epic_id: AggregateId,
    pub project_id: AggregateId,
    pub title: String,
    pub points: Option<u32>,
}

/// Story status changed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryStatusChanged {
    pub story_id: AggregateId,
    pub epic_id: AggregateId,
    pub from: StoryStatus,
    pub to: StoryStatus,
}

/// Story assigned to a user (or unassigned).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryAssigned {
    pub story_id: AggregateId,
    pub assignee_id: Option<AggregateId>,
}

/// User added to the platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserAdded {
    pub user_id: AggregateId,
    pub display_name: String,
    pub email: String,
    pub role: UserRole,
}

/// User role changed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRoleChanged {
    pub user_id: AggregateId,
    pub old_role: UserRole,
    pub new_role: UserRole,
}

/// User status changed (active / inactive / suspended).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserStatusChanged {
    pub user_id: AggregateId,
    pub from: UserStatus,
    pub to: UserStatus,
}

/// Feature created.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureCreated {
    pub feature_id: AggregateId,
    pub slug: String,
    pub friendly_name: String,
    pub project_id: Option<AggregateId>,
}

/// Feature state advanced (e.g. Created → Specified).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureStateAdvanced {
    pub feature_id: AggregateId,
    pub from: FeatureState,
    pub to: FeatureState,
}

/// Feature shipped (terminal state reached).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureShipped {
    pub feature_id: AggregateId,
    pub slug: String,
}

/// Work-package created under a feature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkPackageCreated {
    pub wp_id: AggregateId,
    pub feature_id: AggregateId,
    pub title: String,
    pub sequence: i32,
}

/// Work-package state changed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkPackageStateChanged {
    pub wp_id: AggregateId,
    pub feature_id: AggregateId,
    pub from: WpState,
    pub to: WpState,
}

// ──────────────────────────────────────────────────────────────────────────────
// The domain-event enum
// ──────────────────────────────────────────────────────────────────────────────

/// Every observable transition in the AgilePlus domain.
///
/// Add a variant here when a new aggregate is introduced or an existing one
/// gains a new observable transition.  Variants are `#[non_exhaustive]` to
/// allow downstream crates to match without breaking on future additions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum DomainEvent {
    // --- Project ---
    ProjectCreated(ProjectCreated),
    ProjectRenamed(ProjectRenamed),
    ProjectArchived(ProjectArchived),

    // --- Epic ---
    EpicCreated(EpicCreated),
    EpicStatusChanged(EpicStatusChanged),

    // --- Story ---
    StoryCreated(StoryCreated),
    StoryStatusChanged(StoryStatusChanged),
    StoryAssigned(StoryAssigned),

    // --- User ---
    UserAdded(UserAdded),
    UserRoleChanged(UserRoleChanged),
    UserStatusChanged(UserStatusChanged),

    // --- Feature ---
    FeatureCreated(FeatureCreated),
    FeatureStateAdvanced(FeatureStateAdvanced),
    FeatureShipped(FeatureShipped),

    // --- WorkPackage ---
    WorkPackageCreated(WorkPackageCreated),
    WorkPackageStateChanged(WorkPackageStateChanged),
}

impl DomainEvent {
    /// Human-readable event type string (useful for logging / routing).
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::ProjectCreated(_) => "project.created",
            DomainEvent::ProjectRenamed(_) => "project.renamed",
            DomainEvent::ProjectArchived(_) => "project.archived",
            DomainEvent::EpicCreated(_) => "epic.created",
            DomainEvent::EpicStatusChanged(_) => "epic.status_changed",
            DomainEvent::StoryCreated(_) => "story.created",
            DomainEvent::StoryStatusChanged(_) => "story.status_changed",
            DomainEvent::StoryAssigned(_) => "story.assigned",
            DomainEvent::UserAdded(_) => "user.added",
            DomainEvent::UserRoleChanged(_) => "user.role_changed",
            DomainEvent::UserStatusChanged(_) => "user.status_changed",
            DomainEvent::FeatureCreated(_) => "feature.created",
            DomainEvent::FeatureStateAdvanced(_) => "feature.state_advanced",
            DomainEvent::FeatureShipped(_) => "feature.shipped",
            DomainEvent::WorkPackageCreated(_) => "work_package.created",
            DomainEvent::WorkPackageStateChanged(_) => "work_package.state_changed",
        }
    }

    /// Stable machine-readable wire code (e.g. for cross-repo bus routing).
    ///
    /// Returns a `&'static str` so it can be used directly as a routing key,
    /// log field, or NATS subject without allocating.  The same vocabulary is
    /// used by every consumer repo of `phenotype-error-core`.
    pub fn wire_code(&self) -> &'static str {
        self.event_type()
    }

    /// The aggregate type this event belongs to (e.g. `"Project"`).
    pub fn aggregate_type(&self) -> &'static str {
        match self {
            DomainEvent::ProjectCreated(_)
            | DomainEvent::ProjectRenamed(_)
            | DomainEvent::ProjectArchived(_) => "Project",

            DomainEvent::EpicCreated(_) | DomainEvent::EpicStatusChanged(_) => "Epic",

            DomainEvent::StoryCreated(_)
            | DomainEvent::StoryStatusChanged(_)
            | DomainEvent::StoryAssigned(_) => "Story",

            DomainEvent::UserAdded(_)
            | DomainEvent::UserRoleChanged(_)
            | DomainEvent::UserStatusChanged(_) => "User",

            DomainEvent::FeatureCreated(_)
            | DomainEvent::FeatureStateAdvanced(_)
            | DomainEvent::FeatureShipped(_) => "Feature",

            DomainEvent::WorkPackageCreated(_) | DomainEvent::WorkPackageStateChanged(_) => {
                "WorkPackage"
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// EventEnvelope — wraps a DomainEvent with routing metadata
// ──────────────────────────────────────────────────────────────────────────────

/// An immutable, serialisable wrapper around a [`DomainEvent`].
///
/// The envelope carries the metadata needed by infrastructure (bus routing,
/// idempotency checks, audit log) without leaking those concerns into the
/// domain payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    /// Globally unique identifier for this envelope instance.
    pub id: Uuid,
    /// Wall-clock time when the envelope was created.
    pub occurred_at: DateTime<Utc>,
    /// The primary aggregate's id (entity that produced the event).
    pub aggregate_id: AggregateId,
    /// The aggregate type name (`"Project"`, `"Epic"`, …).
    pub aggregate_type: String,
    /// Causal chain: the id of the command/event that triggered this event.
    pub causation_id: Option<Uuid>,
    /// Correlation id that spans a logical operation across multiple events.
    pub correlation_id: Option<Uuid>,
    /// The event payload.
    pub payload: DomainEvent,
}

impl EventEnvelope {
    /// Wrap a [`DomainEvent`] in an envelope with a new UUID and current timestamp.
    pub fn new(aggregate_id: impl Into<AggregateId>, payload: DomainEvent) -> Self {
        let aggregate_id = aggregate_id.into();
        let aggregate_type = payload.aggregate_type().to_string();
        Self {
            id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            aggregate_id,
            aggregate_type,
            causation_id: None,
            correlation_id: None,
            payload,
        }
    }

    /// Builder: attach a causation id.
    pub fn with_causation(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    /// Builder: attach a correlation id.
    pub fn with_correlation(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// EventHandler port — hexagonal outbound port; NO concrete bus here
// ──────────────────────────────────────────────────────────────────────────────

/// Subscriber / handler port for domain events (hexagonal outbound port).
///
/// Infrastructure adapters (NATS publisher, in-memory bus, test spy, …) implement
/// this trait.  The domain and application layers only depend on this trait, never
/// on concrete message-broker libraries.
///
/// Implementations MUST be idempotent: the same envelope may be delivered more
/// than once.
pub trait EventHandler: Send + Sync {
    /// Handle a single domain-event envelope synchronously.
    ///
    /// Return `Err` only for unrecoverable failures; transient errors should be
    /// retried by the caller.
    fn handle(&self, envelope: &EventEnvelope) -> Result<(), EventHandlerError>;
}

/// Async version of [`EventHandler`] for I/O-bound subscribers.
#[async_trait::async_trait]
pub trait AsyncEventHandler: Send + Sync {
    async fn handle(&self, envelope: &EventEnvelope) -> Result<(), EventHandlerError>;
}

/// Error returned by an [`EventHandler`].
#[derive(Debug, thiserror::Error)]
pub enum EventHandlerError {
    #[error("handler rejected event: {0}")]
    Rejected(String),
    #[error("handler encountered a transient error: {0}")]
    Transient(String),
}

impl EventHandlerError {}

// ──────────────────────────────────────────────────────────────────────────────
// EventBus port — dispatches to registered handlers (no concrete impl here)
// ──────────────────────────────────────────────────────────────────────────────

/// Port for publishing domain-event envelopes to zero or more handlers.
///
/// Concrete adapters (in-process fan-out, NATS, etc.) implement this trait.
pub trait EventBus: Send + Sync {
    /// Publish an envelope to all registered handlers.
    fn publish(&self, envelope: EventEnvelope) -> Result<(), EventHandlerError>;
}

/// Async version of [`EventBus`].
#[async_trait::async_trait]
pub trait AsyncEventBus: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventHandlerError>;
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::epic::EpicStatus;
    use agileplus_domain::domain::story::StoryStatus;
    use agileplus_domain::domain::user::{UserRole, UserStatus};
    use std::sync::{Arc, Mutex};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn project_created() -> DomainEvent {
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: 1.into(),
            slug: "my-project".into(),
            name: "My Project".into(),
        })
    }

    fn epic_status_changed() -> DomainEvent {
        DomainEvent::EpicStatusChanged(EpicStatusChanged {
            epic_id: 2.into(),
            project_id: 1.into(),
            from: EpicStatus::Backlog,
            to: EpicStatus::Active,
        })
    }

    fn story_created() -> DomainEvent {
        DomainEvent::StoryCreated(StoryCreated {
            story_id: 10.into(),
            epic_id: 2.into(),
            project_id: 1.into(),
            title: "User can log in".into(),
            points: Some(3),
        })
    }

    fn story_status_changed() -> DomainEvent {
        DomainEvent::StoryStatusChanged(StoryStatusChanged {
            story_id: 10.into(),
            epic_id: 2.into(),
            from: StoryStatus::Todo,
            to: StoryStatus::InProgress,
        })
    }

    fn user_added() -> DomainEvent {
        DomainEvent::UserAdded(UserAdded {
            user_id: 5.into(),
            display_name: "Alice".into(),
            email: "alice@example.com".into(),
            role: UserRole::Member,
        })
    }

    // ── event construction ────────────────────────────────────────────────────

    #[test]
    fn project_created_event_type_string() {
        assert_eq!(project_created().event_type(), "project.created");
    }

    #[test]
    fn epic_status_changed_aggregate_type() {
        assert_eq!(epic_status_changed().aggregate_type(), "Epic");
    }

    #[test]
    fn story_created_aggregate_type() {
        assert_eq!(story_created().aggregate_type(), "Story");
    }

    #[test]
    fn user_added_event_type_string() {
        assert_eq!(user_added().event_type(), "user.added");
    }

    #[test]
    fn all_event_type_strings_are_unique() {
        let variants: Vec<&str> = vec![
            "project.created",
            "project.renamed",
            "project.archived",
            "epic.created",
            "epic.status_changed",
            "story.created",
            "story.status_changed",
            "story.assigned",
            "user.added",
            "user.role_changed",
            "user.status_changed",
            "feature.created",
            "feature.state_advanced",
            "feature.shipped",
            "work_package.created",
            "work_package.state_changed",
        ];
        let mut seen = std::collections::HashSet::new();
        for v in variants {
            assert!(seen.insert(v), "duplicate event type string: {v}");
        }
    }

    // ── EventEnvelope round-trip ──────────────────────────────────────────────

    #[test]
    fn envelope_round_trip_serde() {
        let env = EventEnvelope::new(1i64, project_created());
        let json = serde_json::to_string(&env).expect("serialize");
        let decoded: EventEnvelope = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(env.id, decoded.id);
        assert_eq!(env.aggregate_id, decoded.aggregate_id);
        assert_eq!(env.aggregate_type, decoded.aggregate_type);
        match decoded.payload {
            DomainEvent::ProjectCreated(p) => {
                assert_eq!(p.slug, "my-project");
                assert_eq!(p.name, "My Project");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn epic_status_changed_serde_round_trip() {
        let env = EventEnvelope::new(2i64, epic_status_changed());
        let json = serde_json::to_string(&env).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        match decoded.payload {
            DomainEvent::EpicStatusChanged(e) => {
                assert_eq!(e.from, EpicStatus::Backlog);
                assert_eq!(e.to, EpicStatus::Active);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn story_status_changed_serde_round_trip() {
        let env = EventEnvelope::new(10i64, story_status_changed());
        let json = serde_json::to_string(&env).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        match decoded.payload {
            DomainEvent::StoryStatusChanged(s) => {
                assert_eq!(s.from, StoryStatus::Todo);
                assert_eq!(s.to, StoryStatus::InProgress);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn envelope_causation_correlation_builders() {
        let cause = Uuid::new_v4();
        let corr = Uuid::new_v4();
        let env = EventEnvelope::new(1i64, project_created())
            .with_causation(cause)
            .with_correlation(corr);
        assert_eq!(env.causation_id, Some(cause));
        assert_eq!(env.correlation_id, Some(corr));
    }

    #[test]
    fn envelope_ids_are_unique() {
        let a = EventEnvelope::new(1i64, project_created());
        let b = EventEnvelope::new(1i64, project_created());
        assert_ne!(a.id, b.id);
    }

    // ── EventHandler port ─────────────────────────────────────────────────────

    /// Test spy that records every envelope it receives.
    struct SpyHandler {
        received: Arc<Mutex<Vec<EventEnvelope>>>,
    }

    impl SpyHandler {
        fn new() -> (Self, Arc<Mutex<Vec<EventEnvelope>>>) {
            let received = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    received: received.clone(),
                },
                received,
            )
        }
    }

    impl EventHandler for SpyHandler {
        fn handle(&self, envelope: &EventEnvelope) -> Result<(), EventHandlerError> {
            self.received.lock().unwrap().push(envelope.clone());
            Ok(())
        }
    }

    #[test]
    fn handler_receives_emitted_events() {
        let (handler, received) = SpyHandler::new();

        let envelopes = vec![
            EventEnvelope::new(1i64, project_created()),
            EventEnvelope::new(2i64, epic_status_changed()),
            EventEnvelope::new(10i64, story_created()),
        ];

        for env in &envelopes {
            handler.handle(env).expect("handle should succeed");
        }

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].aggregate_type, "Project");
        assert_eq!(got[1].aggregate_type, "Epic");
        assert_eq!(got[2].aggregate_type, "Story");
    }

    #[test]
    fn handler_event_types_match_payloads() {
        let (handler, received) = SpyHandler::new();

        handler
            .handle(&EventEnvelope::new(5i64, user_added()))
            .unwrap();

        let got = received.lock().unwrap();
        assert_eq!(got[0].payload.event_type(), "user.added");
    }

    // ── in-process EventBus ───────────────────────────────────────────────────

    /// Minimal synchronous fan-out bus (in-process; no external deps).
    struct InProcessBus {
        handlers: Vec<Box<dyn EventHandler>>,
    }

    impl InProcessBus {
        fn new() -> Self {
            Self {
                handlers: Vec::new(),
            }
        }

        fn register(&mut self, h: impl EventHandler + 'static) {
            self.handlers.push(Box::new(h));
        }
    }

    impl EventBus for InProcessBus {
        fn publish(&self, envelope: EventEnvelope) -> Result<(), EventHandlerError> {
            for h in &self.handlers {
                h.handle(&envelope)?;
            }
            Ok(())
        }
    }

    #[test]
    fn bus_fan_out_to_multiple_handlers() {
        let (h1, r1) = SpyHandler::new();
        let (h2, r2) = SpyHandler::new();

        let mut bus = InProcessBus::new();
        bus.register(h1);
        bus.register(h2);

        bus.publish(EventEnvelope::new(1i64, project_created()))
            .unwrap();

        assert_eq!(r1.lock().unwrap().len(), 1);
        assert_eq!(r2.lock().unwrap().len(), 1);
    }

    #[test]
    fn work_package_events_round_trip() {
        let wp_created = DomainEvent::WorkPackageCreated(WorkPackageCreated {
            wp_id: 20.into(),
            feature_id: 3.into(),
            title: "Implement login endpoint".into(),
            sequence: 1,
        });
        let env = EventEnvelope::new(20i64, wp_created);
        let json = serde_json::to_string(&env).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.aggregate_type, "WorkPackage");
        assert_eq!(decoded.payload.event_type(), "work_package.created");
    }

    #[test]
    fn user_status_changed_round_trip() {
        let ev = DomainEvent::UserStatusChanged(UserStatusChanged {
            user_id: 5.into(),
            from: UserStatus::Active,
            to: UserStatus::Suspended,
        });
        let env = EventEnvelope::new(5i64, ev);
        let json = serde_json::to_string(&env).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        match decoded.payload {
            DomainEvent::UserStatusChanged(u) => {
                assert_eq!(u.from, UserStatus::Active);
                assert_eq!(u.to, UserStatus::Suspended);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    // ── wire_code (cross-repo error-core integration) ─────────────────────────

    #[test]
    fn wire_code_matches_event_type_for_all_variants() {
        // wire_code() must always agree with event_type() so logs and routing
        // keys stay in sync across repos.
        let events = vec![
            project_created(),
            epic_status_changed(),
            story_created(),
            story_status_changed(),
            user_added(),
            DomainEvent::WorkPackageCreated(WorkPackageCreated {
                wp_id: 20.into(),
                feature_id: 3.into(),
                title: "wp".into(),
                sequence: 1,
            }),
        ];
        for ev in events {
            assert_eq!(ev.wire_code(), ev.event_type());
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use agileplus_domain::domain::epic::EpicStatus;
    use agileplus_domain::domain::story::StoryStatus;
    use agileplus_domain::domain::user::{UserRole, UserStatus};
    use agileplus_domain::domain::work_package::WpState;
    use std::sync::{Arc, Mutex};

    fn all_variants() -> Vec<DomainEvent> {
        vec![
            DomainEvent::ProjectCreated(ProjectCreated {
                project_id: 1.into(),
                slug: "s".into(),
                name: "n".into(),
            }),
            DomainEvent::ProjectRenamed(ProjectRenamed {
                project_id: 1.into(),
                old_name: "old".into(),
                new_name: "new".into(),
            }),
            DomainEvent::ProjectArchived(ProjectArchived { project_id: 1.into() }),
            DomainEvent::EpicCreated(EpicCreated {
                epic_id: 2.into(),
                project_id: 1.into(),
                title: "e".into(),
            }),
            DomainEvent::EpicStatusChanged(EpicStatusChanged {
                epic_id: 2.into(),
                project_id: 1.into(),
                from: EpicStatus::Backlog,
                to: EpicStatus::Active,
            }),
            DomainEvent::StoryCreated(StoryCreated {
                story_id: 3.into(),
                epic_id: 2.into(),
                project_id: 1.into(),
                title: "s".into(),
                points: None,
            }),
            DomainEvent::StoryStatusChanged(StoryStatusChanged {
                story_id: 3.into(),
                epic_id: 2.into(),
                from: StoryStatus::Todo,
                to: StoryStatus::Done,
            }),
            DomainEvent::StoryAssigned(StoryAssigned {
                story_id: 3.into(),
                assignee_id: Some(9.into()),
            }),
            DomainEvent::UserAdded(UserAdded {
                user_id: 4.into(),
                display_name: "u".into(),
                email: "u@x".into(),
                role: UserRole::Member,
            }),
            DomainEvent::UserRoleChanged(UserRoleChanged {
                user_id: 4.into(),
                old_role: UserRole::Viewer,
                new_role: UserRole::Admin,
            }),
            DomainEvent::UserStatusChanged(UserStatusChanged {
                user_id: 4.into(),
                from: UserStatus::Active,
                to: UserStatus::Suspended,
            }),
            DomainEvent::FeatureCreated(FeatureCreated {
                feature_id: 5.into(),
                slug: "f".into(),
                friendly_name: "F".into(),
                project_id: None,
            }),
            DomainEvent::FeatureStateAdvanced(FeatureStateAdvanced {
                feature_id: 5.into(),
                from: FeatureState::Created,
                to: FeatureState::Specified,
            }),
            DomainEvent::FeatureShipped(FeatureShipped {
                feature_id: 5.into(),
                slug: "f".into(),
            }),
            DomainEvent::WorkPackageCreated(WorkPackageCreated {
                wp_id: 6.into(),
                feature_id: 5.into(),
                title: "wp".into(),
                sequence: 1,
            }),
            DomainEvent::WorkPackageStateChanged(WorkPackageStateChanged {
                wp_id: 6.into(),
                feature_id: 5.into(),
                from: WpState::Planned,
                to: WpState::Doing,
            }),
        ]
    }

    fn variant(n: usize) -> DomainEvent {
        all_variants().into_iter().nth(n).expect("variant index in range")
    }

    #[test]
    fn aggregate_id_from_i64() {
        let id: AggregateId = 7i64.into();
        assert_eq!(id, AggregateId(7));
    }

    #[test]
    fn aggregate_id_serde_roundtrip() {
        let id = AggregateId(42);
        let json = serde_json::to_string(&id).unwrap();
        let back: AggregateId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn aggregate_id_hash_usable_in_set() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AggregateId(1));
        set.insert(AggregateId(1));
        set.insert(AggregateId(2));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn all_variants_event_type_is_unique_and_snake_case() {
        let mut seen = std::collections::HashSet::new();
        for ev in all_variants() {
            let t = ev.event_type();
            assert!(t.contains('.'), "expected dotted type, got {t}");
            assert!(
                t.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
                "not machine-readable: {t}"
            );
            assert!(seen.insert(t), "duplicate event type {t}");
        }
        assert_eq!(seen.len(), 16);
    }

    #[test]
    fn wire_code_matches_event_type_for_all_variants() {
        for ev in all_variants() {
            assert_eq!(ev.wire_code(), ev.event_type());
        }
    }

    #[test]
    fn aggregate_type_for_project_variants() {
        for ev in all_variants().into_iter().take(3) {
            assert_eq!(ev.aggregate_type(), "Project");
        }
    }

    #[test]
    fn aggregate_type_for_epic_variants() {
        for ev in all_variants().into_iter().skip(3).take(2) {
            assert_eq!(ev.aggregate_type(), "Epic");
        }
    }

    #[test]
    fn aggregate_type_for_story_variants() {
        for ev in all_variants().into_iter().skip(5).take(3) {
            assert_eq!(ev.aggregate_type(), "Story");
        }
    }

    #[test]
    fn aggregate_type_for_user_variants() {
        for ev in all_variants().into_iter().skip(8).take(3) {
            assert_eq!(ev.aggregate_type(), "User");
        }
    }

    #[test]
    fn aggregate_type_for_feature_variants() {
        for ev in all_variants().into_iter().skip(11).take(3) {
            assert_eq!(ev.aggregate_type(), "Feature");
        }
    }

    #[test]
    fn aggregate_type_for_work_package_variants() {
        for ev in all_variants().into_iter().skip(14).take(2) {
            assert_eq!(ev.aggregate_type(), "WorkPackage");
        }
    }

    #[test]
    fn envelope_new_derives_aggregate_type_from_payload() {
        let env = EventEnvelope::new(AggregateId(3), DomainEvent::ProjectArchived(ProjectArchived { project_id: 3.into() }));
        assert_eq!(env.aggregate_type, "Project");
        assert_eq!(env.aggregate_id, AggregateId(3));
        assert!(env.causation_id.is_none());
        assert!(env.correlation_id.is_none());
    }

    #[test]
    fn envelope_occurred_at_is_recent() {
        let before = Utc::now();
        let env = EventEnvelope::new(1i64, DomainEvent::ProjectArchived(ProjectArchived { project_id: 1.into() }));
        let after = Utc::now();
        assert!(env.occurred_at >= before && env.occurred_at <= after);
    }

    #[test]
    fn envelope_builders_set_ids() {
        let c = Uuid::new_v4();
        let r = Uuid::new_v4();
        let env = EventEnvelope::new(1i64, variant(0))
            .with_causation(c)
            .with_correlation(r);
        assert_eq!(env.causation_id, Some(c));
        assert_eq!(env.correlation_id, Some(r));
    }

    #[test]
    fn envelope_serde_roundtrip_preserves_optional_ids() {
        let env = EventEnvelope::new(9i64, variant(7))
            .with_causation(Uuid::new_v4())
            .with_correlation(Uuid::new_v4());
        let json = serde_json::to_string(&env).unwrap();
        let back: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, env);
    }

    #[test]
    fn envelope_clone_preserves_payload() {
        let env = EventEnvelope::new(1i64, variant(0));
        assert_eq!(env.clone().payload, env.payload);
    }

    #[test]
    fn all_variant_payloads_serde_roundtrip() {
        for ev in all_variants() {
            let env = EventEnvelope::new(1i64, ev);
            let json = serde_json::to_string(&env).unwrap();
            let back: EventEnvelope = serde_json::from_str(&json).unwrap();
            assert_eq!(back, env);
        }
    }

    #[test]
    fn story_assigned_none_assignee_roundtrip() {
        let ev = DomainEvent::StoryAssigned(StoryAssigned { story_id: 1.into(), assignee_id: None });
        let json = serde_json::to_string(&ev).unwrap();
        let back: DomainEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
    }

    #[test]
    fn handler_error_rejected_display() {
        let e = EventHandlerError::Rejected("no".into());
        assert!(e.to_string().contains("handler rejected event: no"));
    }

    #[test]
    fn handler_error_transient_display() {
        let e = EventHandlerError::Transient("later".into());
        assert!(e.to_string().contains("handler encountered a transient error: later"));
    }

    struct SpyHandler {
        seen: Arc<Mutex<usize>>,
        fail: bool,
    }

    impl EventHandler for SpyHandler {
        fn handle(&self, _envelope: &EventEnvelope) -> Result<(), EventHandlerError> {
            if self.fail {
                return Err(EventHandlerError::Rejected("forced".into()));
            }
            *self.seen.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[test]
    fn sync_handler_counts_events() {
        let seen = Arc::new(Mutex::new(0));
        let handler = SpyHandler { seen: seen.clone(), fail: false };
        handler.handle(&EventEnvelope::new(1i64, variant(0))).unwrap();
        handler.handle(&EventEnvelope::new(1i64, variant(1))).unwrap();
        assert_eq!(*seen.lock().unwrap(), 2);
    }

    #[test]
    fn sync_handler_can_reject() {
        let handler = SpyHandler { seen: Arc::new(Mutex::new(0)), fail: true };
        let err = handler.handle(&EventEnvelope::new(1i64, variant(0))).unwrap_err();
        assert!(matches!(err, EventHandlerError::Rejected(_)));
    }

    struct AsyncSpy {
        seen: Arc<Mutex<usize>>,
    }

    #[async_trait::async_trait]
    impl AsyncEventHandler for AsyncSpy {
        async fn handle(&self, _envelope: &EventEnvelope) -> Result<(), EventHandlerError> {
            *self.seen.lock().unwrap() += 1;
            Ok(())
        }
    }

    struct AsyncFanout {
        handlers: Vec<Arc<AsyncSpy>>,
    }

    #[async_trait::async_trait]
    impl AsyncEventBus for AsyncFanout {
        async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventHandlerError> {
            for h in &self.handlers {
                h.handle(&envelope).await?;
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn async_handler_receives_event() {
        let seen = Arc::new(Mutex::new(0));
        let h = AsyncSpy { seen: seen.clone() };
        h.handle(&EventEnvelope::new(1i64, variant(0))).await.unwrap();
        assert_eq!(*seen.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn async_event_bus_fans_out() {
        let a = Arc::new(AsyncSpy { seen: Arc::new(Mutex::new(0)) });
        let b = Arc::new(AsyncSpy { seen: Arc::new(Mutex::new(0)) });
        let bus = AsyncFanout { handlers: vec![a.clone(), b.clone()] };
        bus.publish(EventEnvelope::new(1i64, variant(0))).await.unwrap();
        assert_eq!(*a.seen.lock().unwrap(), 1);
        assert_eq!(*b.seen.lock().unwrap(), 1);
    }

    #[test]
    fn event_type_for_feature_state_advanced() {
        let ev = DomainEvent::FeatureStateAdvanced(FeatureStateAdvanced {
            feature_id: 1.into(),
            from: FeatureState::Planned,
            to: FeatureState::Implementing,
        });
        assert_eq!(ev.event_type(), "feature.state_advanced");
    }

    #[test]
    fn event_type_for_work_package_state_changed() {
        let ev = DomainEvent::WorkPackageStateChanged(WorkPackageStateChanged {
            wp_id: 1.into(),
            feature_id: 1.into(),
            from: WpState::Review,
            to: WpState::Done,
        });
        assert_eq!(ev.event_type(), "work_package.state_changed");
    }

    #[test]
    fn event_type_for_user_role_changed() {
        let ev = DomainEvent::UserRoleChanged(UserRoleChanged {
            user_id: 1.into(),
            old_role: UserRole::Member,
            new_role: UserRole::Admin,
        });
        assert_eq!(ev.event_type(), "user.role_changed");
    }

    #[test]
    fn event_type_for_story_assigned() {
        let ev = DomainEvent::StoryAssigned(StoryAssigned { story_id: 1.into(), assignee_id: None });
        assert_eq!(ev.event_type(), "story.assigned");
    }

    #[test]
    fn event_type_for_project_archived() {
        let ev = DomainEvent::ProjectArchived(ProjectArchived { project_id: 1.into() });
        assert_eq!(ev.event_type(), "project.archived");
    }

    // ── wire-format contract (consumed by other repos / brokers) ──────────────

    #[test]
    fn aggregate_id_serializes_as_bare_integer() {
        assert_eq!(serde_json::to_string(&AggregateId(7)).unwrap(), "7");
        assert_eq!(
            serde_json::from_str::<AggregateId>("7").unwrap(),
            AggregateId(7)
        );
    }

    #[test]
    fn envelope_json_exposes_the_documented_field_set() {
        let env = EventEnvelope::new(
            AggregateId(2),
            DomainEvent::ProjectArchived(ProjectArchived {
                project_id: AggregateId(2),
            }),
        );

        let value = serde_json::to_value(&env).unwrap();
        let object = value.as_object().expect("envelope serialises to an object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "aggregate_id",
                "aggregate_type",
                "causation_id",
                "correlation_id",
                "id",
                "occurred_at",
                "payload",
            ]
        );

        assert_eq!(object["id"], serde_json::json!(env.id.to_string()));
        assert_eq!(object["occurred_at"], serde_json::json!(env.occurred_at));
        assert_eq!(object["aggregate_id"], serde_json::json!(2));
        assert_eq!(object["aggregate_type"], serde_json::json!("Project"));
        assert_eq!(object["causation_id"], serde_json::Value::Null);
        assert_eq!(object["correlation_id"], serde_json::Value::Null);
        assert_eq!(
            object["payload"],
            serde_json::json!({"ProjectArchived": {"project_id": 2}})
        );
    }

    #[test]
    fn envelope_deserialises_from_literal_wire_json() {
        let id = Uuid::new_v4();
        let cause = Uuid::new_v4();
        let literal = format!(
            "{{\"id\":\"{id}\",\"occurred_at\":\"2026-03-02T00:00:00Z\",\"aggregate_id\":7,\"aggregate_type\":\"Feature\",\"causation_id\":\"{cause}\",\"correlation_id\":null,\"payload\":{{\"FeatureShipped\":{{\"feature_id\":7,\"slug\":\"f-7\"}}}}}}",
            id = id,
            cause = cause
        );

        let env: EventEnvelope = serde_json::from_str(&literal).unwrap();
        assert_eq!(env.id, id);
        assert_eq!(env.causation_id, Some(cause));
        assert_eq!(env.correlation_id, None);
        assert_eq!(env.aggregate_id, AggregateId(7));
        assert_eq!(
            env.occurred_at,
            DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        match &env.payload {
            DomainEvent::FeatureShipped(shipped) => {
                assert_eq!(shipped.feature_id, AggregateId(7));
                assert_eq!(shipped.slug, "f-7");
            }
            other => panic!("unexpected payload: {other:?}"),
        }
        assert_eq!(env.payload.event_type(), "feature.shipped");
        assert_eq!(env.aggregate_type, env.payload.aggregate_type());
    }
}
