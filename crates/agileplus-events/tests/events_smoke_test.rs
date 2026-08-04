//! Public API smoke tests for the in-process AgilePlus event bus.

use agileplus_events::{DomainEvent, EventBus};

#[test]
fn public_event_variants_round_trip_through_json() {
    let events = [
        DomainEvent::FeatureCreated { id: 1 },
        DomainEvent::FeatureStateChanged {
            id: 2,
            from: "draft".into(),
            to: "review".into(),
        },
        DomainEvent::CycleStarted {
            cycle_id: 3,
            module_id: 4,
        },
        DomainEvent::CycleEnded { cycle_id: 3 },
        DomainEvent::WorkPackageLinked {
            work_package_id: 5,
            feature_id: 1,
        },
        DomainEvent::UserLoggedIn {
            user_id: "user-6".into(),
        },
        DomainEvent::PlaneWebhookReceived {
            issue_id: "issue-7".into(),
            action: "updated".into(),
        },
        DomainEvent::Custom {
            name: "test.event".into(),
            payload: serde_json::json!({"key": "value"}),
        },
    ];

    for event in events {
        let encoded = serde_json::to_string(&event).expect("serialize event");
        let decoded: DomainEvent = serde_json::from_str(&encoded).expect("deserialize event");
        assert_eq!(decoded, event);
    }
}

#[tokio::test]
async fn public_bus_fans_out_to_each_subscriber() {
    let bus = EventBus::new(4);
    let mut first = bus.subscribe();
    let mut second = bus.subscribe();

    let event = DomainEvent::FeatureCreated { id: 42 };
    assert_eq!(bus.publish(event.clone()).expect("publish event"), 2);
    assert_eq!(first.recv().await.expect("first subscriber event"), event);
    assert_eq!(second.recv().await.expect("second subscriber event"), event);
}

#[tokio::test]
async fn public_async_publish_preserves_payload() {
    let bus = EventBus::new(1);
    let mut subscriber = bus.subscribe();
    let event = DomainEvent::PlaneWebhookReceived {
        issue_id: "issue-42".into(),
        action: "created".into(),
    };

    bus.publish_async(event.clone())
        .await
        .expect("publish event");
    assert_eq!(subscriber.recv().await.expect("subscriber event"), event);
}
