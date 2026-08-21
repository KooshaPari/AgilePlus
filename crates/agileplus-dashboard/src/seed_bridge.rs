//! Seed bridge — builds a populated `DashboardStore` from `~/.agileplus/agileplus.db`.
//!
//! This is the canonical construction path for the dashboard daemon.  On any
//! failure it falls back to `DashboardStore::default()` (empty store) so the
//! daemon never refuses to boot.
//!
//! Traceability: WP12 (T071–T077).

use std::collections::HashMap;

use crate::app_state::{DashboardStore, FeatureState, ServiceHealth, WpState};

/// Build the dashboard store by hydrating from the canonical SQLite DB.
/// On any failure returns `DashboardStore::default()` (no recursion).
pub fn build_dashboard_store() -> DashboardStore {
    let path = resolve_db_path();
    match hydrate_from_db(&path) {
        Ok(store) => {
            let wp_total: usize = store.work_packages.values().map(|v| v.len()).sum();
            tracing::info!(
                features = store.features.len(),
                work_packages = wp_total,
                db_path = %path.display(),
                "build_dashboard_store: hydrated from canonical DB"
            );
            store
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                db_path = %path.display(),
                "build_dashboard_store: DB load failed; using empty default"
            );
            DashboardStore::default()
        }
    }
}

fn resolve_db_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("AGILEPLUS_DB") {
        return std::path::PathBuf::from(p);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(home)
            .join(".agileplus")
            .join("agileplus.db");
    }
    std::path::PathBuf::from("./agileplus.db")
}

fn hydrate_from_db(path: &std::path::Path) -> Result<DashboardStore, String> {
    use rusqlite::{Connection, OpenFlags};

    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("open({}): {}", path.display(), e))?;

    let features = load_features(&conn)?;
    let work_packages = load_work_packages(&conn)?;
    let projects = load_projects(&conn)?;

    Ok(DashboardStore {
        features,
        work_packages,
        modules: Vec::new(),
        cycles: Vec::new(),
        cycle_features: HashMap::new(),
        health: default_health(),
        projects,
        active_project_id: None,
        governance_client: None,
        plane_client: None,
        plane_daemon: None,
    })
}

fn load_features(conn: &rusqlite::Connection) -> Result<Vec<crate::app_state::Feature>, String> {
    use crate::app_state::Feature;
    use chrono::{DateTime, TimeZone, Utc};

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, target_branch, created_at, updated_at, labels \
             FROM features ORDER BY id",
        )
        .map_err(|e| format!("features.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(|e| format!("features.query_map: {e}"))?;
    let mut out = Vec::new();
    for row in rows {
        let (id, slug, friendly_name, state_str, target_branch, created_at, updated_at, labels_json) =
            row.map_err(|e| format!("features.row: {e}"))?;
        let tb = target_branch.as_deref().unwrap_or("main");
        let mut f = Feature::new(&slug, &friendly_name, [0u8; 32], Some(tb));
        f.id = id;
        f.state = parse_feature_state(&state_str);
        f.target_branch = tb.to_string();
        f.labels = serde_json::from_str::<Vec<String>>(&labels_json).unwrap_or_default();
        if let Ok(dt) = DateTime::parse_from_rfc3339(&created_at) {
            f.created_at = dt.with_timezone(&Utc);
        }
        if let Ok(dt) = DateTime::parse_from_rfc3339(&updated_at) {
            f.updated_at = dt.with_timezone(&Utc);
        }
        out.push(f);
    }
    Ok(out)
}

fn load_work_packages(
    conn: &rusqlite::Connection,
) -> Result<HashMap<i64, Vec<crate::app_state::WorkPackage>>, String> {
    use crate::app_state::WorkPackage;
    use chrono::{DateTime, TimeZone, Utc};

    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, title, state, file_scope, created_at, updated_at \
             FROM work_packages ORDER BY feature_id, id",
        )
        .map_err(|e| format!("wps.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| format!("wps.query_map: {e}"))?;
    let mut out: HashMap<i64, Vec<WorkPackage>> = HashMap::new();
    for row in rows {
        let (id, feature_id, title, state_str, file_scope, created_at, updated_at) =
            row.map_err(|e| format!("wps.row: {e}"))?;
        let mut wp = WorkPackage::new(feature_id, &title, 0, "");
        wp.id = id;
        wp.state = parse_wp_state(&state_str);
        wp.file_scope = serde_json::from_str::<Vec<String>>(&file_scope)
            .unwrap_or_default();
        if let Ok(dt) = DateTime::parse_from_rfc3339(&created_at) {
            wp.created_at = dt.with_timezone(&Utc);
        }
        if let Ok(dt) = DateTime::parse_from_rfc3339(&updated_at) {
            wp.updated_at = dt.with_timezone(&Utc);
        }
        out.entry(feature_id).or_default().push(wp);
    }
    Ok(out)
}

fn load_projects(conn: &rusqlite::Connection) -> Result<Vec<crate::app_state::Project>, String> {
    use crate::app_state::Project;

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, name, description, created_at, updated_at FROM projects ORDER BY id",
        )
        .map_err(|e| format!("projects.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| format!("projects.query_map: {e}"))?;
    let mut out = Vec::new();
    for row in rows {
        let (id, slug, name, description, _created, _updated) =
            row.map_err(|e| format!("projects.row: {e}"))?;
        let mut p = Project::new(&name, &slug)
            .map_err(|e| format!("project.new({id}): {e}"))?;
        p.id = id;
        if let Some(d) = description {
            p.description = Some(d);
        }
        out.push(p);
    }
    Ok(out)
}

fn default_health() -> Vec<ServiceHealth> {
    let now = chrono::Utc::now();
    vec![
        ServiceHealth { name: "SQLite".into(), healthy: true, degraded: false, latency_ms: Some(0), last_check: now },
        ServiceHealth { name: "NATS".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
        ServiceHealth { name: "Dragonfly".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
        ServiceHealth { name: "Neo4j".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
        ServiceHealth { name: "MinIO".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
        ServiceHealth { name: "AgilePlus API".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
        ServiceHealth { name: "Plane API".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
        ServiceHealth { name: "Plane Web".into(), healthy: false, degraded: false, latency_ms: None, last_check: now },
    ]
}

fn parse_feature_state(s: &str) -> FeatureState {
    use FeatureState::*;
    match s.to_ascii_lowercase().as_str() {
        "created" => Created,
        "specified" => Specified,
        "researched" => Researched,
        "planned" => Planned,
        "implementing" => Implementing,
        "validated" => Validated,
        "shipped" => Shipped,
        "retrospected" => Retrospected,
        _ => Created,
    }
}

fn parse_wp_state(s: &str) -> WpState {
    use WpState::*;
    match s.to_ascii_lowercase().as_str() {
        "planned" => Planned,
        "doing" | "in_progress" => Doing,
        "review" => Review,
        "done" | "completed" => Done,
        "blocked" => Blocked,
        _ => Planned,
    }
}
