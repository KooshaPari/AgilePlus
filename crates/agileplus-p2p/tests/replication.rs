//! Integration tests for the replication module.
//!
//! Tests wire format types (EventBatch, ReplicationResult),
//! NATS subject generation, SyncError formatting, and serde roundtrips.
//! Skips tests requiring actual NATS connections.

use agileplus_domain::domain::event::Event;
use agileplus_p2p::error::SyncError;
use agileplus_p2p::replication::{EventBatch, ReplicationResult, device_subject};

// ── device_subject ─────────────────────────────────────────────────────────

#[test]
fn device_subject_format() {
    assert_eq!(device_subject("abc-123"), "agileplus.sync.device.abc-123");
}

#[test]
fn device_subject_with_uuid() {
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    assert_eq!(
        device_subject(uuid),
        "agileplus.sync.device.550e8400-e29b-41d4-a716-446655440000"
    );
}

#[test]
fn device_subject_empty_string() {
    assert_eq!(device_subject(""), "agileplus.sync.device.");
}

#[test]
fn device_subject_special_characters() {
    assert_eq!(device_subject("dev-1"), "agileplus.sync.device.dev-1");
}

// ── EventBatch ─────────────────────────────────────────────────────────────

#[test]
fn event_batch_serde_roundtrip() {
    let mut ev = Event::new(
        "Feature",
        1,
        "created",
        serde_json::json!({"title": "T1"}),
        "actor-1",
    );
    ev.sequence = 42;

    let batch = EventBatch {
        sender_device_id: "device-abc".into(),
        events: vec![ev.clone()],
    };

    let json = serde_json::to_string(&batch).unwrap();
    let back: EventBatch = serde_json::from_str(&json).unwrap();

    assert_eq!(back.sender_device_id, "device-abc");
    assert_eq!(back.events.len(), 1);
    assert_eq!(back.events[0].entity_type, "Feature");
    assert_eq!(back.events[0].entity_id, 1);
    assert_eq!(back.events[0].sequence, 42);
}

#[test]
fn event_batch_empty_events() {
    let batch = EventBatch {
        sender_device_id: "d1".into(),
        events: vec![],
    };
    let json = serde_json::to_string(&batch).unwrap();
    let back: EventBatch = serde_json::from_str(&json).unwrap();
    assert!(back.events.is_empty());
}

#[test]
fn event_batch_multiple_events() {
    let mut ev1 = Event::new("Feature", 1, "created", serde_json::json!({}), "a");
    ev1.sequence = 1;
    let mut ev2 = Event::new("Feature", 1, "updated", serde_json::json!({}), "b");
    ev2.sequence = 2;
    let mut ev3 = Event::new("WorkPackage", 5, "created", serde_json::json!({}), "c");
    ev3.sequence = 1;

    let batch = EventBatch {
        sender_device_id: "d1".into(),
        events: vec![ev1, ev2, ev3],
    };

    let json = serde_json::to_string(&batch).unwrap();
    let back: EventBatch = serde_json::from_str(&json).unwrap();
    assert_eq!(back.events.len(), 3);
    assert_eq!(back.events[0].event_type, "created");
    assert_eq!(back.events[1].event_type, "updated");
    assert_eq!(back.events[2].entity_type, "WorkPackage");
}

#[test]
fn event_batch_debug() {
    let batch = EventBatch {
        sender_device_id: "d1".into(),
        events: vec![],
    };
    let dbg = format!("{:?}", batch);
    assert!(dbg.contains("EventBatch"));
}

// ── ReplicationResult ──────────────────────────────────────────────────────

#[test]
fn replication_result_default() {
    let r = ReplicationResult::default();
    assert_eq!(r.events_sent, 0);
    assert_eq!(r.events_received, 0);
}

#[test]
fn replication_result_with_values() {
    let r = ReplicationResult {
        events_sent: 10,
        events_received: 5,
    };
    assert_eq!(r.events_sent, 10);
    assert_eq!(r.events_received, 5);
}

#[test]
fn replication_result_debug() {
    let r = ReplicationResult {
        events_sent: 3,
        events_received: 2,
    };
    let dbg = format!("{:?}", r);
    assert!(dbg.contains("ReplicationResult"));
    assert!(dbg.contains("3"));
    assert!(dbg.contains("2"));
}

// ── SyncError ──────────────────────────────────────────────────────────────

#[test]
fn sync_error_connection_failed_display() {
    let err = SyncError::ConnectionFailed {
        peer_id: "peer-1".into(),
        reason: "timeout".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("NATS connection failed"));
    assert!(msg.contains("peer-1"));
    assert!(msg.contains("timeout"));
}

#[test]
fn sync_error_publish_failed_display() {
    let err = SyncError::PublishFailed("stream full".into());
    let msg = format!("{}", err);
    assert!(msg.contains("NATS publish failed"));
    assert!(msg.contains("stream full"));
}

#[test]
fn sync_error_serialization_display() {
    let json_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
    let err: SyncError = json_err.into();
    let msg = format!("{}", err);
    assert!(msg.contains("Serialization error"));
}

#[test]
fn sync_error_event_store_display() {
    let err = SyncError::EventStore("table locked".into());
    let msg = format!("{}", err);
    assert!(msg.contains("Event store error"));
    assert!(msg.contains("table locked"));
}

#[test]
fn sync_error_nats_display() {
    let err = SyncError::Nats("connection refused".into());
    let msg = format!("{}", err);
    assert!(msg.contains("NATS error"));
    assert!(msg.contains("connection refused"));
}

#[test]
fn sync_error_timeout_display() {
    let err = SyncError::Timeout {
        peer_id: "peer-2".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Timeout"));
    assert!(msg.contains("peer-2"));
}

#[test]
fn sync_error_from_peer_discovery() {
    let pd_err = agileplus_p2p::error::PeerDiscoveryError::UnsupportedPlatform;
    let err: SyncError = pd_err.into();
    match err {
        SyncError::Discovery(_) => {}
        other => panic!("Expected Discovery, got {:?}", other),
    }
}

#[test]
fn sync_error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("bad!!!").unwrap_err();
    let err: SyncError = json_err.into();
    match err {
        SyncError::Serialization(_) => {}
        other => panic!("Expected Serialization, got {:?}", other),
    }
}

// ── SyncError conversions from the NATS client ─────────────────────────────
//
// `SyncError` implements `From<async_nats::ConnectError>`. That conversion is
// reached here through a server address that cannot be parsed, so the error
// type is real while no broker and no socket are involved.

#[tokio::test]
async fn malformed_server_address_becomes_nats_error() {
    let connect_error = async_nats::connect("not a nats server address")
        .await
        .expect_err("an address that cannot be parsed must not connect");

    // Pins the reason: the failure happened while parsing, before any socket
    // work, so this test cannot be satisfied by an unreachable host.
    assert_eq!(
        connect_error.kind(),
        async_nats::ConnectErrorKind::ServerParse
    );

    let sync_error: SyncError = connect_error.into();
    match sync_error {
        SyncError::Nats(ref message) => assert!(
            !message.is_empty(),
            "the client's description must survive the conversion"
        ),
        other => panic!("expected Nats, got {other:?}"),
    }
    assert!(sync_error.to_string().starts_with("NATS error:"));
}

// ── EventBatch rejection of malformed payloads ─────────────────────────────

#[test]
fn event_batch_rejects_missing_events_field() {
    let error = serde_json::from_str::<EventBatch>(r#"{"sender_device_id":"d"}"#)
        .expect_err("a batch without events must not decode");
    assert!(
        error.to_string().contains("events"),
        "the missing field should be named: {error}"
    );
}

#[test]
fn event_batch_rejects_non_object_payload() {
    assert!(serde_json::from_str::<EventBatch>("[]").is_err());
    assert!(serde_json::from_str::<EventBatch>("null").is_err());
    assert!(serde_json::from_str::<EventBatch>(r#""d""#).is_err());
}

#[test]
fn event_batch_rejects_events_that_are_not_events() {
    let payload = r#"{"sender_device_id":"d","events":[{"nope":1}]}"#;
    let error =
        serde_json::from_str::<EventBatch>(payload).expect_err("a malformed event must not decode");
    assert!(!error.to_string().is_empty());
}
