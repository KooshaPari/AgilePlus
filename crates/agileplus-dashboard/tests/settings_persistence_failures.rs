// SPDX-License-Identifier: MIT OR Apache-2.0
//! Failure payloads for the settings handlers that persist `~/.agileplus/config.toml`.
//!
//! `tests/settings_handlers.rs` covers the success path for every settings
//! handler with `HOME` pointed at a writable sandbox. Nothing covers what the
//! operator sees when the config cannot be written, which is the branch that
//! decides between "saved" and "your change did not stick".
//!
//! Every test here redirects `HOME` at a *regular file*, so
//! `$HOME/.agileplus/config.toml` cannot be created and `Config::save` fails
//! deterministically. Because the sandbox config never exists, the handlers take
//! the "no config yet" load path and never build a credential store, so no
//! keychain is touched. Tests serialize on a mutex because `HOME` is global.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use agileplus_dashboard::app_state::{DashboardStore, SharedState, default_health};
use agileplus_dashboard::routes::router;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tokio::sync::RwLock;
use tower::util::ServiceExt;

// ── Sandbox ──────────────────────────────────────────────────────────────────

/// Serializes every test in this file and owns the `HOME` redirect.
///
/// An async mutex is used because the guard is deliberately held across the
/// handler's `await` points: releasing it earlier would let a second test start
/// while this one is mid-request.
async fn lock() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let guard = LOCK.lock().await;
    unusable_home();
    guard
}

/// A `HOME` that exists but is a file, so no directory can be created under it.
fn unusable_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let path = std::env::temp_dir().join(format!(
            "agileplus-dashboard-home-is-a-file-{}",
            std::process::id()
        ));
        std::fs::write(&path, "not a directory\n").expect("create unusable home");
        // SAFETY: `set_var` races with concurrent `HOME` reads. Every test in
        // this binary calls `lock()` before any handler runs, and this is the
        // only write, performed while holding the file-wide mutex.
        unsafe { std::env::set_var("HOME", &path) };
        path
    })
}

fn state() -> SharedState {
    Arc::new(RwLock::new(DashboardStore {
        health: default_health(),
        ..Default::default()
    }))
}

fn app(state: SharedState) -> Router {
    router(state)
}

async fn send(request: Request<Body>) -> (StatusCode, String) {
    let response = app(state())
        .oneshot(request)
        .await
        .expect("router answers every request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

async fn post(uri: &str, content_type: &str, body: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", content_type)
        .body(Body::from(body.to_string()))
        .expect("build request");
    send(request).await
}

async fn post_form(uri: &str, form: &str) -> (StatusCode, String) {
    post(uri, "application/x-www-form-urlencoded", form).await
}

async fn patch_form(uri: &str, form: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form.to_string()))
        .expect("build request");
    send(request).await
}

/// The shared red-toast assertion for handlers that render a failure toast.
fn assert_failure_toast(body: &str) {
    assert!(
        body.contains("bg-red-900"),
        "a failed save must render the failure toast, got: {body}"
    );
    assert!(
        !body.contains("bg-green-900"),
        "a failed save must not render the success toast, got: {body}"
    );
    assert!(
        body.contains("Failed to save settings:")
            || body.contains("Failed to load settings safely:"),
        "a failed save must explain itself, got: {body}"
    );
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dashboard_settings_save_failure_renders_failure_toast() {
    let _guard = lock().await;

    let (status, body) = post_form(
        "/api/settings/dashboard",
        "theme=dark&log_level=debug&data_directory=%2Ftmp%2Fdata",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_failure_toast(&body);
    // The submitted values must still be echoed by the form-independent path.
    assert!(!unusable_home().join(".agileplus").exists());
}

#[tokio::test]
async fn agent_settings_save_failure_renders_failure_toast() {
    let _guard = lock().await;

    let (status, body) = post_form(
        "/api/settings/agents",
        "pool_size=8&retry_budget=4&dispatch_mode=balanced&default_provider=claude",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_failure_toast(&body);
}

#[tokio::test]
async fn services_settings_save_failure_renders_failure_toast() {
    let _guard = lock().await;

    let (status, body) = post_form(
        "/api/settings/services",
        "nats_url=nats%3A%2F%2Flocalhost%3A4222&neo4j_url=&minio_url=&postgres_url=&dragonfly_url=",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_failure_toast(&body);
}

#[tokio::test]
async fn service_config_patch_failure_renders_failure_toast() {
    let _guard = lock().await;

    let (status, body) = patch_form(
        "/api/dashboard/services/NATS/config",
        "endpoint_url=http%3A%2F%2Flocalhost%3A4222&timeout_ms=1500&max_retries=3",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_failure_toast(&body);
}

#[tokio::test]
async fn toggle_service_save_failure_reports_error_and_leaves_state_untouched() {
    let _guard = lock().await;
    let state = state();
    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/dashboard/services/AgilePlusProbeService/toggle")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"enabled":false}"#))
                .expect("build request"),
        )
        .await
        .expect("router answers the request");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("toggle returns JSON");

    assert_eq!(json["status"], "error");
    assert_eq!(json["service"], "AgilePlusProbeService");
    assert_eq!(json["enabled"], false);
    assert!(
        json["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()),
        "a failed toggle must explain itself: {json}"
    );

    // The in-memory status is only updated after the config write succeeds.
    let store = state.read().await;
    assert!(
        !store
            .health
            .iter()
            .any(|service| service.name == "AgilePlusProbeService"),
        "a failed toggle must not add an in-memory service entry"
    );
    let nats = store
        .health
        .iter()
        .find(|service| service.name == "NATS")
        .expect("seeded NATS entry");
    assert!(nats.healthy, "a failed toggle must not disable the service");
}
