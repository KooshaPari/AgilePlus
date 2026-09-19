// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end lifecycle of the real `agileplus-api` binary.
//!
//! `src/main.rs` is a binary target: no library test can call
//! `load_runtime_config`, `bind_address`, or `ensure_database_parent`, and the
//! existing binary suite only ever drives the *refusal* paths (a bad
//! `DATABASE_URL`, a missing operator key). A successful startup — the branch
//! every deployment actually takes — was therefore unexecuted.
//!
//! These tests spawn the real artifact against a SQLite database inside a
//! sandboxed `HOME`, wait for the listener, and then talk to it over a real
//! TCP socket:
//!
//! - `DATABASE_URL=sqlite:<nested path>` must be translated into a config path
//!   whose parent directory is created before storage is opened.
//! - `API_HOST` / `AGILEPLUS_API_HOST` select the bind address and `API_PORT`
//!   the port (the fact that the socket answers proves both).
//! - `/health`, `/info`, `/detailed-health` are public; the protected routes
//!   answer 401 with the documented envelope and 200 with the operator key.
//! - A created feature survives in SQLite and is read back through the API, so
//!   the real storage adapter, not a mock, is what served the request.
//!
//! The sandbox keeps the OS keychain out of the loop (file credential backend)
//! and all writes inside a per-test temporary directory.
//!
//! Traceability: WP11-T064, WP11-T069, WP15-T086

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_agileplus-api");
const API_KEY: &str = "operator-api-key-for-lifecycle-tests";
const CREDENTIAL_KEY: &str = "operator-passphrase-for-lifecycle-tests";

/// Environment variables the binary reads that could leak in from the
/// developer's shell. Cleared for every child process.
const RUNTIME_VARS: [&str; 11] = [
    "DATABASE_URL",
    "AGILEPLUS_API_KEY",
    "AGILEPLUS_CREDENTIAL_KEY",
    "API_HOST",
    "AGILEPLUS_API_HOST",
    "API_PORT",
    "AGILEPLUS_HTTP_PORT",
    "AGILEPLUS_API_PORT",
    "AGILEPLUS_GRPC_PORT",
    "AGILEPLUS_TELEMETRY_LOG_LEVEL",
    "AGILEPLUS_CORE_DB_PATH",
];

/// `http.port` in the config belongs to the fixed binding path; the runtime
/// probes below only consider these variables unset.
const SERVICE_PROBE_VARS: [&str; 5] = [
    "NATS_URL",
    "DRAGONFLY_URL",
    "REDIS_URL",
    "NEO4J_URI",
    "S3_ENDPOINT",
];

// ── Sandbox ──────────────────────────────────────────────────────────────────

struct Sandbox {
    home: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let home = std::env::temp_dir().join(format!(
            "agileplus-api-lifecycle-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(home.join(".agileplus")).expect("create the sandbox home");
        Self { home }
    }

    /// A child with a clean environment and `HOME` inside the sandbox, so the
    /// operator's real `~/.agileplus` can neither satisfy nor break the run.
    fn command(&self) -> Command {
        let mut command = Command::new(BIN);
        for key in RUNTIME_VARS {
            command.env_remove(key);
        }
        for key in SERVICE_PROBE_VARS {
            command.env_remove(key);
        }
        command.env("HOME", &self.home);
        command
    }

    /// Nested database directory, so `ensure_database_parent` has real work.
    fn database_path(&self) -> PathBuf {
        self.home.join("data").join("db").join("agileplus.db")
    }

    fn database_parent(&self) -> PathBuf {
        self.home.join("data").join("db")
    }

    fn credentials_file(&self) -> PathBuf {
        self.home.join(".agileplus").join("credentials.enc")
    }

    /// Select the file credential backend so the child never reaches for the
    /// OS keychain (which would prompt on the developer's machine).
    fn write_file_backend_config(&self) {
        let config = format!(
            "[credentials]\nbackend = \"file\"\nfile_path = \"{}\"\n",
            self.credentials_file().display()
        );
        std::fs::write(self.home.join(".agileplus").join("config.toml"), config)
            .expect("write the sandbox config");
    }

    fn spawn(&self, host_var: &str, port: u16) -> Child {
        self.write_file_backend_config();
        self.command()
            .env(
                "DATABASE_URL",
                format!("sqlite:{}", self.database_path().display()),
            )
            .env("AGILEPLUS_CREDENTIAL_KEY", CREDENTIAL_KEY)
            .env("AGILEPLUS_API_KEY", API_KEY)
            .env(host_var, "127.0.0.1")
            .env("API_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the built binary starts")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

/// A `TcpListener` bound to port 0 reports a free port; dropping it frees the
/// port for the child. The poll loop below tolerates the small race.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    listener.local_addr().expect("local addr").port()
}

// ── Minimal HTTP/1.1 client ──────────────────────────────────────────────────

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpResponse {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|e| panic!("body is not JSON ({e}): {}", self.body))
    }
}

/// Send one request with `Connection: close` and read the reply.
///
/// The reader is deliberately tolerant: with a `Content-Length` it stops as
/// soon as the announced body is complete, and otherwise it reads to EOF,
/// treating a read timeout as the end of the response. That keeps the helper
/// correct whether or not the server honours the close request.
fn request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> std::io::Result<HttpResponse> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;

    let mut raw = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    if let Some(body) = body {
        raw.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    for (name, value) in headers {
        raw.push_str(&format!("{name}: {value}\r\n"));
    }
    raw.push_str("\r\n");
    if let Some(body) = body {
        raw.push_str(body);
    }

    stream.write_all(raw.as_bytes())?;
    stream.flush()?;

    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut expected_body: Option<usize> = None;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                bytes.extend_from_slice(&chunk[..n]);
                let text = String::from_utf8_lossy(&bytes).into_owned();
                if let Some((head, body)) = text.split_once("\r\n\r\n") {
                    if expected_body.is_none() {
                        expected_body = head.lines().find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            if name.eq_ignore_ascii_case("content-length") {
                                value.trim().parse::<usize>().ok()
                            } else {
                                None
                            }
                        });
                    }
                    if let Some(expected) = expected_body
                        && body.len() >= expected
                    {
                        break;
                    }
                }
            }
            // A timeout is the end of the response when the server keeps the
            // connection open without announcing a length.
            Err(_) => break,
        }
    }

    let text = String::from_utf8_lossy(&bytes).into_owned();
    let (head, body) = text
        .split_once("\r\n\r\n")
        .unwrap_or_else(|| panic!("malformed HTTP response: {text}"));

    let mut lines = head.split("\r\n");
    let status_line = lines.next().expect("status line");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or_else(|| panic!("unparseable status line: {status_line}"));

    let headers = lines
        .filter_map(|line| {
            line.split_once(':')
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        })
        .collect();

    Ok(HttpResponse {
        status,
        headers,
        body: body.to_string(),
    })
}

/// Wait until the server answers `/health`, or fail with the child's stderr.
fn await_ready(port: u16, child: &mut Child) -> HttpResponse {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Ok(resp) = request(port, "GET", "/health", &[], None)
            && resp.status == 200
        {
            return resp;
        }
        if let Ok(Some(status)) = child.try_wait() {
            let mut stderr = String::new();
            if let Some(mut pipe) = child.stderr.take() {
                let _ = pipe.read_to_string(&mut stderr);
            }
            panic!("the server exited with {status} before answering /health: {stderr}");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("the server did not answer /health on port {port} within 20s");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Full happy path: bind, serve, authenticate, persist, and read back.
#[tokio::test]
async fn a_configured_binary_serves_authenticated_traffic_over_the_database_url_api_host_and_port() {
    let sandbox = Sandbox::new("happy-path");
    let port = free_port();
    let mut child = sandbox.spawn("API_HOST", port);

    let health = await_ready(port, &mut child);

    // The nested database directory must exist before storage was opened.
    assert!(
        sandbox.database_parent().is_dir(),
        "ensure_database_parent must create {}",
        sandbox.database_parent().display()
    );
    assert!(
        sandbox.database_path().is_file(),
        "the SQLite adapter must have created {}",
        sandbox.database_path().display()
    );

    // ── /health ──────────────────────────────────────────────────────────────
    assert_eq!(health.status, 200);
    assert!(
        health.header("content-type").unwrap_or_default().contains("application/json"),
        "health must be JSON, got: {:?}",
        health.header("content-type")
    );
    let health = health.json();
    assert_eq!(health["status"], "healthy");
    assert_eq!(health["service"], "agileplus-api");
    assert_eq!(health["version"], env!("CARGO_PKG_VERSION"));

    // ── /info ────────────────────────────────────────────────────────────────
    let info = request(port, "GET", "/info", &[], None).expect("GET /info");
    assert_eq!(info.status, 200);
    let info = info.json();
    assert_eq!(info["name"], "agileplus-api");
    assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));

    // ── /detailed-health ─────────────────────────────────────────────────────
    let detailed = request(port, "GET", "/detailed-health", &[], None).expect("GET /detailed-health");
    assert_eq!(detailed.status, 200);
    let detailed = detailed.json();
    assert_eq!(
        detailed["status"], "healthy",
        "with no probes configured the deployment is healthy: {detailed}"
    );
    assert_eq!(detailed["api"]["status"], "healthy");
    assert_eq!(
        detailed["services"]["sqlite"]["status"], "healthy",
        "the storage probe must have reached the real database: {detailed}"
    );
    assert!(
        detailed["services"]["sqlite"]["latency_ms"].is_u64(),
        "a healthy probe reports a latency: {detailed}"
    );
    for service in ["nats", "dragonfly", "neo4j", "minio"] {
        assert_eq!(
            detailed["services"][service]["status"], "not_configured",
            "{service} is unset in this sandbox: {detailed}"
        );
    }

    // ── Auth contract ────────────────────────────────────────────────────────
    let anonymous = request(port, "GET", "/api/v1/features", &[], None).expect("GET without a key");
    assert_eq!(anonymous.status, 401);
    assert_eq!(
        anonymous.json()["error"],
        "Missing API key (Authorization Bearer, X-API-Key header, or ?api_key= param required)"
    );

    let wrong = request(
        port,
        "GET",
        "/api/v1/features",
        &[("X-API-Key", "not-the-operator-key")],
        None,
    )
    .expect("GET with a wrong key");
    assert_eq!(wrong.status, 401);
    assert_eq!(wrong.json()["error"], "Invalid API key");

    // ── Authorized read against the real (empty) database ────────────────────
    let empty = request(
        port,
        "GET",
        "/api/v1/features",
        &[("X-API-Key", API_KEY)],
        None,
    )
    .expect("GET with the operator key");
    assert_eq!(empty.status, 200);
    assert_eq!(empty.json(), serde_json::json!([]));

    // ── Write, then read it back through the API ─────────────────────────────
    let created = request(
        port,
        "POST",
        "/api/v1/features",
        &[("X-API-Key", API_KEY)],
        Some(r#"{"title":"Lifecycle Feature"}"#),
    )
    .expect("POST /api/v1/features");
    assert_eq!(
        created.status, 201,
        "creating a feature must answer 201: {}",
        created.body
    );
    let created = created.json();
    assert_eq!(created["slug"], "lifecycle-feature");
    assert_eq!(created["name"], "Lifecycle Feature");
    assert_eq!(created["state"], "created");
    assert_eq!(created["target_branch"], "main");
    let id = created["id"].as_i64().expect("the created feature has an id");
    assert!(id > 0, "the database assigned an id, got {id}");

    let by_slug = request(
        port,
        "GET",
        "/api/v1/features/lifecycle-feature",
        &[("X-API-Key", API_KEY)],
        None,
    )
    .expect("GET the feature back");
    assert_eq!(by_slug.status, 200);
    assert_eq!(by_slug.json()["id"], id);

    let listed = request(
        port,
        "GET",
        "/api/v1/features",
        &[("X-API-Key", API_KEY)],
        None,
    )
    .expect("GET the feature list");
    assert_eq!(listed.status, 200);
    let listed = listed.json();
    assert_eq!(listed.as_array().map(|a| a.len()), Some(1));
    assert_eq!(listed[0]["slug"], "lifecycle-feature");

    // A read-only route that must not be mistaken for a protected one.
    let missing = request(
        port,
        "GET",
        "/api/v1/features/not-a-real-slug",
        &[("X-API-Key", API_KEY)],
        None,
    )
    .expect("GET an unknown feature");
    assert_eq!(missing.status, 404);
    assert!(
        missing.json()["error"]
            .as_str()
            .unwrap_or_default()
            .contains("not-a-real-slug")
    );

    stop(&mut child);
}

/// `AGILEPLUS_API_HOST` is the fallback when `API_HOST` is absent; the `or_else`
/// arm of `bind_address` is otherwise unreachable.
#[tokio::test]
async fn agileplus_api_host_is_honoured_when_api_host_is_unset() {
    let sandbox = Sandbox::new("host-fallback");
    let port = free_port();
    let mut child = sandbox.spawn("AGILEPLUS_API_HOST", port);

    let health = await_ready(port, &mut child);
    assert_eq!(health.status, 200);
    assert_eq!(health.json()["status"], "healthy");

    stop(&mut child);
}

/// Stop the child and confirm it terminated on a signal rather than exiting
/// successfully — a server that had already crashed would have been caught by
/// `await_ready`, and this pins that the process was still serving when asked
/// to stop.
fn stop(child: &mut Child) {
    child.kill().expect("the server accepts a stop signal");
    let status = child.wait().expect("the child is reaped");
    assert!(
        !status.success(),
        "a stopped server must not report a successful exit, got: {status}"
    );
}
