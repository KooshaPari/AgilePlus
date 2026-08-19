//! Live-data wiring: load the dashboard store from the on-box governance
//! database (wsm3d.db) with a seed fallback, plus shared DB path resolution.
//!
//! The dashboard historically rendered from `DashboardStore::seeded()`
//! (in-memory dogfood fixtures). This module makes the store read real
//! governance data (features, work packages, projects) straight out of the
//! AgilePlus SQLite store. If the DB is missing/unreadable/empty the caller
//! falls back to the seed store, so the dashboard always renders.
//!
//! Reads are READ-ONLY: we never run migrations or write to the store here.
//! (Full StoragePort integration via SqliteStorageAdapter is a follow-up;
//! that path runs migrations on open and is intentionally NOT used by the
//! dashboard until the migration set against wsm3d.db is validated.)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use agileplus_domain::domain::{
    feature::Feature,
    project::Project,
    state_machine::FeatureState,
    work_package::{PrState, WorkPackage, WpState},
};
use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::app_state::DashboardStore;
use crate::seed_bridge::build_dashboard_store;

/// Resolve the on-box governance DB path.
///
/// Precedence: `AGILEPLUS_DB_PATH` env > `~/.agileplus/wsm3d.db` >
/// `./wsm3d.db` > `./agileplus.db`. Returns `None` only when no candidate
/// exists on disk (callers then fall back to the seed store).
pub fn resolve_db_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AGILEPLUS_DB_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let candidate = PathBuf::from(home).join(".agileplus").join("wsm3d.db");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let candidate = PathBuf::from(profile).join(".agileplus").join("wsm3d.db");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    for name in ["wsm3d.db", "agileplus.db"] {
        let candidate = PathBuf::from(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn parse_ts(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn map_wp_state(raw: &str) -> WpState {
    match raw {
        "doing" => WpState::Doing,
        "review" => WpState::Review,
        "done" => WpState::Done,
        "blocked" => WpState::Blocked,
        _ => WpState::Planned,
    }
}

fn map_pr_state(raw: Option<String>) -> Option<PrState> {
    match raw.as_deref() {
        Some("open") => Some(PrState::Open),
        Some("review") => Some(PrState::Review),
        Some("changes_requested") => Some(PrState::ChangesRequested),
        Some("approved") => Some(PrState::Approved),
        Some("merged") => Some(PrState::Merged),
        _ => None,
    }
}

fn load_features(conn: &Connection) -> Vec<Feature> {
    let mut out = Vec::new();
    let sql = "SELECT id, slug, friendly_name, state, spec_hash, target_branch, \
               created_at, updated_at, module_id FROM features ORDER BY id";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let spec_blob: Vec<u8> = row.get(4).unwrap_or_default();
            let mut spec_hash = [0_u8; 32];
            if spec_blob.len() == 32 {
                spec_hash.copy_from_slice(&spec_blob);
            }
            let state: String = row.get(3)?;
            Ok(Feature {
                id: row.get(0)?,
                slug: row.get(1)?,
                friendly_name: row.get(2)?,
                state: state
                    .parse::<FeatureState>()
                    .unwrap_or(FeatureState::Created),
                spec_hash,
                target_branch: row.get(5)?,
                plane_issue_id: None,
                plane_state_id: None,
                labels: Vec::new(),
                module_id: row.get(8).unwrap_or(None),
                project_id: None,
                created_at_commit: None,
                last_modified_commit: None,
                created_at: parse_ts(&row.get::<_, String>(6).unwrap_or_default()),
                updated_at: parse_ts(&row.get::<_, String>(7).unwrap_or_default()),
            })
        }) {
            out = rows.filter_map(Result::ok).collect();
        }
    }
    out
}

fn load_work_packages(conn: &Connection) -> HashMap<i64, Vec<WorkPackage>> {
    let mut out: HashMap<i64, Vec<WorkPackage>> = HashMap::new();
    let sql = "SELECT id, feature_id, title, state, sequence, file_scope, acceptance_criteria, \
               agent_id, pr_url, pr_state, worktree_path, created_at, updated_at \
               FROM work_packages ORDER BY sequence, id";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let file_scope_raw: String = row.get(5).unwrap_or_default();
            let file_scope: Vec<String> = serde_json::from_str(&file_scope_raw).unwrap_or_default();
            Ok(WorkPackage {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                title: row.get(2)?,
                state: map_wp_state(&row.get::<_, String>(3).unwrap_or_default()),
                sequence: row.get(4).unwrap_or(0),
                file_scope,
                acceptance_criteria: row.get(6).unwrap_or_default(),
                agent_id: row.get(7)?,
                pr_url: row.get(8)?,
                pr_state: map_pr_state(row.get(9)?),
                worktree_path: row.get(10)?,
                plane_sub_issue_id: None,
                base_commit: None,
                head_commit: None,
                created_at: parse_ts(&row.get::<_, String>(11).unwrap_or_default()),
                updated_at: parse_ts(&row.get::<_, String>(12).unwrap_or_default()),
            })
        }) {
            for wp in rows.filter_map(Result::ok) {
                out.entry(wp.feature_id).or_default().push(wp);
            }
        }
    }
    out
}

fn load_projects(conn: &Connection) -> Vec<Project> {
    let mut out = Vec::new();
    let sql =
        "SELECT id, slug, name, description, created_at, updated_at FROM projects ORDER BY id";
    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3).unwrap_or(None),
                created_at: parse_ts(&row.get::<_, String>(4).unwrap_or_default()),
                updated_at: parse_ts(&row.get::<_, String>(5).unwrap_or_default()),
            })
        }) {
            out = rows.filter_map(Result::ok).collect();
        }
    }
    out
}

/// Build a [`DashboardStore`] backed by the live DB.
///
/// Returns `None` when the DB is missing, unreadable, or has no features, so
/// callers fall back to [`DashboardStore::seeded`]. The seed store is used as
/// the base (health/modules/cycles surfaces) and the DB overrides the
/// governance surfaces (features, work packages, projects).
pub fn load_live_store() -> Option<DashboardStore> {
    let path: PathBuf = resolve_db_path()?;
    load_store_from_path(&path)
}

/// Load a store from an explicit DB path (used by tests).
pub fn load_store_from_path(path: &Path) -> Option<DashboardStore> {
    if !path.exists() {
        return None;
    }
    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;

    let features = load_features(&conn);
    if features.is_empty() {
        return None;
    }
    let work_packages = load_work_packages(&conn);
    let projects = load_projects(&conn);

    let mut store = build_dashboard_store();
    store.features = features;
    store.work_packages = work_packages;
    if !projects.is_empty() {
        store.projects = projects;
        // The live DB features carry no project_id (schema has no column),
        // so a project filter would hide every feature from the board.
        // Show all features; the project switcher still lists projects.
        store.active_project_id = None;
    }
    Some(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temp DB with the same shape as wsm3d.db (features,
    /// work_packages, projects) and seed a few rows.
    fn make_db(path: &Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE features (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT UNIQUE NOT NULL,
                friendly_name TEXT NOT NULL,
                state TEXT NOT NULL,
                spec_hash BLOB NOT NULL,
                target_branch TEXT NOT NULL DEFAULT 'main',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                module_id INTEGER
            );
            CREATE TABLE work_packages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feature_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                state TEXT NOT NULL,
                sequence INTEGER NOT NULL DEFAULT 0,
                file_scope TEXT NOT NULL DEFAULT '[]',
                acceptance_criteria TEXT NOT NULL DEFAULT '',
                agent_id TEXT,
                pr_url TEXT,
                pr_state TEXT,
                worktree_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                slug TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .unwrap();
        let ts = "2026-06-11T08:01:37.753224700+00:00";
        conn.execute(
            "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
             VALUES ('f1','Feature One','planned',x'00','main',?1,?1)",
            [ts],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
             VALUES ('f2','Feature Two','implementing',x'00','main',?1,?1)",
            [ts],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO work_packages (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at) \
             VALUES (1,'WP One','doing',1,'[\"src/a.rs\"]','accept a',?1,?1)",
            [ts],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO work_packages (feature_id, title, state, sequence, file_scope, created_at, updated_at) \
             VALUES (2,'WP Two','done',1,'[]',?1,?1)",
            [ts],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO projects (slug, name, description, created_at, updated_at) VALUES ('p1','Proj One','',?1,?1)",
            [ts],
        )
        .unwrap();
    }

    #[test]
    fn load_store_from_db_populates_governance_surfaces() {
        let dir = std::env::temp_dir().join(format!("agileplus-dash-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");
        make_db(&path);

        let store = load_store_from_path(&path).expect("store should load");
        assert_eq!(store.features.len(), 2, "features loaded from db");
        assert_eq!(store.features[0].slug, "f1");
        assert_eq!(store.features[1].state, FeatureState::Implementing);
        let wps = store.work_packages.get(&1).expect("wp group for feature 1");
        assert_eq!(wps.len(), 1);
        assert_eq!(wps[0].state, WpState::Doing);
        assert_eq!(wps[0].file_scope, vec!["src/a.rs".to_string()]);
        assert_eq!(store.projects.len(), 1);
        assert_eq!(store.projects[0].slug, "p1");
        // DB features are not project-scoped: no active-project filter.
        assert_eq!(store.active_project_id, None);
        assert_eq!(store.features_for_active_project().len(), 2);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_store_missing_or_empty_returns_none() {
        let dir = std::env::temp_dir().join(format!("agileplus-dash-empty-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.db");
        assert!(load_store_from_path(&path).is_none(), "missing db -> None");

        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE features (id INTEGER PRIMARY KEY, slug TEXT, friendly_name TEXT, \
             state TEXT, spec_hash BLOB, target_branch TEXT, created_at TEXT, updated_at TEXT, module_id INTEGER);",
        )
        .unwrap();
        drop(conn);
        assert!(
            load_store_from_path(&path).is_none(),
            "empty features -> None"
        );

        fs::remove_dir_all(&dir).ok();
    }
}
