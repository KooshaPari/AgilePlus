//! Integration tests for SHA-256 hash chain — `compute_hash` and `verify_chain`.

use agileplus_domain::domain::event::Event;
use agileplus_events::hash::{compute_hash, verify_chain, HashError};
use chrono::{DateTime, Utc};

/// Build a properly chained sequence of events with valid hashes.
fn build_valid_chain(count: usize, entity_id: i64) -> Vec<Event> {
    let ts = DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let mut events = Vec::new();
    let mut prev_hash = [0u8; 32];

    for i in 0..count {
        let seq = (i + 1) as i64;
        let payload = serde_json::json!({"step": i});
        let hash = compute_hash(
            entity_id,
            "Feature",
            "created",
            &payload,
            ts,
            "tester",
            &prev_hash,
        )
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

// ── compute_hash ────────────────────────────────────────────────────────────

#[test]
fn compute_hash_deterministic() {
    let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let h1 = compute_hash(
        1,
        "Feature",
        "created",
        &serde_json::json!({"name": "test"}),
        ts,
        "user1",
        &[0u8; 32],
    )
    .unwrap();
    let h2 = compute_hash(
        1,
        "Feature",
        "created",
        &serde_json::json!({"name": "test"}),
        ts,
        "user1",
        &[0u8; 32],
    )
    .unwrap();
    assert_eq!(h1, h2);
    assert_ne!(h1, [0u8; 32]);
}

#[test]
fn compute_hash_different_inputs_differ() {
    let ts = Utc::now();
    let h1 = compute_hash(1, "Feature", "created", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    let h2 = compute_hash(2, "Feature", "created", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h2);

    let h3 = compute_hash(1, "WorkPackage", "created", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h3);

    let h4 = compute_hash(1, "Feature", "shipped", &serde_json::json!({}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h4);

    let h5 = compute_hash(1, "Feature", "created", &serde_json::json!({"x": 1}), ts, "a", &[0u8; 32]).unwrap();
    assert_ne!(h1, h5);

    let h6 = compute_hash(1, "Feature", "created", &serde_json::json!({}), ts, "b", &[0u8; 32]).unwrap();
    assert_ne!(h1, h6);
}

#[test]
fn compute_hash_depends_on_prev_hash() {
    let ts = Utc::now();
    let mut prev1 = [0u8; 32];
    prev1[0] = 1;
    let mut prev2 = [0u8; 32];
    prev2[0] = 2;

    let h1 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &prev1).unwrap();
    let h2 = compute_hash(1, "F", "c", &serde_json::json!({}), ts, "a", &prev2).unwrap();
    assert_ne!(h1, h2);
}

// ── verify_chain ────────────────────────────────────────────────────────────

#[test]
fn verify_chain_empty_is_ok() {
    verify_chain(&[]).unwrap();
}

#[test]
fn verify_chain_single_valid_event() {
    let events = build_valid_chain(1, 1);
    verify_chain(&events).unwrap();
}

#[test]
fn verify_chain_multi_event_valid() {
    let events = build_valid_chain(5, 42);
    verify_chain(&events).unwrap();
}

#[test]
fn verify_chain_fails_on_wrong_hash() {
    let mut events = build_valid_chain(2, 1);
    // Corrupt the first event's hash
    events[0].hash[0] ^= 0xFF;
    let result = verify_chain(&events);
    assert!(result.is_err());
}

#[test]
fn verify_chain_fails_on_broken_chain() {
    let mut events = build_valid_chain(3, 1);
    // Corrupt prev_hash of second event (should match first event's hash)
    events[1].prev_hash[0] ^= 0xFF;
    let result = verify_chain(&events);
    assert!(result.is_err());
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
            sequence: 3, // Gap: should be 2
        },
    ];

    let result = verify_chain(&events);
    assert!(result.is_err());
    match result.unwrap_err() {
        HashError::ChainBroken { sequence } => assert_eq!(sequence, 3),
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
fn verify_chain_reports_hash_mismatch_at_correct_sequence() {
    let mut events = build_valid_chain(3, 1);
    // Tamper payload of event 2 (this will invalidate its hash but keep chain link intact)
    events[1].payload = serde_json::json!({"tampered": true});

    let result = verify_chain(&events);
    assert!(result.is_err());
    match result.unwrap_err() {
        HashError::HashMismatch { sequence } => assert_eq!(sequence, 2),
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

// ── HashError Display ───────────────────────────────────────────────────────

#[test]
fn hash_error_display_messages() {
    let e1 = HashError::ChainBroken { sequence: 5 };
    assert!(e1.to_string().contains("5"));

    let e2 = HashError::HashMismatch { sequence: 3 };
    assert!(e2.to_string().contains("3"));

    let e3 = HashError::InvalidHashLength(16);
    assert!(e3.to_string().contains("16"));
}
