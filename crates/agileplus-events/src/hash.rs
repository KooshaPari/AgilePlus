//! SHA-256 hash chain computation and verification.

use agileplus_domain::domain::event::Event;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Hash chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },
    #[error("Invalid hash length: expected 32, got {0}")]
    InvalidHashLength(usize),
    #[error("Hash mismatch at sequence {sequence}")]
    HashMismatch { sequence: i64 },
}

/// Compute SHA-256 hash for a new event.
///
/// Hash inputs (in order, length-prefixed where noted):
/// 1. entity_id (8 bytes, big-endian)
/// 2. entity_type (length-prefixed UTF-8)
/// 3. event_type (length-prefixed UTF-8)
/// 4. payload (length-prefixed JSON)
/// 5. timestamp (length-prefixed ISO 8601)
/// 6. actor (length-prefixed UTF-8)
/// 7. prev_hash (32 bytes)
pub fn compute_hash(
    entity_id: i64,
    entity_type: &str,
    event_type: &str,
    payload: &serde_json::Value,
    timestamp: DateTime<Utc>,
    actor: &str,
    prev_hash: &[u8; 32],
) -> Result<[u8; 32], HashError> {
    let mut hasher = Sha256::new();

    hasher.update(entity_id.to_be_bytes());

    hasher.update((entity_type.len() as u32).to_be_bytes());
    hasher.update(entity_type.as_bytes());

    hasher.update((event_type.len() as u32).to_be_bytes());
    hasher.update(event_type.as_bytes());

    let payload_json =
        serde_json::to_string(payload).map_err(|_| HashError::InvalidHashLength(0))?;
    hasher.update((payload_json.len() as u32).to_be_bytes());
    hasher.update(payload_json.as_bytes());

    let timestamp_str = timestamp.to_rfc3339();
    hasher.update((timestamp_str.len() as u32).to_be_bytes());
    hasher.update(timestamp_str.as_bytes());

    hasher.update((actor.len() as u32).to_be_bytes());
    hasher.update(actor.as_bytes());

    hasher.update(prev_hash);

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result[..]);
    Ok(hash)
}

/// Verify the integrity of an event chain.
///
/// Ensures each event's hash is correctly computed and chains to its predecessor.
pub fn verify_chain(events: &[Event]) -> Result<(), HashError> {
    if events.is_empty() {
        return Ok(());
    }

    // First event must chain from zeros
    if events[0].prev_hash != [0u8; 32] {
        return Err(HashError::ChainBroken {
            sequence: events[0].sequence,
        });
    }

    let expected = compute_hash(
        events[0].entity_id,
        &events[0].entity_type,
        &events[0].event_type,
        &events[0].payload,
        events[0].timestamp,
        &events[0].actor,
        &[0u8; 32],
    )?;
    if expected != events[0].hash {
        return Err(HashError::HashMismatch {
            sequence: events[0].sequence,
        });
    }

    for i in 1..events.len() {
        let prev = &events[i - 1];
        let curr = &events[i];

        if curr.sequence != prev.sequence + 1 {
            return Err(HashError::ChainBroken {
                sequence: curr.sequence,
            });
        }
        if curr.prev_hash != prev.hash {
            return Err(HashError::ChainBroken {
                sequence: curr.sequence,
            });
        }

        let expected = compute_hash(
            curr.entity_id,
            &curr.entity_type,
            &curr.event_type,
            &curr.payload,
            curr.timestamp,
            &curr.actor,
            &prev.hash,
        )?;
        if expected != curr.hash {
            return Err(HashError::HashMismatch {
                sequence: curr.sequence,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_deterministic() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let h1 = compute_hash(
            1,
            "Feature",
            "created",
            &serde_json::json!({"n": "t"}),
            ts,
            "u1",
            &[0u8; 32],
        )
        .unwrap();
        let h2 = compute_hash(
            1,
            "Feature",
            "created",
            &serde_json::json!({"n": "t"}),
            ts,
            "u1",
            &[0u8; 32],
        )
        .unwrap();
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn verify_chain_empty() {
        verify_chain(&[]).unwrap();
    }

    #[test]
    fn verify_chain_single() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let payload = serde_json::json!({"x": 1});
        let hash = compute_hash(1, "F", "c", &payload, ts, "a", &[0u8; 32]).unwrap();
        let event = Event {
            id: 1,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "c".into(),
            payload,
            actor: "a".into(),
            timestamp: ts,
            prev_hash: [0u8; 32],
            hash,
            sequence: 1,
        };
        verify_chain(&[event]).unwrap();
    }

    #[test]
    fn verify_chain_two_events() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let p1 = serde_json::json!({"v": 1});
        let h1 = compute_hash(1, "F", "c", &p1, ts, "a", &[0u8; 32]).unwrap();
        let e1 = Event {
            id: 1,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "c".into(),
            payload: p1,
            actor: "a".into(),
            timestamp: ts,
            prev_hash: [0u8; 32],
            hash: h1,
            sequence: 1,
        };

        let p2 = serde_json::json!({"v": 2});
        let h2 = compute_hash(1, "F", "u", &p2, ts, "a", &h1).unwrap();
        let e2 = Event {
            id: 2,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "u".into(),
            payload: p2,
            actor: "a".into(),
            timestamp: ts,
            prev_hash: h1,
            hash: h2,
            sequence: 2,
        };

        verify_chain(&[e1, e2]).unwrap();
    }

    #[test]
    fn verify_chain_detects_tamper() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let p1 = serde_json::json!({"v": 1});
        let h1 = compute_hash(1, "F", "c", &p1, ts, "a", &[0u8; 32]).unwrap();
        let mut e1 = Event {
            id: 1,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "c".into(),
            payload: p1,
            actor: "a".into(),
            timestamp: ts,
            prev_hash: [0u8; 32],
            hash: h1,
            sequence: 1,
        };
        // Tamper
        e1.hash[0] ^= 0xFF;
        assert!(verify_chain(&[e1]).is_err());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn ts() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn base_hash() -> [u8; 32] {
        compute_hash(
            1,
            "Feature",
            "created",
            &serde_json::json!({"n": 1}),
            ts(),
            "actor",
            &[0u8; 32],
        )
        .unwrap()
    }

    fn make_event(sequence: i64, payload: serde_json::Value, prev_hash: [u8; 32]) -> Event {
        let hash = compute_hash(1, "Feature", "updated", &payload, ts(), "actor", &prev_hash).unwrap();
        Event {
            id: sequence,
            entity_type: "Feature".into(),
            entity_id: 1,
            event_type: "updated".into(),
            payload,
            actor: "actor".into(),
            timestamp: ts(),
            prev_hash,
            hash,
            sequence,
        }
    }

    fn make_chain(n: i64) -> Vec<Event> {
        let mut events = Vec::new();
        let mut prev = [0u8; 32];
        for seq in 1..=n {
            let ev = make_event(seq, serde_json::json!({"seq": seq}), prev);
            prev = ev.hash;
            events.push(ev);
        }
        events
    }

    #[test]
    fn compute_hash_deterministic_same_inputs() {
        assert_eq!(base_hash(), base_hash());
    }

    #[test]
    fn compute_hash_not_all_zero() {
        assert_ne!(base_hash(), [0u8; 32]);
    }

    #[test]
    fn compute_hash_changes_with_entity_id() {
        let a = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(2, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compute_hash_changes_with_entity_type() {
        let a = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "G", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compute_hash_changes_with_event_type() {
        let a = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "F", "f", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compute_hash_changes_with_payload() {
        let a = compute_hash(1, "F", "e", &serde_json::json!({"x": 1}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "F", "e", &serde_json::json!({"x": 2}), ts(), "a", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compute_hash_changes_with_timestamp() {
        let other = DateTime::parse_from_rfc3339("2026-03-03T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let a = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "F", "e", &serde_json::json!({}), other, "a", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compute_hash_changes_with_actor() {
        let a = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "b", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compute_hash_changes_with_prev_hash() {
        let a = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "F", "e", &serde_json::json!({}), ts(), "a", &[1u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn length_prefix_prevents_field_boundary_collision() {
        // "ab"+"c" must not hash the same as "a"+"bc" because each field is length-prefixed.
        let a = compute_hash(1, "ab", "c", &serde_json::json!({}), ts(), "x", &[0u8; 32]).unwrap();
        let b = compute_hash(1, "a", "bc", &serde_json::json!({}), ts(), "x", &[0u8; 32]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn hash_error_chain_broken_display() {
        let e = HashError::ChainBroken { sequence: 5 };
        assert_eq!(e.to_string(), "Hash chain broken at sequence 5");
    }

    #[test]
    fn hash_error_invalid_length_display() {
        let e = HashError::InvalidHashLength(7);
        assert_eq!(e.to_string(), "Invalid hash length: expected 32, got 7");
    }

    #[test]
    fn hash_error_hash_mismatch_display() {
        let e = HashError::HashMismatch { sequence: 3 };
        assert_eq!(e.to_string(), "Hash mismatch at sequence 3");
    }

    #[test]
    fn verify_chain_empty_is_ok() {
        verify_chain(&[]).unwrap();
    }

    #[test]
    fn verify_chain_single_event_ok() {
        verify_chain(&make_chain(1)).unwrap();
    }

    #[test]
    fn verify_chain_two_events_ok() {
        verify_chain(&make_chain(2)).unwrap();
    }

    #[test]
    fn verify_chain_three_events_ok() {
        verify_chain(&make_chain(3)).unwrap();
    }

    #[test]
    fn verify_chain_first_prev_hash_nonzero_is_broken() {
        let mut events = make_chain(1);
        events[0].prev_hash = [9u8; 32];
        assert!(matches!(
            verify_chain(&events),
            Err(HashError::ChainBroken { .. })
        ));
    }

    #[test]
    fn verify_chain_sequence_gap_detected() {
        let mut events = make_chain(2);
        events[1].sequence = 5;
        assert!(matches!(
            verify_chain(&events),
            Err(HashError::ChainBroken { sequence: 5 })
        ));
    }

    #[test]
    fn verify_chain_prev_hash_mismatch_detected() {
        let mut events = make_chain(2);
        events[1].prev_hash = [7u8; 32];
        assert!(matches!(
            verify_chain(&events),
            Err(HashError::ChainBroken { .. })
        ));
    }

    #[test]
    fn verify_chain_hash_mismatch_detected_on_second() {
        let mut events = make_chain(2);
        events[1].payload = serde_json::json!({"tampered": true});
        assert!(matches!(
            verify_chain(&events),
            Err(HashError::HashMismatch { sequence: 2 })
        ));
    }

    #[test]
    fn verify_chain_tampered_first_hash_detected() {
        let mut events = make_chain(1);
        events[0].hash[0] ^= 0xFF;
        assert!(matches!(
            verify_chain(&events),
            Err(HashError::HashMismatch { sequence: 1 })
        ));
    }

    #[test]
    fn verify_chain_middle_tamper_detected() {
        let mut events = make_chain(3);
        events[1].hash[5] ^= 0x01;
        assert!(verify_chain(&events).is_err());
    }

    #[test]
    fn compute_hash_handles_multibyte_fields() {
        let payload = serde_json::json!({"note": "café ☕"});
        let first =
            compute_hash(1, "Fëature", "créated", &payload, ts(), "álice", &[0u8; 32]).unwrap();
        let second =
            compute_hash(1, "Fëature", "créated", &payload, ts(), "álice", &[0u8; 32]).unwrap();
        assert_eq!(first, second, "multi-byte inputs must hash deterministically");

        let ascii = compute_hash(
            1,
            "Feature",
            "created",
            &serde_json::json!({"note": "cafe"}),
            ts(),
            "alice",
            &[0u8; 32],
        )
        .unwrap();
        assert_ne!(first, ascii, "accented and ASCII text must not collide");

        let split_a =
            compute_hash(1, "é", "ab", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        let split_b =
            compute_hash(1, "éa", "b", &serde_json::json!({}), ts(), "a", &[0u8; 32]).unwrap();
        assert_ne!(split_a, split_b, "field boundaries must stay unambiguous");
    }

    fn multibyte_chain() -> Vec<Event> {
        let payloads = [
            serde_json::json!({"title": "café ☕"}),
            serde_json::json!({"title": "日本語"}),
        ];
        let mut events = Vec::new();
        let mut prev = [0u8; 32];
        for (index, payload) in payloads.into_iter().enumerate() {
            let sequence = (index + 1) as i64;
            let hash = compute_hash(1, "Fëaturé", "créé", &payload, ts(), "Álice", &prev).unwrap();
            events.push(Event {
                id: sequence,
                entity_type: "Fëaturé".into(),
                entity_id: 1,
                event_type: "créé".into(),
                payload,
                actor: "Álice".into(),
                timestamp: ts(),
                prev_hash: prev,
                hash,
                sequence,
            });
            prev = hash;
        }
        events
    }

    #[test]
    fn verify_chain_accepts_multibyte_fields() {
        let events = multibyte_chain();
        verify_chain(&events).unwrap();
        assert_ne!(events[0].hash, events[1].hash);
    }

    #[test]
    fn verify_chain_detects_tampered_multibyte_payload() {
        let mut events = multibyte_chain();
        events[1].payload = serde_json::json!({"title": "tampered"});
        assert!(matches!(
            verify_chain(&events),
            Err(HashError::HashMismatch { sequence: 2 })
        ));
    }
}
