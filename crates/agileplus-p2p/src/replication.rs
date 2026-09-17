//! Event replication over NATS between AgilePlus peers.
//!
//! Connects to a peer's NATS instance (at `nats://{peer_tailscale_ip}:4222`),
//! publishes local events to the peer's device subject, and subscribes to
//! receive events the peer wants to push to us.
//!
//! Uses JetStream for reliable delivery with message persistence.
//!
//! Traceability: WP16 / T098

use async_nats::jetstream;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

use agileplus_domain::domain::event::Event;

use crate::discovery::PeerInfo;
use crate::error::SyncError;

// ── NATS subject helpers ──────────────────────────────────────────────────────

/// Subject to which a device publishes events destined for `target_device_id`.
pub fn device_subject(target_device_id: &str) -> String {
    format!("agileplus.sync.device.{target_device_id}")
}

// ── Wire format ───────────────────────────────────────────────────────────────

/// Serialisable wrapper around a batch of events for over-the-wire transfer.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventBatch {
    pub sender_device_id: String,
    pub events: Vec<Event>,
}

// ── Result type ───────────────────────────────────────────────────────────────

/// Outcome of a single replication attempt with one peer.
#[derive(Debug, Default)]
pub struct ReplicationResult {
    pub events_sent: usize,
    pub events_received: usize,
}

// ── Core replication logic ────────────────────────────────────────────────────

/// Replicate `events` to `peer` and collect any events the peer sends back.
///
/// Connection is attempted with exponential backoff (3 retries: 1 s, 2 s, 4 s).
pub async fn replicate_events(
    local_device_id: &str,
    peer: &PeerInfo,
    events: Vec<Event>,
) -> Result<ReplicationResult, SyncError> {
    let url = format!("nats://{}:4222", peer.tailscale_ip);
    let client = connect_with_retry(&url, &peer.device_id).await?;
    let js = jetstream::new(client);

    // Ensure the peer's stream exists (or create it).
    let peer_stream_name = format!(
        "AGILEPLUS_SYNC_{}",
        peer.device_id.replace('-', "_").to_uppercase()
    );
    let peer_subject = device_subject(&peer.device_id);
    let _ = js
        .get_or_create_stream(jetstream::stream::Config {
            name: peer_stream_name.clone(),
            subjects: vec![peer_subject.clone()],
            ..Default::default()
        })
        .await;

    // Publish events to the peer's subject.
    let events_sent = events.len();
    let batch = EventBatch {
        sender_device_id: local_device_id.to_string(),
        events,
    };
    let payload = serde_json::to_vec(&batch)?;

    js.publish(peer_subject.clone(), payload.into())
        .await
        .map_err(|e| SyncError::PublishFailed(e.to_string()))?
        .await
        .map_err(|e| SyncError::PublishFailed(e.to_string()))?;

    info!(
        "Sent {events_sent} events to peer {} on {url}",
        peer.device_id
    );

    // Attempt to drain any events the peer already published to our subject.
    let local_subject = device_subject(local_device_id);
    let local_stream_name = format!(
        "AGILEPLUS_SYNC_{}",
        local_device_id.replace('-', "_").to_uppercase()
    );
    let _ = js
        .get_or_create_stream(jetstream::stream::Config {
            name: local_stream_name.clone(),
            subjects: vec![local_subject.clone()],
            ..Default::default()
        })
        .await;

    let events_received = drain_pending(&js, &local_stream_name, &local_subject).await;

    Ok(ReplicationResult {
        events_sent,
        events_received,
    })
}

/// Connect to a NATS server with up to 3 retries (1 s / 2 s / 4 s).
async fn connect_with_retry(url: &str, peer_id: &str) -> Result<async_nats::Client, SyncError> {
    let delays = [
        Duration::from_secs(1),
        Duration::from_secs(2),
        Duration::from_secs(4),
    ];
    let mut last_err = SyncError::ConnectionFailed {
        peer_id: peer_id.to_string(),
        reason: "no attempts made".to_string(),
    };

    for (attempt, delay) in delays.iter().enumerate() {
        match async_nats::connect(url).await {
            Ok(c) => {
                debug!("Connected to NATS at {url} on attempt {}", attempt + 1);
                return Ok(c);
            }
            Err(e) => {
                warn!("NATS connect attempt {} to {url} failed: {e}", attempt + 1);
                last_err = SyncError::ConnectionFailed {
                    peer_id: peer_id.to_string(),
                    reason: e.to_string(),
                };
                sleep(*delay).await;
            }
        }
    }
    error!("All NATS connect attempts to {url} failed for peer {peer_id}");
    Err(last_err)
}

/// Pull and count any messages already queued in `stream` / `subject`.
async fn drain_pending(js: &jetstream::Context, stream_name: &str, subject: &str) -> usize {
    use tokio::time::timeout;

    let consumer_cfg = jetstream::consumer::pull::Config {
        filter_subject: subject.to_string(),
        deliver_policy: jetstream::consumer::DeliverPolicy::All,
        ..Default::default()
    };

    let stream = match js.get_stream(stream_name).await {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let consumer = match stream.create_consumer(consumer_cfg).await {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let mut count = 0usize;
    loop {
        let batch = match timeout(
            Duration::from_millis(250),
            consumer.fetch().max_messages(50).messages(),
        )
        .await
        {
            Ok(Ok(b)) => b,
            _ => break,
        };

        let msgs: Vec<_> = batch
            .take_until(tokio::time::sleep(Duration::from_millis(100)))
            .collect()
            .await;

        if msgs.is_empty() {
            break;
        }

        for msg in msgs.into_iter().flatten() {
            count += 1;
            let _ = msg.ack().await;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_subject_format() {
        let s = device_subject("device-abc-123");
        assert_eq!(s, "agileplus.sync.device.device-abc-123");
    }

    #[test]
    fn event_batch_roundtrip() {
        use agileplus_domain::domain::event::Event;
        let batch = EventBatch {
            sender_device_id: "dev-1".to_string(),
            events: vec![Event::new(
                "Feature",
                42,
                "created",
                serde_json::json!({}),
                "test",
            )],
        };
        let json = serde_json::to_string(&batch).unwrap();
        let back: EventBatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sender_device_id, "dev-1");
        assert_eq!(back.events.len(), 1);
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use agileplus_domain::domain::event::Event;

    fn peer(id: &str, ip: &str) -> PeerInfo {
        PeerInfo {
            device_id: id.to_string(),
            hostname: format!("{id}.ts.net"),
            tailscale_ip: ip.to_string(),
            status: crate::discovery::PeerStatus::Offline,
        }
    }

    fn event(entity_id: i64, seq: i64) -> Event {
        let mut e = Event::new("Feature", entity_id, "created", serde_json::json!({}), "t");
        e.sequence = seq;
        e
    }

    #[test]
    fn device_subject_simple_id() {
        assert_eq!(device_subject("abc"), "agileplus.sync.device.abc");
    }

    #[test]
    fn device_subject_uuid_like_id() {
        let id = "1b4e28ba-2fa1-11d2-883f-0016d3cca427";
        assert_eq!(device_subject(id), format!("agileplus.sync.device.{id}"));
    }

    #[test]
    fn device_subject_empty_id() {
        assert_eq!(device_subject(""), "agileplus.sync.device.");
    }

    #[test]
    fn event_batch_empty_events_roundtrip() {
        let batch = EventBatch {
            sender_device_id: "s".into(),
            events: vec![],
        };
        let json = serde_json::to_string(&batch).unwrap();
        let back: EventBatch = serde_json::from_str(&json).unwrap();
        assert!(back.events.is_empty());
        assert_eq!(back.sender_device_id, "s");
    }

    #[test]
    fn event_batch_multiple_events_roundtrip() {
        let batch = EventBatch {
            sender_device_id: "dev-a".into(),
            events: vec![event(1, 1), event(1, 2), event(2, 1)],
        };
        let json = serde_json::to_string(&batch).unwrap();
        let back: EventBatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back.events.len(), 3);
        assert_eq!(back.events[1].sequence, 2);
        assert_eq!(back.events[2].entity_id, 2);
    }

    #[test]
    fn event_batch_debug() {
        let batch = EventBatch {
            sender_device_id: "dbg".into(),
            events: vec![],
        };
        assert!(format!("{batch:?}").contains("dbg"));
    }

    #[test]
    fn replication_result_default_zeroed() {
        let r = ReplicationResult::default();
        assert_eq!(r.events_sent, 0);
        assert_eq!(r.events_received, 0);
    }

    #[test]
    fn replication_result_debug() {
        let r = ReplicationResult {
            events_sent: 3,
            events_received: 4,
        };
        let s = format!("{r:?}");
        assert!(s.contains('3'));
        assert!(s.contains('4'));
    }

    #[tokio::test]
    async fn replicate_events_unreachable_peer_errors() {
        // TEST-NET-1 address: connection attempts fail, exercising the
        // retry/backoff path and returning a ConnectionFailed error.
        let p = peer("dev-unreachable", "192.0.2.1");
        let err = replicate_events("local", &p, vec![event(1, 1)])
            .await
            .unwrap_err();
        match err {
            SyncError::ConnectionFailed { peer_id, .. } => {
                assert_eq!(peer_id, "dev-unreachable")
            }
            other => panic!("expected ConnectionFailed, got {other:?}"),
        }
    }
}
