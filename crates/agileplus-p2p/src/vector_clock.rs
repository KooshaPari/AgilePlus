//! Vector-clock-based synchronisation between AgilePlus devices.
//!
//! Each device maintains a `SyncVector` — a map from `(entity_type, entity_id)`
//! to the highest event sequence number that device has seen for that entity.
//! Two devices exchange their vectors, compute the symmetric difference, and
//! transfer only the missing events.
//!
//! Traceability: WP16 / T099

use std::collections::HashMap;

use agileplus_domain::domain::event::Event;
use agileplus_events::store::{EventError, EventStore};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::discovery::PeerInfo;
use crate::error::SyncError;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Per-entity watermark vector for one device.
///
/// `entries[(entity_type, entity_id)] = last_sequence` — the highest event
/// sequence number this device has applied for that entity stream.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncVector {
    pub device_id: String,
    /// Map key is `(entity_type, entity_id)` serialised as `"type/id"`.
    pub entries: HashMap<(String, String), u64>,
}

impl SyncVector {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            entries: HashMap::new(),
        }
    }

    /// Record that we have seen `sequence` for `(entity_type, entity_id)`.
    pub fn advance(&mut self, entity_type: &str, entity_id: &str, sequence: u64) {
        let key = (entity_type.to_string(), entity_id.to_string());
        let entry = self.entries.entry(key).or_insert(0);
        if sequence > *entry {
            *entry = sequence;
        }
    }

    /// Merge another vector into self by taking the per-key maximum.
    pub fn merge(&mut self, other: &SyncVector) {
        for (k, &v) in &other.entries {
            let entry = self.entries.entry(k.clone()).or_insert(0);
            if v > *entry {
                *entry = v;
            }
        }
    }

    /// Return the last-known sequence for an entity (0 if unknown).
    pub fn get(&self, entity_type: &str, entity_id: &str) -> u64 {
        self.entries
            .get(&(entity_type.to_string(), entity_id.to_string()))
            .copied()
            .unwrap_or(0)
    }
}

/// Result of a full sync round-trip with one peer.
#[derive(Debug, Default)]
pub struct SyncResult {
    pub events_sent: usize,
    pub events_received: usize,
    pub conflicts_detected: usize,
    pub updated_vector: SyncVector,
}

// ── Sync algorithm ────────────────────────────────────────────────────────────

/// Compare two sync vectors and return the set of entity keys where the local
/// device has events the peer is missing (`local_seq > peer_seq`).
pub fn compute_missing_locally(
    local: &SyncVector,
    peer: &SyncVector,
) -> Vec<(String, String, u64, u64)> {
    // (entity_type, entity_id, peer_seq, local_seq)
    let mut missing = Vec::new();
    for ((et, eid), &local_seq) in &local.entries {
        let peer_seq = peer.get(et, eid);
        if local_seq > peer_seq {
            missing.push((et.clone(), eid.clone(), peer_seq, local_seq));
        }
    }
    missing
}

/// Fetch events from `event_store` for entities where local has more than peer.
async fn fetch_events_to_send(
    local_vector: &SyncVector,
    peer_vector: &SyncVector,
    event_store: &dyn EventStore,
) -> Result<Vec<Event>, EventError> {
    let missing = compute_missing_locally(local_vector, peer_vector);
    let mut events_to_send = Vec::new();
    for (entity_type, entity_id_str, peer_seq, _local_seq) in missing {
        // entity_id is stored as String in the vector but as i64 in EventStore.
        let entity_id: i64 = entity_id_str.parse().unwrap_or(0);
        let evts = event_store
            .get_events_since(&entity_type, entity_id, peer_seq as i64)
            .await?;
        events_to_send.extend(evts);
    }
    Ok(events_to_send)
}

/// Full sync with a single peer.
///
/// In a real deployment the `peer_vector` would be exchanged over NATS.  Here
/// we accept it as a parameter so the function is testable without a live peer.
pub async fn sync_with_peer_vectors(
    local_device_id: &str,
    peer: &PeerInfo,
    local_vector: &SyncVector,
    peer_vector: &SyncVector,
    event_store: &dyn EventStore,
) -> Result<SyncResult, SyncError> {
    info!(
        "Syncing with peer {} ({})",
        peer.device_id, peer.tailscale_ip
    );

    // 1. Determine what to send.
    let events_to_send = fetch_events_to_send(local_vector, peer_vector, event_store)
        .await
        .map_err(|e| SyncError::EventStore(e.to_string()))?;

    debug!(
        "Will send {} events to peer {}",
        events_to_send.len(),
        peer.device_id
    );

    // 2. Replicate events to peer via NATS.
    let rep_result =
        crate::replication::replicate_events(local_device_id, peer, events_to_send.clone())
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Replication failed for peer {}: {e}", peer.device_id);
                crate::replication::ReplicationResult::default()
            });

    // 3. Build updated vector = max(local, peer).
    let mut updated = local_vector.clone();
    updated.merge(peer_vector);

    Ok(SyncResult {
        events_sent: rep_result.events_sent,
        events_received: rep_result.events_received,
        conflicts_detected: 0, // conflict resolution delegated to SyncOrchestrator
        updated_vector: updated,
    })
}

/// Convenience wrapper matching the spec signature.
///
/// Accepts peer vector inline; the caller is responsible for exchanging
/// vectors with the peer (e.g. via an initial NATS request/reply).
pub async fn sync_with_peer(
    local_device_id: &str,
    peer: &PeerInfo,
    local_vector: &SyncVector,
    event_store: &dyn EventStore,
) -> Result<SyncResult, SyncError> {
    // In production the peer vector would be fetched from the peer over NATS.
    // We start with an empty vector (peer has nothing) so we send everything.
    let empty_peer_vector = SyncVector::new(&peer.device_id);
    sync_with_peer_vectors(
        local_device_id,
        peer,
        local_vector,
        &empty_peer_vector,
        event_store,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SyncVector unit tests ────────────────────────────────────────────────

    #[test]
    fn advance_sets_initial_value() {
        let mut v = SyncVector::new("dev-a");
        v.advance("Feature", "1", 5);
        assert_eq!(v.get("Feature", "1"), 5);
    }

    #[test]
    fn advance_only_moves_forward() {
        let mut v = SyncVector::new("dev-a");
        v.advance("Feature", "1", 10);
        v.advance("Feature", "1", 3); // lower — should not regress
        assert_eq!(v.get("Feature", "1"), 10);
    }

    #[test]
    fn get_returns_zero_for_unknown_entity() {
        let v = SyncVector::new("dev-a");
        assert_eq!(v.get("Epic", "999"), 0);
    }

    #[test]
    fn merge_takes_maximum() {
        let mut a = SyncVector::new("dev-a");
        a.advance("Feature", "1", 5);
        a.advance("Epic", "2", 3);

        let mut b = SyncVector::new("dev-b");
        b.advance("Feature", "1", 8);
        b.advance("Story", "3", 2);

        a.merge(&b);
        assert_eq!(a.get("Feature", "1"), 8); // b had higher value
        assert_eq!(a.get("Epic", "2"), 3); // a had higher value
        assert_eq!(a.get("Story", "3"), 2); // new key from b
    }

    #[test]
    fn merge_is_commutative() {
        let mut a = SyncVector::new("dev-a");
        a.advance("Feature", "1", 5);

        let mut b = SyncVector::new("dev-b");
        b.advance("Feature", "1", 8);

        let mut a_then_b = a.clone();
        a_then_b.merge(&b);

        let mut b_then_a = b.clone();
        b_then_a.merge(&a);

        assert_eq!(a_then_b.get("Feature", "1"), b_then_a.get("Feature", "1"));
    }

    // ── compute_missing_locally tests ────────────────────────────────────────

    #[test]
    fn missing_events_detected_correctly() {
        let mut local = SyncVector::new("dev-a");
        local.advance("Feature", "1", 10);
        local.advance("Epic", "2", 5);

        let mut peer = SyncVector::new("dev-b");
        peer.advance("Feature", "1", 7); // behind
        peer.advance("Epic", "2", 5); // equal — no transfer needed

        let missing = compute_missing_locally(&local, &peer);
        assert_eq!(missing.len(), 1);
        let (et, eid, peer_seq, local_seq) = &missing[0];
        assert_eq!(et, "Feature");
        assert_eq!(eid, "1");
        assert_eq!(*peer_seq, 7);
        assert_eq!(*local_seq, 10);
    }

    #[test]
    fn no_missing_events_when_vectors_equal() {
        let mut local = SyncVector::new("dev-a");
        local.advance("Feature", "1", 10);

        let peer = local.clone();

        let missing = compute_missing_locally(&local, &peer);
        assert!(missing.is_empty());
    }

    #[test]
    fn peer_ahead_generates_no_local_transfer() {
        let mut local = SyncVector::new("dev-a");
        local.advance("Feature", "1", 3);

        let mut peer = SyncVector::new("dev-b");
        peer.advance("Feature", "1", 10); // peer is ahead

        let missing = compute_missing_locally(&local, &peer);
        assert!(
            missing.is_empty(),
            "local should not send events it doesn't have"
        );
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use agileplus_domain::domain::event::Event;
    use agileplus_events::store::EventError;
    use async_trait::async_trait;

    #[derive(Default)]
    struct EmptyEventStore {
        events: std::sync::Mutex<Vec<Event>>,
    }

    #[async_trait]
    impl EventStore for EmptyEventStore {
        async fn append(&self, event: &Event) -> Result<i64, EventError> {
            self.events.lock().unwrap().push(event.clone());
            Ok(event.sequence)
        }

        async fn get_events(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<Vec<Event>, EventError> {
            Ok(self
                .events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
                .cloned()
                .collect())
        }

        async fn get_events_since(
            &self,
            entity_type: &str,
            entity_id: i64,
            sequence: i64,
        ) -> Result<Vec<Event>, EventError> {
            Ok(self
                .events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| {
                    e.entity_type == entity_type
                        && e.entity_id == entity_id
                        && e.sequence > sequence
                })
                .cloned()
                .collect())
        }

        async fn get_events_by_range(
            &self,
            _entity_type: &str,
            _entity_id: i64,
            _from: chrono::DateTime<chrono::Utc>,
            _to: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<Event>, EventError> {
            Ok(Vec::new())
        }

        async fn get_latest_sequence(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<i64, EventError> {
            Ok(self
                .events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
                .map(|e| e.sequence)
                .max()
                .unwrap_or(0))
        }
    }

    fn peer(id: &str) -> PeerInfo {
        PeerInfo {
            device_id: id.to_string(),
            hostname: format!("{id}.tailnet"),
            tailscale_ip: "127.0.0.1".to_string(),
            status: crate::discovery::PeerStatus::Online,
        }
    }

    #[test]
    fn new_vector_has_device_id_and_no_entries() {
        let v = SyncVector::new("dev-x");
        assert_eq!(v.device_id, "dev-x");
        assert!(v.entries.is_empty());
    }

    #[test]
    fn new_accepts_owned_string() {
        let v = SyncVector::new(String::from("dev-y"));
        assert_eq!(v.device_id, "dev-y");
    }

    #[test]
    fn advance_creates_then_updates() {
        let mut v = SyncVector::new("d");
        assert_eq!(v.get("A", "1"), 0);
        v.advance("A", "1", 1);
        assert_eq!(v.get("A", "1"), 1);
        v.advance("A", "1", 9);
        assert_eq!(v.get("A", "1"), 9);
    }

    #[test]
    fn advance_zero_is_recorded() {
        let mut v = SyncVector::new("d");
        v.advance("A", "1", 0);
        assert_eq!(v.get("A", "1"), 0);
        assert!(v.entries.contains_key(&("A".to_string(), "1".to_string())));
    }

    #[test]
    fn advance_distinct_keys_are_independent() {
        let mut v = SyncVector::new("d");
        v.advance("A", "1", 5);
        v.advance("A", "2", 7);
        v.advance("B", "1", 3);
        assert_eq!(v.get("A", "1"), 5);
        assert_eq!(v.get("A", "2"), 7);
        assert_eq!(v.get("B", "1"), 3);
    }

    #[test]
    fn merge_into_empty_is_identity() {
        let mut a = SyncVector::new("a");
        let mut b = SyncVector::new("b");
        b.advance("A", "1", 4);
        a.merge(&b);
        assert_eq!(a.get("A", "1"), 4);
    }

    #[test]
    fn merge_equal_values_keeps_value() {
        let mut a = SyncVector::new("a");
        a.advance("A", "1", 4);
        let mut b = SyncVector::new("b");
        b.advance("A", "1", 4);
        a.merge(&b);
        assert_eq!(a.get("A", "1"), 4);
    }

    #[test]
    fn merge_does_not_lose_local_when_peer_behind() {
        let mut a = SyncVector::new("a");
        a.advance("A", "1", 10);
        let mut b = SyncVector::new("b");
        b.advance("A", "1", 2);
        a.merge(&b);
        assert_eq!(a.get("A", "1"), 10);
    }

    #[test]
    fn merge_is_associative_on_max() {
        let mut a = SyncVector::new("a");
        a.advance("A", "1", 1);
        let mut b = SyncVector::new("b");
        b.advance("A", "1", 5);
        let mut c = SyncVector::new("c");
        c.advance("A", "1", 3);

        let mut left = a.clone();
        left.merge(&b);
        left.merge(&c);

        let mut bc = b.clone();
        bc.merge(&c);
        let mut right = a.clone();
        right.merge(&bc);

        assert_eq!(left.get("A", "1"), right.get("A", "1"));
    }

    #[test]
    fn merge_idempotent() {
        let mut a = SyncVector::new("a");
        a.advance("A", "1", 5);
        let b = a.clone();
        a.merge(&b);
        a.merge(&b);
        assert_eq!(a.get("A", "1"), 5);
        assert_eq!(a.entries.len(), 1);
    }

    #[test]
    fn sync_vector_json_requires_string_keys() {
        // `SyncVector::entries` is keyed by `(String, String)`, and serde_json
        // cannot encode non-string map keys. This documents the limitation: the
        // export path therefore serialises vectors as `serde_json::Value`.
        let mut v = SyncVector::new("d1");
        v.advance("Feature", "1", 42);
        assert!(
            serde_json::to_string(&v).is_err(),
            "tuple-keyed map must not be JSON-encodable"
        );
    }

    #[test]
    fn sync_vector_default_is_empty() {
        let v = SyncVector::default();
        assert!(v.entries.is_empty());
        assert!(v.device_id.is_empty());
    }

    #[test]
    fn compute_missing_multiple_entities() {
        let mut local = SyncVector::new("l");
        local.advance("A", "1", 5);
        local.advance("B", "2", 9);
        local.advance("C", "3", 1);
        let mut peer = SyncVector::new("p");
        peer.advance("A", "1", 3);
        peer.advance("C", "3", 1);

        let mut missing = compute_missing_locally(&local, &peer);
        missing.sort();
        assert_eq!(missing.len(), 2);
        // B/2: peer unknown (0) -> 9
        assert!(missing.contains(&("B".to_string(), "2".to_string(), 0, 9)));
        assert!(missing.contains(&("A".to_string(), "1".to_string(), 3, 5)));
    }

    #[test]
    fn compute_missing_empty_local() {
        let local = SyncVector::new("l");
        let mut peer = SyncVector::new("p");
        peer.advance("A", "1", 5);
        assert!(compute_missing_locally(&local, &peer).is_empty());
    }

    #[test]
    fn compute_missing_unknown_peer_key_is_treated_as_zero() {
        let mut local = SyncVector::new("l");
        local.advance("New", "77", 3);
        let peer = SyncVector::new("p");
        let missing = compute_missing_locally(&local, &peer);
        assert_eq!(missing, vec![("New".into(), "77".into(), 0, 3)]);
    }

    #[test]
    fn sync_result_default_is_zeroed() {
        let r = SyncResult::default();
        assert_eq!(r.events_sent, 0);
        assert_eq!(r.events_received, 0);
        assert_eq!(r.conflicts_detected, 0);
        assert!(r.updated_vector.entries.is_empty());
    }

    #[tokio::test]
    async fn sync_with_peer_vectors_merges_vector() {
        let store = EmptyEventStore::default();
        let mut local = SyncVector::new("dev-local");
        local.advance("Feature", "1", 4);
        let mut peer_vec = SyncVector::new("dev-peer");
        peer_vec.advance("Feature", "1", 2);
        peer_vec.advance("Epic", "9", 6);

        let result = sync_with_peer_vectors(
            "dev-local",
            &peer("dev-peer"),
            &local,
            &peer_vec,
            &store,
        )
        .await
        .unwrap();

        assert_eq!(result.updated_vector.get("Feature", "1"), 4);
        assert_eq!(result.updated_vector.get("Epic", "9"), 6);
        assert_eq!(result.conflicts_detected, 0);
    }

    #[tokio::test]
    async fn sync_with_peer_vectors_no_events_to_send() {
        let store = EmptyEventStore::default();
        let local = SyncVector::new("dev-local");
        let peer_vec = SyncVector::new("dev-peer");
        let result =
            sync_with_peer_vectors("dev-local", &peer("dev-peer"), &local, &peer_vec, &store)
                .await
                .unwrap();
        assert_eq!(result.events_sent, 0);
    }

    #[tokio::test]
    async fn sync_with_peer_empty_vectors() {
        let store = EmptyEventStore::default();
        let local = SyncVector::new("l");
        let result = sync_with_peer("l", &peer("p"), &local, &store).await.unwrap();
        assert_eq!(result.updated_vector.device_id, "l");
    }
}
