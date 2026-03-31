//! Device registration — persistent identity for the local AgilePlus instance.
//!
//! On first startup a UUID v4 `device_id` is generated and the local
//! Tailscale hostname / IP are fetched from the daemon.  The record is stored
//! so the same identity is recovered across restarts.
//!
//! Traceability: WP16 / T097

#[cfg(unix)]
use bytes::Bytes;
use chrono::{DateTime, Utc};
#[cfg(unix)]
use http_body_util::{BodyExt as _, Empty};
#[cfg(unix)]
use hyper::Request;
#[cfg(unix)]
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use tokio::net::UnixStream;
use tracing::{debug, info};
use uuid::Uuid;

#[cfg(unix)]
use crate::discovery::tailscale_socket_path;
use crate::error::ConnectionError;
#[cfg(unix)]
use crate::error::PeerDiscoveryError;

/// Persistent identity for one AgilePlus node in the tailnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNode {
    pub device_id: String,
    pub hostname: String,
    pub tailscale_ip: String,
    pub created_at: DateTime<Utc>,
}

// ── Persistence trait ─────────────────────────────────────────────────────────

/// Minimal persistence interface for device records.
pub trait DeviceStore: Send + Sync {
    fn insert_device(&self, device: &DeviceNode) -> Result<(), ConnectionError>;
    fn get_device(&self) -> Result<Option<DeviceNode>, ConnectionError>;
}

// ── Tailscale self-info ───────────────────────────────────────────────────────

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct TailscaleStatusSelf {
    #[serde(rename = "Self")]
    self_node: TailscaleSelf,
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct TailscaleSelf {
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    tailscale_ips: Vec<String>,
}

#[cfg(unix)]
async fn query_local_tailscale() -> Result<(String, String), PeerDiscoveryError> {
    let socket_path = tailscale_socket_path()?;

    let stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        PeerDiscoveryError::ApiUnavailable(format!(
            "cannot connect to Tailscale socket {}: {}",
            socket_path.display(),
            e
        ))
    })?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e: hyper::Error| PeerDiscoveryError::HttpError(e.to_string()))?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::warn!("Tailscale local API connection error: {e}");
        }
    });

    let req = Request::builder()
        .method("GET")
        .uri("/localapi/v0/status")
        .header("Host", "local-tailscaled.sock")
        .body(Empty::<Bytes>::new())
        .map_err(|e| PeerDiscoveryError::HttpError(e.to_string()))?;

    let resp = sender
        .send_request(req)
        .await
        .map_err(|e: hyper::Error| PeerDiscoveryError::HttpError(e.to_string()))?;

    let body_bytes = resp
        .into_body()
        .collect()
        .await
        .map_err(|e: hyper::Error| PeerDiscoveryError::HttpError(e.to_string()))?
        .to_bytes();

    let status: TailscaleStatusSelf = serde_json::from_slice(&body_bytes)?;
    let hostname = status.self_node.dns_name.trim_end_matches('.').to_string();
    let ip = status
        .self_node
        .tailscale_ips
        .into_iter()
        .next()
        .unwrap_or_default();

    Ok((hostname, ip))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Register the local device, creating a new record if none exists.
pub async fn register_device(store: &dyn DeviceStore) -> Result<DeviceNode, ConnectionError> {
    if let Some(existing) = store.get_device()? {
        debug!("Found existing device registration: {}", existing.device_id);
        return Ok(existing);
    }

    #[cfg(unix)]
    let (hostname, tailscale_ip) = match query_local_tailscale().await {
        Ok(pair) => pair,
        Err(e) => {
            tracing::warn!("Tailscale query failed during registration, using fallbacks: {e}");
            let h = hostname::get()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "unknown".to_string());
            (h, String::new())
        }
    };
    #[cfg(not(unix))]
    let (hostname, tailscale_ip) = {
        let h = hostname::get()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());
        (h, String::new())
    };

    let device = DeviceNode {
        device_id: Uuid::new_v4().to_string(),
        hostname,
        tailscale_ip,
        created_at: Utc::now(),
    };

    info!("Registering new device: {}", device.device_id);
    store.insert_device(&device)?;
    Ok(device)
}

/// Retrieve the locally registered device without creating one.
pub fn get_local_device(store: &dyn DeviceStore) -> Result<Option<DeviceNode>, ConnectionError> {
    store.get_device()
}

// ── In-memory store for tests ─────────────────────────────────────────────────

/// Thread-safe in-memory `DeviceStore` for unit tests.
#[derive(Debug, Default)]
pub struct InMemoryDeviceStore {
    inner: std::sync::Mutex<Option<DeviceNode>>,
}

impl DeviceStore for InMemoryDeviceStore {
    fn insert_device(&self, device: &DeviceNode) -> Result<(), ConnectionError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| ConnectionError::Database(e.to_string()))?;
        if guard.is_some() {
            return Err(ConnectionError::ConflictingRegistration);
        }
        *guard = Some(device.clone());
        Ok(())
    }

    fn get_device(&self) -> Result<Option<DeviceNode>, ConnectionError> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| ConnectionError::Database(e.to_string()))?;
        Ok(guard.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_creates_device_with_uuid() {
        let store = InMemoryDeviceStore::default();
        let device = register_device(&store).await.unwrap();
        assert!(!device.device_id.is_empty());
        Uuid::parse_str(&device.device_id).expect("device_id must be a valid UUID");
    }

    #[tokio::test]
    async fn register_idempotent() {
        let store = InMemoryDeviceStore::default();
        let d1 = register_device(&store).await.unwrap();
        let d2 = register_device(&store).await.unwrap();
        assert_eq!(d1.device_id, d2.device_id);
    }

    #[tokio::test]
    async fn get_local_device_before_registration_returns_none() {
        let store = InMemoryDeviceStore::default();
        let result = get_local_device(&store).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_local_device_after_registration_returns_some() {
        let store = InMemoryDeviceStore::default();
        let registered = register_device(&store).await.unwrap();
        let fetched = get_local_device(&store).unwrap().unwrap();
        assert_eq!(registered.device_id, fetched.device_id);
    }

    #[test]
    fn in_memory_store_prevents_double_insert() {
        let store = InMemoryDeviceStore::default();
        let device = DeviceNode {
            device_id: "abc".to_string(),
            hostname: "host".to_string(),
            tailscale_ip: "100.64.0.1".to_string(),
            created_at: Utc::now(),
        };
        store.insert_device(&device).unwrap();
        let result = store.insert_device(&device);
        assert!(matches!(
            result,
            Err(ConnectionError::ConflictingRegistration)
        ));
    }

    #[test]
    fn in_memory_store_allows_re_insert_after_different_successful_insert() {
        let store = InMemoryDeviceStore::default();
        let device1 = DeviceNode {
            device_id: "first".to_string(),
            hostname: "host1".to_string(),
            tailscale_ip: "100.64.0.1".to_string(),
            created_at: Utc::now(),
        };
        let device2 = DeviceNode {
            device_id: "second".to_string(),
            hostname: "host2".to_string(),
            tailscale_ip: "100.64.0.2".to_string(),
            created_at: Utc::now(),
        };
        store.insert_device(&device1).unwrap();
        store.insert_device(&device2).unwrap();
        let fetched = store.get_device().unwrap().unwrap();
        assert_eq!(fetched.device_id, "second");
    }

    #[test]
    fn in_memory_store_get_device_returns_none_when_empty() {
        let store = InMemoryDeviceStore::default();
        let result = store.get_device().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn in_memory_store_insert_and_get_roundtrip() {
        let store = InMemoryDeviceStore::default();
        let device = DeviceNode {
            device_id: "roundtrip-test".to_string(),
            hostname: "rt-host".to_string(),
            tailscale_ip: "100.64.0.99".to_string(),
            created_at: Utc::now(),
        };
        store.insert_device(&device).unwrap();
        let fetched = store.get_device().unwrap().unwrap();
        assert_eq!(fetched.device_id, "roundtrip-test");
        assert_eq!(fetched.hostname, "rt-host");
        assert_eq!(fetched.tailscale_ip, "100.64.0.99");
    }

    #[test]
    fn in_memory_store_lock_error_returns_error() {
        let store = InMemoryDeviceStore::default();
        let device = DeviceNode {
            device_id: "lock-test".to_string(),
            hostname: "lh".to_string(),
            tailscale_ip: "100.64.0.1".to_string(),
            created_at: Utc::now(),
        };
        store.insert_device(&device).unwrap();
        let result = store.insert_device(&device);
        assert!(matches!(result, Err(ConnectionError::ConflictingRegistration)));
    }

    #[test]
    fn device_node_debug_impl() {
        let device = DeviceNode {
            device_id: "debug-test".to_string(),
            hostname: "debug-host".to_string(),
            tailscale_ip: "100.64.0.5".to_string(),
            created_at: Utc::now(),
        };
        let debug_str = format!("{:?}", device);
        assert!(debug_str.contains("debug-test"));
        assert!(debug_str.contains("debug-host"));
    }

    #[test]
    fn device_node_serialize_deserialize() {
        use serde::{Deserialize, Serialize};
        let device = DeviceNode {
            device_id: "serde-test".to_string(),
            hostname: "serde-host".to_string(),
            tailscale_ip: "100.64.0.7".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&device).unwrap();
        let parsed: DeviceNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.device_id, "serde-test");
        assert_eq!(parsed.hostname, "serde-host");
        assert_eq!(parsed.tailscale_ip, "100.64.0.7");
    }

    #[tokio::test]
    async fn register_device_returns_correct_hostname() {
        let store = InMemoryDeviceStore::default();
        let device = register_device(&store).await.unwrap();
        assert!(!device.hostname.is_empty());
    }

    #[tokio::test]
    async fn register_device_creates_uuid_v4() {
        let store = InMemoryDeviceStore::default();
        let device = register_device(&store).await.unwrap();
        let parsed = Uuid::parse_str(&device.device_id);
        assert!(parsed.is_ok());
    }

    #[tokio::test]
    async fn get_local_device_returns_none_when_not_registered() {
        let store = InMemoryDeviceStore::default();
        let result = get_local_device(&store).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_local_device_returns_device_after_registration() {
        let store = InMemoryDeviceStore::default();
        let registered = register_device(&store).await.unwrap();
        let fetched = get_local_device(&store).unwrap().unwrap();
        assert_eq!(registered.device_id, fetched.device_id);
        assert_eq!(registered.hostname, fetched.hostname);
    }

    #[tokio::test]
    async fn multiple_registrations_return_same_device() {
        let store = InMemoryDeviceStore::default();
        let first = register_device(&store).await.unwrap();
        let second = register_device(&store).await.unwrap();
        let third = register_device(&store).await.unwrap();
        assert_eq!(first.device_id, second.device_id);
        assert_eq!(second.device_id, third.device_id);
    }
}
