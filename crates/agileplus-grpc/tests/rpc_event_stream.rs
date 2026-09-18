//! RPC handler tests for the `StreamAgentEvents` server stream.
//!
//! The handler body itself (scope enforcement, bus subscription, and the
//! `agent_event_stream` wiring) is exercised here; the pure framing helpers are
//! covered separately by `streaming_comprehensive.rs`.
//!
//! Traceability: WP14-T083

mod support;

use std::pin::Pin;
use std::time::Duration;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_grpc::event_bus::AgentEvent;
use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreService;
use agileplus_proto::agileplus::v1::{StreamAgentEventsRequest, StreamAgentEventsResponse};
use futures::{Stream, StreamExt};
use support::{Harness, scope};
use tokio::time::timeout;
use tonic::{Code, Request, Status};

type EventStream = Pin<Box<dyn Stream<Item = Result<StreamAgentEventsResponse, Status>> + Send>>;

/// Open the stream for `feature_slug` against the harness server.
async fn subscribe(harness: &Harness, feature_slug: &str) -> EventStream {
    harness
        .server
        .stream_agent_events(Request::new(StreamAgentEventsRequest {
            feature_slug: feature_slug.to_string(),
            project_scope: scope(),
        }))
        .await
        .expect("streaming should open")
        .into_inner()
}

/// Await the next event, failing the test if nothing arrives promptly.
async fn next_event(stream: &mut EventStream) -> agileplus_proto::agileplus::v1::AgentEvent {
    timeout(Duration::from_secs(5), stream.next())
        .await
        .expect("an event should arrive before the timeout")
        .expect("the stream should yield an item")
        .expect("the event should not fail")
        .event
        .expect("event payload should be present")
}

#[tokio::test]
async fn stream_agent_events_requires_project_scope() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .stream_agent_events(Request::new(StreamAgentEventsRequest {
            feature_slug: "alpha".into(),
            project_scope: None,
        }))
        .await
        .map(|_| ())
        .expect_err("a scope-less subscription must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn stream_agent_events_delivers_matching_agent_progress() {
    let harness = Harness::new().await;
    harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let mut stream = subscribe(&harness, "alpha").await;

    harness.bus.publish(AgentEvent::AgentStarted {
        feature_slug: "alpha".into(),
        wp_sequence: 1,
        agent_id: "agent-7".into(),
    });
    harness.bus.publish(AgentEvent::AgentCompleted {
        feature_slug: "alpha".into(),
        wp_sequence: 1,
        success: true,
    });

    let started = next_event(&mut stream).await;
    assert_eq!(started.event_type, "agent_started");
    assert_eq!(started.feature_slug, "alpha");
    assert_eq!(started.wp_sequence, 1);
    assert_eq!(started.agent_id, "agent-7");
    assert!(!started.timestamp.is_empty());
    let payload: serde_json::Value =
        serde_json::from_str(&started.payload).expect("payload should be JSON");
    assert!(payload.is_object());

    let completed = next_event(&mut stream).await;
    assert_eq!(completed.event_type, "agent_completed");
    assert_eq!(completed.wp_sequence, 1);
    // `agent_completed` carries no agent id on the wire.
    assert!(completed.agent_id.is_empty());
}

#[tokio::test]
async fn stream_agent_events_filters_out_other_features() {
    let harness = Harness::new().await;
    harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let mut stream = subscribe(&harness, "alpha").await;

    harness.bus.publish(AgentEvent::AgentStarted {
        feature_slug: "beta".into(),
        wp_sequence: 9,
        agent_id: "agent-beta".into(),
    });
    harness.bus.publish(AgentEvent::WpStateChanged {
        feature_slug: "alpha".into(),
        wp_sequence: 2,
        old_state: "doing".into(),
        new_state: "review".into(),
    });

    let event = next_event(&mut stream).await;
    assert_eq!(event.feature_slug, "alpha");
    assert_eq!(event.event_type, "wp_state_changed");
    assert_eq!(event.wp_sequence, 2);

    // The beta event was dropped, so nothing else is pending.
    let quiet = timeout(Duration::from_millis(150), stream.next()).await;
    assert!(
        quiet.is_err(),
        "only matching features may be delivered, got {quiet:?}"
    );
}

#[tokio::test]
async fn stream_agent_events_empty_filter_delivers_every_feature() {
    let harness = Harness::new().await;
    let mut stream = subscribe(&harness, "").await;

    harness.bus.publish(AgentEvent::AgentFixing {
        feature_slug: "alpha".into(),
        wp_sequence: 1,
        cycle: 2,
    });
    harness.bus.publish(AgentEvent::PrCreated {
        feature_slug: "beta".into(),
        wp_sequence: 4,
        pr_url: "https://example.invalid/pr/4".into(),
    });

    let first = next_event(&mut stream).await;
    let second = next_event(&mut stream).await;

    assert_eq!(first.event_type, "agent_fixing");
    assert_eq!(first.feature_slug, "alpha");
    assert_eq!(second.event_type, "pr_created");
    assert_eq!(second.feature_slug, "beta");
    assert_eq!(second.wp_sequence, 4);
}

#[tokio::test]
async fn stream_agent_events_does_not_replay_events_published_before_subscription() {
    let harness = Harness::new().await;
    harness.bus.publish(AgentEvent::AgentStarted {
        feature_slug: "alpha".into(),
        wp_sequence: 1,
        agent_id: "agent-early".into(),
    });

    let mut stream = subscribe(&harness, "alpha").await;

    let quiet = timeout(Duration::from_millis(150), stream.next()).await;
    assert!(
        quiet.is_err(),
        "a fresh subscription must not replay earlier events, got {quiet:?}"
    );
}
