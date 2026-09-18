//! Health check handlers and service probe utilities.
//!
//! Provides:
//! - Simple health check (`/health`)
//! - Detailed health check with service probes (`/detailed-health`)
//! - TCP connection probing for external services (NATS, Dragonfly, Neo4j, MinIO)

use std::time::Instant;

use axum::Json;

use crate::responses::{DetailedHealthResponse, SimpleHealthResponse};
use crate::state::AppState;
use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::ports::{ObservabilityPort, StoragePort};

/// `GET /health` — simple health check, no auth required.
pub async fn simple_health_handler() -> Json<SimpleHealthResponse> {
    Json(SimpleHealthResponse::healthy())
}

/// `GET /detailed-health` — aggregated health check, no auth required (T070).
pub async fn health_handler<S, V, O>(
    axum::extract::State(app): axum::extract::State<AppState<S, V, O>>,
) -> Json<DetailedHealthResponse>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    use std::collections::HashMap;

    // Probe storage with a lightweight call.
    let mut services: HashMap<String, crate::responses::ServiceHealth> = HashMap::new();

    let t0 = Instant::now();
    let sqlite_health = match app.storage.list_all_features().await {
        Ok(_) => crate::responses::ServiceHealth::healthy(t0.elapsed().as_millis() as u64),
        Err(e) => crate::responses::ServiceHealth::unavailable(e.to_string()),
    };
    services.insert("sqlite".to_owned(), sqlite_health);

    // --- Env-gated service probes (2 s timeout each) ---
    let probe_timeout = std::time::Duration::from_secs(2);

    // NATS — check NATS_URL, attempt TCP connect
    services.insert(
        "nats".to_owned(),
        probe_tcp_env("NATS_URL", probe_timeout).await,
    );

    // Dragonfly / Redis — check DRAGONFLY_URL then REDIS_URL
    services.insert(
        "dragonfly".to_owned(),
        probe_tcp_env_multi(&["DRAGONFLY_URL", "REDIS_URL"], probe_timeout).await,
    );

    // Neo4j — check NEO4J_URI, attempt TCP connect to host:port
    services.insert(
        "neo4j".to_owned(),
        probe_tcp_env("NEO4J_URI", probe_timeout).await,
    );

    // MinIO/S3 — check S3_ENDPOINT, attempt TCP connect
    services.insert(
        "minio".to_owned(),
        probe_tcp_env("S3_ENDPOINT", probe_timeout).await,
    );

    let overall = DetailedHealthResponse::compute_status(&services).to_string();

    Json(DetailedHealthResponse {
        status: overall,
        timestamp: chrono::Utc::now().to_rfc3339(),
        services,
        api: crate::responses::ApiHealth {
            status: "healthy".to_owned(),
            uptime_seconds: 0, // uptime tracking requires a startup timestamp in AppState
        },
    })
}

/// Probe a single env var: if set, TCP-connect to host:port with timeout.
/// Returns `not_configured` if the env var is absent.
async fn probe_tcp_env(
    env_key: &str,
    timeout: std::time::Duration,
) -> crate::responses::ServiceHealth {
    let url = match std::env::var(env_key) {
        Ok(v) => v,
        Err(_) => return crate::responses::ServiceHealth::not_configured(),
    };
    probe_tcp_url(&url, timeout).await
}

/// Try multiple env var names in order; return the first that is set and probed.
/// If none are set, return `not_configured`.
async fn probe_tcp_env_multi(
    env_keys: &[&str],
    timeout: std::time::Duration,
) -> crate::responses::ServiceHealth {
    for key in env_keys {
        if let Ok(url) = std::env::var(key) {
            return probe_tcp_url(&url, timeout).await;
        }
    }
    crate::responses::ServiceHealth::not_configured()
}

/// Parse `host:port` from a URL string and TCP-connect with the given timeout.
/// Accepts schemes like `http://host:port`, `nats://host:port`, or bare `host:port`.
async fn probe_tcp_url(url: &str, timeout: std::time::Duration) -> crate::responses::ServiceHealth {
    let addr = extract_host_port(url);
    let t0 = Instant::now();
    match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => crate::responses::ServiceHealth::healthy(t0.elapsed().as_millis() as u64),
        Ok(Err(e)) => crate::responses::ServiceHealth::unavailable(format!("{addr}: {e}")),
        Err(_) => {
            crate::responses::ServiceHealth::unavailable(format!("{addr}: connection timed out"))
        }
    }
}

/// Extract `host:port` from a URL or bare address string.
fn extract_host_port(url: &str) -> String {
    // Strip scheme prefix (e.g. "nats://", "http://")
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("nats://"))
        .or_else(|| url.strip_prefix("bolt://"))
        .or_else(|| url.strip_prefix("bolt+routing://"))
        .unwrap_or(url);
    // Strip trailing path/query
    let host_port = stripped.split('/').next().unwrap_or(stripped);
    // If port is missing, default to 4222 for nats-like schemes, 80 otherwise
    if host_port.contains(':') {
        host_port.to_string()
    } else if url.starts_with("nats://") {
        format!("{host_port}:4222")
    } else {
        format!("{host_port}:80")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_host_port_bare() {
        assert_eq!(extract_host_port("localhost:5432"), "localhost:5432");
    }

    #[test]
    fn extract_host_port_http() {
        assert_eq!(extract_host_port("http://10.0.0.1:8080"), "10.0.0.1:8080");
    }

    #[test]
    fn extract_host_port_https() {
        assert_eq!(
            extract_host_port("https://example.com:443"),
            "example.com:443"
        );
    }

    #[test]
    fn extract_host_port_nats() {
        assert_eq!(
            extract_host_port("nats://nats.local:4222"),
            "nats.local:4222"
        );
    }

    #[test]
    fn extract_host_port_nats_no_port() {
        assert_eq!(extract_host_port("nats://nats.local"), "nats.local:4222");
    }

    #[test]
    fn extract_host_port_bolt() {
        assert_eq!(extract_host_port("bolt://neo4j:7687"), "neo4j:7687");
    }

    #[test]
    fn extract_host_port_bolt_routing() {
        assert_eq!(extract_host_port("bolt+routing://neo4j:7687"), "neo4j:7687");
    }

    #[test]
    fn extract_host_port_bare_no_port() {
        assert_eq!(extract_host_port("myhost"), "myhost:80");
    }

    #[test]
    fn extract_host_port_strips_path() {
        assert_eq!(
            extract_host_port("http://host:8080/api/health"),
            "host:8080"
        );
    }

    #[test]
    fn extract_host_port_preserves_query() {
        // The function strips trailing paths via split('/') but does not
        // strip query strings (which lack a leading '/').
        assert_eq!(
            extract_host_port("http://host:8080?key=val"),
            "host:8080?key=val"
        );
    }

    #[test]
    fn simple_health_handler_returns_200() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let resp = simple_health_handler().await;
            assert_eq!(resp.0.status, "healthy");
        });
    }

    #[test]
    fn extract_host_port_http_without_port_defaults_80() {
        assert_eq!(extract_host_port("http://host"), "host:80");
    }

    #[test]
    fn extract_host_port_bare_ipv4_defaults_80() {
        assert_eq!(extract_host_port("10.0.0.1"), "10.0.0.1:80");
    }

    #[test]
    fn extract_host_port_nats_strips_trailing_path() {
        assert_eq!(
            extract_host_port("nats://nats.local:4222/stream"),
            "nats.local:4222"
        );
    }

    #[test]
    fn extract_host_port_https_without_port_defaults_80() {
        assert_eq!(extract_host_port("https://example.com"), "example.com:80");
    }

    #[test]
    fn simple_health_response_shape_is_stable() {
        let response = crate::responses::SimpleHealthResponse::healthy();
        assert_eq!(response.status, "healthy");
        assert_eq!(response.service, "agileplus-api");
        assert!(!response.version.is_empty());
    }

    // ── Service probes ───────────────────────────────────────────────────────
    //
    // `probe_tcp_url` is what turns a configured external service (NATS,
    // Dragonfly, Neo4j, MinIO) into a `healthy`/`unavailable` entry in the
    // `/detailed-health` payload. Each branch below is driven against a real
    // socket rather than a stub.

    /// Bind an ephemeral listener and hand back its address, keeping the
    /// listener alive so the port stays open.
    async fn live_listener() -> (tokio::net::TcpListener, String) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind an ephemeral port");
        let addr = listener.local_addr().expect("listener address").to_string();
        (listener, addr)
    }

    /// Reserve a port and immediately release it, so connecting is refused.
    async fn closed_port() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind an ephemeral port");
        let addr = listener.local_addr().expect("listener address").to_string();
        drop(listener);
        addr
    }

    #[tokio::test]
    async fn probe_tcp_url_reports_healthy_for_open_listener() {
        let (_listener, addr) = live_listener().await;
        let health = probe_tcp_url(&addr, std::time::Duration::from_secs(2)).await;
        assert_eq!(health.status, "healthy", "got: {:?}", health.error);
        assert!(
            health.latency_ms.is_some(),
            "a successful probe should report latency"
        );
        assert!(health.error.is_none());
    }

    #[tokio::test]
    async fn probe_tcp_url_reports_unavailable_for_closed_port() {
        let addr = closed_port().await;
        let health = probe_tcp_url(&addr, std::time::Duration::from_secs(2)).await;
        assert_eq!(health.status, "unavailable");
        let error = health.error.expect("refused probes carry the reason");
        assert!(
            error.starts_with(&addr),
            "the error should identify the probed address, got: {error}"
        );
        assert!(health.latency_ms.is_none());
    }

    #[tokio::test]
    async fn probe_tcp_url_reports_timeout_for_unroutable_address() {
        // TEST-NET-1 (RFC 5737) is guaranteed not to be reachable; a
        // nanosecond budget guarantees the timeout branch rather than a
        // connection refusal.
        let health = probe_tcp_url("192.0.2.1:80", std::time::Duration::from_nanos(1)).await;
        assert_eq!(health.status, "unavailable");
        assert_eq!(
            health.error.as_deref(),
            Some("192.0.2.1:80: connection timed out")
        );
    }

    #[tokio::test]
    async fn probe_tcp_url_accepts_a_url_with_scheme_and_path() {
        let (_listener, addr) = live_listener().await;
        let health = probe_tcp_url(
            &format!("http://{addr}/health"),
            std::time::Duration::from_secs(2),
        )
        .await;
        assert_eq!(health.status, "healthy", "got: {:?}", health.error);
    }

    #[tokio::test]
    async fn probe_tcp_env_reports_not_configured_for_absent_key() {
        let health = probe_tcp_env(
            "AGILEPLUS_PROBE_ABSENT_9F31",
            std::time::Duration::from_secs(1),
        )
        .await;
        assert_eq!(health.status, "not_configured");
        assert!(health.latency_ms.is_none());
    }

    #[tokio::test]
    async fn probe_tcp_env_multi_reports_not_configured_when_none_set() {
        let health = probe_tcp_env_multi(
            &["AGILEPLUS_PROBE_ABSENT_9F31", "AGILEPLUS_PROBE_ABSENT_9F32"],
            std::time::Duration::from_secs(1),
        )
        .await;
        assert_eq!(health.status, "not_configured");
    }

    #[tokio::test]
    async fn detailed_health_marks_unconfigured_services_not_configured() {
        // `compute_status` ignores unconfigured services, so a deployment with
        // no external services wired up still reports healthy overall.
        let mut services = std::collections::HashMap::new();
        services.insert(
            "sqlite".to_owned(),
            crate::responses::ServiceHealth::healthy(1),
        );
        for key in ["nats", "dragonfly", "neo4j", "minio"] {
            services.insert(
                key.to_owned(),
                crate::responses::ServiceHealth::not_configured(),
            );
        }
        assert_eq!(
            crate::responses::DetailedHealthResponse::compute_status(&services),
            "healthy"
        );
    }
}
