//! Peer discovery via Tailscale local API.
//!
//! Connects to the Tailscale daemon's local UNIX socket and queries
//! `/localapi/v0/status` to enumerate peers on the tailnet.
//! Traceability: WP16 / T096

#[cfg(unix)]
use bytes::Bytes;
#[cfg(unix)]
use http_body_util::{BodyExt as _, Empty};
#[cfg(unix)]
use hyper::Request;
#[cfg(unix)]
use hyper_util::rt::TokioIo;
#[cfg(unix)]
use serde::Deserialize;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(unix)]
use tracing::{debug, warn};

use crate::error::PeerDiscoveryError;

/// Information about a discovered peer on the Tailscale network.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub device_id: String,
    pub hostname: String,
    pub tailscale_ip: String,
    pub status: PeerStatus,
}

/// Availability status of a discovered peer.
#[derive(Debug, Clone, PartialEq)]
pub enum PeerStatus {
    /// Peer is online and AgilePlus is detected.
    Online,
    /// Peer is reachable on the tailnet but AgilePlus is not running.
    Offline,
    /// Peer status could not be determined.
    Unknown,
}

// ── Tailscale JSON response shapes ──────────────────────────────────────────

#[cfg(unix)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TailscaleStatus {
    #[serde(rename = "Peer", default)]
    peer: std::collections::HashMap<String, TailscalePeer>,
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct TailscalePeer {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    tailscale_ips: Vec<String>,
    #[serde(rename = "Online", default)]
    online: bool,
}

// ── Socket path resolution ───────────────────────────────────────────────────

/// Return the path of the Tailscale daemon UNIX socket for the current platform.
pub fn tailscale_socket_path() -> Result<std::path::PathBuf, PeerDiscoveryError> {
    #[cfg(target_os = "linux")]
    {
        Ok(std::path::PathBuf::from(
            "/var/run/tailscale/tailscaled.sock",
        ))
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(p) = std::env::var("TAILSCALE_SOCKET") {
            return Ok(std::path::PathBuf::from(p));
        }
        Ok(std::path::PathBuf::from(
            "/var/run/tailscale/tailscaled.sock",
        ))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(PeerDiscoveryError::UnsupportedPlatform)
    }
}

// ── HTTP-over-UNIX-socket client ─────────────────────────────────────────────

#[cfg(unix)]
async fn tailscale_get(path: &str) -> Result<String, PeerDiscoveryError> {
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
            warn!("Tailscale local API connection error: {e}");
        }
    });

    let req = Request::builder()
        .method("GET")
        .uri(path)
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

    Ok(String::from_utf8_lossy(&body_bytes).into_owned())
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Discover peers on the local Tailscale network.
#[cfg(unix)]
pub async fn discover_peers() -> Result<Vec<PeerInfo>, PeerDiscoveryError> {
    let body = tailscale_get("/localapi/v0/status").await?;
    debug!("Tailscale status response: {} bytes", body.len());

    let status: TailscaleStatus = serde_json::from_str(&body)?;
    let mut peers = Vec::new();

    for (_key, peer) in status.peer {
        let tailscale_ip = peer.tailscale_ips.into_iter().next().unwrap_or_default();
        if tailscale_ip.is_empty() {
            continue;
        }

        let hostname = peer.dns_name.trim_end_matches('.').to_string();

        let status = if peer.online {
            probe_agileplus(&tailscale_ip).await
        } else {
            PeerStatus::Offline
        };

        peers.push(PeerInfo {
            device_id: peer.id,
            hostname,
            tailscale_ip,
            status,
        });
    }

    Ok(peers)
}

/// Attempt a short TCP connection to `ip:3000` to detect AgilePlus.
#[cfg(unix)]
async fn probe_agileplus(ip: &str) -> PeerStatus {
    use tokio::net::TcpStream;
    use tokio::time::{Duration, timeout};

    let addr = format!("{ip}:3000");
    match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => PeerStatus::Online,
        Ok(Err(_)) => PeerStatus::Offline,
        Err(_) => PeerStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_status_equality() {
        assert_eq!(PeerStatus::Online, PeerStatus::Online);
        assert_ne!(PeerStatus::Online, PeerStatus::Offline);
    }

    #[test]
    fn socket_path_not_empty() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let p = tailscale_socket_path().unwrap();
            assert!(!p.as_os_str().is_empty());
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            assert!(tailscale_socket_path().is_err());
        }
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn peer_status_online_offline_unknown_distinct() {
        assert_ne!(PeerStatus::Online, PeerStatus::Offline);
        assert_ne!(PeerStatus::Offline, PeerStatus::Unknown);
        assert_ne!(PeerStatus::Online, PeerStatus::Unknown);
    }

    #[test]
    fn peer_status_clone_and_debug() {
        let s = PeerStatus::Online;
        let c = s.clone();
        assert_eq!(s, c);
        assert!(format!("{s:?}").contains("Online"));
    }

    #[test]
    fn peer_info_clone_preserves_fields() {
        let p = PeerInfo {
            device_id: "dev".into(),
            hostname: "host".into(),
            tailscale_ip: "100.0.0.1".into(),
            status: PeerStatus::Offline,
        };
        let c = p.clone();
        assert_eq!(c.device_id, p.device_id);
        assert_eq!(c.hostname, p.hostname);
        assert_eq!(c.tailscale_ip, p.tailscale_ip);
        assert_eq!(c.status, p.status);
    }

    #[test]
    fn peer_info_debug() {
        let p = PeerInfo {
            device_id: "dev".into(),
            hostname: "host".into(),
            tailscale_ip: "100.0.0.1".into(),
            status: PeerStatus::Online,
        };
        let s = format!("{p:?}");
        assert!(s.contains("dev"));
        assert!(s.contains("100.0.0.1"));
    }

    #[cfg(unix)]
    #[test]
    fn tailscale_status_parses_single_peer() {
        let json = r#"{
            "Peer": {
                "node1": {
                    "ID": "node-1",
                    "DNSName": "alpha.tailnet.ts.net.",
                    "TailscaleIPs": ["100.64.0.1", "fd7a::1"],
                    "Online": true
                }
            }
        }"#;
        let status: TailscaleStatus = serde_json::from_str(json).unwrap();
        let peer = status.peer.get("node1").unwrap();
        assert_eq!(peer.id, "node-1");
        assert_eq!(peer.dns_name, "alpha.tailnet.ts.net.");
        assert_eq!(peer.tailscale_ips.len(), 2);
        assert!(peer.online);
    }

    #[cfg(unix)]
    #[test]
    fn tailscale_status_parses_empty_peer_map() {
        let status: TailscaleStatus = serde_json::from_str(r#"{"Peer": {}}"#).unwrap();
        assert!(status.peer.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn tailscale_status_defaults_report_missing_fields() {
        let status: TailscaleStatus =
            serde_json::from_str(r#"{"Peer": {"n": {"ID": "id"}}}"#).unwrap();
        let peer = status.peer.get("n").unwrap();
        assert_eq!(peer.id, "id");
        assert!(peer.dns_name.is_empty());
        assert!(peer.tailscale_ips.is_empty());
        assert!(!peer.online);
    }

    #[cfg(unix)]
    #[test]
    fn tailscale_status_missing_peer_key_defaults_empty() {
        let status: TailscaleStatus = serde_json::from_str("{}").unwrap();
        assert!(status.peer.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn tailscale_status_invalid_json_is_parse_error() {
        let r: Result<TailscaleStatus, _> = serde_json::from_str(": not json");
        assert!(r.is_err());
    }

    #[test]
    fn socket_path_is_absolute_on_supported_platforms() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let p = tailscale_socket_path().unwrap();
            assert!(p.is_absolute(), "socket path should be absolute: {p:?}");
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn probe_agileplus_unroutable_ip_times_out_to_unknown() {
        // 192.0.2.0/24 is TEST-NET-1: guaranteed non-routable, so the 2s
        // connect timeout fires and the peer is reported Unknown.
        let status = probe_agileplus("192.0.2.1").await;
        assert_eq!(status, PeerStatus::Unknown);
    }

    #[test]
    fn discovery_module_compiles_without_unix_gate() {
        // `discover_peers` is unix-gated; the module should still expose
        // PeerInfo / PeerStatus on all platforms.
        let _ = PeerStatus::Unknown;
    }
}
