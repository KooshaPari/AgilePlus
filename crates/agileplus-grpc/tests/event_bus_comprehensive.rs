//! Comprehensive tests for the event bus and AgentEvent API.
//!
//! Complements the inline unit tests in `event_bus.rs`.
//!
//! Traceability: WP14-T083

use agileplus_grpc::event_bus::{AgentEvent, EventBus};

// ---------------------------------------------------------------------------
// AgentEvent accessor methods
// ---------------------------------------------------------------------------

#[test]
fn agent_started_agent_id() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "agent-42".into(),
    };
    assert_eq!(e.agent_id(), "agent-42");
}

#[test]
fn pr_created_agent_id_is_empty() {
    let e = AgentEvent::PrCreated {
        feature_slug: "f".into(),
        wp_sequence: 1,
        pr_url: "url".into(),
    };
    assert_eq!(e.agent_id(), "");
}

#[test]
fn review_received_agent_id_is_empty() {
    let e = AgentEvent::ReviewReceived {
        feature_slug: "f".into(),
        wp_sequence: 1,
        review_status: "approved".into(),
        comments: 3,
    };
    assert_eq!(e.agent_id(), "");
}

#[test]
fn agent_fixing_agent_id_is_empty() {
    let e = AgentEvent::AgentFixing {
        feature_slug: "f".into(),
        wp_sequence: 1,
        cycle: 2,
    };
    assert_eq!(e.agent_id(), "");
}

#[test]
fn agent_completed_agent_id_is_empty() {
    let e = AgentEvent::AgentCompleted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        success: true,
    };
    assert_eq!(e.agent_id(), "");
}

#[test]
fn wp_state_changed_agent_id_is_empty() {
    let e = AgentEvent::WpStateChanged {
        feature_slug: "f".into(),
        wp_sequence: 1,
        old_state: "a".into(),
        new_state: "b".into(),
    };
    assert_eq!(e.agent_id(), "");
}

// ---------------------------------------------------------------------------
// AgentEvent event_type strings
// ---------------------------------------------------------------------------

#[test]
fn event_type_strings_for_all_variants() {
    let cases: Vec<(AgentEvent, &str)> = vec![
        (
            AgentEvent::AgentStarted {
                feature_slug: "f".into(),
                wp_sequence: 1,
                agent_id: "a".into(),
            },
            "agent_started",
        ),
        (
            AgentEvent::PrCreated {
                feature_slug: "f".into(),
                wp_sequence: 1,
                pr_url: "u".into(),
            },
            "pr_created",
        ),
        (
            AgentEvent::ReviewReceived {
                feature_slug: "f".into(),
                wp_sequence: 1,
                review_status: "ok".into(),
                comments: 0,
            },
            "review_received",
        ),
        (
            AgentEvent::AgentFixing {
                feature_slug: "f".into(),
                wp_sequence: 1,
                cycle: 1,
            },
            "agent_fixing",
        ),
        (
            AgentEvent::AgentCompleted {
                feature_slug: "f".into(),
                wp_sequence: 1,
                success: true,
            },
            "agent_completed",
        ),
        (
            AgentEvent::WpStateChanged {
                feature_slug: "f".into(),
                wp_sequence: 1,
                old_state: "x".into(),
                new_state: "y".into(),
            },
            "wp_state_changed",
        ),
    ];

    for (event, expected) in cases {
        assert_eq!(event.event_type(), expected);
    }
}

// ---------------------------------------------------------------------------
// AgentEvent wp_sequence accessor
// ---------------------------------------------------------------------------

#[test]
fn wp_sequence_accessor_for_all_variants() {
    let seq = 42;
    let events = [
        AgentEvent::AgentStarted {
            feature_slug: "f".into(),
            wp_sequence: seq,
            agent_id: "a".into(),
        },
        AgentEvent::PrCreated {
            feature_slug: "f".into(),
            wp_sequence: seq,
            pr_url: "u".into(),
        },
        AgentEvent::ReviewReceived {
            feature_slug: "f".into(),
            wp_sequence: seq,
            review_status: "ok".into(),
            comments: 0,
        },
        AgentEvent::AgentFixing {
            feature_slug: "f".into(),
            wp_sequence: seq,
            cycle: 1,
        },
        AgentEvent::AgentCompleted {
            feature_slug: "f".into(),
            wp_sequence: seq,
            success: true,
        },
        AgentEvent::WpStateChanged {
            feature_slug: "f".into(),
            wp_sequence: seq,
            old_state: "a".into(),
            new_state: "b".into(),
        },
    ];

    for event in &events {
        assert_eq!(event.wp_sequence(), seq);
    }
}

// ---------------------------------------------------------------------------
// AgentEvent payload (JSON serialization)
// ---------------------------------------------------------------------------

#[test]
fn payload_returns_valid_json_for_all_variants() {
    let events = [
        AgentEvent::AgentStarted {
            feature_slug: "f".into(),
            wp_sequence: 1,
            agent_id: "a".into(),
        },
        AgentEvent::PrCreated {
            feature_slug: "f".into(),
            wp_sequence: 1,
            pr_url: "u".into(),
        },
        AgentEvent::ReviewReceived {
            feature_slug: "f".into(),
            wp_sequence: 1,
            review_status: "ok".into(),
            comments: 5,
        },
        AgentEvent::AgentFixing {
            feature_slug: "f".into(),
            wp_sequence: 1,
            cycle: 3,
        },
        AgentEvent::AgentCompleted {
            feature_slug: "f".into(),
            wp_sequence: 1,
            success: false,
        },
        AgentEvent::WpStateChanged {
            feature_slug: "f".into(),
            wp_sequence: 1,
            old_state: "planned".into(),
            new_state: "doing".into(),
        },
    ];

    for event in &events {
        let json = event.payload();
        assert!(!json.is_empty());
        // Verify it's valid JSON by parsing
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed.is_object());
    }
}

#[test]
fn payload_contains_event_type_variant_name() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    let json = e.payload();
    assert!(json.contains("AgentStarted"));
}

// ---------------------------------------------------------------------------
// AgentEvent feature_slug accessor
// ---------------------------------------------------------------------------

#[test]
fn feature_slug_accessor_for_all_variants() {
    let slug = "my-feature";
    let events = [
        AgentEvent::AgentStarted {
            feature_slug: slug.into(),
            wp_sequence: 1,
            agent_id: "a".into(),
        },
        AgentEvent::PrCreated {
            feature_slug: slug.into(),
            wp_sequence: 1,
            pr_url: "u".into(),
        },
        AgentEvent::ReviewReceived {
            feature_slug: slug.into(),
            wp_sequence: 1,
            review_status: "ok".into(),
            comments: 0,
        },
        AgentEvent::AgentFixing {
            feature_slug: slug.into(),
            wp_sequence: 1,
            cycle: 1,
        },
        AgentEvent::AgentCompleted {
            feature_slug: slug.into(),
            wp_sequence: 1,
            success: true,
        },
        AgentEvent::WpStateChanged {
            feature_slug: slug.into(),
            wp_sequence: 1,
            old_state: "a".into(),
            new_state: "b".into(),
        },
    ];

    for event in &events {
        assert_eq!(event.feature_slug(), slug);
    }
}

// ---------------------------------------------------------------------------
// AgentEvent matches_feature
// ---------------------------------------------------------------------------

#[test]
fn matches_feature_exact_match() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "feat-abc".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    assert!(e.matches_feature("feat-abc"));
}

#[test]
fn matches_feature_empty_filter_matches_all() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "feat-abc".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    assert!(e.matches_feature(""));
}

#[test]
fn matches_feature_wrong_slug() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "feat-abc".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    assert!(!e.matches_feature("feat-xyz"));
}

#[test]
fn matches_feature_partial_slug_does_not_match() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "feature-abc".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    assert!(!e.matches_feature("feature"));
}

// ---------------------------------------------------------------------------
// EventBus capacity and subscriber behavior
// ---------------------------------------------------------------------------

#[test]
fn event_bus_default_capacity() {
    let bus = EventBus::default();
    let mut rx = bus.subscribe();

    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });

    let event = rx.try_recv().unwrap();
    assert_eq!(event.feature_slug(), "f");
}

#[tokio::test]
async fn event_bus_capacity_overflow_drops_oldest() {
    // Small capacity of 2 — 3rd event should push out the oldest
    let bus = EventBus::new(2);
    let mut rx = bus.subscribe();

    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "first".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "second".into(),
        wp_sequence: 2,
        agent_id: "a".into(),
    });
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "third".into(),
        wp_sequence: 3,
        agent_id: "a".into(),
    });

    // Drop the bus to close the channel so recv() terminates after remaining events
    drop(bus);

    // The receiver may get Lagged(1) when capacity overflows. Handle it gracefully.
    let mut received = Vec::new();
    loop {
        match rx.recv().await {
            Ok(event) => received.push(event),
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
    // After lagging past 1 event, we should get events 2 and 3
    assert_eq!(received.len(), 2);
    assert_eq!(received[0].feature_slug(), "second");
    assert_eq!(received[1].feature_slug(), "third");
}

#[tokio::test]
async fn event_bus_multiple_independent_subscribers() {
    let bus = EventBus::new(32);
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    bus.publish(AgentEvent::AgentCompleted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        success: true,
    });

    // Both subscribers should receive the event independently
    let e1 = rx1.recv().await.unwrap();
    let e2 = rx2.recv().await.unwrap();
    assert_eq!(e1.event_type(), "agent_completed");
    assert_eq!(e2.event_type(), "agent_completed");
}

#[tokio::test]
async fn event_bus_publish_no_receivers() {
    let bus = EventBus::new(16);
    // No subscriber — publish should not panic
    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });
}

#[tokio::test]
async fn event_bus_many_events_ordering() {
    let bus = EventBus::new(128);
    let mut rx = bus.subscribe();

    for i in 0..50 {
        bus.publish(AgentEvent::AgentStarted {
            feature_slug: "f".into(),
            wp_sequence: i,
            agent_id: format!("agent-{i}"),
        });
    }

    for i in 0..50 {
        let e = rx.recv().await.unwrap();
        assert_eq!(e.wp_sequence(), i);
    }
}

#[test]
fn event_bus_clone_shares_channel() {
    let bus1 = EventBus::new(16);
    let bus2 = bus1.clone();
    let mut rx = bus2.subscribe();

    bus1.publish(AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    });

    let event = rx.try_recv().unwrap();
    assert_eq!(event.feature_slug(), "f");
}

#[test]
fn agent_event_clone() {
    let e = AgentEvent::AgentStarted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        agent_id: "a".into(),
    };
    let e2 = e.clone();
    assert_eq!(e.feature_slug(), e2.feature_slug());
    assert_eq!(e.event_type(), e2.event_type());
    assert_eq!(e.wp_sequence(), e2.wp_sequence());
}

#[test]
fn agent_event_debug() {
    let e = AgentEvent::AgentCompleted {
        feature_slug: "f".into(),
        wp_sequence: 1,
        success: true,
    };
    let debug = format!("{:?}", e);
    assert!(debug.contains("AgentCompleted"));
    assert!(debug.contains("f"));
}
