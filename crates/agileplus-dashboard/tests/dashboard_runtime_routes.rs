// SPDX-License-Identifier: MIT OR Apache-2.0
//! Routes whose response is resolved at request time from the environment, the
//! working directory, or a live stream rather than from dashboard state.
//!
//! Covered here:
//! - `GET /api/dashboard/epics-stories.json` — the `DATABASE_URL` / `DATABASE_PATH`
//!   / default-file resolution that the raw `epics_stories_json_for_path` unit
//!   tests never reach.
//! - `GET /api/stream` — the Server-Sent Events body itself. The existing route
//!   tests only assert that the response opens; nothing polls the frames.
//! - `POST /api/dashboard/services/{name}/restart` — the fallback template used
//!   when `AGILEPLUS_SERVICE_RESTART_CMD` is unset, and the failure payload.
//!
//! Database resolution depends on the process working directory and the restart
//! fallback depends on process environment, so every test in this file runs
//! under one mutex that owns both.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use agileplus_dashboard::app_state::{DashboardStore, ServiceHealth, SharedState};
use agileplus_dashboard::routes::{restart_service, router, sse_stream};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path as RoutePath, State};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use rusqlite::Connection;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tower::util::ServiceExt;

// ── Sandbox / environment guard ──────────────────────────────────────────────

fn sandbox_root() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "agileplus-dashboard-runtime-routes-{}",
            std::process::id()
        ));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear previous sandbox");
        }
        std::fs::create_dir_all(&dir).expect("create sandbox root");
        dir
    })
}

/// Serializes the whole file and owns both the working directory and the
/// `DATABASE_*` environment variables.
struct EnvGuard {
    previous_cwd: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous_cwd);
    }
}

/// Enter a fresh, empty working directory for one test.
fn enter_sandbox(name: &str) -> (EnvGuard, PathBuf) {
    static LOCK: Mutex<()> = Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_cwd = std::env::current_dir().expect("read current dir");

    let work = sandbox_root().join(name);
    if work.exists() {
        std::fs::remove_dir_all(&work).expect("clear test dir");
    }
    std::fs::create_dir_all(&work).expect("create test dir");
    std::env::set_current_dir(&work).expect("enter test dir");

    (
        EnvGuard {
            previous_cwd,
            _lock: guard,
        },
        work,
    )
}

fn set_var(key: &str, value: &str) {
    // SAFETY: every test in this binary holds `EnvGuard` before touching the
    // environment, and `DATABASE_URL`/`DATABASE_PATH` are read by no other
    // thread in this process.
    unsafe { std::env::set_var(key, value) };
}

fn unset_var(key: &str) {
    // SAFETY: see `set_var`.
    unsafe { std::env::remove_var(key) };
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn state(health: Vec<ServiceHealth>) -> SharedState {
    Arc::new(RwLock::new(DashboardStore {
        health,
        ..Default::default()
    }))
}

fn app(state: SharedState) -> Router {
    router(state)
}

async fn get_json(app: &Router, uri: &str) -> serde_json::Value {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("router answers every request");
    assert_eq!(response.status(), StatusCode::OK, "GET {uri} must succeed");
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("response is JSON")
}

async fn body_text(response: Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    String::from_utf8(bytes.to_vec()).expect("body is utf-8")
}

/// Create an `agileplus`-style database with epics and stories tables.
fn create_epics_db(path: &Path, epic_titles: &[&str]) {
    let connection = Connection::open(path).expect("open sqlite database");
    connection
        .execute_batch(
            "CREATE TABLE epics (
                 id INTEGER PRIMARY KEY,
                 title TEXT NOT NULL,
                 status TEXT NOT NULL,
                 requirement_id TEXT
             );
             CREATE TABLE stories (
                 id INTEGER PRIMARY KEY,
                 epic_id INTEGER,
                 title TEXT NOT NULL,
                 status TEXT NOT NULL,
                 requirement_id TEXT
             );",
        )
        .expect("create schema");

    for (index, title) in epic_titles.iter().enumerate() {
        let id = index as i64 + 1;
        connection
            .execute(
                "INSERT INTO epics (id, title, status, requirement_id) VALUES (?1, ?2, 'active', ?3)",
                rusqlite::params![id, title, format!("FR-{id}")],
            )
            .expect("insert epic");
        connection
            .execute(
                "INSERT INTO stories (id, epic_id, title, status, requirement_id) \
                 VALUES (?1, ?2, ?3, 'planned', ?4)",
                rusqlite::params![id, id, format!("story for {title}"), format!("FR-{id}-S1")],
            )
            .expect("insert story");
    }
}

// ── /api/dashboard/epics-stories.json ────────────────────────────────────────

#[tokio::test]
async fn epics_stories_route_reads_database_path_env() {
    let (_guard, work) = enter_sandbox("database-path");
    let db = work.join("runtime-path.db");
    create_epics_db(&db, &["Epic from DATABASE_PATH"]);

    set_var("DATABASE_PATH", db.to_str().expect("utf-8 path"));
    unset_var("DATABASE_URL");

    let json = get_json(&app(state(Vec::new())), "/api/dashboard/epics-stories.json").await;

    assert!(json.get("error").is_none(), "unexpected error: {json}");
    assert_eq!(json["epic_count"], 1);
    assert_eq!(json["story_count"], 1);
    assert_eq!(json["epics"][0]["id"], 1);
    assert_eq!(json["epics"][0]["title"], "Epic from DATABASE_PATH");
    assert_eq!(json["epics"][0]["requirement_id"], "FR-1");
    assert_eq!(json["stories"][0]["epic_id"], 1);
    assert_eq!(json["stories"][0]["requirement_id"], "FR-1-S1");
    assert!(
        json["timestamp"]
            .as_str()
            .is_some_and(|ts| ts.contains('T')),
        "timestamp must be an RFC3339 instant: {json}"
    );
}

#[tokio::test]
async fn epics_stories_route_strips_sqlite_scheme_from_database_url() {
    let (_guard, work) = enter_sandbox("database-url");
    let db = work.join("runtime-url.db");
    create_epics_db(&db, &["First epic", "Second epic"]);

    set_var("DATABASE_URL", &format!("sqlite:{}", db.display()));
    unset_var("DATABASE_PATH");

    let json = get_json(&app(state(Vec::new())), "/api/dashboard/epics-stories.json").await;

    assert!(json.get("error").is_none(), "unexpected error: {json}");
    assert_eq!(json["epic_count"], 2);
    assert_eq!(json["stories"][1]["title"], "story for Second epic");
}

#[tokio::test]
async fn epics_stories_route_reads_plain_database_url_without_scheme() {
    let (_guard, work) = enter_sandbox("database-url-plain");
    let db = work.join("runtime-plain.db");
    create_epics_db(&db, &["Plain path epic"]);

    // `strip_prefix("sqlite:")` must leave a scheme-less absolute path alone.
    set_var("DATABASE_URL", db.to_str().expect("utf-8 path"));
    unset_var("DATABASE_PATH");

    let json = get_json(&app(state(Vec::new())), "/api/dashboard/epics-stories.json").await;

    assert_eq!(json["epic_count"], 1);
    assert_eq!(json["epics"][0]["title"], "Plain path epic");
}

#[tokio::test]
async fn epics_stories_route_reports_missing_schema_for_default_database() {
    let (_guard, work) = enter_sandbox("default-database");
    unset_var("DATABASE_URL");
    unset_var("DATABASE_PATH");

    let json = get_json(&app(state(Vec::new())), "/api/dashboard/epics-stories.json").await;

    assert_eq!(json["epic_count"], 0);
    assert_eq!(json["story_count"], 0);
    assert!(
        json["epics"].as_array().expect("epics array").is_empty(),
        "no epics can exist in a fresh database: {json}"
    );
    assert!(
        json["error"]
            .as_str()
            .is_some_and(|error| error.contains("epics query failed")),
        "missing schema must surface as an error: {json}"
    );
    assert!(
        work.join("agileplus.db").exists(),
        "the default database path must resolve against the working directory"
    );
}

// ── /api/stream ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn sse_stream_emits_live_feature_and_health_frames() {
    let (_guard, _work) = enter_sandbox("sse");

    let seeded = DashboardStore::seeded();
    let feature_count = seeded.features.len();
    assert!(feature_count > 0, "seed must provide features");

    let store = DashboardStore {
        health: vec![
            ServiceHealth {
                name: "NATS".into(),
                healthy: true,
                degraded: false,
                latency_ms: Some(2),
                last_check: Utc::now(),
            },
            ServiceHealth {
                name: "Neo4j".into(),
                healthy: false,
                degraded: true,
                latency_ms: None,
                last_check: Utc::now(),
            },
        ],
        ..seeded
    };
    let state = Arc::new(RwLock::new(store));

    let response = sse_stream(State(state)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);

    let mut frames = String::new();
    let mut body = response.into_body().into_data_stream();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline
        && !(frames.contains("feature_updated") && frames.contains("health_changed"))
    {
        let chunk = tokio::time::timeout(Duration::from_secs(5), body.next())
            .await
            .expect("stream must yield a frame before the timeout")
            .expect("stream must stay open")
            .expect("frame is not an error");
        frames.push_str(&String::from_utf8_lossy(&chunk));
    }

    assert!(frames.contains("feature_updated"), "got: {frames}");
    assert!(
        frames.contains(&format!("\"features\":{feature_count}")),
        "heartbeat must report the live feature count: {frames}"
    );
    assert!(frames.contains("health_changed"), "got: {frames}");
    assert!(
        frames.contains("\"all_healthy\":false"),
        "a degraded service must flip all_healthy: {frames}"
    );
    assert!(frames.contains("\"healthy\":1"), "got: {frames}");
    assert!(frames.contains("\"total\":2"), "got: {frames}");
}

// ── restart_service fallback ─────────────────────────────────────────────────

#[tokio::test]
async fn restart_service_without_template_reports_configured_fallback_command() {
    let (_guard, _work) = enter_sandbox("restart-fallback");
    unset_var("AGILEPLUS_SERVICE_RESTART_CMD");

    let response = restart_service(
        State(state(Vec::new())),
        RoutePath("AgilePlusCoverageProbe".to_string()),
    )
    .await
    .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value =
        serde_json::from_str(&body_text(response).await).expect("response is JSON");

    assert_eq!(json["service"], "AgilePlusCoverageProbe");
    assert_eq!(json["command"], "systemctl restart AgilePlusCoverageProbe");
    assert_eq!(
        json["status"], "error",
        "restarting a unit that does not exist must not report success: {json}"
    );
    let detail = json["error"]
        .as_str()
        .or_else(|| json["stderr"].as_str())
        .unwrap_or_default();
    assert!(
        !detail.is_empty(),
        "a failed restart must explain itself: {json}"
    );
}
