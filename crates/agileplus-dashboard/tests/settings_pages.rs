// SPDX-License-Identifier: MIT OR Apache-2.0
//! GET-side coverage for the settings pages, which resolve their content from
//! the config file and the process environment at request time.
//!
//! The existing route tests render these pages only with an empty config and an
//! empty environment, so the "configured" half of each page was never executed:
//! the persisted agent pool, the plane workspace/key hint/sync-mode branch, and
//! the working-directory fallback in `Config::config_path`.
//!
//! These handlers read `~/.agileplus/config.toml`, so every test runs with
//! `HOME` pointed at a private sandbox and serializes on one mutex, exactly like
//! `tests/settings_handlers.rs`. Nothing here can touch the operator's real
//! configuration, and the pages are asserted to be read-only.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use agileplus_dashboard::app_state::{DashboardStore, SharedState};
use agileplus_dashboard::routes::agents as agent_routes;
use agileplus_dashboard::routes::router;
use axum::Router;
use axum::body::Body;
use axum::extract::Form;
use axum::http::{Request, StatusCode};
use tokio::sync::RwLock;
use tower::util::ServiceExt;

/// Environment variables the plane settings page reads.
const PLANE_VARS: [&str; 6] = [
    "PLANE_WORKSPACE",
    "PLANE_PROJECT",
    "PLANE_API_KEY",
    "PLANE_API_URL",
    "PLANE_WEB_URL",
    "PLANE_SYNC_BIDIRECTIONAL",
];

// ── Sandbox ──────────────────────────────────────────────────────────────────

/// Serializes every test in this file and installs the sandbox `HOME`.
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
            "agileplus-dashboard-settings-pages-{}",
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

fn config_file() -> PathBuf {
    sandbox_home().join(".agileplus").join("config.toml")
}

fn reset_config() {
    let _ = std::fs::remove_file(config_file());
}

fn write_config(contents: &str) {
    std::fs::create_dir_all(config_file().parent().expect("config parent")).expect("create config dir");
    std::fs::write(config_file(), contents).expect("write sandbox config");
}

fn read_config() -> String {
    std::fs::read_to_string(config_file()).unwrap_or_default()
}

fn clear_plane_env() {
    for key in PLANE_VARS {
        // SAFETY: every test in this binary holds the file-wide mutex before
        // touching the environment.
        unsafe { std::env::remove_var(key) };
    }
}

fn set_var(key: &str, value: &str) {
    // SAFETY: see `clear_plane_env`.
    unsafe { std::env::set_var(key, value) };
}

// ── Harness ──────────────────────────────────────────────────────────────────

fn seeded_state() -> SharedState {
    Arc::new(RwLock::new(DashboardStore::seeded()))
}

fn app() -> Router {
    router(seeded_state())
}

async fn get(uri: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
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

async fn body_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    String::from_utf8(bytes.to_vec()).expect("body is utf-8")
}

// ── Agent settings page ──────────────────────────────────────────────────────

#[tokio::test]
async fn agent_settings_page_falls_back_to_defaults_without_a_config_file() {
    let _guard = lock();
    reset_config();

    let (status, html) = get("/settings/agents").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Agent Settings"), "missing page title: {html}");
    // The documented fallback pool configuration.
    assert!(html.contains(r#"value="6""#), "missing default pool size: {html}");
    assert!(html.contains(r#"value="3""#), "missing default retry budget: {html}");
    assert!(
        html.contains(r#"value="balanced" selected"#),
        "missing default dispatch mode: {html}"
    );
    assert!(!config_file().exists(), "rendering settings must not create a config");
}

#[tokio::test]
async fn agent_settings_page_renders_the_persisted_pool_configuration() {
    let _guard = lock();
    write_config(
        "[agents]\n\
         pool_size = 12\n\
         retry_budget = 7\n\
         dispatch_mode = \"priority\"\n\
         default_provider = \"gemini\"\n",
    );
    let before = read_config();

    let (status, html) = get("/settings/agents").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains(r#"value="12""#), "persisted pool size missing: {html}");
    assert!(html.contains(r#"value="7""#), "persisted retry budget missing: {html}");
    assert!(
        html.contains(r#"value="priority" selected"#),
        "persisted dispatch mode missing: {html}"
    );
    // The summary cards render the same values as the form.
    assert!(html.contains("Priority"), "dispatch mode label missing: {html}");
    assert_eq!(read_config(), before, "a settings page must be read-only");
}

#[tokio::test]
async fn agent_settings_save_route_persists_and_the_page_reflects_it() {
    let _guard = lock();
    reset_config();

    let (status, toast) = post_form(
        "/api/settings/agents",
        "pool_size=9&retry_budget=4&dispatch_mode=round-robin&default_provider=local",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        toast.contains("Agent settings saved successfully"),
        "missing success toast: {toast}"
    );
    assert!(
        toast.contains("bg-green-900"),
        "a successful save must render the success toast: {toast}"
    );

    let saved = read_config();
    assert!(saved.contains("pool_size = 9"), "config not persisted: {saved}");
    assert!(saved.contains("retry_budget = 4"), "config not persisted: {saved}");
    assert!(
        saved.contains(r#"dispatch_mode = "round-robin""#),
        "config not persisted: {saved}"
    );

    let (status, html) = get("/settings/agents").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains(r#"value="9""#), "page ignored the saved pool size: {html}");
    assert!(
        html.contains(r#"value="round-robin" selected"#),
        "page ignored the saved dispatch mode: {html}"
    );
}

#[tokio::test]
async fn duplicated_agent_handlers_in_the_agents_module_persist_the_same_configuration() {
    let _guard = lock();
    reset_config();

    // `routes::agents` carries its own copies of the agent settings handlers;
    // the router re-exports the `routes::settings` copies, so these two were
    // previously never executed by any test.
    let html = body_text(agent_routes::agent_settings_page().await).await;
    assert!(html.contains("Agent Settings"), "missing page title: {html}");
    assert!(html.contains(r#"value="6""#), "missing default pool size: {html}");

    let toast = body_text(
        agent_routes::save_agent_settings(Form(agent_routes::AgentSettingsForm {
            pool_size: 11,
            retry_budget: 2,
            dispatch_mode: "  manual  ".to_string(),
            default_provider: "  gemini  ".to_string(),
        }))
        .await,
    )
    .await;

    assert!(
        toast.contains("Agent settings saved successfully"),
        "missing success toast: {toast}"
    );
    // Surrounding whitespace from the form is trimmed before it is written.
    let saved = read_config();
    assert!(saved.contains("pool_size = 11"), "config not persisted: {saved}");
    assert!(
        saved.contains(r#"dispatch_mode = "manual""#),
        "dispatch mode must be trimmed before persisting: {saved}"
    );
    assert!(
        saved.contains(r#"default_provider = "gemini""#),
        "provider must be trimmed before persisting: {saved}"
    );

    let html = body_text(agent_routes::agent_settings_page().await).await;
    assert!(html.contains(r#"value="11""#), "page ignored the saved value: {html}");
}

// ── Services settings page ───────────────────────────────────────────────────

#[tokio::test]
async fn services_settings_page_lists_health_and_survives_a_persisted_endpoint_config() {
    let _guard = lock();
    write_config(
        "[[services]]\n\
         name = \"NATS\"\n\
         endpoint_url = \"nats://sandbox:4222\"\n\
         enabled = true\n\
         timeout_ms = 2500\n\
         max_retries = 5\n",
    );
    let before = read_config();

    let (status, html) = get("/settings/services").await;

    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Service Endpoints"), "missing page title: {html}");
    for service in ["NATS", "Dragonfly", "Neo4j", "MinIO", "SQLite"] {
        assert!(
            html.contains(service),
            "health card for {service} missing from the page"
        );
    }
    assert_eq!(read_config(), before, "a settings page must be read-only");
}

// ── Plane settings page ──────────────────────────────────────────────────────

#[tokio::test]
async fn plane_settings_page_reports_a_fully_configured_workspace() {
    let _guard = lock();
    reset_config();
    clear_plane_env();

    set_var("PLANE_WORKSPACE", "acme-workspace");
    set_var("PLANE_PROJECT", "acme-project");
    set_var("PLANE_API_KEY", "test-key");
    set_var("PLANE_API_URL", "https://plane.internal/");
    set_var("PLANE_WEB_URL", "https://plane.internal/web/");
    set_var("PLANE_SYNC_BIDIRECTIONAL", "true");

    let (status, html) = get("/settings/plane").await;

    assert_eq!(status, StatusCode::OK);
    // Trailing slashes are trimmed from both URLs before rendering.
    assert!(
        html.contains(r#"value="https://plane.internal""#),
        "api url not normalized: {html}"
    );
    assert!(
        !html.contains(r#"value="https://plane.internal/""#),
        "api url must be rendered without a trailing slash: {html}"
    );
    assert!(
        html.contains(r#"value="acme-workspace""#),
        "workspace slug not rendered: {html}"
    );
    assert!(
        html.contains(r#"value="acme-project""#),
        "project slug not rendered: {html}"
    );
    // The configured key is never echoed, only its first/last characters.
    assert!(
        html.contains("Current key: t••••••y"),
        "api key hint missing: {html}"
    );
    assert!(
        !html.contains("test-key"),
        "the raw api key must never be rendered: {html}"
    );
    // A configured workspace produces no configuration warnings.
    assert!(
        !html.contains("Missing PLANE_API_KEY"),
        "a configured workspace must not warn about its key: {html}"
    );
    assert!(
        !html.contains("Plane sync disabled until required settings are provided"),
        "a configured workspace must not be reported as disabled: {html}"
    );

    // Plane health endpoints are mapped from the store with their latencies.
    let store = DashboardStore::seeded();
    let total_features = store.features.len();
    let total_work_packages: usize = store.work_packages.values().map(Vec::len).sum();
    assert!(
        html.contains(&format!("0/{total_features} (0%)")),
        "feature coverage line missing: {html}"
    );
    assert!(
        html.contains(&format!("0/{total_work_packages} (0%)")),
        "work package coverage line missing: {html}"
    );
    assert!(html.contains(">Plane API<"), "plane api endpoint missing: {html}");
    assert!(html.contains(">API<"), "api endpoint missing: {html}");
    assert!(html.contains("(12ms)"), "plane api latency missing: {html}");

    clear_plane_env();
}

#[tokio::test]
async fn plane_settings_page_warns_when_the_workspace_is_not_configured() {
    let _guard = lock();
    reset_config();
    clear_plane_env();

    let (status, html) = get("/settings/plane").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        html.contains("Missing PLANE_API_KEY"),
        "a missing key must be reported: {html}"
    );
    assert!(
        html.contains("Missing PLANE_WORKSPACE"),
        "a missing workspace must be reported: {html}"
    );
    assert!(
        html.contains("Plane sync disabled until required settings are provided"),
        "an unconfigured page must say sync is disabled: {html}"
    );
    assert!(html.contains("Not configured"), "workspace must read unconfigured: {html}");
    // No key is configured, so the hint block is skipped entirely.
    assert!(
        !html.contains("Current key:"),
        "an unconfigured page must not render a key hint: {html}"
    );
    // The api url falls back to the public default.
    assert!(
        html.contains(r#"value="https://app.plane.so""#),
        "default api url missing: {html}"
    );
}

// ── config_path fallback ─────────────────────────────────────────────────────

#[tokio::test]
async fn config_path_falls_back_to_the_working_directory_when_home_is_unset() {
    let _guard = lock();
    reset_config();

    // `Config::config_path` joins `HOME`, so with `HOME` unset it must resolve
    // `.agileplus/config.toml` against the working directory instead.
    let work = sandbox_home().join("relative-home");
    let relative_config = work.join(".agileplus").join("config.toml");
    std::fs::create_dir_all(relative_config.parent().expect("config parent"))
        .expect("create relative config dir");
    std::fs::write(
        &relative_config,
        "[agents]\npool_size = 21\nretry_budget = 5\ndispatch_mode = \"manual\"\n\
         default_provider = \"local\"\n",
    )
    .expect("write relative config");

    let previous_cwd = std::env::current_dir().expect("read current dir");
    let previous_home = std::env::var("HOME").ok();
    std::env::set_current_dir(&work).expect("enter relative-home sandbox");
    // SAFETY: the file-wide mutex is held, so no other test reads `HOME`.
    unsafe { std::env::remove_var("HOME") };

    let (status, html) = get("/settings/agents").await;

    std::env::set_current_dir(&previous_cwd).expect("restore working directory");
    if let Some(home) = &previous_home {
        // SAFETY: see above.
        unsafe { std::env::set_var("HOME", home) };
    }

    assert_eq!(status, StatusCode::OK);
    assert!(
        html.contains(r#"value="21""#),
        "the working-directory config was not read: {html}"
    );
    assert!(
        html.contains(r#"value="manual" selected"#),
        "the working-directory dispatch mode was not read: {html}"
    );
}
