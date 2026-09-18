// SPDX-License-Identifier: MIT OR Apache-2.0
//! Routes backed by live `GovernanceClient` / `PlaneClient` instances, and the
//! startup bridge that merges the `agileplus-api` project/module/cycle lists
//! into the seed store.
//!
//! In production `main.rs` installs a governance client and (when `PLANE_*` is
//! configured) a plane client, then serves `/api/dashboard/governance/status`
//! and `/api/dashboard/plane/sync` from them. Every existing test builds a store
//! without either client, so only the `not_initialized` half of those handlers is
//! exercised. The same is true of `seed_bridge::try_merge_from_api`, which no test
//! calls at all.
//!
//! The API merge and the plane client are driven against a loopback stub server
//! so no external network is required. The governance client keeps its local
//! audit database inside a private sandbox, and `AGILEPLUS_GOVERNANCE_ENABLED` is
//! pinned to `false` so the client never dials a remote governance service.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use agileplus_dashboard::app_state::{DashboardStore, ServiceHealth, SharedState};
use agileplus_dashboard::routes::router;
use agileplus_dashboard::seed_bridge::try_merge_from_api;
use agileplus_governance::GovernanceClient;
use agileplus_plane::PlaneClient;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::sync::RwLock;
use tower::util::ServiceExt;

/// API key the stub projects endpoint requires.
const STUB_API_KEY: &str = "stub-api-key";

// ── Sandbox / environment guard ──────────────────────────────────────────────

fn sandbox_root() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!(
            "agileplus-dashboard-live-clients-{}",
            std::process::id()
        ));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clear previous sandbox");
        }
        std::fs::create_dir_all(&dir).expect("create sandbox root");
        dir
    })
}

/// Serializes the file and owns both the working directory and the
/// `AGILEPLUS_*` / `PLANE_*` variables these tests install.
struct EnvGuard {
    previous_cwd: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous_cwd);
    }
}

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
    // environment; no other thread in this process reads these variables.
    unsafe { std::env::set_var(key, value) };
}

fn unset_var(key: &str) {
    // SAFETY: see `set_var`.
    unsafe { std::env::remove_var(key) };
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn state(store: DashboardStore) -> SharedState {
    Arc::new(RwLock::new(store))
}

fn store_with_health() -> DashboardStore {
    DashboardStore {
        health: vec![ServiceHealth {
            name: "API".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(1),
            last_check: chrono::Utc::now(),
        }],
        ..Default::default()
    }
}

/// Serve `routes` on an ephemeral loopback port and return its base URL.
async fn serve(routes: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stub server");
    let addr = listener.local_addr().expect("stub server address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, routes).await;
    });
    format!("http://{addr}")
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

// ── Stub API fixtures ────────────────────────────────────────────────────────

fn stub_module() -> serde_json::Value {
    let seed = DashboardStore::seeded();
    let mut module = seed.modules.first().cloned().expect("seed module");
    module.id = 900;
    module.slug = "stub-module".into();
    module.friendly_name = "Stub Module".into();
    module.description = Some("from the stub api".into());
    serde_json::to_value(module).expect("module json")
}

fn stub_cycle() -> serde_json::Value {
    let seed = DashboardStore::seeded();
    let mut cycle = seed.cycles.first().cloned().expect("seed cycle");
    cycle.id = 42;
    cycle.name = "Stub Sprint".into();
    cycle.description = Some("from the stub api".into());
    serde_json::to_value(cycle).expect("cycle json")
}

fn stub_project() -> serde_json::Value {
    let mut project =
        agileplus_domain::domain::project::Project::new("Stub Project", "stub-project")
            .expect("valid project");
    project.id = 7;
    project.description = Some("from the stub api".into());
    serde_json::to_value(project).expect("project json")
}

/// Stub `agileplus-api` that serves `modules`, `cycles`, and (when
/// `with_projects`) a key-protected project list.
async fn serve_agileplus_api(modules: serde_json::Value, cycles: serde_json::Value) -> String {
    let projects = stub_project();
    let routes = Router::new()
        .route(
            "/api/modules",
            get({
                let modules = modules.clone();
                move || {
                    let modules = modules.clone();
                    async move { axum::Json(modules) }
                }
            }),
        )
        .route(
            "/api/cycles",
            get({
                let cycles = cycles.clone();
                move || {
                    let cycles = cycles.clone();
                    async move { axum::Json(cycles) }
                }
            }),
        )
        .route(
            "/api/v1/projects",
            get(move |headers: HeaderMap| {
                let projects = projects.clone();
                async move {
                    let key = headers
                        .get("X-API-Key")
                        .and_then(|value| value.to_str().ok());
                    if key != Some(STUB_API_KEY) {
                        return StatusCode::UNAUTHORIZED.into_response();
                    }
                    axum::Json(serde_json::Value::Array(vec![projects])).into_response()
                }
            }),
        );
    serve(routes).await
}

fn seed_slugs(store: &DashboardStore) -> (Vec<String>, Vec<String>, Vec<String>) {
    (
        store.modules.iter().map(|m| m.slug.clone()).collect(),
        store.cycles.iter().map(|c| c.name.clone()).collect(),
        store.projects.iter().map(|p| p.slug.clone()).collect(),
    )
}

// ── Governance client ────────────────────────────────────────────────────────

#[tokio::test]
async fn governance_status_reports_an_installed_client() {
    let (_guard, work) = enter_sandbox("governance");
    set_var("AGILEPLUS_GOVERNANCE_ENABLED", "false");
    set_var(
        "AGILEPLUS_LOCAL_DB_PATH",
        work.join("governance.db").to_str().expect("utf-8 path"),
    );

    let client = GovernanceClient::with_defaults()
        .await
        .expect("local governance client must initialize without a remote service");
    let app = router(state(store_with_health().with_governance(client)));

    let json = get_json(&app, "/api/dashboard/governance/status").await;

    assert_eq!(json["available"], true);
    assert_eq!(json["initialized"], true);
    assert_eq!(json["remote_enabled"], false);
    assert_eq!(json["local_enabled"], true);
    assert_eq!(json["sync_enabled"], true);
    assert_eq!(json["connection_status"], "Disabled");
    assert_eq!(json["audits_total"], 0);
    assert_eq!(json["audits_today"], 0);
    assert_eq!(json["audit_errors"], 0);
    assert_eq!(json["pending_operations"], 0);
}

// ── Plane client ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn plane_sync_status_counts_work_items_from_a_reachable_api() {
    let (_guard, _work) = enter_sandbox("plane-reachable");

    let routes = Router::new().route(
        "/api/v1/workspaces/{workspace}/projects/{project}/work-items/",
        get(|| async {
            axum::Json(serde_json::json!([
                {"id": "wi-1", "name": "First work item"},
                {"id": "wi-2", "name": "Second work item"},
            ]))
        }),
    );
    let base = serve(routes).await;

    let client = PlaneClient::new(base, "plane-key".into(), "acme".into(), "proj-1".into());
    let app = router(state(store_with_health().with_plane(client)));

    let json = get_json(&app, "/api/dashboard/plane/sync").await;

    assert_eq!(json["available"], true);
    assert_eq!(json["work_items"], 2);
    assert!(json.get("error").is_none(), "unexpected error: {json}");
    assert!(
        json["synced_at"]
            .as_str()
            .is_some_and(|ts| ts.contains('T')),
        "synced_at must be an RFC3339 instant: {json}"
    );
}

#[tokio::test]
async fn plane_sync_status_stays_available_when_the_api_is_unreachable() {
    let (_guard, _work) = enter_sandbox("plane-unreachable");

    // Port 1 is privileged and never served, so the request fails fast.
    let client = PlaneClient::new(
        "http://127.0.0.1:1".into(),
        "plane-key".into(),
        "acme".into(),
        "proj-1".into(),
    );
    let app = router(state(store_with_health().with_plane(client)));

    let json = get_json(&app, "/api/dashboard/plane/sync").await;

    assert_eq!(json["available"], true);
    assert!(
        json.get("work_items").is_none(),
        "a failed fetch must not report a count: {json}"
    );
    assert!(
        json["error"]
            .as_str()
            .is_some_and(|error| !error.is_empty()),
        "a failed fetch must explain itself: {json}"
    );
    assert!(json["synced_at"].as_str().is_some());
}

// ── seed_bridge::try_merge_from_api ──────────────────────────────────────────

#[tokio::test]
async fn merge_from_api_replaces_modules_cycles_and_projects() {
    let (_guard, _work) = enter_sandbox("merge");
    set_var("AGILEPLUS_API_KEY", STUB_API_KEY);
    let base = serve_agileplus_api(
        serde_json::Value::Array(vec![stub_module()]),
        serde_json::Value::Array(vec![stub_cycle()]),
    )
    .await;
    set_var("AGILEPLUS_API_BASE", &base);

    let seeded = DashboardStore::seeded();
    let feature_count = seeded.features.len();
    let merged = try_merge_from_api(seeded).await;

    // Public lists replace the seed.
    assert_eq!(merged.modules.len(), 1);
    assert_eq!(merged.modules[0].slug, "stub-module");
    assert_eq!(merged.modules[0].id, 900);

    assert_eq!(merged.cycles.len(), 1);
    assert_eq!(merged.cycles[0].id, 42);
    assert_eq!(merged.cycles[0].name, "Stub Sprint");

    // The cycle -> feature index is rebuilt, so the seed's cycle 1 disappears.
    let mut cycle_ids: Vec<i64> = merged.cycle_features.keys().copied().collect();
    cycle_ids.sort_unstable();
    assert_eq!(cycle_ids, vec![42]);
    assert!(
        merged.cycle_feature_ids(1).is_empty(),
        "the seed cycle index must be dropped when the api answers"
    );

    // The key-protected project list replaces the seed projects.
    assert_eq!(merged.projects.len(), 1);
    assert_eq!(merged.projects[0].slug, "stub-project");
    assert_eq!(merged.projects[0].id, 7);
    assert!(
        merged.active_project().is_none(),
        "seed active project 1 no longer exists once the api list replaces it"
    );

    // Features and work packages are never touched by the merge.
    assert_eq!(merged.features.len(), feature_count);
    let mut merged_keys: Vec<i64> = merged.work_packages.keys().copied().collect();
    let mut seed_keys: Vec<i64> = DashboardStore::seeded()
        .work_packages
        .keys()
        .copied()
        .collect();
    merged_keys.sort_unstable();
    seed_keys.sort_unstable();
    assert_eq!(merged_keys, seed_keys);
    assert!(merged_keys.iter().all(|key| {
        !merged
            .work_packages
            .get(key)
            .expect("merged work packages")
            .is_empty()
    }));
}

#[tokio::test]
async fn merge_from_api_activates_the_first_project_when_none_is_active() {
    let (_guard, _work) = enter_sandbox("merge-activate");
    set_var("AGILEPLUS_API_KEY", STUB_API_KEY);
    let base = serve_agileplus_api(
        serde_json::Value::Array(vec![stub_module()]),
        serde_json::Value::Array(vec![stub_cycle()]),
    )
    .await;
    set_var("AGILEPLUS_API_BASE", &base);

    let mut seeded = DashboardStore::seeded();
    seeded.active_project_id = None;

    let merged = try_merge_from_api(seeded).await;

    assert_eq!(merged.active_project_id, Some(7));
    assert_eq!(
        merged.active_project().map(|project| project.slug.as_str()),
        Some("stub-project")
    );
}

#[tokio::test]
async fn merge_from_api_keeps_seed_projects_without_an_api_key() {
    let (_guard, _work) = enter_sandbox("merge-no-key");
    unset_var("AGILEPLUS_API_KEY");
    let base = serve_agileplus_api(
        serde_json::Value::Array(vec![stub_module()]),
        serde_json::Value::Array(vec![stub_cycle()]),
    )
    .await;
    set_var("AGILEPLUS_API_BASE", &base);

    let seeded = DashboardStore::seeded();
    let (_, _, seed_projects) = seed_slugs(&seeded);
    let merged = try_merge_from_api(seeded).await;

    assert_eq!(merged.modules[0].slug, "stub-module");
    assert_eq!(
        merged
            .projects
            .iter()
            .map(|project| project.slug.clone())
            .collect::<Vec<_>>(),
        seed_projects,
        "without a key the protected project list must stay seeded"
    );
    assert_eq!(merged.active_project_id, Some(1));
}

#[tokio::test]
async fn merge_from_api_ignores_empty_api_collections() {
    let (_guard, _work) = enter_sandbox("merge-empty");
    set_var("AGILEPLUS_API_KEY", STUB_API_KEY);
    let base = serve_agileplus_api(
        serde_json::Value::Array(Vec::new()),
        serde_json::Value::Array(Vec::new()),
    )
    .await;
    set_var("AGILEPLUS_API_BASE", &base);

    let seeded = DashboardStore::seeded();
    let (seed_modules, seed_cycles, _seed_projects) = seed_slugs(&seeded);
    let seed_cycle_features = seeded.cycle_features.clone();
    let merged = try_merge_from_api(seeded).await;

    let (modules, cycles, projects) = seed_slugs(&merged);
    assert_eq!(modules, seed_modules, "an empty module list is not a delta");
    assert_eq!(cycles, seed_cycles, "an empty cycle list is not a delta");
    assert_eq!(
        merged.cycle_features, seed_cycle_features,
        "an empty cycle list must not clear the seed index"
    );
    assert_eq!(
        projects,
        vec!["stub-project".to_string()],
        "the project list is merged independently of the module and cycle lists"
    );
}

#[tokio::test]
async fn merge_from_api_falls_back_to_seed_when_the_api_is_unreachable() {
    let (_guard, _work) = enter_sandbox("merge-unreachable");
    unset_var("AGILEPLUS_API_KEY");
    set_var("AGILEPLUS_API_BASE", "http://127.0.0.1:1");

    let seeded = DashboardStore::seeded();
    let (seed_modules, seed_cycles, seed_projects) = seed_slugs(&seeded);
    let seed_cycle_features = seeded.cycle_features.clone();
    let merged = try_merge_from_api(seeded).await;

    let (modules, cycles, projects) = seed_slugs(&merged);
    assert_eq!(modules, seed_modules);
    assert_eq!(cycles, seed_cycles);
    assert_eq!(projects, seed_projects);
    assert_eq!(merged.cycle_features, seed_cycle_features);
    assert_eq!(merged.active_project_id, Some(1));
}
