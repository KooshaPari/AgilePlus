//! Comprehensive tests for the streaming module.
//!
//! Complements the inline unit tests in `streaming.rs`.
//!
//! Traceability: WP14-T083

use agileplus_grpc::event_bus::{AgentEvent, EventBus};
use agileplus_grpc::streaming::{agent_event_stream, domain_event_to_proto};

// ---------------------------------------------------------------------------
// domain_event_to_proto - all event types
// ---------------------------------------------------------------------------

#[test]
fn proto_agent_started_fields() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "feat-a".into(),
        wp_sequence: 5,
        agent_id: "agent-7".into(),
    };
    let proto = domain_event_to_proto(e);
    assert_eq!(proto.event_type, "agent_started");
    assert_eq!(proto.feature_slug, "feat-a");
    assert_eq!(proto.wp_sequence, 5);
    assert_eq!(proto.agent_id, "agent-7");
    assert!(!proto.timestamp.is_empty());
    assert!(!proto.payload.is_empty());
}

#[test]
fn proto_pr_created_fields() {
    let e = AgentEvent::PrCreated {
        feature_slug: "feat-b".into(),
        wp_sequence: 3,
        pr_url: "https://github.com/pr/42".into(),
    };
    let proto = domain_event_to_proto(e);
    assert_eq!(proto.event_type, "pr_created");
    assert_eq!(proto.feature_slug, "feat-b");
    assert_eq!(proto.wp_sequence, 3);
    assert_eq!(proto.agent_id, ""); // PrCreated has no agent_id
}

#[test]
fn proto_review_received_fields() {
    let e = AgentEvent::ReviewReceived {
        feature_slug: "feat-c".into(),
        wp_sequence: 7,
        review_status: "approved".into(),
        comments: 12,
    };
    let proto = domain_event_to_proto(e);
    assert_eq!(proto.event_type, "review_received");
    assert_eq!(proto.feature_slug, "feat-c");
    assert_eq!(proto.wp_sequence, 7);
    assert_eq!(proto.agent_id, "");
}

#[test]
fn proto_agent_fixing_fields() {
    let e = AgentEvent::AgentFixing {
        feature_slug: "feat-d".into(),
        wp_sequence: 2,
        cycle: 4,
    };
    let proto = domain_event_to_proto(e);
    assert_eq!(proto.event_type, "agent_fixing");
    assert_eq!(proto.feature_slug, "feat-d");
    assert_eq!(proto.wp_sequence, 2);
}

#[test]
fn proto_agent_completed_fields() {
    let e = AgentEvent::AgentCompleted {
        feature_slug: "feat-e".into(),
        wp_sequence: 9,
        success: true,
    };
    let proto = domain_event_to_proto(e);
    assert_eq!(proto.event_type, "agent_completed");
    assert_eq!(proto.feature_slug, "feat-e");
    assert_eq!(proto.wp_sequence, 9);
}

#[test]
fn proto_wp_state_changed_fields() {
    let e = AgentEvent::WpStateChanged {
        feature_slug: "feat-f".into(),
        wp_sequence: 1,
        old_state: "planned".into(),
        new_state: "doing".into(),
    };
    let proto = domain_event_to_proto(e);
    assert_eq!(proto.event_type, "wp_state_changed");
    assert_eq!(proto.feature_slug, "feat-f");
    assert_eq!(proto.wp_sequence, 1);
}

#[test]
fn proto_timestamp_is_rfc3339() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    let proto = domain_event_to_proto(e);
    assert!(proto.timestamp.contains('T'));
    assert!(proto.timestamp.starts_with("20"));
}

#[test]
fn proto_payload_is_valid_json() {
    let e = AgentEvent::AgentCompleted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        success: false,
    };
    let proto = domain_event_to_proto(e);
    let parsed: serde_json::Value =
        serde_json::from_str(&proto.payload).expect("payload should be valid JSON");
    assert!(parsed.is_object());
}

// ---------------------------------------------------------------------------
// agent_event_stream - filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stream_filters_out_non_matching_events() {
    let bus = EventBus::new(16);
    let rx = bus.subscribe();
    let mut stream = agent_event_stream(rx, "target-feat".to_string());

    // Publish events for different features
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "other-feat".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "another-feat".into(),
        wp_sequence: 2,
        agent_id: "a".into(),
    });
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "target-feat".into(),
        wp_sequence: 3,
        agent_id: "a".into(),
    });
    drop(bus);

    let mut received = Vec::new();
    while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
        received.push(item.unwrap());
    }
    assert_eq!(received.len(), 1);
    assert_eq!(
        received[0].event.as_ref().unwrap().feature_slug,
        "target-feat"
    );
}

#[tokio::test]
async fn stream_empty_filter_matches_all_events() {
    let bus = EventBus::new(16);
    let rx = bus.subscribe();
    let mut stream = agent_event_stream(rx, String::new());

    for i in 0..5 {
        bus.publish(AgentEvent::AgentStarted {
            feature_slug: format!("feat-{i}"),
            wp_sequence: i,
            agent_id: "a".into(),
        });
    }
    drop(bus);

    let mut count = 0;
    while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
        item.unwrap();
        count += 1;
    }
    assert_eq!(count, 5);
}

#[tokio::test]
async fn stream_terminates_when_sender_dropped() {
    let bus = EventBus::new(16);
    let rx = bus.subscribe();
    let mut stream = agent_event_stream(rx, String::new());

    // Drop the bus (sender) immediately
    drop(bus);

    // Stream should terminate
    let mut count = 0;
    while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
        item.unwrap();
        count += 1;
    }
    assert_eq!(count, 0);
}

#[tokio::test]
async fn stream_delivers_events_in_order() {
    let bus = EventBus::new(16);
    let rx = bus.subscribe();
    let mut stream = agent_event_stream(rx, "f".to_string());

    for i in 0..10 {
        bus.publish(AgentEvent::WpStateChanged {
            feature_slug: "f".into(),
            wp_sequence: i,
            old_state: "a".into(),
            new_state: "b".into(),
        });
    }
    drop(bus);

    let mut sequences = Vec::new();
    while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
        let resp = item.unwrap();
        sequences.push(resp.event.as_ref().unwrap().wp_sequence);
    }
    assert_eq!(sequences, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[tokio::test]
async fn stream_all_event_types_pass_through() {
    let bus = EventBus::new(32);
    let rx = bus.subscribe();
    let mut stream = agent_event_stream(rx, String::new());

    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });
    bus.publish(AgentEvent::PrCreated {
        feature_slug: "f".into(),
        wp_sequence: 2,
        pr_url: "u".into(),
    });
    bus.publish(AgentEvent::ReviewReceived {
        feature_slug: "f".into(),
        wp_sequence: 3,
        review_status: "ok".into(),
        comments: 0,
    });
    bus.publish(AgentEvent::AgentFixing {
        feature_slug: "f".into(),
        wp_sequence: 4,
        cycle: 1,
    });
    bus.publish(AgentEvent::AgentCompleted {
        feature_slug: "f".into(),
        wp_sequence: 5,
        success: true,
    });
    bus.publish(AgentEvent::WpStateChanged {
        feature_slug: "f".into(),
        wp_sequence: 6,
        old_state: "a".into(),
        new_state: "b".into(),
    });
    drop(bus);

    let mut types = Vec::new();
    while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
        let resp = item.unwrap();
        types.push(resp.event.as_ref().unwrap().event_type.clone());
    }
    assert_eq!(
        types,
        vec![
            "agent_started",
            "pr_created",
            "review_received",
            "agent_fixing",
            "agent_completed",
            "wp_state_changed"
        ]
    );
}

#[tokio::test]
async fn stream_mixed_filtering_multiple_features() {
    let bus = EventBus::new(32);
    let rx = bus.subscribe();
    let mut stream = agent_event_stream(rx, "alpha".to_string());

    // Publish interleaved events
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "alpha".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "beta".into(),
        wp_sequence: 2,
        agent_id: "b".into(),
    });
    bus.publish(AgentEvent::AgentCompleted {
        feature_slug: "alpha".into(),
        wp_sequence: 1,
        success: true,
    });
    bus.publish(AgentEvent::WpStateChanged {
        feature_slug: "gamma".into(),
        wp_sequence: 1,
        old_state: "a".into(),
        new_state: "b".into(),
    });
    drop(bus);

    let mut received = Vec::new();
    while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
        let resp = item.unwrap();
        received.push(resp.event.as_ref().unwrap().event_type.clone());
    }
    assert_eq!(received, vec!["agent_started", "agent_completed"]);
}
