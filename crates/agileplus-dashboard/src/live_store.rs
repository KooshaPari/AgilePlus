// SPDX-License-Identifier: MIT OR Apache-2.0
//! Honest dashboard data-source selection and query-only project projection.
//!
//! The native Askama dashboard is not an acceptance authority. Until the
//! device service owns all mutations, production-mode dashboard requests are
//! read-only. Seeded data remains available only through an explicit demo flag.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use agileplus_domain::domain::{
    feature::Feature,
    project::Project,
    state_machine::FeatureState,
    work_package::{PrState, WorkPackage, WpState},
};
use axum::extract::{Extension, Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags, Row};
use serde::Serialize;

use crate::app_state::{DashboardStore, ServiceHealth};

const DEMO_ENV: &str = "AGILEPLUS_DASHBOARD_DEMO";
const PROJECT_ROOT_ENV: &str = "AGILEPLUS_PROJECT_ROOT";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardSourceMode {
    Disconnected,
    Demo,
    ProjectDatabaseReadOnly,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSource {
    pub mode: DashboardSourceMode,
    pub connected: bool,
    pub read_only: bool,
    pub project_name: Option<String>,
    pub message: String,
    pub loaded_at: DateTime<Utc>,
    pub feature_count: usize,
    pub work_package_count: usize,
    pub project_count: usize,
    #[serde(skip)]
    pub diagnostic: Option<String>,
}

impl DashboardSource {
    fn disconnected(diagnostic: impl Into<String>) -> Self {
        Self {
            mode: DashboardSourceMode::Disconnected,
            connected: false,
            read_only: true,
            project_name: None,
            message: "No live AgilePlus project is connected. Select or configure a project; seeded data is not shown.".into(),
            loaded_at: Utc::now(),
            feature_count: 0,
            work_package_count: 0,
            project_count: 0,
            diagnostic: Some(diagnostic.into()),
        }
    }

    fn demo(store: &DashboardStore) -> Self {
        Self {
            mode: DashboardSourceMode::Demo,
            connected: false,
            read_only: false,
            project_name: Some("AgilePlus demo fixtures".into()),
            message: "Explicit demo mode: all displayed records are fixtures, not live project state.".into(),
            loaded_at: Utc::now(),
            feature_count: store.features.len(),
            work_package_count: store.work_packages.values().map(Vec::len).sum(),
            project_count: store.projects.len(),
            diagnostic: None,
        }
    }

    fn live(project_root: &Path, store: &DashboardStore) -> Self {
        Self {
            mode: DashboardSourceMode::ProjectDatabaseReadOnly,
            connected: true,
            read_only: true,
            project_name: project_root
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_owned),
            message: "Live project database projection. Mutations are disabled until routed through the AgilePlus device service.".into(),
            loaded_at: Utc::now(),
            feature_count: store.features.len(),
            work_package_count: store.work_packages.values().map(Vec::len).sum(),
            project_count: store.projects.len(),
            diagnostic: None,
        }
    }

    pub fn allows_mutation(&self) -> bool {
        self.mode == DashboardSourceMode::Demo
    }
}

pub struct DashboardRuntime {
    pub store: DashboardStore,
    pub source: DashboardSource,
}

pub fn initialize_from_environment() -> DashboardRuntime {
    let demo = std::env::var(DEMO_ENV).is_ok_and(|value| value == "1");
    let configured_root = std::env::var_os(PROJECT_ROOT_ENV);
    let cwd = std::env::current_dir().unwrap_or_default();
    initialize(demo, configured_root.as_deref(), &cwd)
}

fn initialize(demo: bool, configured_root: Option<&OsStr>, cwd: &Path) -> DashboardRuntime {
    if demo {
        let store = DashboardStore::seeded();
        let source = DashboardSource::demo(&store);
        return DashboardRuntime { store, source };
    }

    let root = match configured_root {
        Some(root) => Some(PathBuf::from(root)),
        None => find_project_root(cwd),
    };

    let Some(root) = root else {
        return DashboardRuntime {
            store: DashboardStore::default(),
            source: DashboardSource::disconnected(format!(
                "no .agileplus/agileplus.db found from {} and {PROJECT_ROOT_ENV} is unset",
                cwd.display()
            )),
        };
    };

    match load_project_store(&root) {
        Ok((canonical_root, store)) => {
            let source = DashboardSource::live(&canonical_root, &store);
            DashboardRuntime { store, source }
        }
        Err(error) => DashboardRuntime {
            store: DashboardStore::default(),
            source: DashboardSource::disconnected(error),
        },
    }
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".agileplus").join("agileplus.db").is_file() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn load_project_store(project_root: &Path) -> Result<(PathBuf, DashboardStore), String> {
    let canonical_root = canonical_git_root(project_root)?;
    let database = canonical_root.join(".agileplus").join("agileplus.db");
    if !database.is_file() {
        return Err(format!(
            "project database does not exist at {}",
            database.display()
        ));
    }

    let connection = Connection::open_with_flags(&database, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("cannot open project database read-only: {error}"))?;
    connection
        .execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON;")
        .map_err(|error| format!("cannot configure query-only projection: {error}"))?;

    let features = load_features(&connection)?;
    let work_packages = load_work_packages(&connection)?;
    let projects = load_projects(&connection)?;
    let work_packages = work_packages.into_iter().fold(
        HashMap::<i64, Vec<WorkPackage>>::new(),
        |mut grouped, work_package| {
            grouped
                .entry(work_package.feature_id)
                .or_default()
                .push(work_package);
            grouped
        },
    );

    let store = DashboardStore {
        features,
        work_packages,
        modules: Vec::new(),
        cycles: Vec::new(),
        cycle_features: HashMap::new(),
        health: vec![ServiceHealth {
            name: "Project SQLite projection".into(),
            healthy: true,
            degraded: false,
            latency_ms: None,
            last_check: Utc::now(),
        }],
        projects,
        // Current feature rows do not carry a reliable project foreign key.
        // Leaving selection unset shows every live feature instead of silently
        // filtering them all out under a fabricated association.
        active_project_id: None,
        governance_client: None,
        plane_client: None,
        plane_daemon: None,
    };

    Ok((canonical_root, store))
}

fn canonical_git_root(project_root: &Path) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(project_root).map_err(|error| {
        format!(
            "cannot canonicalize dashboard project root {}: {error}",
            project_root.display()
        )
    })?;
    let output = Command::new("git")
        .args(["-C"])
        .arg(&canonical)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| format!("cannot run git project-root discovery: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "dashboard project root is not a Git worktree: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let discovered = String::from_utf8(output.stdout)
        .map_err(|_| "git returned a non-UTF-8 project root".to_string())?;
    let discovered = fs::canonicalize(discovered.trim())
        .map_err(|error| format!("cannot canonicalize Git project root: {error}"))?;
    if discovered != canonical {
        return Err(format!(
            "configured dashboard root {} resolves to Git worktree {}",
            canonical.display(),
            discovered.display()
        ));
    }
    Ok(canonical)
}

fn load_features(connection: &Connection) -> Result<Vec<Feature>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at, module_id, labels
             FROM features ORDER BY created_at DESC",
        )
        .map_err(|error| format!("cannot prepare live feature projection: {error}"))?;
    statement
        .query_map([], row_to_feature)
        .map_err(|error| format!("cannot query live features: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot decode live feature row: {error}"))
}

fn row_to_feature(row: &Row<'_>) -> rusqlite::Result<Feature> {
    let state_text: String = row.get(3)?;
    let state = state_text.parse::<FeatureState>().map_err(|error| {
        conversion_error(3, rusqlite::types::Type::Text, error)
    })?;
    let spec_hash_bytes: Vec<u8> = row.get(4)?;
    let spec_hash: [u8; 32] = spec_hash_bytes.try_into().map_err(|bytes: Vec<u8>| {
        conversion_error(
            4,
            rusqlite::types::Type::Blob,
            format!("spec hash must contain 32 bytes, found {}", bytes.len()),
        )
    })?;
    let labels_json: String = row.get(9)?;
    let labels = serde_json::from_str(&labels_json)
        .map_err(|error| conversion_error(9, rusqlite::types::Type::Text, error))?;

    Ok(Feature {
        id: row.get(0)?,
        slug: row.get(1)?,
        friendly_name: row.get(2)?,
        state,
        spec_hash,
        target_branch: row.get(5)?,
        plane_issue_id: None,
        plane_state_id: None,
        labels,
        module_id: row.get(8)?,
        project_id: None,
        created_at_commit: None,
        last_modified_commit: None,
        created_at: parse_datetime(row, 6)?,
        updated_at: parse_datetime(row, 7)?,
    })
}

fn load_work_packages(connection: &Connection) -> Result<Vec<WorkPackage>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, feature_id, title, state, sequence, file_scope, acceptance_criteria,
                    agent_id, pr_url, pr_state, worktree_path, created_at, updated_at
             FROM work_packages ORDER BY feature_id, sequence",
        )
        .map_err(|error| format!("cannot prepare live work-package projection: {error}"))?;
    statement
        .query_map([], row_to_work_package)
        .map_err(|error| format!("cannot query live work packages: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot decode live work-package row: {error}"))
}

fn row_to_work_package(row: &Row<'_>) -> rusqlite::Result<WorkPackage> {
    let state_text: String = row.get(3)?;
    let state = match state_text.as_str() {
        "planned" => WpState::Planned,
        "doing" => WpState::Doing,
        "review" => WpState::Review,
        "done" => WpState::Done,
        "blocked" => WpState::Blocked,
        _ => {
            return Err(conversion_error(
                3,
                rusqlite::types::Type::Text,
                format!("unknown work-package state {state_text:?}"),
            ));
        }
    };
    let file_scope_json: String = row.get(5)?;
    let file_scope = serde_json::from_str(&file_scope_json)
        .map_err(|error| conversion_error(5, rusqlite::types::Type::Text, error))?;
    let pr_state_text: Option<String> = row.get(9)?;
    let pr_state = match pr_state_text.as_deref() {
        None => None,
        Some("open") => Some(PrState::Open),
        Some("review") => Some(PrState::Review),
        Some("changes_requested") => Some(PrState::ChangesRequested),
        Some("approved") => Some(PrState::Approved),
        Some("merged") => Some(PrState::Merged),
        Some(other) => {
            return Err(conversion_error(
                9,
                rusqlite::types::Type::Text,
                format!("unknown PR state {other:?}"),
            ));
        }
    };

    Ok(WorkPackage {
        id: row.get(0)?,
        feature_id: row.get(1)?,
        title: row.get(2)?,
        state,
        sequence: row.get(4)?,
        file_scope,
        acceptance_criteria: row.get(6)?,
        agent_id: row.get(7)?,
        pr_url: row.get(8)?,
        pr_state,
        worktree_path: row.get(10)?,
        plane_sub_issue_id: None,
        base_commit: None,
        head_commit: None,
        created_at: parse_datetime(row, 11)?,
        updated_at: parse_datetime(row, 12)?,
    })
}

fn load_projects(connection: &Connection) -> Result<Vec<Project>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, slug, name, COALESCE(description, ''), created_at, updated_at
             FROM projects ORDER BY created_at ASC",
        )
        .map_err(|error| format!("cannot prepare live project projection: {error}"))?;
    statement
        .query_map([], |row| {
            let description: String = row.get(3)?;
            Ok(Project {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                description: (!description.is_empty()).then_some(description),
                created_at: parse_datetime(row, 4)?,
                updated_at: parse_datetime(row, 5)?,
            })
        })
        .map_err(|error| format!("cannot query live projects: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot decode live project row: {error}"))
}

fn parse_datetime(row: &Row<'_>, index: usize) -> rusqlite::Result<DateTime<Utc>> {
    let value: String = row.get(index)?;
    value
        .parse::<DateTime<Utc>>()
        .map_err(|error| conversion_error(index, rusqlite::types::Type::Text, error))
}

fn conversion_error(
    index: usize,
    value_type: rusqlite::types::Type,
    error: impl std::fmt::Display,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        value_type,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error.to_string(),
        )),
    )
}

pub async fn source_status(
    Extension(source): Extension<Arc<DashboardSource>>,
) -> Json<DashboardSource> {
    Json((*source).clone())
}

pub async fn mutation_guard(
    State(source): State<Arc<DashboardSource>>,
    request: Request,
    next: Next,
) -> Response {
    if matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    ) || source.allows_mutation()
    {
        return next.run(request).await;
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "dashboard_read_only",
            "source_mode": source.mode,
            "message": "Dashboard mutation is disabled until the request is routed through the canonical AgilePlus device service."
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::middleware;
    use axum::routing::post;
    use axum::Router;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tower::ServiceExt;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "agileplus-dashboard-live-{name}-{}-{suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(path.join(".agileplus")).expect("create fixture root");
            let init = Command::new("git")
                .args(["init", "-q", "-b", "main"])
                .current_dir(&path)
                .output()
                .expect("git init");
            assert!(init.status.success());
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn seed_database(&self, feature_state: &str) {
            let database = self.path().join(".agileplus").join("agileplus.db");
            let connection = Connection::open(database).expect("create fixture database");
            connection
                .execute_batch(
                    "CREATE TABLE features (
                        id INTEGER PRIMARY KEY, slug TEXT NOT NULL, friendly_name TEXT NOT NULL,
                        state TEXT NOT NULL, spec_hash BLOB NOT NULL, target_branch TEXT NOT NULL,
                        created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                        module_id INTEGER, labels TEXT NOT NULL
                    );
                    CREATE TABLE work_packages (
                        id INTEGER PRIMARY KEY, feature_id INTEGER NOT NULL, title TEXT NOT NULL,
                        state TEXT NOT NULL, sequence INTEGER NOT NULL, file_scope TEXT NOT NULL,
                        acceptance_criteria TEXT NOT NULL, agent_id TEXT, pr_url TEXT, pr_state TEXT,
                        worktree_path TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
                    );
                    CREATE TABLE projects (
                        id INTEGER PRIMARY KEY, slug TEXT NOT NULL, name TEXT NOT NULL,
                        description TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
                    );",
                )
                .expect("create fixture schema");
            let now = Utc::now().to_rfc3339();
            connection
                .execute(
                    "INSERT INTO features VALUES (1, 'live-feature', 'Live Feature', ?1, ?2, 'main', ?3, ?3, NULL, '[\"live\"]')",
                    rusqlite::params![feature_state, vec![7_u8; 32], now],
                )
                .expect("insert feature");
            connection
                .execute(
                    "INSERT INTO work_packages VALUES (1, 1, 'Live WP', 'planned', 1, '[\"src/**\"]', 'FR-1 done', NULL, NULL, NULL, NULL, ?1, ?1)",
                    [&now],
                )
                .expect("insert work package");
            connection
                .execute(
                    "INSERT INTO projects VALUES (1, 'live-project', 'Live Project', '', ?1, ?1)",
                    [&now],
                )
                .expect("insert project");
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn live_store_loads_real_rows_without_seed_fallback() {
        let scratch = Scratch::new("load");
        scratch.seed_database("created");
        let (root, store) = load_project_store(scratch.path()).expect("load live store");
        assert_eq!(root, fs::canonicalize(scratch.path()).unwrap());
        assert_eq!(store.features.len(), 1);
        assert_eq!(store.features[0].slug, "live-feature");
        assert_eq!(store.work_packages[&1][0].title, "Live WP");
        assert_eq!(store.projects[0].slug, "live-project");
        assert!(store.modules.is_empty());
        assert!(store.cycles.is_empty());
    }

    #[test]
    fn invalid_live_row_disconnects_instead_of_silently_dropping_data() {
        let scratch = Scratch::new("invalid");
        scratch.seed_database("invented_state");
        let error = load_project_store(scratch.path())
            .err()
            .expect("invalid state must fail the projection");
        assert!(error.contains("cannot decode live feature row"));
    }

    #[test]
    fn demo_requires_explicit_opt_in() {
        let scratch = Scratch::new("demo");
        let disconnected = initialize(false, None, scratch.path());
        assert_eq!(
            disconnected.source.mode,
            DashboardSourceMode::Disconnected
        );
        assert!(disconnected.store.features.is_empty());

        let demo = initialize(true, None, scratch.path());
        assert_eq!(demo.source.mode, DashboardSourceMode::Demo);
        assert!(!demo.store.features.is_empty());
        assert!(demo.source.allows_mutation());
    }

    #[tokio::test]
    async fn live_mode_blocks_non_get_requests() {
        let source = Arc::new(DashboardSource {
            mode: DashboardSourceMode::ProjectDatabaseReadOnly,
            connected: true,
            read_only: true,
            project_name: Some("project".into()),
            message: "live".into(),
            loaded_at: Utc::now(),
            feature_count: 1,
            work_package_count: 1,
            project_count: 1,
            diagnostic: None,
        });
        let app = Router::new()
            .route("/mutate", post(|| async { StatusCode::NO_CONTENT }))
            .layer(middleware::from_fn_with_state(source, mutation_guard));
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mutate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn explicit_demo_mode_allows_fixture_mutation() {
        let store = DashboardStore::seeded();
        let source = Arc::new(DashboardSource::demo(&store));
        let app = Router::new()
            .route("/mutate", post(|| async { StatusCode::NO_CONTENT }))
            .layer(middleware::from_fn_with_state(source, mutation_guard));
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mutate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
