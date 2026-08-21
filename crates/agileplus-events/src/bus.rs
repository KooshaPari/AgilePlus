//! In-process pub/sub event bus backed by `tokio::sync::broadcast`.
//!
//! Use this when multiple subsystems need to react to a domain event in
//! real time (e.g. webhook fan-out, cockpit updates, plane sync). The
//! append-only `EventStore` in this crate handles durability + replay;
//! the `EventBus` handles live fan-out.

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::SendError;

use super::domain_event::{
    EpicCreated, EpicStatusChanged, FeatureCreated, FeatureShipped, FeatureStateAdvanced,
    ProjectArchived, ProjectCreated, ProjectRenamed, StoryAssigned, StoryCreated,
    StoryStatusChanged, UserAdded, UserRoleChanged, UserStatusChanged, WorkPackageCreated,
    WorkPackageStateChanged,
};

/// Domain event variants that flow through the bus. Each variant wraps a
/// typed struct from `domain_event.rs` so that downstream consumers get
/// strong typing and serde round-trip safety.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DomainEvent {
    // --- Existing bus-level events ---
    FeatureCreatedLegacy {
        id: i64,
    },
    FeatureStateChanged {
        id: i64,
        from: String,
        to: String,
    },
    CycleStarted {
        cycle_id: i64,
        module_id: i64,
    },
    CycleEnded {
        cycle_id: i64,
    },
    WorkPackageLinked {
        work_package_id: i64,
        feature_id: i64,
    },
    UserLoggedIn {
        user_id: String,
    },
    PlaneWebhookReceived {
        issue_id: String,
        action: String,
    },
    // --- Typed variants wrapping domain_event structs ---
    ProjectCreated(ProjectCreated),
    ProjectRenamed(ProjectRenamed),
    ProjectArchived(ProjectArchived),
    EpicCreated(EpicCreated),
    EpicStatusChanged(EpicStatusChanged),
    StoryCreated(StoryCreated),
    StoryStatusChanged(StoryStatusChanged),
    StoryAssigned(StoryAssigned),
    UserAdded(UserAdded),
    UserRoleChanged(UserRoleChanged),
    UserStatusChanged(UserStatusChanged),
    FeatureCreated(FeatureCreated),
    FeatureStateAdvanced(FeatureStateAdvanced),
    FeatureShipped(FeatureShipped),
    WorkPackageCreated(WorkPackageCreated),
    WorkPackageStateChanged(WorkPackageStateChanged),
}

impl DomainEvent {
    /// A short machine-readable tag like `"project.created"`.
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::FeatureCreatedLegacy { .. } => "feature.created",
            Self::FeatureStateChanged { .. } => "feature.state_changed",
            Self::CycleStarted { .. } => "cycle.started",
            Self::CycleEnded { .. } => "cycle.ended",
            Self::WorkPackageLinked { .. } => "work_package.linked",
            Self::UserLoggedIn { .. } => "user.logged_in",
            Self::PlaneWebhookReceived { .. } => "plane.webhook_received",
            Self::ProjectCreated(_) => "project.created",
            Self::ProjectRenamed(_) => "project.renamed",
            Self::ProjectArchived(_) => "project.archived",
            Self::EpicCreated(_) => "epic.created",
            Self::EpicStatusChanged(_) => "epic.status_changed",
            Self::StoryCreated(_) => "story.created",
            Self::StoryStatusChanged(_) => "story.status_changed",
            Self::StoryAssigned(_) => "story.assigned",
            Self::UserAdded(_) => "user.added",
            Self::UserRoleChanged(_) => "user.role_changed",
            Self::UserStatusChanged(_) => "user.status_changed",
            Self::FeatureCreated(_) => "feature.created",
            Self::FeatureStateAdvanced(_) => "feature.state_advanced",
            Self::FeatureShipped(_) => "feature.shipped",
            Self::WorkPackageCreated(_) => "work_package.created",
            Self::WorkPackageStateChanged(_) => "work_package.state_changed",
        }
    }

    /// The aggregate root type (e.g. `"Project"`, `"Feature"`).
    pub fn aggregate_type(&self) -> &'static str {
        match self {
            Self::FeatureCreatedLegacy { .. } => "Feature",
            Self::FeatureStateChanged { .. } => "Feature",
            Self::CycleStarted { .. } => "Cycle",
            Self::CycleEnded { .. } => "Cycle",
            Self::WorkPackageLinked { .. } => "WorkPackage",
            Self::UserLoggedIn { .. } => "User",
            Self::PlaneWebhookReceived { .. } => "Plane",
            Self::ProjectCreated(_) => "Project",
            Self::ProjectRenamed(_) => "Project",
            Self::ProjectArchived(_) => "Project",
            Self::EpicCreated(_) => "Epic",
            Self::EpicStatusChanged(_) => "Epic",
            Self::StoryCreated(_) => "Story",
            Self::StoryStatusChanged(_) => "Story",
            Self::StoryAssigned(_) => "Story",
            Self::UserAdded(_) => "User",
            Self::UserRoleChanged(_) => "User",
            Self::UserStatusChanged(_) => "User",
            Self::FeatureCreated(_) => "Feature",
            Self::FeatureStateAdvanced(_) => "Feature",
            Self::FeatureShipped(_) => "Feature",
            Self::WorkPackageCreated(_) => "WorkPackage",
            Self::WorkPackageStateChanged(_) => "WorkPackage",
        }
    }
}

/// In-process event bus. Publish an event and every active subscriber
/// receives a copy.
pub struct EventBus {
    tx: broadcast::Sender<DomainEvent>,
}

impl EventBus {
    /// Create a new bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event to all subscribers. Returns `Err(event)` if there
    /// are no active subscribers.
    pub fn publish(&self, event: DomainEvent) -> Result<(), SendError<DomainEvent>> {
        self.tx.send(event).map(|_| ())
    }

    /// Create a new subscriber handle.
    pub fn subscribe(&self) -> EventSubscriber {
        EventSubscriber {
            inner: self.tx.subscribe(),
        }
    }

    /// Number of active subscriber handles.
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// A handle to the event stream. Each subscriber receives every event
/// independently. Not `Clone` — receivers must be obtained from
/// `EventBus::subscribe()`.
pub struct EventSubscriber {
    inner: broadcast::Receiver<DomainEvent>,
}

impl EventSubscriber {
    /// Receive the next event, awaiting if necessary.
    pub async fn recv(&mut self) -> Result<DomainEvent, broadcast::error::RecvError> {
        self.inner.recv().await
    }

    /// Non-blocking variant. Returns `None` if no event is currently buffered.
    pub fn try_recv(&mut self) -> Option<Result<DomainEvent, broadcast::error::RecvError>> {
        match self.inner.try_recv() {
            Ok(ev) => Some(Ok(ev)),
            Err(broadcast::error::TryRecvError::Empty) => None,
            Err(broadcast::error::TryRecvError::Lagged(n)) => {
                Some(Err(broadcast::error::RecvError::Lagged(n)))
            }
            Err(broadcast::error::TryRecvError::Closed) => {
                Some(Err(broadcast::error::RecvError::Closed))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn publish_delivers_to_subscriber() {
        let bus = EventBus::new(8);
        let mut sub = bus.subscribe();

        bus.publish(DomainEvent::FeatureCreatedLegacy { id: 42 }).unwrap();

        let ev = tokio::time::timeout(Duration::from_millis(100), sub.recv())
            .await
            .expect("no timeout")
            .expect("no recv error");
        assert_eq!(ev, DomainEvent::FeatureCreatedLegacy { id: 42 });
    }

    #[tokio::test]
    async fn multiple_subscribers_each_receive() {
        let bus = EventBus::new(8);
        let mut a = bus.subscribe();
        let mut b = bus.subscribe();

        bus.publish(DomainEvent::CycleStarted {
            cycle_id: 1,
            module_id: 2,
        })
        .unwrap();

        let ev_a = a.recv().await.unwrap();
        let ev_b = b.recv().await.unwrap();
        assert_eq!(ev_a, ev_b);
    }

    #[tokio::test]
    async fn subscriber_count_reflects_handles() {
        let bus = EventBus::new(4);
        assert_eq!(bus.subscriber_count(), 0);
        let _s1 = bus.subscribe();
        let _s2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }

    #[tokio::test]
    async fn try_recv_returns_none_when_empty() {
        let bus = EventBus::new(4);
        let mut sub = bus.subscribe();
        assert!(sub.try_recv().is_none());
    }

    #[tokio::test]
    async fn serde_round_trip_on_event() {
        let ev = DomainEvent::FeatureStateChanged {
            id: 7,
            from: "draft".into(),
            to: "review".into(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: DomainEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }
}
