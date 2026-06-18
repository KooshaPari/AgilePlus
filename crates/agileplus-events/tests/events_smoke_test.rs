//! Smoke tests for DomainEvent and EventEnvelope serialization.
//!
//! Verifies:
//! - DomainEvent construction
//! - EventEnvelope wrapping
//! - serde_json round-trip (serialize → deserialize)
//! - Traceability: FR-008 / WP02

use agileplus_events::{
    AggregateId, DomainEvent, EventEnvelope, ProjectCreated, StoryCreated, UserAdded,
};
use agileplus_domain::domain::user::UserRole;

#[test]
fn test_domain_event_construction() {
    // Construct a ProjectCreated event
    let project_id = AggregateId(1);
    let event = DomainEvent::ProjectCreated(ProjectCreated {
        project_id,
        slug: "my-project".to_string(),
        name: "My Project".to_string(),
    });

    // Verify event_type and aggregate_type accessors
    assert_eq!(event.event_type(), "project.created");
    assert_eq!(event.aggregate_type(), "Project");
}

#[test]
fn test_story_created_event() {
    let story_id = AggregateId(42);
    let epic_id = AggregateId(10);
    let project_id = AggregateId(1);

    let event = DomainEvent::StoryCreated(StoryCreated {
        story_id,
        epic_id,
        project_id,
        title: "As a user, I want to...".to_string(),
        points: Some(5),
    });

    assert_eq!(event.event_type(), "story.created");
    assert_eq!(event.aggregate_type(), "Story");
}

#[test]
fn test_user_added_event() {
    let user_id = AggregateId(99);

    let event = DomainEvent::UserAdded(UserAdded {
        user_id,
        display_name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        role: UserRole::Member,
    });

    assert_eq!(event.event_type(), "user.added");
    assert_eq!(event.aggregate_type(), "User");
}

#[test]
fn test_event_envelope_wrapping() {
    let project_id = AggregateId(5);
    let payload = DomainEvent::ProjectCreated(ProjectCreated {
        project_id,
        slug: "test-proj".to_string(),
        name: "Test Project".to_string(),
    });

    let envelope = EventEnvelope::new(project_id, payload.clone());

    assert_eq!(envelope.aggregate_id, project_id);
    assert_eq!(envelope.aggregate_type, "Project");
    assert!(envelope.id.is_nil() == false); // UUIDs should be generated
    assert!(envelope.causation_id.is_none());
    assert!(envelope.correlation_id.is_none());
}

#[test]
fn test_event_envelope_serde_round_trip() {
    let project_id = AggregateId(7);
    let payload = DomainEvent::ProjectCreated(ProjectCreated {
        project_id,
        slug: "round-trip-test".to_string(),
        name: "Round Trip Test".to_string(),
    });

    let envelope = EventEnvelope::new(project_id, payload);

    // Serialize to JSON
    let json = serde_json::to_string(&envelope).expect("Failed to serialize envelope");

    // Deserialize back
    let deserialized: EventEnvelope =
        serde_json::from_str(&json).expect("Failed to deserialize envelope");

    // Verify round-trip integrity
    assert_eq!(deserialized.id, envelope.id);
    assert_eq!(deserialized.aggregate_id, envelope.aggregate_id);
    assert_eq!(deserialized.aggregate_type, envelope.aggregate_type);
    assert_eq!(deserialized.occurred_at, envelope.occurred_at);
}

#[test]
fn test_multiple_event_types_serde() {
    let events = vec![
        DomainEvent::ProjectCreated(ProjectCreated {
            project_id: AggregateId(1),
            slug: "proj1".to_string(),
            name: "Project 1".to_string(),
        }),
        DomainEvent::StoryCreated(StoryCreated {
            story_id: AggregateId(10),
            epic_id: AggregateId(5),
            project_id: AggregateId(1),
            title: "Story 1".to_string(),
            points: Some(3),
        }),
        DomainEvent::UserAdded(UserAdded {
            user_id: AggregateId(100),
            display_name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            role: UserRole::Admin,
        }),
    ];

    for event in events {
        let json = serde_json::to_string(&event).expect("Failed to serialize event");
        let _deserialized: DomainEvent =
            serde_json::from_str(&json).expect("Failed to deserialize event");
        // If we get here without panicking, the round-trip succeeded
    }
}

#[test]
fn test_event_envelope_with_correlation_ids() {
    let project_id = AggregateId(12);
    let payload = DomainEvent::ProjectCreated(ProjectCreated {
        project_id,
        slug: "corr-test".to_string(),
        name: "Correlation Test".to_string(),
    });

    let mut envelope = EventEnvelope::new(project_id, payload);

    // Set correlation IDs manually
    let causation_uuid = uuid::Uuid::new_v4();
    let correlation_uuid = uuid::Uuid::new_v4();
    envelope.causation_id = Some(causation_uuid);
    envelope.correlation_id = Some(correlation_uuid);

    // Verify they persist through serialization
    let json = serde_json::to_string(&envelope).expect("Failed to serialize");
    let deserialized: EventEnvelope =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.causation_id, Some(causation_uuid));
    assert_eq!(deserialized.correlation_id, Some(correlation_uuid));
}
