//! Integration tests for the device and discovery modules.
//!
//! Tests DeviceNode construction, InMemoryDeviceStore behavior,
//! PeerInfo/PeerStatus types, error formatting, and device
//! registration flow (via mock stores).

use agileplus_p2p::device::{DeviceNode, InMemoryDeviceStore, DeviceStore, register_device};
use agileplus_p2p::discovery::{PeerInfo, PeerStatus};
use agileplus_p2p::error::{ConnectionError, PeerDiscoveryError};

// ── DeviceNode ─────────────────────────────────────────────────────────────

#[test]
fn device_node_serde_roundtrip() {
    let node = DeviceNode {
        device_id: "abc-123".into(),
        hostname: "laptop.tailnet".into(),
        tailscale_ip: "100.64.0.1".into(),
        created_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&node).unwrap();
    let back: DeviceNode = serde_json::from_str(&json).unwrap();
    assert_eq!(back.device_id, "abc-123");
    assert_eq!(back.hostname, "laptop.tailnet");
    assert_eq!(back.tailscale_ip, "100.64.0.1");
}

#[test]
fn device_node_clone_produces_equal_copy() {
    let node = DeviceNode {
        device_id: "id-1".into(),
        hostname: "host-1".into(),
        tailscale_ip: "100.0.0.1".into(),
        created_at: chrono::Utc::now(),
    };
    let cloned = node.clone();
    assert_eq!(node.device_id, cloned.device_id);
    assert_eq!(node.hostname, cloned.hostname);
    assert_eq!(node.tailscale_ip, cloned.tailscale_ip);
}

#[test]
fn device_node_debug_format() {
    let node = DeviceNode {
        device_id: "id-1".into(),
        hostname: "host-1".into(),
        tailscale_ip: "100.0.0.1".into(),
        created_at: chrono::Utc::now(),
    };
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("DeviceNode"));
    assert!(dbg.contains("id-1"));
}

// ── InMemoryDeviceStore ────────────────────────────────────────────────────

#[test]
fn in_memory_store_empty_initially() {
    let store = InMemoryDeviceStore::default();
    assert!(store.get_device().unwrap().is_none());
}

#[test]
fn in_memory_store_insert_and_get() {
    let store = InMemoryDeviceStore::default();
    let node = DeviceNode {
        device_id: "test-id".into(),
        hostname: "test-host".into(),
        tailscale_ip: "100.0.0.1".into(),
        created_at: chrono::Utc::now(),
    };
    store.insert_device(&node).unwrap();
    let retrieved = store.get_device().unwrap().unwrap();
    assert_eq!(retrieved.device_id, "test-id");
    assert_eq!(retrieved.hostname, "test-host");
}

#[test]
fn in_memory_store_rejects_second_insert() {
    let store = InMemoryDeviceStore::default();
    let node = DeviceNode {
        device_id: "id-1".into(),
        hostname: "host-1".into(),
        tailscale_ip: "100.0.0.1".into(),
        created_at: chrono::Utc::now(),
    };
    store.insert_device(&node).unwrap();
    let result = store.insert_device(&DeviceNode {
        device_id: "id-2".into(),
        hostname: "host-2".into(),
        tailscale_ip: "100.0.0.2".into(),
        created_at: chrono::Utc::now(),
    });
    assert!(result.is_err());
    match result.unwrap_err() {
        ConnectionError::ConflictingRegistration => {}
        other => panic!("Expected ConflictingRegistration, got {:?}", other),
    }
}

#[test]
fn in_memory_store_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<InMemoryDeviceStore>();
}

// ── register_device ────────────────────────────────────────────────────────

#[tokio::test]
async fn register_device_creates_new() {
    let store = InMemoryDeviceStore::default();
    let node = register_device(&store).await.unwrap();
    assert!(!node.device_id.is_empty());
    // Validate it's a UUID
    uuid::Uuid::parse_str(&node.device_id).expect("must be valid UUID");
}

#[tokio::test]
async fn register_device_idempotent() {
    let store = InMemoryDeviceStore::default();
    let d1 = register_device(&store).await.unwrap();
    let d2 = register_device(&store).await.unwrap();
    assert_eq!(d1.device_id, d2.device_id);
    assert_eq!(d1.hostname, d2.hostname);
}

#[tokio::test]
async fn register_device_persists_timestamp() {
    let store = InMemoryDeviceStore::default();
    let before = chrono::Utc::now();
    let node = register_device(&store).await.unwrap();
    let after = chrono::Utc::now();
    assert!(node.created_at >= before);
    assert!(node.created_at <= after);
}

// ── PeerInfo / PeerStatus ──────────────────────────────────────────────────

#[test]
fn peer_status_eq() {
    assert_eq!(PeerStatus::Online, PeerStatus::Online);
    assert_eq!(PeerStatus::Offline, PeerStatus::Offline);
    assert_eq!(PeerStatus::Unknown, PeerStatus::Unknown);
    assert_ne!(PeerStatus::Online, PeerStatus::Offline);
    assert_ne!(PeerStatus::Online, PeerStatus::Unknown);
    assert_ne!(PeerStatus::Offline, PeerStatus::Unknown);
}

#[test]
fn peer_status_clone() {
    let s = PeerStatus::Online;
    let c = s.clone();
    assert_eq!(s, c);
}

#[test]
fn peer_status_debug() {
    let statuses = [PeerStatus::Online, PeerStatus::Offline, PeerStatus::Unknown];
    for s in &statuses {
        let dbg = format!("{:?}", s);
        assert!(!dbg.is_empty());
    }
}

#[test]
fn peer_info_clone() {
    let info = PeerInfo {
        device_id: "peer-1".into(),
        hostname: "laptop.tailnet".into(),
        tailscale_ip: "100.64.0.2".into(),
        status: PeerStatus::Online,
    };
    let cloned = info.clone();
    assert_eq!(cloned.device_id, "peer-1");
    assert_eq!(cloned.hostname, "laptop.tailnet");
    assert_eq!(cloned.tailscale_ip, "100.64.0.2");
    assert_eq!(cloned.status, PeerStatus::Online);
}

#[test]
fn peer_info_debug() {
    let info = PeerInfo {
        device_id: "p1".into(),
        hostname: "h1".into(),
        tailscale_ip: "100.0.0.1".into(),
        status: PeerStatus::Offline,
    };
    let dbg = format!("{:?}", info);
    assert!(dbg.contains("PeerInfo"));
    assert!(dbg.contains("p1"));
}

// ── PeerDiscoveryError ─────────────────────────────────────────────────────

#[test]
fn peer_discovery_error_api_unavailable_display() {
    let err = PeerDiscoveryError::ApiUnavailable("socket not found".into());
    let msg = format!("{}", err);
    assert!(msg.contains("Tailscale local API unavailable"));
    assert!(msg.contains("socket not found"));
}

#[test]
fn peer_discovery_error_http_display() {
    let err = PeerDiscoveryError::HttpError("timeout".into());
    let msg = format!("{}", err);
    assert!(msg.contains("HTTP error"));
    assert!(msg.contains("timeout"));
}

#[test]
fn peer_discovery_error_unsupported_platform_display() {
    let err = PeerDiscoveryError::UnsupportedPlatform;
    let msg = format!("{}", err);
    assert!(msg.contains("Unsupported platform"));
}

#[test]
fn peer_discovery_error_from_serde_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid!!!").unwrap_err();
    let err: PeerDiscoveryError = json_err.into();
    match err {
        PeerDiscoveryError::ParseError(_) => {}
        other => panic!("Expected ParseError, got {:?}", other),
    }
}

#[test]
fn peer_discovery_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err: PeerDiscoveryError = io_err.into();
    match err {
        PeerDiscoveryError::Io(_) => {}
        other => panic!("Expected Io, got {:?}", other),
    }
}

// ── ConnectionError ────────────────────────────────────────────────────────

#[test]
fn connection_error_registry_unavailable_display() {
    let err = ConnectionError::RegistryUnavailable("timeout".into());
    let msg = format!("{}", err);
    assert!(msg.contains("Could not connect to device registry"));
    assert!(msg.contains("timeout"));
}

#[test]
fn connection_error_conflicting_display() {
    let err = ConnectionError::ConflictingRegistration;
    let msg = format!("{}", err);
    assert!(msg.contains("already registered"));
}

#[test]
fn connection_error_database_display() {
    let err = ConnectionError::Database("corrupt".into());
    let msg = format!("{}", err);
    assert!(msg.contains("SQLite"));
    assert!(msg.contains("corrupt"));
}

#[test]
fn connection_error_from_peer_discovery() {
    let pd_err = PeerDiscoveryError::UnsupportedPlatform;
    let err: ConnectionError = pd_err.into();
    match err {
        ConnectionError::TailscaleQuery(_) => {}
        other => panic!("Expected TailscaleQuery, got {:?}", other),
    }
}

#[test]
fn connection_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "disk full");
    let err: ConnectionError = io_err.into();
    match err {
        ConnectionError::Io(_) => {}
        other => panic!("Expected Io, got {:?}", other),
    }
}

// ── PeerDiscoveryError display with Display impl ───────────────────────────

#[test]
fn peer_discovery_error_parse_error_display() {
    let json_err = serde_json::from_str::<serde_json::Value>("bad").unwrap_err();
    let err: PeerDiscoveryError = json_err.into();
    let msg = format!("{}", err);
    assert!(msg.contains("Failed to parse Tailscale response"));
}

#[test]
fn peer_discovery_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "something broke");
    let err: PeerDiscoveryError = io_err.into();
    let msg = format!("{}", err);
    assert!(msg.contains("IO error"));
    assert!(msg.contains("something broke"));
}
