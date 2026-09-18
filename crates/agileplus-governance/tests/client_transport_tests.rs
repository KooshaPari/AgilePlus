//! Transport-level tests for `GovernanceClient`: remote health checks,
//! detached audit sync, and the config-file builder path.
//!
//! These exercise the four `test_connection` outcomes (success, non-2xx
//! status, transport failure, timeout) plus the `log_audit` remote-sync
//! branch against a real in-process TCP listener. No mocked transports:
//! the client speaks HTTP to a socket that this test owns.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use agileplus_governance::audit::{AuditEvent, AuditFilter};
use agileplus_governance::client::GovernanceClientBuilder;
use agileplus_governance::config::{
    AuthSettings, GovernanceConfig, GovernanceSettings, LocalSettings, SyncSettings,
};
use agileplus_governance::{ConnectionStatus, GovernanceClient, GovernanceError};

/// Minimal HTTP/1.1 server: answers every request with `status` after
/// optionally sleeping `delay` (used to force the client's timeout path).
struct TestServer {
    base_url: String,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(status: &'static str, delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local_addr");
        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                tokio::spawn(async move {
                    let mut buf = [0u8; 2048];
                    let _ =
                        tokio::time::timeout(Duration::from_secs(2), socket.read(&mut buf)).await;
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    }
                    let response =
                        format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        Self {
            base_url: format!("http://{addr}"),
            handle,
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn remote_config(base_url: &str, timeout_secs: u64, dir: &std::path::Path) -> GovernanceConfig {
    GovernanceConfig {
        governance: GovernanceSettings {
            enabled: true,
            base_url: base_url.to_string(),
            auth: AuthSettings::default(),
            timeout_secs,
            retry_attempts: 1,
        },
        local: LocalSettings {
            enabled: true,
            db_path: dir.join("gov.db").to_string_lossy().into_owned(),
            retention_days: 7,
        },
        sync: SyncSettings {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[tokio::test]
async fn new_reports_connected_when_health_check_succeeds() {
    let server = TestServer::start("200 OK", Duration::ZERO).await;
    let dir = tempfile::tempdir().unwrap();

    let client = GovernanceClient::new(remote_config(&server.base_url, 5, dir.path()))
        .await
        .expect("health check should succeed against a 200 responder");

    assert_eq!(
        client.connection_status().await,
        ConnectionStatus::Connected
    );
    assert!(client.status().await.remote_enabled);
}

#[tokio::test]
async fn new_surfaces_non_success_health_status() {
    let server = TestServer::start("503 Service Unavailable", Duration::ZERO).await;
    let dir = tempfile::tempdir().unwrap();

    let err = GovernanceClient::new(remote_config(&server.base_url, 5, dir.path()))
        .await
        .err()
        .expect("a 503 health check must not be treated as connected");

    assert!(
        matches!(err, GovernanceError::Network(_)),
        "expected Network error, got {err:?}"
    );
    assert_eq!(
        err.to_string(),
        "Network error: Health check failed with status 503 Service Unavailable"
    );
    assert_eq!(err.status_code(), 503);
}

#[tokio::test]
async fn new_surfaces_transport_failure_when_endpoint_is_unreachable() {
    let dir = tempfile::tempdir().unwrap();
    // Port 1 is privileged and effectively never listening.
    let err = GovernanceClient::new(remote_config("http://127.0.0.1:1", 5, dir.path()))
        .await
        .err()
        .expect("connection to a closed port must fail");

    assert!(matches!(err, GovernanceError::Network(_)));
    let message = err.to_string();
    assert!(
        !message.contains("Health check failed"),
        "transport failure must not be reported as an HTTP status: {message}"
    );
    assert!(
        !message.contains("timeout"),
        "transport failure must not be reported as a timeout: {message}"
    );
}

#[tokio::test]
async fn new_reports_timeout_when_health_endpoint_hangs() {
    // Server accepts the connection but stalls well past the client timeout.
    let server = TestServer::start("200 OK", Duration::from_secs(5)).await;
    let dir = tempfile::tempdir().unwrap();

    let err = GovernanceClient::new(remote_config(&server.base_url, 1, dir.path()))
        .await
        .err()
        .expect("a hanging health check must time out");

    assert_eq!(err.to_string(), "Network error: Connection timeout");
    assert_eq!(err.status_code(), 503);
}

#[tokio::test]
async fn log_audit_syncs_to_remote_and_records_last_sync() {
    let server = TestServer::start("200 OK", Duration::ZERO).await;
    let dir = tempfile::tempdir().unwrap();
    let client = GovernanceClient::new(remote_config(&server.base_url, 5, dir.path()))
        .await
        .unwrap();

    assert!(client.status().await.last_sync.is_none());
    client
        .log_audit(AuditEvent::success("sync-me"))
        .await
        .unwrap();

    // Sync happens on a detached task; poll briefly for its observable effect.
    let mut synced = false;
    for _ in 0..100 {
        if client.status().await.last_sync.is_some() {
            synced = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(synced, "detached sync task never recorded last_sync");
}

#[tokio::test]
async fn log_audit_tolerates_remote_sync_failure() {
    let server = TestServer::start("200 OK", Duration::ZERO).await;
    let dir = tempfile::tempdir().unwrap();
    let client = GovernanceClient::new(remote_config(&server.base_url, 5, dir.path()))
        .await
        .unwrap();
    // Remote disappears after the client connected.
    drop(server);

    client
        .log_audit(AuditEvent::success("no-remote"))
        .await
        .expect("remote sync failures must not propagate to the caller");
    // Give the detached sync task time to attempt and fail its POST.
    tokio::time::sleep(Duration::from_millis(300)).await;

    let events = client.query_audit(AuditFilter::new()).await.unwrap();
    assert_eq!(events.len(), 1, "local audit entry must survive sync failure");
    assert!(client.status().await.last_sync.is_none());
}

#[tokio::test]
async fn builder_config_file_loads_settings_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("from-file.db");
    let config_path = dir.path().join("governance.json");
    std::fs::write(
        &config_path,
        format!(
            r#"{{"governance":{{"enabled":false,"base_url":"http://127.0.0.1:1","auth":{{"method":"api-key","api_key":"","bearer_token":""}},"timeout_secs":5,"retry_attempts":1}},"local":{{"enabled":true,"db_path":"{}","retention_days":3}},"sync":{{"enabled":false,"interval_ms":1000,"batch_size":1,"timeout_secs":1}},"policy":{{"enabled":true,"default_action":"allow","enforce_gates":true,"enforce_rate_limits":true}},"rate_limit":{{"enabled":false,"max_requests":10,"window_ms":1000}}}}"#,
            db_path.display()
        ),
    )
    .unwrap();

    let client = GovernanceClientBuilder::new()
        .config_file(&config_path)
        .expect("config_file should load a valid JSON config")
        .build()
        .await
        .expect("client should build from file config");

    let status = client.status().await;
    assert!(status.local_enabled);
    assert!(!status.remote_enabled);
    assert!(!status.sync_enabled);
    assert_eq!(status.config.governance_url, "http://127.0.0.1:1");
    assert_eq!(client.connection_status().await, ConnectionStatus::Disabled);
    assert!(db_path.exists(), "audit DB from the file config should be created");

    let missing = GovernanceClientBuilder::new().config_file(dir.path().join("absent.json"));
    assert!(missing.is_err(), "missing config file must fail the builder");
}
