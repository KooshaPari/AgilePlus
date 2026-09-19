// SPDX-License-Identifier: MIT OR Apache-2.0
//! Degradation paths for `seed_bridge::try_merge_from_api`.
//!
//! The existing `tests/live_clients.rs` covers the happy path (a well-formed
//! api replaces the seed), an unreachable api, and empty collections. It never
//! covers an api that *answers* with something unusable: a body that is not
//! JSON, a body of the wrong shape, or a rejected credential. Those are the
//! paths that decide whether a broken api can corrupt the dashboard's startup
//! state, and they are the ones the merge's `if let Some(..)` guards exist for.
//!
//! The cycle-index rebuild (`features whose module_id matches the cycle id`) is
//! also only exercised with an empty result today, so the second test drives it
//! with features that actually match.
//!
//! Every test redirects `HOME` to a private sandbox and serializes on one mutex;
//! `AGILEPLUS_API_*` is owned by the same guard.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use agileplus_dashboard::app_state::DashboardStore;
use agileplus_dashboard::seed_bridge::try_merge_from_api;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

// ── Sandbox ──────────────────────────────────────────────────────────────────

/// Serializes the file and installs the sandbox `HOME`.
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
            "agileplus-dashboard-seed-bridge-degradation-{}",
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
    // touching the environment, and no other thread in this process reads these
    // variables.
    unsafe { std::env::set_var(key, value) };
}

fn unset_var(key: &str) {
    // SAFETY: see `set_var`.
    unsafe { std::env::remove_var(key) };
}

// ── Stub api ─────────────────────────────────────────────────────────────────

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

// ── Malformed payloads ───────────────────────────────────────────────────────

#[tokio::test]
async fn unusable_api_payloads_leave_every_seeded_collection_intact() {
    let _guard = lock();
    unset_var("AGILEPLUS_API_KEY");

    let routes = Router::new()
        // 200 with a body that is not JSON at all.
        .route(
            "/api/modules",
            get(|| async { (StatusCode::OK, "this is not json") }),
        )
        // 200 with valid JSON of the wrong shape for the target type.
        .route(
            "/api/cycles",
            get(|| async { axum::Json(serde_json::json!({ "cycles": [] })) }),
        )
        // A key-protected endpoint that rejects the caller.
        .route(
            "/api/v1/projects",
            get(|| async { (StatusCode::UNAUTHORIZED, "unauthorized") }),
        );
    let base = serve(routes).await;
    set_var("AGILEPLUS_API_BASE", &base);
    set_var("AGILEPLUS_API_KEY", "rejected-key");

    let seeded = DashboardStore::seeded();
    let seed_modules: Vec<String> = seeded.modules.iter().map(|m| m.slug.clone()).collect();
    let seed_cycles: Vec<String> = seeded.cycles.iter().map(|c| c.name.clone()).collect();
    let seed_projects: Vec<String> = seeded.projects.iter().map(|p| p.slug.clone()).collect();
    let seed_cycle_features = seeded.cycle_features.clone();
    let seed_feature_count = seeded.features.len();

    let merged = try_merge_from_api(seeded).await;

    assert_eq!(
        merged.modules.iter().map(|m| m.slug.clone()).collect::<Vec<_>>(),
        seed_modules,
        "a non-JSON module payload must not replace the seed"
    );
    assert_eq!(
        merged.cycles.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
        seed_cycles,
        "a wrong-shaped cycle payload must not replace the seed"
    );
    assert_eq!(
        merged
            .cycle_features
            .keys()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        seed_cycle_features
            .keys()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        "a wrong-shaped cycle payload must not clear the cycle index"
    );
    assert_eq!(merged.cycle_features, seed_cycle_features);
    assert_eq!(
        merged.projects.iter().map(|p| p.slug.clone()).collect::<Vec<_>>(),
        seed_projects,
        "a rejected credential must leave the seed projects in place"
    );
    assert_eq!(merged.active_project_id, Some(1));
    assert_eq!(merged.features.len(), seed_feature_count);

    unset_var("AGILEPLUS_API_KEY");
    unset_var("AGILEPLUS_API_BASE");
}

// ── Cycle index rebuild ──────────────────────────────────────────────────────

fn module_feature(id: i64, module_id: Option<i64>) -> Feature {
    let mut feature = Feature::new(
        &format!("feat-{id}"),
        &format!("Feature {id}"),
        [0; 32],
        None,
    );
    feature.id = id;
    feature.state = FeatureState::Created;
    feature.module_id = module_id;
    feature
}

#[tokio::test]
async fn merge_rebuilds_the_cycle_index_from_feature_module_ids() {
    let _guard = lock();
    unset_var("AGILEPLUS_API_KEY");

    let cycle = serde_json::json!([{
        "id": 42,
        "name": "Cycle 42",
        "description": null,
        "start_date": "2026-01-01",
        "end_date": "2026-01-14",
        "state": "Active",
        "module_scope_id": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z"
    }]);
    let routes = Router::new()
        .route(
            "/api/modules",
            get(|| async { axum::Json(serde_json::Value::Array(Vec::new())) }),
        )
        .route(
            "/api/cycles",
            get(move || {
                let cycle = cycle.clone();
                async move { axum::Json(cycle) }
            }),
        );
    let base = serve(routes).await;
    set_var("AGILEPLUS_API_BASE", &base);

    let mut store = DashboardStore::default();
    store.features = vec![
        module_feature(1, Some(42)),
        module_feature(2, Some(7)),
        module_feature(3, Some(42)),
        module_feature(4, None),
    ];

    let merged = try_merge_from_api(store).await;

    assert_eq!(merged.cycles.len(), 1);
    assert_eq!(merged.cycles[0].id, 42);
    assert_eq!(
        merged.cycle_feature_ids(42),
        vec![1, 3],
        "features whose module_id matches the cycle id form the cycle index"
    );
    assert!(
        merged.cycle_feature_ids(7).is_empty(),
        "a feature in another module must not be indexed into this cycle"
    );
    assert_eq!(merged.features.len(), 4, "features are never rewritten by a merge");

    unset_var("AGILEPLUS_API_BASE");
}
