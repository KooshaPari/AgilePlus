//! Integration tests for the Tailscale local API client.
//!
//! Both `discover_peers` and `register_device` read the daemon's
//! `/localapi/v0/status` endpoint over a UNIX socket. These tests stand up a
//! minimal HTTP/1.1 server on a UNIX socket inside a temporary directory and
//! point `TAILSCALE_SOCKET` at it, so the request/response path, the response
//! parsing and the peer mapping are exercised for real without contacting the
//! machine's actual daemon or opening a TCP connection.
//!
//! The socket-path override exists only on macOS: `tailscale_socket_path`
//! hard-codes `/var/run/tailscale/tailscaled.sock` on Linux. The whole file is
//! therefore gated on macOS rather than silently skipping.
#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use agileplus_p2p::device::{DeviceNode, DeviceStore, InMemoryDeviceStore, register_device};
use agileplus_p2p::discovery::{PeerStatus, discover_peers};
use agileplus_p2p::error::{ConnectionError, PeerDiscoveryError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

// ── Environment override ───────────────────────────────────────────────────

/// Serialises every test in this binary that rewrites the process environment.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Points `TAILSCALE_SOCKET` at a fake daemon socket and restores the previous
/// value on drop, so a failing test cannot leave the variable behind.
struct SocketOverride {
    previous: Option<String>,
    // Released only after `restore` has run (field order is not relied upon:
    // `Drop::drop` runs first, while the guard is still held).
    _guard: MutexGuard<'static, ()>,
}

impl SocketOverride {
    fn set(socket_path: &Path) -> Self {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::env::var("TAILSCALE_SOCKET").ok();
        // SAFETY: the variable is process-global. ENV_LOCK serialises every test
        // in this binary that touches it, and the guard in this struct restores
        // the previous value on drop.
        unsafe { std::env::set_var("TAILSCALE_SOCKET", socket_path) };
        Self {
            previous,
            _guard: guard,
        }
    }
}

impl Drop for SocketOverride {
    fn drop(&mut self) {
        // SAFETY: as above; the mutex guard is still held at this point.
        unsafe {
            match self.previous.take() {
                Some(value) => std::env::set_var("TAILSCALE_SOCKET", value),
                None => std::env::remove_var("TAILSCALE_SOCKET"),
            }
        }
    }
}

// ── Fake daemon ────────────────────────────────────────────────────────────

/// A one-shot HTTP/1.1 server bound to a UNIX socket in a temporary directory.
struct FakeDaemon {
    _dir: tempfile::TempDir,
    socket_path: PathBuf,
}

impl FakeDaemon {
    /// Answer the first incoming request with `body`.
    ///
    /// The socket directory is real, so this is a genuine socket round-trip;
    /// only the daemon behind it is fake.
    fn serving(body: Vec<u8>) -> Self {
        let dir = tempfile::tempdir().expect("temporary directory for the fake socket");
        let socket_path = dir.path().join("tailscaled.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind the fake tailscale socket");

        tokio::spawn(async move {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };

            // Consume the request head, which hyper terminates with a blank line.
            let mut request = Vec::new();
            let mut chunk = [0u8; 256];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                match stream.read(&mut chunk).await {
                    Ok(0) | Err(_) => break,
                    Ok(read) => request.extend_from_slice(&chunk[..read]),
                }
            }

            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes()).await;
            let _ = stream.write_all(&body).await;
            let _ = stream.flush().await;

            // Keep the connection open until the client hangs up, so the
            // response cannot be truncated by an early close. Nothing awaits
            // this task, so a client that never closes only leaks a task in the
            // test's own runtime.
            let mut sink = [0u8; 256];
            while let Ok(read) = stream.read(&mut sink).await {
                if read == 0 {
                    break;
                }
            }
        });

        Self {
            _dir: dir,
            socket_path,
        }
    }

    fn json(body: &str) -> Self {
        Self::serving(body.as_bytes().to_vec())
    }

    /// Take the process-wide environment lock for a daemon socket, returning
    /// both so the socket outlives the override.
    fn activated(self) -> (Self, SocketOverride) {
        let guard = SocketOverride::set(&self.socket_path);
        (self, guard)
    }
}

/// Path inside a fresh temporary directory that deliberately has no listener.
fn unused_socket_path(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("missing.sock")
}

fn device_node(device_id: &str, hostname: &str) -> DeviceNode {
    DeviceNode {
        device_id: device_id.to_string(),
        hostname: hostname.to_string(),
        tailscale_ip: "100.64.0.42".to_string(),
        created_at: chrono::Utc::now(),
    }
}

// ── discover_peers ─────────────────────────────────────────────────────────

#[tokio::test]
async fn discover_peers_maps_the_daemon_status_payload() {
    let (_daemon, _guard) = FakeDaemon::json(
        r#"{
            "Peer": {
                "node-1": {
                    "ID": "peer-1",
                    "DNSName": "alpha.tailnet.ts.net.",
                    "TailscaleIPs": ["100.64.0.1", "fd7a:115c::1"],
                    "Online": false
                },
                "node-2": {
                    "ID": "peer-2",
                    "DNSName": "beta.tailnet.ts.net",
                    "TailscaleIPs": ["100.64.0.2"],
                    "Online": false
                },
                "node-3": {
                    "ID": "peer-3",
                    "DNSName": "gamma.tailnet.ts.net.",
                    "TailscaleIPs": [],
                    "Online": true
                },
                "node-4": {
                    "ID": "peer-4",
                    "DNSName": "delta.tailnet.ts.net.",
                    "Online": true
                }
            }
        }"#,
    )
    .activated();

    let mut peers = discover_peers()
        .await
        .expect("fake daemon answers with JSON");
    peers.sort_by(|a, b| a.device_id.cmp(&b.device_id));

    // Peers without any address are dropped: they cannot be dialled.
    assert_eq!(
        peers.len(),
        2,
        "peers with no tailscale address must be skipped: {peers:?}"
    );

    assert_eq!(peers[0].device_id, "peer-1");
    // The daemon reports fully qualified names; the trailing dot is stripped.
    assert_eq!(peers[0].hostname, "alpha.tailnet.ts.net");
    // Only the first address is kept.
    assert_eq!(peers[0].tailscale_ip, "100.64.0.1");
    assert_eq!(peers[0].status, PeerStatus::Offline);

    assert_eq!(peers[1].device_id, "peer-2");
    // A name that already lacks the trailing dot is left alone.
    assert_eq!(peers[1].hostname, "beta.tailnet.ts.net");
    assert_eq!(peers[1].tailscale_ip, "100.64.0.2");
    assert_eq!(peers[1].status, PeerStatus::Offline);
}

#[tokio::test]
async fn discover_peers_returns_nothing_for_an_empty_peer_map() {
    let (_daemon, _guard) = FakeDaemon::json(r#"{"Peer": {}}"#).activated();

    let peers = discover_peers().await.expect("an empty map is valid");
    assert!(peers.is_empty());
}

#[tokio::test]
async fn discover_peers_returns_nothing_when_the_peer_key_is_absent() {
    let (_daemon, _guard) = FakeDaemon::json("{}").activated();

    let peers = discover_peers().await.expect("a missing Peer key is valid");
    assert!(peers.is_empty());
}

#[tokio::test]
async fn discover_peers_never_reports_an_unreachable_address_online() {
    // 192.0.2.0/24 is TEST-NET-1, reserved for documentation, so no host can
    // answer the AgilePlus probe. The peer must survive the mapping but must
    // not be advertised as online.
    let (_daemon, _guard) = FakeDaemon::json(
        r#"{
            "Peer": {
                "node-1": {
                    "ID": "peer-doc",
                    "DNSName": "unreachable.tailnet.ts.net.",
                    "TailscaleIPs": ["192.0.2.1"],
                    "Online": true
                }
            }
        }"#,
    )
    .activated();

    let peers = discover_peers()
        .await
        .expect("fake daemon answers with JSON");
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].device_id, "peer-doc");
    assert_eq!(peers[0].tailscale_ip, "192.0.2.1");
    // Either the connect is refused (Offline) or it times out (Unknown); what
    // matters is that an unreachable peer is never reported as online.
    assert_ne!(
        peers[0].status,
        PeerStatus::Online,
        "an unroutable address must not be reported as online"
    );
}

#[tokio::test]
async fn discover_peers_reports_a_missing_socket_as_api_unavailable() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = unused_socket_path(&dir);
    let _guard = SocketOverride::set(&socket_path);

    let error = discover_peers()
        .await
        .expect_err("a missing daemon socket must not be treated as success");

    match error {
        PeerDiscoveryError::ApiUnavailable(message) => {
            assert!(
                message.contains("cannot connect"),
                "the failure kind should be named: {message}"
            );
            assert!(
                message.contains(&socket_path.display().to_string()),
                "the failing socket should be named: {message}"
            );
        }
        other => panic!("expected ApiUnavailable, got {other:?}"),
    }
}

#[tokio::test]
async fn discover_peers_rejects_a_body_that_is_not_json() {
    let (_daemon, _guard) = FakeDaemon::json("this is not json").activated();

    let error = discover_peers()
        .await
        .expect_err("a non-JSON body must not decode into a status");

    assert!(
        matches!(error, PeerDiscoveryError::ParseError(_)),
        "expected ParseError, got {error:?}"
    );
}

#[tokio::test]
async fn discover_peers_rejects_a_body_that_is_not_utf8() {
    // httparse gives raw bytes; the client converts them lossily, so invalid
    // UTF-8 must surface as a parse failure rather than a panic.
    let body = vec![0xff, 0xfe, 0x00, b'{', b'}', 0x80];
    let (_daemon, _guard) = FakeDaemon::serving(body).activated();

    let error = discover_peers()
        .await
        .expect_err("a non-UTF-8 body must not decode into a status");

    assert!(
        matches!(error, PeerDiscoveryError::ParseError(_)),
        "expected ParseError, got {error:?}"
    );
}

#[tokio::test]
async fn discover_peers_rejects_a_status_of_the_wrong_shape() {
    // Parses as JSON but `Peer` is a list, not a map: this must be a typed
    // parse error, not a silent empty result.
    let (_daemon, _guard) = FakeDaemon::json(r#"{"Peer": []}"#).activated();

    let error = discover_peers()
        .await
        .expect_err("a mistyped Peer field must not decode");

    assert!(
        matches!(error, PeerDiscoveryError::ParseError(_)),
        "expected ParseError, got {error:?}"
    );
}

// ── register_device against a live socket ──────────────────────────────────

#[tokio::test]
async fn register_device_adopts_the_identity_reported_by_the_daemon() {
    let (_daemon, _guard) = FakeDaemon::json(
        r#"{
            "Self": {
                "DNSName": "laptop.tailnet.ts.net.",
                "TailscaleIPs": ["100.64.0.9", "fd7a:115c::9"]
            }
        }"#,
    )
    .activated();
    let store = InMemoryDeviceStore::default();

    let device = register_device(&store)
        .await
        .expect("registration succeeds");

    assert_eq!(device.hostname, "laptop.tailnet.ts.net");
    assert_eq!(device.tailscale_ip, "100.64.0.9");
    uuid::Uuid::parse_str(&device.device_id).expect("device id must be a UUID");

    // The record is persisted, not just returned.
    let stored = store.get_device().unwrap().expect("record was persisted");
    assert_eq!(stored.device_id, device.device_id);
    assert_eq!(stored.hostname, device.hostname);
    assert_eq!(stored.tailscale_ip, device.tailscale_ip);
    assert_eq!(stored.created_at, device.created_at);
}

#[tokio::test]
async fn register_device_falls_back_when_the_daemon_answers_garbage() {
    let (_daemon, _guard) = FakeDaemon::json("not a status document").activated();
    let store = InMemoryDeviceStore::default();

    let device = register_device(&store)
        .await
        .expect("a broken daemon must not block registration");

    // Fallback: no address from the daemon, hostname from the OS.
    assert!(
        device.tailscale_ip.is_empty(),
        "no address is available from a broken daemon: {}",
        device.tailscale_ip
    );
    assert!(
        !device.hostname.is_empty(),
        "the OS hostname must be used as a fallback"
    );
    uuid::Uuid::parse_str(&device.device_id).expect("device id must be a UUID");
    assert_eq!(
        store.get_device().unwrap().unwrap().device_id,
        device.device_id
    );
}

#[tokio::test]
async fn register_device_returns_the_stored_record_without_asking_the_daemon() {
    // The socket path deliberately has no listener: if registration consulted
    // the daemon it would fall back to the OS hostname, so the stored hostname
    // coming back verbatim proves the daemon was never queried.
    let dir = tempfile::tempdir().unwrap();
    let socket_path = unused_socket_path(&dir);
    let _guard = SocketOverride::set(&socket_path);

    let store = InMemoryDeviceStore::default();
    store
        .insert_device(&device_node("stored-device", "stored-host"))
        .expect("insert into an empty store");

    let device = register_device(&store).await.unwrap();

    assert_eq!(device.device_id, "stored-device");
    assert_eq!(device.hostname, "stored-host");
    assert_eq!(device.tailscale_ip, "100.64.0.42");
}

#[tokio::test]
async fn register_device_surfaces_a_missing_daemon_as_a_fallback_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = unused_socket_path(&dir);
    let _guard = SocketOverride::set(&socket_path);
    let store = InMemoryDeviceStore::default();

    let device = register_device(&store)
        .await
        .expect("an absent daemon must degrade, not fail");

    assert!(device.tailscale_ip.is_empty());
    assert!(!device.hostname.is_empty());
}

#[tokio::test]
async fn register_device_still_reports_store_conflicts_after_a_daemon_query() {
    let (_daemon, _guard) = FakeDaemon::json(
        r#"{"Self": {"DNSName": "host.tailnet.ts.net.", "TailscaleIPs": ["100.64.0.7"]}}"#,
    )
    .activated();

    // A store that already holds a record refuses the insert; registration must
    // propagate that rather than swallow it behind the daemon result.
    struct RejectingStore;

    impl DeviceStore for RejectingStore {
        fn insert_device(&self, _device: &DeviceNode) -> Result<(), ConnectionError> {
            Err(ConnectionError::ConflictingRegistration)
        }

        fn get_device(&self) -> Result<Option<DeviceNode>, ConnectionError> {
            Ok(None)
        }
    }

    let error = register_device(&RejectingStore)
        .await
        .expect_err("a conflicting registration must fail");

    assert!(
        matches!(error, ConnectionError::ConflictingRegistration),
        "expected ConflictingRegistration, got {error:?}"
    );
}
