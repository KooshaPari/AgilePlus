//! Comprehensive unit tests for agileplus-events.
//!
//! Covers: EventBus, DomainEvent (both bus and domain_event variants),
//! hash computation/verification, EventStore, EventQuery, replay,
//! snapshot management, EventEnvelope, AggregateId, and error types.
//!
//! Target: 50+ tests, 1000+ lines.

use agileplus_domain::domain::epic::EpicStatus;
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::StoryStatus;
use agileplus_domain::domain::user::{UserRole, UserStatus};
use agileplus_domain::domain::work_package::WpState;
use agileplus_events::bus::{DomainEvent as BusDomainEvent, EventBus};
use agileplus_events::domain_event::{
    AggregateId, DomainEvent, EpicCreated, EpicStatusChanged,
    EventEnvelope, EventHandler, EventHandlerError, FeatureCreated, FeatureShipped,
    FeatureStateAdvanced, ProjectArchived, ProjectCreated, ProjectRenamed, StoryAssigned,
    StoryCreated, StoryStatusChanged, UserAdded, UserRoleChanged, UserStatusChanged,
    WorkPackageCreated, WorkPackageStateChanged,
};
use agileplus_events::hash::{compute_hash, verify_chain, HashError};
use agileplus_events::query::EventQuery;
use agileplus_events::replay::{replay_events, replay_events_since, Aggregate, ReplayError};
use agileplus_events::snapshot::{
    InMemorySnapshotStore, LoadedState, SnapshotConfig, SnapshotStore,
    should_snapshot,
};
use agileplus_events::store::{EventError, EventStore, InMemoryEventStore};
use agileplus_events::EventSourcingError;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

// ══════════════════════════════════════════════════════════════════════════════
// Helpers
// ══════════════════════════════════════════════════════════════════════════════

fn make_event(seq: i64, entity_type: &str, entity_id: i64, event_type: &str, actor: &str) -> Event {
    Event {
        id: seq,
        entity_type: entity_type.into(),
        entity_id,
        event_type: event_type.into(),
        payload: serde_json::json!({"seq": seq}),
        actor: actor.into(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: seq,
    }
}

fn make_event_at(
    seq: i64,
    entity_type: &str,
    entity_id: i64,
    event_type: &str,
    actor: &str,
    timestamp: DateTime<Utc>,
) -> Event {
    Event {
        id: seq,
        entity_type: entity_type.into(),
        entity_id,
        event_type: event_type.into(),
        payload: serde_json::json!({"seq": seq}),
        actor: actor.into(),
        timestamp,
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: seq,
    }
}

fn make_snapshot(entity_type: &str, entity_id: i64, seq: i64) -> agileplus_domain::domain::snapshot::Snapshot {
    agileplus_domain::domain::snapshot::Snapshot::new(
        entity_type,
        entity_id,
        serde_json::json!({"state": "test", "seq": seq}),
        seq,
    )
}

/// Build a valid chain of events with correct hashes.
fn build_hash_chain(count: usize, entity_id: i64) -> Vec<Event> {
    let ts = DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let mut events = Vec::new();
    let mut prev_hash = [0u8; 32];

    for i in 0..count {
        let seq = (i + 1) as i64;
        let payload = serde_json::json!({"step": i});
        let hash = compute_hash(entity_id, "Feature", "created", &payload, ts, "tester", &prev_hash)
            .unwrap();
        events.push(Event {
            id: seq,
            entity_type: "Feature".into(),
            entity_id,
            event_type: "created".into(),
            payload,
            actor: "tester".into(),
            timestamp: ts,
            prev_hash,
            hash,
            sequence: seq,
        });
        prev_hash = hash;
    }
    events
}

// ══════════════════════════════════════════════════════════════════════════════
// 1. EventBus (bus.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn bus_publish_no_subscribers_returns_err() {
    let bus = EventBus::new(8);
    let result = bus.publish(BusDomainEvent::FeatureCreatedLegacy { id: 1 });
    assert!(result.is_err(), "publishing with no subscribers should fail");
}

#[tokio::test]
async fn bus_publish_delivers_to_subscriber() {
    let bus = EventBus::new(8);
    let mut sub = bus.subscribe();
    bus.publish(BusDomainEvent::UserLoggedIn {
        user_id: "u1".into(),
    })
    .unwrap();
    let ev = tokio::time::timeout(std::time::Duration::from_millis(100), sub.recv())
        .await
        .expect("timeout")
        .expect("recv error");
    match ev {
        BusDomainEvent::UserLoggedIn { user_id } => assert_eq!(user_id, "u1"),
        other => panic!("unexpected variant: {other:?}"),
    }
}

#[tokio::test]
async fn bus_subscriber_count_after_drops() {
    let bus = EventBus::new(4);
    assert_eq!(bus.subscriber_count(), 0);
    {
        let _s1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);
        let _s2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }
    // After drops, count should be 0
    assert_eq!(bus.subscriber_count(), 0);
}

#[tokio::test]
async fn bus_multiple_subscribers_each_receive_independent_copy() {
    let bus = EventBus::new(8);
    let mut a = bus.subscribe();
    let mut b = bus.subscribe();
    let mut c = bus.subscribe();

    bus.publish(BusDomainEvent::CycleStarted {
        cycle_id: 10,
        module_id: 20,
    })
    .unwrap();

    let ev_a = a.recv().await.unwrap();
    let ev_b = b.recv().await.unwrap();
    let ev_c = c.recv().await.unwrap();
    assert_eq!(ev_a, ev_b);
    assert_eq!(ev_b, ev_c);
}

#[tokio::test]
async fn bus_publishes_received_in_order() {
    let bus = EventBus::new(16);
    let mut sub = bus.subscribe();

    let events = vec![
        BusDomainEvent::FeatureCreatedLegacy { id: 1 },
        BusDomainEvent::FeatureCreatedLegacy { id: 2 },
        BusDomainEvent::FeatureCreatedLegacy { id: 3 },
    ];

    for ev in &events {
        bus.publish(ev.clone()).unwrap();
    }

    for expected in &events {
        let received = sub.recv().await.unwrap();
        assert_eq!(&received, expected);
    }
}

#[tokio::test]
async fn bus_try_recv_returns_none_when_empty() {
    let bus = EventBus::new(4);
    let mut sub = bus.subscribe();
    assert!(sub.try_recv().is_none());
}

#[tokio::test]
async fn bus_try_recv_returns_event_after_publish() {
    let bus = EventBus::new(4);
    let mut sub = bus.subscribe();
    bus.publish(BusDomainEvent::PlaneWebhookReceived {
        issue_id: "i1".into(),
        action: "opened".into(),
    })
    .unwrap();
    let result = sub.try_recv();
    assert!(result.is_some());
    let ev = result.unwrap().unwrap();
    match ev {
        BusDomainEvent::PlaneWebhookReceived { issue_id, action } => {
            assert_eq!(issue_id, "i1");
            assert_eq!(action, "opened");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn bus_serde_round_trip_all_bus_variants() {
    let variants = vec![
        BusDomainEvent::FeatureCreatedLegacy { id: 1 },
        BusDomainEvent::FeatureStateChanged {
            id: 2,
            from: "a".into(),
            to: "b".into(),
        },
        BusDomainEvent::CycleStarted {
            cycle_id: 3,
            module_id: 4,
        },
        BusDomainEvent::CycleEnded { cycle_id: 5 },
        BusDomainEvent::WorkPackageLinked {
            work_package_id: 6,
            feature_id: 7,
        },
        BusDomainEvent::UserLoggedIn {
            user_id: "u1".into(),
        },
        BusDomainEvent::PlaneWebhookReceived {
            issue_id: "i1".into(),
            action: "closed".into(),
        },
    ];

    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let decoded: BusDomainEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, &decoded, "round-trip failed for {json}");
    }
}

#[tokio::test]
async fn bus_domain_event_all_event_types() {
    let variants = vec![
        (BusDomainEvent::FeatureCreatedLegacy { id: 1 }, "feature.created"),
        (
            BusDomainEvent::FeatureStateChanged { id: 1, from: "a".into(), to: "b".into() },
            "feature.state_changed",
        ),
        (BusDomainEvent::CycleStarted { cycle_id: 1, module_id: 1 }, "cycle.started"),
        (BusDomainEvent::CycleEnded { cycle_id: 1 }, "cycle.ended"),
        (
            BusDomainEvent::WorkPackageLinked { work_package_id: 1, feature_id: 1 },
            "work_package.linked",
        ),
        (BusDomainEvent::UserLoggedIn { user_id: "x".into() }, "user.logged_in"),
        (
            BusDomainEvent::PlaneWebhookReceived { issue_id: "x".into(), action: "y".into() },
            "plane.webhook_received",
        ),
    ];

    for (variant, expected_type) in &variants {
        assert_eq!(variant.event_type(), *expected_type);
    }
}

#[tokio::test]
async fn bus_domain_event_all_aggregate_types() {
    let variants = vec![
        (BusDomainEvent::FeatureCreatedLegacy { id: 1 }, "Feature"),
        (
            BusDomainEvent::FeatureStateChanged { id: 1, from: "a".into(), to: "b".into() },
            "Feature",
        ),
        (BusDomainEvent::CycleStarted { cycle_id: 1, module_id: 1 }, "Cycle"),
        (BusDomainEvent::CycleEnded { cycle_id: 1 }, "Cycle"),
        (
            BusDomainEvent::WorkPackageLinked { work_package_id: 1, feature_id: 1 },
            "WorkPackage",
        ),
        (BusDomainEvent::UserLoggedIn { user_id: "x".into() }, "User"),
        (
            BusDomainEvent::PlaneWebhookReceived { issue_id: "x".into(), action: "y".into() },
            "Plane",
        ),
    ];

    for (variant, expected_type) in &variants {
        assert_eq!(variant.aggregate_type(), *expected_type);
    }
}

#[tokio::test]
async fn bus_domain_event_typed_variant_aggregate_types() {
    let typed = vec![
        (
            BusDomainEvent::ProjectCreated(ProjectCreated {
                project_id: AggregateId(1),
                slug: "s".into(),
                name: "n".into(),
            }),
            "Project",
        ),
        (
            BusDomainEvent::ProjectRenamed(ProjectRenamed {
                project_id: AggregateId(1),
                old_name: "o".into(),
                new_name: "n".into(),
            }),
            "Project",
        ),
        (
            BusDomainEvent::ProjectArchived(ProjectArchived {
                project_id: AggregateId(1),
            }),
            "Project",
        ),
        (
            BusDomainEvent::EpicCreated(EpicCreated {
                epic_id: AggregateId(1),
                project_id: AggregateId(2),
                title: "t".into(),
            }),
            "Epic",
        ),
        (
            BusDomainEvent::EpicStatusChanged(EpicStatusChanged {
                epic_id: AggregateId(1),
                project_id: AggregateId(2),
                from: EpicStatus::Backlog,
                to: EpicStatus::Active,
            }),
            "Epic",
        ),
        (
            BusDomainEvent::StoryCreated(StoryCreated {
                story_id: AggregateId(1),
                epic_id: AggregateId(2),
                project_id: AggregateId(3),
                title: "t".into(),
                points: Some(5),
            }),
            "Story",
        ),
        (
            BusDomainEvent::StoryStatusChanged(StoryStatusChanged {
                story_id: AggregateId(1),
                epic_id: AggregateId(2),
                from: StoryStatus::Todo,
                to: StoryStatus::InProgress,
            }),
            "Story",
        ),
        (
            BusDomainEvent::StoryAssigned(StoryAssigned {
                story_id: AggregateId(1),
                assignee_id: Some(AggregateId(2)),
            }),
            "Story",
        ),
        (
            BusDomainEvent::UserAdded(UserAdded {
                user_id: AggregateId(1),
                display_name: "A".into(),
                email: "a@b.com".into(),
                role: UserRole::Admin,
            }),
            "User",
        ),
        (
            BusDomainEvent::UserRoleChanged(UserRoleChanged {
                user_id: AggregateId(1),
                old_role: UserRole::Member,
                new_role: UserRole::Admin,
            }),
            "User",
        ),
        (
            BusDomainEvent::UserStatusChanged(UserStatusChanged {
                user_id: AggregateId(1),
                from: UserStatus::Active,
                to: UserStatus::Suspended,
            }),
            "User",
        ),
        (
            BusDomainEvent::FeatureCreated(FeatureCreated {
                feature_id: AggregateId(1),
                slug: "s".into(),
                friendly_name: "f".into(),
                project_id: Some(AggregateId(2)),
            }),
            "Feature",
        ),
        (
            BusDomainEvent::FeatureStateAdvanced(FeatureStateAdvanced {
                feature_id: AggregateId(1),
                from: FeatureState::Created,
                to: FeatureState::Specified,
            }),
            "Feature",
        ),
        (
            BusDomainEvent::FeatureShipped(FeatureShipped {
                feature_id: AggregateId(1),
                slug: "s".into(),
            }),
            "Feature",
        ),
        (
            BusDomainEvent::WorkPackageCreated(WorkPackageCreated {
                wp_id: AggregateId(1),
                feature_id: AggregateId(2),
                title: "t".into(),
                sequence: 1,
            }),
            "WorkPackage",
        ),
        (
            BusDomainEvent::WorkPackageStateChanged(WorkPackageStateChanged {
                wp_id: AggregateId(1),
                feature_id: AggregateId(2),
                from: WpState::Planned,
                to: WpState::Doing,
            }),
            "WorkPackage",
        ),
    ];

    for (variant, expected) in &typed {
        assert_eq!(variant.aggregate_type(), *expected);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 2. DomainEvent (domain_event.rs) — serde roundtrips for ALL variants
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn de_project_created_serde_roundtrip() {
    let ev = DomainEvent::ProjectCreated(ProjectCreated {
        project_id: AggregateId(1),
        slug: "proj".into(),
        name: "Project".into(),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_project_renamed_serde_roundtrip() {
    let ev = DomainEvent::ProjectRenamed(ProjectRenamed {
        project_id: AggregateId(1),
        old_name: "Old".into(),
        new_name: "New".into(),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_project_archived_serde_roundtrip() {
    let ev = DomainEvent::ProjectArchived(ProjectArchived {
        project_id: AggregateId(1),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_epic_created_serde_roundtrip() {
    let ev = DomainEvent::EpicCreated(EpicCreated {
        epic_id: AggregateId(2),
        project_id: AggregateId(1),
        title: "My Epic".into(),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_epic_status_changed_serde_roundtrip() {
    let ev = DomainEvent::EpicStatusChanged(EpicStatusChanged {
        epic_id: AggregateId(2),
        project_id: AggregateId(1),
        from: EpicStatus::Backlog,
        to: EpicStatus::Active,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_story_created_serde_roundtrip() {
    let ev = DomainEvent::StoryCreated(StoryCreated {
        story_id: AggregateId(42),
        epic_id: AggregateId(2),
        project_id: AggregateId(1),
        title: "Login".into(),
        points: Some(3),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_story_created_no_points_serde_roundtrip() {
    let ev = DomainEvent::StoryCreated(StoryCreated {
        story_id: AggregateId(42),
        epic_id: AggregateId(2),
        project_id: AggregateId(1),
        title: "Login".into(),
        points: None,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    match decoded {
        DomainEvent::StoryCreated(s) => assert!(s.points.is_none()),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn de_story_status_changed_serde_roundtrip() {
    let ev = DomainEvent::StoryStatusChanged(StoryStatusChanged {
        story_id: AggregateId(42),
        epic_id: AggregateId(2),
        from: StoryStatus::Todo,
        to: StoryStatus::InProgress,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_story_assigned_serde_roundtrip() {
    let ev = DomainEvent::StoryAssigned(StoryAssigned {
        story_id: AggregateId(42),
        assignee_id: Some(AggregateId(5)),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_story_unassigned_serde_roundtrip() {
    let ev = DomainEvent::StoryAssigned(StoryAssigned {
        story_id: AggregateId(42),
        assignee_id: None,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    match decoded {
        DomainEvent::StoryAssigned(s) => assert!(s.assignee_id.is_none()),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn de_user_added_serde_roundtrip() {
    let ev = DomainEvent::UserAdded(UserAdded {
        user_id: AggregateId(99),
        display_name: "Alice".into(),
        email: "alice@example.com".into(),
        role: UserRole::Member,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_user_role_changed_serde_roundtrip() {
    let ev = DomainEvent::UserRoleChanged(UserRoleChanged {
        user_id: AggregateId(5),
        old_role: UserRole::Member,
        new_role: UserRole::Admin,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_user_status_changed_serde_roundtrip() {
    let ev = DomainEvent::UserStatusChanged(UserStatusChanged {
        user_id: AggregateId(5),
        from: UserStatus::Active,
        to: UserStatus::Suspended,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_feature_created_serde_roundtrip() {
    let ev = DomainEvent::FeatureCreated(FeatureCreated {
        feature_id: AggregateId(10),
        slug: "feat-slug".into(),
        friendly_name: "Feature Name".into(),
        project_id: Some(AggregateId(1)),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_feature_created_no_project_serde_roundtrip() {
    let ev = DomainEvent::FeatureCreated(FeatureCreated {
        feature_id: AggregateId(10),
        slug: "feat".into(),
        friendly_name: "Name".into(),
        project_id: None,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    match decoded {
        DomainEvent::FeatureCreated(f) => assert!(f.project_id.is_none()),
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn de_feature_state_advanced_serde_roundtrip() {
    let ev = DomainEvent::FeatureStateAdvanced(FeatureStateAdvanced {
        feature_id: AggregateId(10),
        from: FeatureState::Created,
        to: FeatureState::Specified,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_feature_shipped_serde_roundtrip() {
    let ev = DomainEvent::FeatureShipped(FeatureShipped {
        feature_id: AggregateId(10),
        slug: "shipped-feat".into(),
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_work_package_created_serde_roundtrip() {
    let ev = DomainEvent::WorkPackageCreated(WorkPackageCreated {
        wp_id: AggregateId(20),
        feature_id: AggregateId(3),
        title: "Implement login".into(),
        sequence: 1,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

#[test]
fn de_work_package_state_changed_serde_roundtrip() {
    let ev = DomainEvent::WorkPackageStateChanged(WorkPackageStateChanged {
        wp_id: AggregateId(20),
        feature_id: AggregateId(3),
        from: WpState::Planned,
        to: WpState::Doing,
    });
    let json = serde_json::to_string(&ev).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, decoded);
}

// ══════════════════════════════════════════════════════════════════════════════
// 3. DomainEvent event_type and aggregate_type for all variants
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn de_all_event_types_are_unique() {
    let events = vec![
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
        DomainEvent::ProjectRenamed(ProjectRenamed {
            project_id: AggregateId(1),
            old_name: "o".into(),
            new_name: "n".into(),
        }),
        DomainEvent::ProjectArchived(ProjectArchived {
            project_id: AggregateId(1),
        }),
        DomainEvent::EpicCreated(EpicCreated {
            epic_id: AggregateId(1),
            project_id: AggregateId(2),
            title: "t".into(),
        }),
        DomainEvent::EpicStatusChanged(EpicStatusChanged {
            epic_id: AggregateId(1),
            project_id: AggregateId(2),
            from: EpicStatus::Backlog,
            to: EpicStatus::Active,
        }),
        DomainEvent::StoryCreated(StoryCreated {
            story_id: AggregateId(1),
            epic_id: AggregateId(2),
            project_id: AggregateId(3),
            title: "t".into(),
            points: Some(1),
        }),
        DomainEvent::StoryStatusChanged(StoryStatusChanged {
            story_id: AggregateId(1),
            epic_id: AggregateId(2),
            from: StoryStatus::Todo,
            to: StoryStatus::InProgress,
        }),
        DomainEvent::StoryAssigned(StoryAssigned {
            story_id: AggregateId(1),
            assignee_id: None,
        }),
        DomainEvent::UserAdded(UserAdded {
            user_id: AggregateId(1),
            display_name: "A".into(),
            email: "a@b".into(),
            role: UserRole::Admin,
        }),
        DomainEvent::UserRoleChanged(UserRoleChanged {
            user_id: AggregateId(1),
            old_role: UserRole::Member,
            new_role: UserRole::Admin,
        }),
        DomainEvent::UserStatusChanged(UserStatusChanged {
            user_id: AggregateId(1),
            from: UserStatus::Active,
            to: UserStatus::Suspended,
        }),
        DomainEvent::FeatureCreated(FeatureCreated {
            feature_id: AggregateId(1),
            slug: "s".into(),
            friendly_name: "f".into(),
            project_id: None,
        }),
        DomainEvent::FeatureStateAdvanced(FeatureStateAdvanced {
            feature_id: AggregateId(1),
            from: FeatureState::Created,
            to: FeatureState::Specified,
        }),
        DomainEvent::FeatureShipped(FeatureShipped {
            feature_id: AggregateId(1),
            slug: "s".into(),
        }),
        DomainEvent::WorkPackageCreated(WorkPackageCreated {
            wp_id: AggregateId(1),
            feature_id: AggregateId(2),
            title: "t".into(),
            sequence: 1,
        }),
        DomainEvent::WorkPackageStateChanged(WorkPackageStateChanged {
            wp_id: AggregateId(1),
            feature_id: AggregateId(2),
            from: WpState::Planned,
            to: WpState::Doing,
        }),
    ];

    let mut seen = std::collections::HashSet::new();
    for ev in &events {
        let et = ev.event_type();
        assert!(seen.insert(et), "duplicate event_type: {et}");
    }
    assert_eq!(seen.len(), events.len());
}

#[test]
fn de_feature_variants_aggregate_type() {
    let cases = vec![
        (
            DomainEvent::FeatureCreated(FeatureCreated {
                feature_id: AggregateId(1),
                slug: "s".into(),
                friendly_name: "f".into(),
                project_id: None,
            }),
            "Feature",
        ),
        (
            DomainEvent::FeatureStateAdvanced(FeatureStateAdvanced {
                feature_id: AggregateId(1),
                from: FeatureState::Created,
                to: FeatureState::Specified,
            }),
            "Feature",
        ),
        (
            DomainEvent::FeatureShipped(FeatureShipped {
                feature_id: AggregateId(1),
                slug: "s".into(),
            }),
            "Feature",
        ),
    ];
    for (ev, expected) in &cases {
        assert_eq!(ev.aggregate_type(), *expected);
    }
}

#[test]
fn de_work_package_variants_aggregate_type() {
    let cases = vec![
        (
            DomainEvent::WorkPackageCreated(WorkPackageCreated {
                wp_id: AggregateId(1),
                feature_id: AggregateId(2),
                title: "t".into(),
                sequence: 1,
            }),
            "WorkPackage",
        ),
        (
            DomainEvent::WorkPackageStateChanged(WorkPackageStateChanged {
                wp_id: AggregateId(1),
                feature_id: AggregateId(2),
                from: WpState::Planned,
                to: WpState::Doing,
            }),
            "WorkPackage",
        ),
    ];
    for (ev, expected) in &cases {
        assert_eq!(ev.aggregate_type(), *expected);
    }
}

#[test]
fn de_wire_code_matches_event_type() {
    let events = vec![
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
        DomainEvent::EpicStatusChanged(EpicStatusChanged {
            epic_id: AggregateId(1),
            project_id: AggregateId(2),
            from: EpicStatus::Backlog,
            to: EpicStatus::Active,
        }),
        DomainEvent::FeatureShipped(FeatureShipped {
            feature_id: AggregateId(1),
            slug: "s".into(),
        }),
    ];
    for ev in &events {
        assert_eq!(ev.wire_code(), ev.event_type());
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 4. AggregateId
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn aggregate_id_from_i64() {
    let id = AggregateId::from(42i64);
    assert_eq!(id.0, 42);
}

#[test]
fn aggregate_id_serde_roundtrip() {
    let id = AggregateId(100);
    let json = serde_json::to_string(&id).unwrap();
    let decoded: AggregateId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, decoded);
}

#[test]
fn aggregate_id_equality() {
    assert_eq!(AggregateId(1), AggregateId(1));
    assert_ne!(AggregateId(1), AggregateId(2));
}

#[test]
fn aggregate_id_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    AggregateId(42).hash(&mut h1);
    AggregateId(42).hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn aggregate_id_clone() {
    let id = AggregateId(7);
    let cloned = id;
    assert_eq!(id, cloned);
}

// ══════════════════════════════════════════════════════════════════════════════
// 5. EventEnvelope
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn envelope_new_creates_with_uuid_and_timestamp() {
    let payload = DomainEvent::ProjectCreated(ProjectCreated {
        project_id: AggregateId(1),
        slug: "s".into(),
        name: "n".into(),
    });
    let env = EventEnvelope::new(AggregateId(1), payload);
    assert!(!env.id.is_nil());
    assert_eq!(env.aggregate_id, AggregateId(1));
    assert_eq!(env.aggregate_type, "Project");
    assert!(env.causation_id.is_none());
    assert!(env.correlation_id.is_none());
    // Timestamp should be recent
    let now = Utc::now();
    let diff = now.signed_duration_since(env.occurred_at);
    assert!(diff.num_seconds() < 5);
}

#[test]
fn envelope_with_causation_builder() {
    let cause = Uuid::new_v4();
    let env = EventEnvelope::new(
        AggregateId(1),
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
    )
    .with_causation(cause);
    assert_eq!(env.causation_id, Some(cause));
    assert!(env.correlation_id.is_none());
}

#[test]
fn envelope_with_correlation_builder() {
    let corr = Uuid::new_v4();
    let env = EventEnvelope::new(
        AggregateId(1),
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
    )
    .with_correlation(corr);
    assert!(env.causation_id.is_none());
    assert_eq!(env.correlation_id, Some(corr));
}

#[test]
fn envelope_chained_builders() {
    let cause = Uuid::new_v4();
    let corr = Uuid::new_v4();
    let env = EventEnvelope::new(
        AggregateId(1),
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
    )
    .with_causation(cause)
    .with_correlation(corr);
    assert_eq!(env.causation_id, Some(cause));
    assert_eq!(env.correlation_id, Some(corr));
}

#[test]
fn envelope_uuids_unique_across_instances() {
    let payload = DomainEvent::ProjectCreated(ProjectCreated {
        project_id: AggregateId(1),
        slug: "s".into(),
        name: "n".into(),
    });
    let a = EventEnvelope::new(AggregateId(1), payload.clone());
    let b = EventEnvelope::new(AggregateId(1), payload);
    assert_ne!(a.id, b.id);
}

#[test]
fn envelope_serde_roundtrip_preserves_all_fields() {
    let cause = Uuid::new_v4();
    let corr = Uuid::new_v4();
    let env = EventEnvelope::new(
        AggregateId(42),
        DomainEvent::EpicCreated(EpicCreated {
            epic_id: AggregateId(10),
            project_id: AggregateId(1),
            title: "Test".into(),
        }),
    )
    .with_causation(cause)
    .with_correlation(corr);

    let json = serde_json::to_string(&env).unwrap();
    let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.id, env.id);
    assert_eq!(decoded.aggregate_id, AggregateId(42));
    assert_eq!(decoded.aggregate_type, "Epic");
    assert_eq!(decoded.occurred_at, env.occurred_at);
    assert_eq!(decoded.causation_id, Some(cause));
    assert_eq!(decoded.correlation_id, Some(corr));
}

// ══════════════════════════════════════════════════════════════════════════════
// 6. EventHandler port
// ══════════════════════════════════════════════════════════════════════════════

struct RecordingHandler {
    received: std::sync::Mutex<Vec<EventEnvelope>>,
}

impl RecordingHandler {
    fn new() -> Self {
        Self {
            received: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn count(&self) -> usize {
        self.received.lock().unwrap().len()
    }

    fn last_aggregate_type(&self) -> String {
        self.received
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .aggregate_type
            .clone()
    }
}

impl EventHandler for RecordingHandler {
    fn handle(&self, envelope: &EventEnvelope) -> Result<(), EventHandlerError> {
        self.received.lock().unwrap().push(envelope.clone());
        Ok(())
    }
}

struct FailingHandler;

impl EventHandler for FailingHandler {
    fn handle(&self, _envelope: &EventEnvelope) -> Result<(), EventHandlerError> {
        Err(EventHandlerError::Rejected("not allowed".into()))
    }
}

#[test]
fn handler_receives_envelope() {
    let handler = RecordingHandler::new();
    let env = EventEnvelope::new(
        AggregateId(1),
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
    );
    handler.handle(&env).unwrap();
    assert_eq!(handler.count(), 1);
    assert_eq!(handler.last_aggregate_type(), "Project");
}

#[test]
fn handler_reject_returns_error() {
    let handler = FailingHandler;
    let env = EventEnvelope::new(
        AggregateId(1),
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "s".into(),
            name: "n".into(),
        }),
    );
    let result = handler.handle(&env);
    assert!(result.is_err());
}

#[test]
fn handler_error_display() {
    let e1 = EventHandlerError::Rejected("bad".into());
    assert!(e1.to_string().contains("bad"));
    let e2 = EventHandlerError::Transient("retry".into());
    assert!(e2.to_string().contains("retry"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 7. Hash computation and chain verification
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hash_computation_deterministic() {
    let ts = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let payload = serde_json::json!({"key": "value"});
    let h1 = compute_hash(1, "Feature", "created", &payload, ts, "alice", &[0u8; 32]).unwrap();
    let h2 = compute_hash(1, "Feature", "created", &payload, ts, "alice", &[0u8; 32]).unwrap();
    assert_eq!(h1, h2);
    assert_ne!(h1, [0u8; 32]);
}

#[test]
fn hash_differs_with_different_entity_id() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    let h2 = compute_hash(2, "F", "c", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_with_different_entity_type() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "Feature", "c", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    let h2 =
        compute_hash(1, "WorkPackage", "c", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_with_different_event_type() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "F", "created", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    let h2 = compute_hash(1, "F", "shipped", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_with_different_payload() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "F", "c", &serde_json::json!({"a": 1}), ts, "a", &[0u8; 32]).unwrap();
    let h2 = compute_hash(1, "F", "c", &serde_json::json!({"a": 2}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_with_different_actor() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "alice", &[0u8; 32]).unwrap();
    let h2 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "bob", &[0u8; 32]).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_with_different_prev_hash() {
    let ts = Utc::now();
    let mut p1 = [0u8; 32];
    p1[0] = 1;
    let mut p2 = [0u8; 32];
    p2[0] = 2;
    let h1 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &p1).unwrap();
    let h2 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &p2).unwrap();
    assert_ne!(h1, h2);
}

#[test]
fn verify_chain_empty_ok() {
    verify_chain(&[]).unwrap();
}

#[test]
fn verify_chain_single_valid() {
    let events = build_hash_chain(1, 1);
    verify_chain(&events).unwrap();
}

#[test]
fn verify_chain_5_events_valid() {
    let events = build_hash_chain(5, 42);
    verify_chain(&events).unwrap();
}

#[test]
fn verify_chain_fails_on_corrupted_hash() {
    let mut events = build_hash_chain(3, 1);
    events[1].hash[0] ^= 0xFF;
    assert!(verify_chain(&events).is_err());
}

#[test]
fn verify_chain_fails_on_broken_chain_link() {
    let mut events = build_hash_chain(3, 1);
    events[1].prev_hash[0] ^= 0xFF;
    assert!(verify_chain(&events).is_err());
}

#[test]
fn verify_chain_fails_on_sequence_gap() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    let h2 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &h1).unwrap();

    let events = vec![
        Event {
            id: 1,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "c".into(),
            payload: serde_json::json!({}),
            actor: "a".into(),
            timestamp: ts,
            prev_hash: [0u8; 32],
            hash: h1,
            sequence: 1,
        },
        Event {
            id: 2,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "c".into(),
            payload: serde_json::json!({}),
            actor: "a".into(),
            timestamp: ts,
            prev_hash: h1,
            hash: h2,
            sequence: 5, // Gap: should be 2
        },
    ];

    let result = verify_chain(&events);
    assert!(result.is_err());
    match result.unwrap_err() {
        HashError::ChainBroken { sequence } => assert_eq!(sequence, 5),
        other => panic!("expected ChainBroken, got {other:?}"),
    }
}

#[test]
fn verify_chain_fails_on_first_event_nonzero_prev_hash() {
    let ts = Utc::now();
    let mut bad_prev = [0u8; 32];
    bad_prev[0] = 42;

    let events = vec![Event {
        id: 1,
        entity_type: "F".into(),
        entity_id: 1,
        event_type: "c".into(),
        payload: serde_json::json!({}),
        actor: "a".into(),
        timestamp: ts,
        prev_hash: bad_prev,
        hash: [0u8; 32],
        sequence: 1,
    }];

    let result = verify_chain(&events);
    assert!(result.is_err());
    match result.unwrap_err() {
        HashError::ChainBroken { sequence } => assert_eq!(sequence, 1),
        other => panic!("expected ChainBroken, got {other:?}"),
    }
}

#[test]
fn verify_chain_tampered_payload_detected() {
    let mut events = build_hash_chain(3, 1);
    events[2].payload = serde_json::json!({"tampered": true});
    let result = verify_chain(&events);
    assert!(result.is_err());
    match result.unwrap_err() {
        HashError::HashMismatch { sequence } => assert_eq!(sequence, 3),
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

#[test]
fn hash_error_display_contains_sequence() {
    let e1 = HashError::ChainBroken { sequence: 5 };
    assert!(e1.to_string().contains("5"));
    let e2 = HashError::HashMismatch { sequence: 3 };
    assert!(e2.to_string().contains("3"));
    let e3 = HashError::InvalidHashLength(16);
    assert!(e3.to_string().contains("16"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 8. EventStore (InMemoryEventStore)
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn store_new_is_empty() {
    let store = InMemoryEventStore::new();
    let events = store.get_events("Feature", 1).await.unwrap();
    assert!(events.is_empty());
    let seq = store.get_latest_sequence("Feature", 1).await.unwrap();
    assert_eq!(seq, 0);
}

#[tokio::test]
async fn store_append_assigns_incrementing_sequence() {
    let store = InMemoryEventStore::new();
    let seq1 = store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    let seq2 = store
        .append(&make_event(0, "Feature", 1, "updated", "a"))
        .await
        .unwrap();
    let seq3 = store
        .append(&make_event(0, "Feature", 1, "shipped", "a"))
        .await
        .unwrap();
    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    assert_eq!(seq3, 3);
}

#[tokio::test]
async fn store_independent_entities_have_separate_sequences() {
    let store = InMemoryEventStore::new();
    let s1 = store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    let s2 = store
        .append(&make_event(0, "Feature", 2, "created", "a"))
        .await
        .unwrap();
    let s3 = store
        .append(&make_event(0, "WorkPackage", 1, "created", "a"))
        .await
        .unwrap();
    assert_eq!(s1, 1);
    assert_eq!(s2, 1);
    assert_eq!(s3, 1);
}

#[tokio::test]
async fn store_get_events_returns_sorted_by_sequence() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "Feature", 1, "shipped", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "Feature", 1, "specified", "a"))
        .await
        .unwrap();

    let events = store.get_events("Feature", 1).await.unwrap();
    assert_eq!(events.len(), 3);
    assert!(events[0].sequence < events[1].sequence);
    assert!(events[1].sequence < events[2].sequence);
}

#[tokio::test]
async fn store_get_events_does_not_mix_entity_types() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "WorkPackage", 1, "created", "a"))
        .await
        .unwrap();

    let f = store.get_events("Feature", 1).await.unwrap();
    let w = store.get_events("WorkPackage", 1).await.unwrap();
    assert_eq!(f.len(), 1);
    assert_eq!(w.len(), 1);
}

#[tokio::test]
async fn store_get_events_empty_for_unknown() {
    let store = InMemoryEventStore::new();
    let events = store.get_events("Feature", 999).await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn store_get_events_since_filters_correctly() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "Feature", 1, "updated", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "Feature", 1, "shipped", "a"))
        .await
        .unwrap();

    let events = store.get_events_since("Feature", 1, 1).await.unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.sequence > 1));
}

#[tokio::test]
async fn store_get_events_since_beyond_last_returns_empty() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();

    let events = store.get_events_since("Feature", 1, 100).await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn store_get_events_by_range_filters_timestamp() {
    let store = InMemoryEventStore::new();
    let early = Utc::now() - Duration::hours(3);
    let mid = Utc::now() - Duration::hours(1);
    let late = Utc::now() + Duration::hours(1);

    let mut e1 = make_event(0, "Feature", 1, "created", "a");
    e1.timestamp = early;
    store.append(&e1).await.unwrap();

    let mut e2 = make_event(0, "Feature", 1, "updated", "a");
    e2.timestamp = mid;
    store.append(&e2).await.unwrap();

    // Range: early-1h to mid+1h → includes both
    let range_events = store
        .get_events_by_range(
            "Feature",
            1,
            early - Duration::hours(1),
            mid + Duration::hours(1),
        )
        .await
        .unwrap();
    assert_eq!(range_events.len(), 2);

    // Range: only late → empty
    let late_events = store
        .get_events_by_range("Feature", 1, late, late + Duration::hours(1))
        .await
        .unwrap();
    assert!(late_events.is_empty());
}

#[tokio::test]
async fn store_get_latest_sequence_returns_zero_for_empty() {
    let store = InMemoryEventStore::new();
    assert_eq!(
        store.get_latest_sequence("Feature", 1).await.unwrap(),
        0
    );
}

#[tokio::test]
async fn store_get_latest_sequence_returns_last() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "Feature", 1, "shipped", "a"))
        .await
        .unwrap();
    assert_eq!(
        store.get_latest_sequence("Feature", 1).await.unwrap(),
        2
    );
}

#[tokio::test]
async fn store_get_events_by_type_filters_correctly() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "WorkPackage", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event(0, "Feature", 1, "updated", "a"))
        .await
        .unwrap();

    let created = store.get_events_by_type("created").await.unwrap();
    assert_eq!(created.len(), 2);
    assert!(created.iter().all(|e| e.event_type == "created"));
}

#[tokio::test]
async fn store_get_events_by_type_empty_when_not_found() {
    let store = InMemoryEventStore::new();
    store
        .append(&make_event(0, "Feature", 1, "created", "a"))
        .await
        .unwrap();
    let events = store.get_events_by_type("nonexistent").await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn store_append_overwrites_event_sequence() {
    let store = InMemoryEventStore::new();
    let mut ev = make_event(999, "Feature", 1, "created", "a");
    ev.sequence = 999;
    let seq = store.append(&ev).await.unwrap();
    assert_eq!(seq, 1); // Should be assigned 1, not 999
}

// ══════════════════════════════════════════════════════════════════════════════
// 9. EventQuery
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn query_new_matches_all() {
    let events = vec![
        make_event(1, "F", 1, "c", "a"),
        make_event(2, "W", 2, "c", "b"),
    ];
    assert_eq!(EventQuery::new().filter(&events).len(), 2);
}

#[test]
fn query_entity_type_filter() {
    let events = vec![
        make_event(1, "Feature", 1, "c", "a"),
        make_event(2, "WorkPackage", 1, "c", "a"),
        make_event(3, "Feature", 2, "c", "a"),
    ];
    let result = EventQuery::new().entity_type("Feature").filter(&events);
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|e| e.entity_type == "Feature"));
}

#[test]
fn query_entity_id_filter() {
    let events = vec![
        make_event(1, "F", 1, "c", "a"),
        make_event(2, "F", 2, "c", "a"),
        make_event(3, "F", 1, "c", "a"),
    ];
    let result = EventQuery::new().entity_id(1).filter(&events);
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|e| e.entity_id == 1));
}

#[test]
fn query_event_type_filter() {
    let events = vec![
        make_event(1, "F", 1, "created", "a"),
        make_event(2, "F", 1, "shipped", "a"),
        make_event(3, "F", 2, "created", "a"),
    ];
    let result = EventQuery::new().event_type("shipped").filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event_type, "shipped");
}

#[test]
fn query_actor_filter() {
    let events = vec![
        make_event(1, "F", 1, "c", "alice"),
        make_event(2, "F", 1, "c", "bob"),
        make_event(3, "F", 1, "c", "alice"),
    ];
    let result = EventQuery::new().actor("bob").filter(&events);
    assert_eq!(result.len(), 1);
}

#[test]
fn query_time_range_filter() {
    let now = Utc::now();
    let events = vec![
        make_event_at(1, "F", 1, "c", "a", now - Duration::hours(3)),
        make_event_at(2, "F", 1, "c", "a", now - Duration::hours(1)),
        make_event_at(3, "F", 1, "c", "a", now + Duration::hours(1)),
    ];
    let result = EventQuery::new()
        .start_time(now - Duration::hours(2))
        .end_time(now)
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sequence, 2);
}

#[test]
fn query_only_start_time() {
    let now = Utc::now();
    let events = vec![
        make_event_at(1, "F", 1, "c", "a", now - Duration::hours(5)),
        make_event_at(2, "F", 1, "c", "a", now),
    ];
    let result = EventQuery::new()
        .start_time(now - Duration::hours(2))
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sequence, 2);
}

#[test]
fn query_only_end_time() {
    let now = Utc::now();
    let events = vec![
        make_event_at(1, "F", 1, "c", "a", now),
        make_event_at(2, "F", 1, "c", "a", now + Duration::hours(5)),
    ];
    let result = EventQuery::new()
        .end_time(now + Duration::hours(2))
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sequence, 1);
}

#[test]
fn query_sequence_range() {
    let events = vec![
        make_event(1, "F", 1, "c", "a"),
        make_event(2, "F", 1, "c", "a"),
        make_event(3, "F", 1, "c", "a"),
        make_event(4, "F", 1, "c", "a"),
    ];
    let result = EventQuery::new()
        .after_sequence(2)
        .end_sequence(3)
        .filter(&events);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].sequence, 2);
    assert_eq!(result[1].sequence, 3);
}

#[test]
fn query_limit_restricts_results() {
    let events = vec![
        make_event(1, "F", 1, "c", "a"),
        make_event(2, "F", 1, "c", "a"),
        make_event(3, "F", 1, "c", "a"),
    ];
    let result = EventQuery::new().limit(2).filter(&events);
    assert_eq!(result.len(), 2);
}

#[test]
fn query_limit_zero_returns_empty() {
    let events = vec![make_event(1, "F", 1, "c", "a")];
    let result = EventQuery::new().limit(0).filter(&events);
    assert!(result.is_empty());
}

#[test]
fn query_combined_filters_narrow() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "Feature", 2, "created", "bob"),
        make_event(3, "WorkPackage", 1, "created", "alice"),
        make_event(4, "Feature", 1, "shipped", "alice"),
    ];
    let result = EventQuery::new()
        .entity_type("Feature")
        .actor("alice")
        .event_type("created")
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].entity_id, 1);
}

#[test]
fn query_no_matches_returns_empty() {
    let events = vec![make_event(1, "Feature", 1, "created", "alice")];
    let result = EventQuery::new()
        .entity_type("WorkPackage")
        .filter(&events);
    assert!(result.is_empty());
}

#[test]
fn query_empty_input_returns_empty() {
    let events: Vec<Event> = vec![];
    let result = EventQuery::new().entity_type("Feature").filter(&events);
    assert!(result.is_empty());
}

#[test]
fn query_all_filters_combined() {
    let now = Utc::now();
    let events = vec![
        make_event_at(1, "Feature", 1, "created", "alice", now - Duration::hours(2)),
        make_event_at(2, "Feature", 1, "updated", "alice", now),
        make_event_at(3, "WorkPackage", 1, "created", "alice", now),
    ];
    let result = EventQuery::new()
        .entity_type("Feature")
        .entity_id(1)
        .event_type("created")
        .actor("alice")
        .after_sequence(0)
        .end_sequence(10)
        .start_time(now - Duration::hours(3))
        .end_time(now + Duration::hours(1))
        .limit(5)
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sequence, 1);
}

// ══════════════════════════════════════════════════════════════════════════════
// 10. Replay
// ══════════════════════════════════════════════════════════════════════════════

fn make_replay_event(seq: i64, entity_id: i64, payload: serde_json::Value) -> Event {
    Event {
        id: seq,
        entity_type: "T".into(),
        entity_id,
        event_type: "U".into(),
        payload,
        actor: "test".into(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: seq,
    }
}

struct TestAggregate {
    version: i64,
    count: i64,
    last_payload: serde_json::Value,
}

impl TestAggregate {
    fn new() -> Self {
        Self {
            version: 0,
            count: 0,
            last_payload: serde_json::json!({}),
        }
    }
}

#[async_trait]
impl Aggregate for TestAggregate {
    async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
        self.count += 1;
        self.last_payload = event.payload.clone();
        Ok(())
    }
    fn version(&self) -> i64 {
        self.version
    }
    fn set_version(&mut self, v: i64) {
        self.version = v;
    }
}

struct FailingAggregate {
    version: i64,
    fail_on: i64,
}

#[async_trait]
impl Aggregate for FailingAggregate {
    async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
        if event.sequence == self.fail_on {
            return Err(ReplayError::AggregateError("boom".into()));
        }
        self.version = event.sequence;
        Ok(())
    }
    fn version(&self) -> i64 {
        self.version
    }
    fn set_version(&mut self, v: i64) {
        self.version = v;
    }
}

#[tokio::test]
async fn replay_empty_is_noop() {
    let mut agg = TestAggregate::new();
    replay_events(&mut agg, &[]).await.unwrap();
    assert_eq!(agg.version, 0);
    assert_eq!(agg.count, 0);
}

#[tokio::test]
async fn replay_applies_all_events() {
    let mut agg = TestAggregate::new();
    let events = vec![
        make_replay_event(1, 1, serde_json::json!({"v": 1})),
        make_replay_event(2, 1, serde_json::json!({"v": 2})),
        make_replay_event(3, 1, serde_json::json!({"v": 3})),
    ];
    replay_events(&mut agg, &events).await.unwrap();
    assert_eq!(agg.version, 3);
    assert_eq!(agg.count, 3);
    assert_eq!(agg.last_payload, serde_json::json!({"v": 3}));
}

#[tokio::test]
async fn replay_single_event_sets_version() {
    let mut agg = TestAggregate::new();
    let events = vec![make_replay_event(5, 10, serde_json::json!({"x": 1}))];
    replay_events(&mut agg, &events).await.unwrap();
    assert_eq!(agg.version, 5);
    assert_eq!(agg.count, 1);
}

#[tokio::test]
async fn replay_rejects_mixed_entities() {
    let mut agg = TestAggregate::new();
    let events = vec![
        make_replay_event(1, 1, serde_json::json!({})),
        make_replay_event(2, 2, serde_json::json!({})),
    ];
    let result = replay_events(&mut agg, &events).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ReplayError::InvalidState(msg) => assert!(msg.contains("different entities")),
        other => panic!("expected InvalidState, got {other:?}"),
    }
}

#[tokio::test]
async fn replay_propagates_aggregate_errors() {
    let mut agg = FailingAggregate { version: 0, fail_on: 2 };
    let events = vec![
        make_replay_event(1, 1, serde_json::json!({})),
        make_replay_event(2, 1, serde_json::json!({})),
    ];
    let result = replay_events(&mut agg, &events).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ReplayError::AggregateError(msg) => assert_eq!(msg, "boom"),
        other => panic!("expected AggregateError, got {other:?}"),
    }
    assert_eq!(agg.version, 1);
}

#[tokio::test]
async fn replay_since_filters_old_events() {
    let mut agg = TestAggregate::new();
    let events = vec![
        make_replay_event(1, 1, serde_json::json!({"v": 1})),
        make_replay_event(2, 1, serde_json::json!({"v": 2})),
        make_replay_event(3, 1, serde_json::json!({"v": 3})),
        make_replay_event(4, 1, serde_json::json!({"v": 4})),
    ];
    replay_events_since(&mut agg, 2, &events).await.unwrap();
    assert_eq!(agg.version, 4);
    assert_eq!(agg.count, 2);
    assert_eq!(agg.last_payload, serde_json::json!({"v": 4}));
}

#[tokio::test]
async fn replay_since_zero_applies_all() {
    let mut agg = TestAggregate::new();
    let events = vec![
        make_replay_event(1, 1, serde_json::json!({})),
        make_replay_event(2, 1, serde_json::json!({})),
    ];
    replay_events_since(&mut agg, 0, &events).await.unwrap();
    assert_eq!(agg.count, 2);
}

#[tokio::test]
async fn replay_since_beyond_all_applies_none() {
    let mut agg = TestAggregate::new();
    let events = vec![
        make_replay_event(1, 1, serde_json::json!({})),
        make_replay_event(2, 1, serde_json::json!({})),
    ];
    replay_events_since(&mut agg, 100, &events).await.unwrap();
    assert_eq!(agg.count, 0);
    assert_eq!(agg.version, 0);
}

#[tokio::test]
async fn replay_since_empty_events() {
    let mut agg = TestAggregate::new();
    replay_events_since(&mut agg, 5, &[]).await.unwrap();
    assert_eq!(agg.version, 0);
}

#[test]
fn replay_error_display() {
    let e1 = ReplayError::AggregateError("bad".into());
    assert!(e1.to_string().contains("bad"));
    let e2 = ReplayError::InvalidState("mixed".into());
    assert!(e2.to_string().contains("mixed"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 11. Snapshot
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn should_snapshot_event_threshold_true() {
    let config = SnapshotConfig {
        event_threshold: 10,
        time_threshold_secs: 300,
    };
    assert!(should_snapshot(&config, 10, 0, None));
}

#[test]
fn should_snapshot_event_threshold_false() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    assert!(!should_snapshot(&config, 50, 0, None));
}

#[test]
fn should_snapshot_time_threshold_true() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    let old = Utc::now() - Duration::seconds(400);
    assert!(should_snapshot(&config, 5, 0, Some(old)));
}

#[test]
fn should_snapshot_time_threshold_false() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    let recent = Utc::now() - Duration::seconds(10);
    assert!(!should_snapshot(&config, 5, 0, Some(recent)));
}

#[test]
fn should_snapshot_event_threshold_priority() {
    let config = SnapshotConfig {
        event_threshold: 5,
        time_threshold_secs: 300,
    };
    let recent = Utc::now();
    assert!(should_snapshot(&config, 10, 4, Some(recent)));
}

#[test]
fn should_snapshot_no_time_none_below_threshold() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    assert!(!should_snapshot(&config, 50, 0, None));
}

#[test]
fn snapshot_config_default() {
    let config = SnapshotConfig::default();
    assert_eq!(config.event_threshold, 100);
    assert_eq!(config.time_threshold_secs, 300);
}

#[test]
fn snapshot_config_clone() {
    let config = SnapshotConfig {
        event_threshold: 50,
        time_threshold_secs: 60,
    };
    let cloned = config.clone();
    assert_eq!(cloned.event_threshold, 50);
    assert_eq!(cloned.time_threshold_secs, 60);
}

#[tokio::test]
async fn snapshot_store_save_and_load_latest() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 20)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 30)).await.unwrap();
    let loaded = store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 30);
}

#[tokio::test]
async fn snapshot_store_load_none_for_unknown() {
    let store = InMemorySnapshotStore::new();
    assert!(store.load("Feature", 999).await.unwrap().is_none());
}

#[tokio::test]
async fn snapshot_store_load_none_for_different_type() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    assert!(store.load("WorkPackage", 1).await.unwrap().is_none());
}

#[tokio::test]
async fn snapshot_store_delete_before_removes_old() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 20)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 30)).await.unwrap();
    store.delete_before("Feature", 1, 20).await.unwrap();
    let loaded = store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 30);
}

#[tokio::test]
async fn snapshot_store_delete_before_no_panic_on_unknown() {
    let store = InMemorySnapshotStore::new();
    store
        .delete_before("Feature", 999, 10)
        .await
        .unwrap();
}

#[tokio::test]
async fn snapshot_store_independent_entities() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("WorkPackage", 1, 20)).await.unwrap();
    let f = store.load("Feature", 1).await.unwrap().unwrap();
    let w = store.load("WorkPackage", 1).await.unwrap().unwrap();
    assert_eq!(f.event_sequence, 10);
    assert_eq!(w.event_sequence, 20);
}

#[tokio::test]
async fn loaded_state_without_snapshot_returns_all_events() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();
    event_store
        .append(&make_event(0, "Feature", 1, "c", "a"))
        .await
        .unwrap();
    event_store
        .append(&make_event(0, "Feature", 1, "c", "a"))
        .await
        .unwrap();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1)
        .await
        .unwrap();
    assert!(state.snapshot.is_none());
    assert_eq!(state.events_to_replay.len(), 2);
}

#[tokio::test]
async fn loaded_state_with_snapshot_returns_only_newer() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();
    for _i in 0..5 {
        event_store
            .append(&make_event(0, "Feature", 1, "c", "a"))
            .await
            .unwrap();
    }
    snap_store
        .save(&make_snapshot("Feature", 1, 3))
        .await
        .unwrap();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1)
        .await
        .unwrap();
    assert!(state.snapshot.is_some());
    assert_eq!(state.snapshot.as_ref().unwrap().event_sequence, 3);
    assert_eq!(state.events_to_replay.len(), 2);
}

#[tokio::test]
async fn loaded_state_with_snapshot_no_newer_events() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();
    event_store
        .append(&make_event(0, "Feature", 1, "c", "a"))
        .await
        .unwrap();
    snap_store
        .save(&make_snapshot("Feature", 1, 1))
        .await
        .unwrap();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1)
        .await
        .unwrap();
    assert!(state.snapshot.is_some());
    assert!(state.events_to_replay.is_empty());
}

#[tokio::test]
async fn loaded_state_no_events_no_snapshot() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();
    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1)
        .await
        .unwrap();
    assert!(state.snapshot.is_none());
    assert!(state.events_to_replay.is_empty());
}

// ══════════════════════════════════════════════════════════════════════════════
// 12. EventSourcingError conversions
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn sourcing_error_from_store_error() {
    let e = EventSourcingError::Store(EventError::NotFound("x".into()));
    assert!(e.to_string().contains("x"));
}

#[test]
fn sourcing_error_from_hash_error() {
    let e = EventSourcingError::Hash(HashError::ChainBroken { sequence: 1 });
    assert!(e.to_string().contains("1"));
}

#[test]
fn sourcing_error_from_replay_error() {
    let e = EventSourcingError::Replay(ReplayError::AggregateError("bad".into()));
    assert!(e.to_string().contains("bad"));
}

#[test]
fn sourcing_error_from_snapshot_error() {
    let e = EventSourcingError::Snapshot(agileplus_events::snapshot::SnapshotError::NotFound {
        entity_type: "Feature".into(),
        entity_id: 1,
    });
    assert!(e.to_string().contains("Feature"));
}

#[test]
fn sourcing_error_from_query_error() {
    let e = EventSourcingError::Query(agileplus_events::query::QueryError::Error("q".into()));
    assert!(e.to_string().contains("q"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 13. EventError Display
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn event_error_not_found_display() {
    let e = EventError::NotFound("e42".into());
    assert!(e.to_string().contains("e42"));
}

#[test]
fn event_error_duplicate_sequence_display() {
    let e = EventError::DuplicateSequence("dup".into());
    assert!(e.to_string().contains("dup"));
}

#[test]
fn event_error_storage_error_display() {
    let e = EventError::StorageError("io".into());
    assert!(e.to_string().contains("io"));
}

#[test]
fn event_error_invalid_hash_display() {
    let e = EventError::InvalidHash("bad".into());
    assert!(e.to_string().contains("bad"));
}

#[test]
fn event_error_sequence_gap_display() {
    let e = EventError::SequenceGap {
        expected: 3,
        actual: 5,
    };
    let msg = e.to_string();
    assert!(msg.contains("3"));
    assert!(msg.contains("5"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 14. SnapshotError Display
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_error_not_found_display() {
    let e = agileplus_events::snapshot::SnapshotError::NotFound {
        entity_type: "Feature".into(),
        entity_id: 42,
    };
    let msg = e.to_string();
    assert!(msg.contains("Feature"));
    assert!(msg.contains("42"));
}

#[test]
fn snapshot_error_storage_display() {
    let e = agileplus_events::snapshot::SnapshotError::StorageError("disk".into());
    assert!(e.to_string().contains("disk"));
}

#[test]
fn snapshot_error_invalid_display() {
    let e = agileplus_events::snapshot::SnapshotError::Invalid("bad snap".into());
    assert!(e.to_string().contains("bad snap"));
}
