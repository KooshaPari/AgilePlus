// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mutation-path tests for the dashboard settings and service-control handlers.
//!
//! These handlers persist to `~/.agileplus/config.toml`, so every test runs
//! with `HOME` pointed at a private sandbox: nothing here can touch the
//! operator's real configuration. All tests in this file share that one
//! sandbox file, so they serialize on a mutex and reset it before running.
//!
//! The handlers only build a credential store when the loaded config still
//! carries a legacy inline Plane key, so a sandbox config without one keeps
//! these tests credential-free (no OS keychain access).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use agileplus_dashboard::app_state::{DashboardStore, SharedState, default_health};
use agileplus_dashboard::routes::router;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tokio::sync::RwLock;
use tower::util::ServiceExt;

// ── Sandbox ──────────────────────────────────────────────────────────────────

/// Serializes every test in this file and owns the `HOME` redirect.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    sandbox_home();
    guard
}

/// The private `HOME` every test in this binary uses.
fn sandbox_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "agileplus-dashboard-settings-handlers-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join(".agileplus")).expect("create sandbox home");
        // SAFETY: `set_var` is unsafe because a concurrent read of `HOME` is
        // undefined behaviour. Every test in this binary calls `lock()` before
        // any handler runs, and `lock()` performs this one-time write while
        // holding the file-wide mutex, so no test can be reading `HOME` yet.
        unsafe { std::env::set_var("HOME", &dir) };
        dir
    })
}

/// Absolute path of the sandbox config the handlers read and write.
fn config_file() -> PathBuf {
    sandbox_home().join(".agileplus").join("config.toml")
}

/// Start each test from an empty config so tests stay independent.
fn reset_config() {
    let _ = std::fs::remove_file(config_file());
}

fn read_config() -> String {
    std::fs::read_to_string(config_file()).unwrap_or_default()
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn state() -> SharedState {
    Arc::new(RwLock::new(DashboardStore {
        health: default_health(),
        ..Default::default()
    }))
}

fn app() -> Router {
    router(state())
}

async fn send(request: Request<Body>) -> (StatusCode, String) {
    let response = app()
        .oneshot(request)
        .await
        .expect("router should answer every request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

async fn post_form(uri: &str, form: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form.to_string()))
        .expect("build form request");
    send(request).await
}

async fn patch_form(uri: &str, form: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form.to_string()))
        .expect("build patch request");
    send(request).await
}

async fn post_json(uri: &str, json: &str) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json.to_string()))
        .expect("build json request");
    let (status, body) = send(request).await;
    let parsed = serde_json::from_str(&body)
        .unwrap_or_else(|error| panic!("expected JSON from {uri}, got {body:?}: {error}"));
    (status, parsed)
}

// ── Service settings ─────────────────────────────────────────────────────────

#[tokio::test]
async fn save_services_settings_persists_configured_services() {
    let _guard = lock();
    reset_config();

    let (status, body) = post_form(
        "/api/settings/services",
        "nats_url=nats://localhost:4222&neo4j_url=bolt://localhost:7687&minio_url=http%3A%2F%2Flocalhost%3A9000",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "handler should render a toast");
    assert!(
        body.contains("Saved 3 service endpoint(s)"),
        "expected an accurate count in the toast, got: {body}"
    );

    let config = read_config();
    assert!(config.contains("NATS"), "config should list NATS: {config}");
    assert!(
        config.contains("nats://localhost:4222"),
        "config should keep the NATS endpoint: {config}"
    );
    assert!(
        config.contains("bolt://localhost:7687"),
        "config should keep the Neo4j endpoint: {config}"
    );
    assert!(
        config.contains("http://localhost:9000"),
        "config should keep the MinIO endpoint: {config}"
    );
    assert!(
        !config.contains("Dragonfly"),
        "an omitted field must not create an entry: {config}"
    );
}

#[tokio::test]
async fn save_services_settings_skips_blank_fields() {
    let _guard = lock();
    reset_config();

    let (status, body) = post_form(
        "/api/settings/services",
        "nats_url=   &neo4j_url=bolt://keep.example",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("Saved 1 service endpoint(s)"),
        "only the filled field counts as a save, got: {body}"
    );

    let config = read_config();
    assert!(
        config.contains("bolt://keep.example"),
        "the filled service should persist: {config}"
    );
    assert!(
        !config.contains("NATS"),
        "a whitespace-only field must not create an entry: {config}"
    );
}

// ── Agent and dashboard settings ─────────────────────────────────────────────

#[tokio::test]
async fn save_agent_settings_persists_pool_configuration() {
    let _guard = lock();
    reset_config();

    let (status, body) = post_form(
        "/api/settings/agents",
        "pool_size=7&retry_budget=4&dispatch_mode=parallel&default_provider=deepseek",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "handler should render a toast");

    let config = read_config();
    assert!(config.contains("pool_size = 7"), "pool size: {config}");
    assert!(config.contains("retry_budget = 4"), "retry budget: {config}");
    assert!(config.contains("parallel"), "dispatch mode: {config}");
    assert!(config.contains("deepseek"), "provider: {config}");
    assert!(
        body.contains("saved") || body.contains("Saved"),
        "expected a toast message, got: {body}"
    );
}

#[tokio::test]
async fn save_dashboard_settings_persists_ui_configuration() {
    let _guard = lock();
    reset_config();

    let (status, _body) = post_form(
        "/api/settings/dashboard",
        "theme=dark&log_level=debug&data_directory=%2Ftmp%2Fagileplus-data",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let config = read_config();
    assert!(config.contains("dark"), "theme should persist: {config}");
    assert!(config.contains("debug"), "log level: {config}");
    assert!(
        config.contains("agileplus-data"),
        "data directory: {config}"
    );
}

// ── Plane settings (secret-handling branch) ──────────────────────────────────

#[tokio::test]
async fn save_plane_settings_rejects_a_missing_api_key() {
    let _guard = lock();
    reset_config();

    let (status, body) = post_form(
        "/api/settings/plane",
        "api_url=https://app.plane.so&api_key=&workspace_slug=ws&project_slug=proj",
    )
    .await;

    assert_eq!(status, StatusCode::OK, "rejection is a rendered toast");
    assert!(
        body.contains("Plane API key is required"),
        "expected the required-key message, got: {body}"
    );
    assert!(
        !config_file().exists(),
        "a rejected save must not create a config file"
    );
}

// ── Service config PATCH ─────────────────────────────────────────────────────

#[tokio::test]
async fn patch_service_config_creates_entry_with_limits() {
    let _guard = lock();
    reset_config();

    let (status, body) = patch_form(
        "/api/dashboard/services/Plane/config",
        "endpoint_url=https://new.example&timeout_ms=2500&max_retries=5",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("configuration saved"),
        "expected the save toast, got: {body}"
    );

    let config = read_config();
    assert!(config.contains("Plane"), "service entry: {config}");
    assert!(
        config.contains("https://new.example"),
        "endpoint should persist: {config}"
    );
    assert!(config.contains("2500"), "timeout should persist: {config}");
    assert!(config.contains('5'), "retry budget should persist: {config}");
}

#[tokio::test]
async fn patch_service_config_keeps_endpoint_when_blank() {
    let _guard = lock();
    reset_config();

    let (status, _) = patch_form(
        "/api/dashboard/services/Plane/config",
        "endpoint_url=https://keep.example&timeout_ms=1000",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // A blank endpoint must not wipe the stored URL.
    let (status, _) = patch_form(
        "/api/dashboard/services/Plane/config",
        "endpoint_url=&timeout_ms=9000",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let config = read_config();
    assert!(
        config.contains("https://keep.example"),
        "blank endpoint must not clear the stored URL: {config}"
    );
}

// ── Service toggle ───────────────────────────────────────────────────────────

#[tokio::test]
async fn toggle_service_persists_disabled_state() {
    let _guard = lock();
    reset_config();

    let (status, _) = patch_form(
        "/api/dashboard/services/Plane/config",
        "endpoint_url=https://plane.example",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = post_json(
        "/api/dashboard/services/Plane/toggle",
        r#"{"enabled": false}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "toggle body: {body}");

    let config = read_config();
    assert!(
        config.contains("enabled = false"),
        "disabled state should persist: {config}"
    );
}

#[tokio::test]
async fn toggle_service_defaults_to_enabled_for_unknown_service() {
    let _guard = lock();
    reset_config();

    let (status, body) = post_json("/api/dashboard/services/Ghost/toggle", r#"{}"#).await;
    assert_eq!(status, StatusCode::OK, "toggle body: {body}");

    let config = read_config();
    assert!(config.contains("Ghost"), "entry should be created: {config}");
    assert!(
        config.contains("enabled = true"),
        "an omitted flag defaults to enabled: {config}"
    );
}

// ── Service restart ──────────────────────────────────────────────────────────

/// Restart tests mutate a process-wide env var, so they hold the file lock.
struct RestartEnv(&'static str);

impl RestartEnv {
    fn set(value: &'static str) -> Self {
        // SAFETY: `set_var` is unsafe because a concurrent read of this var is
        // undefined behaviour. Only the restart tests touch this variable, and
        // every one of them holds `lock()` for its whole body.
        unsafe { std::env::set_var("AGILEPLUS_SERVICE_RESTART_CMD", value) };
        Self(value)
    }
}

impl Drop for RestartEnv {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("AGILEPLUS_SERVICE_RESTART_CMD") };
    }
}

#[tokio::test]
async fn restart_service_rejects_template_without_placeholder() {
    let _guard = lock();
    let _env = RestartEnv::set("echo restarted");

    let (status, body) = post_json("/api/dashboard/services/Plane/restart", "{}").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "error", "body: {body}");
    assert_eq!(body["service"], "Plane");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("{}"),
        "the error should explain the missing placeholder: {body}"
    );
}

#[tokio::test]
async fn restart_service_rejects_disallowed_program() {
    let _guard = lock();
    let _env = RestartEnv::set("rm -rf {}");

    let (status, body) = post_json("/api/dashboard/services/Plane/restart", "{}").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["status"], "error",
        "an unlisted program must be refused: {body}"
    );
}

#[tokio::test]
async fn restart_service_runs_an_allowed_command() {
    let _guard = lock();
    let _env = RestartEnv::set("echo restarted {}");

    let (status, body) = post_json("/api/dashboard/services/Plane/restart", "{}").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok", "body: {body}");
    assert_eq!(body["service"], "Plane");
    assert!(
        body["stdout"]
            .as_str()
            .unwrap_or_default()
            .contains("restarted Plane"),
        "the command output should be forwarded: {body}"
    );
}
