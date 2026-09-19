// SPDX-License-Identifier: MIT OR Apache-2.0
//! Request-time failure branches for three dashboard routes.
//!
//! Each of these handlers has a rejection path that no existing test reaches:
//!
//! - `POST /api/features/{id}/transition` with an *existing* feature and a
//!   lifecycle step the domain rejects (the state machine is only ever driven
//!   with a valid step today).
//! - `POST /api/dashboard/services/{name}/restart` when the allowlisted program
//!   cannot be spawned at all (the existing tests cover the missing-placeholder
//!   and disallowed-program rejections, both of which happen before `output()`).
//! - `GET /api/dashboard/epics-stories.json` when the resolved database cannot
//!   be opened (only a *missing schema* is covered today, which is a different
//!   branch inside the same function).
//!
//! All three read process-global state (the working directory, `PATH`,
//! `AGILEPLUS_SERVICE_RESTART_CMD`, `DATABASE_*`), and two of them can persist a
//! config, so every test here runs under one mutex with `HOME` redirected to a
//! private sandbox. Nothing can touch the operator's real configuration.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use agileplus_dashboard::app_state::{DashboardStore, SharedState};
use agileplus_dashboard::routes::router;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tokio::sync::RwLock;
use tower::util::ServiceExt;

// ── Sandbox ──────────────────────────────────────────────────────────────────

/// Serializes the file, installs the sandbox `HOME`, and fences the global
/// variables these tests own.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    sandbox_home();
    guard
}

fn sandbox_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "agileplus-dashboard-route-failure-paths-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join(".agileplus")).expect("create sandbox home");
        // SAFETY: `set_var` is unsafe because a concurrent read of `HOME` is
        // undefined behaviour. The file-wide mutex is held here and by every
        // test in this binary, so no test can be reading `HOME` yet.
        unsafe { std::env::set_var("HOME", &dir) };
        dir
    })
}

fn set_var(key: &str, value: &str) {
    // SAFETY: every test in this binary holds the file-wide mutex before
    // touching the environment, and no other thread reads these variables.
    unsafe { std::env::set_var(key, value) };
}

fn unset_var(key: &str) {
    // SAFETY: see `set_var`.
    unsafe { std::env::remove_var(key) };
}

/// Restores one environment variable when the test ends, including on panic.
struct EnvRestore {
    key: &'static str,
    previous: Option<String>,
}

impl EnvRestore {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            previous: std::env::var(key).ok(),
        }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        match &self.previous {
            // SAFETY: see `set_var`.
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn state(store: DashboardStore) -> SharedState {
    Arc::new(RwLock::new(store))
}

fn app(state: SharedState) -> Router {
    router(state)
}

async fn send(request: Request<Body>) -> (StatusCode, String) {
    let response = app(state(DashboardStore::default()))
        .oneshot(request)
        .await
        .expect("router should answer every request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

async fn get(app: &Router, uri: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("router should answer every request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

// ── Feature transition ───────────────────────────────────────────────────────

fn created_feature(id: i64) -> Feature {
    let mut feature = Feature::new(
        &format!("feat-{id}"),
        &format!("Feature {id}"),
        [0; 32],
        None,
    );
    feature.id = id;
    feature.state = FeatureState::Created;
    feature
}

#[tokio::test]
async fn feature_transition_rejects_a_disallowed_lifecycle_step_without_mutating_state() {
    let _guard = lock();

    let store = DashboardStore {
        features: vec![created_feature(1)],
        ..Default::default()
    };
    let state = state(store);
    let app = app(state.clone());

    // Created -> Shipped skips every intermediate state, so the domain rejects it.
    let request = Request::builder()
        .method("POST")
        .uri("/api/features/1/transition")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("target_state=shipped"))
        .expect("build form request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("router should answer every request");

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a step the lifecycle forbids must be rejected"
    );
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let body = String::from_utf8_lossy(&bytes).into_owned();
    assert!(
        body.contains("invalid transition"),
        "the rejection must name the refused transition: {body}"
    );
    assert!(body.contains("Created"), "rejection must name the current state: {body}");
    assert!(body.contains("Shipped"), "rejection must name the target state: {body}");

    // Scoped so the read guard is released before the next request: the
    // handler takes `state.write()`, and tokio's `RwLock` is fair, so a live
    // reader would deadlock the write instead of failing.
    {
        let store = state.read().await;
        assert_eq!(
            store.features[0].state,
            FeatureState::Created,
            "a rejected transition must leave the feature untouched"
        );
    }

    // The same feature still accepts a legal step afterwards. This must go
    // through the *seeded* app - the plain `send` helper builds an empty store,
    // which would (correctly) 404 on a missing feature.
    let request = Request::builder()
        .method("POST")
        .uri("/api/features/1/transition")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("target_state=specified"))
        .expect("build form request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("router should answer every request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let body = String::from_utf8_lossy(&bytes).into_owned();
    assert_eq!(status, StatusCode::OK, "a legal step must still be accepted: {body}");
    assert!(body.contains("kanban-board"), "expected the kanban partial: {body}");
}

// ── restart_service spawn failure ────────────────────────────────────────────

#[tokio::test]
async fn restart_service_reports_a_failed_spawn_for_an_unresolvable_program() {
    let _guard = lock();
    let _command = EnvRestore::capture("AGILEPLUS_SERVICE_RESTART_CMD");
    let _path = EnvRestore::capture("PATH");

    // `echo` is on the approved restart registry, so validation passes and the
    // handler reaches the spawn. An empty PATH makes the spawn itself fail.
    set_var("AGILEPLUS_SERVICE_RESTART_CMD", "echo restart {}");
    set_var("PATH", "");

    let (status, body) = send(Request::builder()
        .method("POST")
        .uri("/api/dashboard/services/AgilePlusSpawnProbe/restart")
        .body(Body::empty())
        .expect("build request"))
    .await;

    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value =
        serde_json::from_str(&body).unwrap_or_else(|error| panic!("expected JSON: {body}: {error}"));

    assert_eq!(json["status"], "error", "a failed spawn must not report success");
    assert_eq!(json["service"], "AgilePlusSpawnProbe");
    assert_eq!(json["command"], "echo restart AgilePlusSpawnProbe");
    let error = json["error"].as_str().unwrap_or_default();
    assert!(
        !error.is_empty(),
        "a failed spawn must explain itself: {json}"
    );
    assert!(
        json.get("stdout").is_none() && json.get("stderr").is_none(),
        "a process that never started has no output to report: {json}"
    );
}

// ── epics-stories database open failure ──────────────────────────────────────

#[tokio::test]
async fn epics_stories_reports_a_database_that_cannot_be_opened() {
    let _guard = lock();
    let _url = EnvRestore::capture("DATABASE_URL");
    let _path = EnvRestore::capture("DATABASE_PATH");

    // Point the resolver at a directory: SQLite cannot open one as a database.
    let directory = sandbox_home().join("not-a-database");
    std::fs::create_dir_all(&directory).expect("create directory-instead-of-db");
    unset_var("DATABASE_URL");
    set_var("DATABASE_PATH", directory.to_str().expect("utf-8 path"));

    let (status, body) = get(
        &app(state(DashboardStore::default())),
        "/api/dashboard/epics-stories.json",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value =
        serde_json::from_str(&body).unwrap_or_else(|error| panic!("expected JSON: {body}: {error}"));

    assert_eq!(json["epic_count"], 0);
    assert_eq!(json["story_count"], 0);
    assert_eq!(json["epics"], serde_json::json!([]));
    assert_eq!(json["stories"], serde_json::json!([]));
    assert!(
        json["error"]
            .as_str()
            .is_some_and(|error| error.contains("db open failed")),
        "an unopenable database must be reported as such, not as a query failure: {json}"
    );
    assert!(
        !directory.join("agileplus.db").exists(),
        "the handler must not create a database inside the directory it was pointed at"
    );
}
