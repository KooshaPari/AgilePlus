//! Build a populated `DashboardStore` for the agileplus-dashboard binary.
//!
// Two construction paths:
//! - **`build_dashboard_store()`** (default): tries the populated SQLite DB at
//!   `$AGILEPLUS_DB` or `~/.agileplus/agileplus.db`; falls back to the
//!   kitty-specs fixture if the DB is missing or unreadable.
//! - **`seeded()`** (DashboardStore::seeded, in app_state.rs): called via
//!   `populate_admin_features()` in the DashboardStore::default impl.
//!
//! Traceability: WP12 (T071-T077).

use std::collections::HashMap;
use std::path::PathBuf;

use crate::app_state::DashboardStore;
use agileplus_domain::cycle::entity::Cycle;
use agileplus_domain::cycle::state::CycleState;
use agileplus_domain::domain::cycle::state as cycle_state;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{PrState, WorkPackage, WpState};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OpenFlags};

/// Resolve the canonical DB path: `$AGILEPLUS_DB` env var → `~/.agileplus/agileplus.db`.
pub fn resolve_db_path() -> PathBuf {
    if let Ok(p) = std::env::var("AGILEPLUS_DB") {
        return PathBuf::from(p);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".agileplus").join("agileplus.db");
    }
    PathBuf::from("./agileplus.db")
}

/// Try the populated DB first; fall back to fixture data on any error.
pub fn build_dashboard_store() -> DashboardStore {
    match load_from_canonical_db() {
        Ok(store) => {
            tracing::info!(
                features = store.features.len(),
                work_packages = store.work_packages.values().map(|v| v.len()).sum::<usize>(),
                "build_dashboard_store: populated from ~/.agileplus/agileplus.db",
            );
            store
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "build_dashboard_store: DB load failed; using fixture data",
            );
            crate::seed::seed_dashboard_store()
        }
    }
}

fn empty_dashboard_store() -> DashboardStore {
    DashboardStore::default()
}

fn load_from_canonical_db() -> Result<DashboardStore, String> {
    let path = resolve_db_path();
    let conn = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("open({}): {}", path.display(), e))?;

    let mut store = empty_dashboard_store();
    load_features(&conn, &mut store)?;
    load_work_packages(&conn, &mut store)?;
    load_projects(&conn, &mut store)?;
    load_modules(&conn, &mut store)?;
    load_cycles(&conn, &mut store)?;
    load_cycle_features(&conn, &mut store)?;
    Ok(store)
}

fn load_features(conn: &Connection, store: &mut DashboardStore) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch,              created_at, updated_at, module_id, labels FROM features ORDER BY id",
        )
        .map_err(|e| format!("features.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, String>(9)?,
            ))
        })
        .map_err(|e| format!("features.query_map: {e}"))?;
    for row in rows {
        let (id, slug, friendly_name, state_str, spec_hash, target_branch,
             created_at, updated_at, module_id, labels_json) =
            row.map_err(|e| format!("features.row: {e}"))?;
        let state = parse_feature_state(&state_str);
        let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
        let mut f = Feature::new(
            &slug,
            &friendly_name,
            pad_spec_hash(&spec_hash),
            Some(target_branch.as_deref().unwrap_or("main")),
        );
        f.id = id;
        f.state = state;
        f.target_branch = target_branch.unwrap_or_else(|| "main".to_string());
        f.created_at = parse_iso_datetime(&created_at).unwrap_or_else(Utc::now);
        f.updated_at = parse_iso_datetime(&updated_at).unwrap_or_else(Utc::now);
        f.module_id = module_id;
        f.labels = labels;
        store.features.push(f);
    }
    Ok(())
}

fn load_work_packages(conn: &Connection, store: &mut DashboardStore) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, title, state, sequence, file_scope,              acceptance_criteria, agent_id, pr_url, pr_state,              worktree_path, created_at, updated_at FROM work_packages ORDER BY feature_id, id",
        )
        .map_err(|e| format!("wps.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, String>(12)?,
            ))
        })
        .map_err(|e| format!("wps.query_map: {e}"))?;
    for row in rows {
        let (id, feature_id, title, state_str, sequence, file_scope,
             acceptance_criteria, agent_id, pr_url, _pr_state,
             worktree_path, created_at, updated_at) =
            row.map_err(|e| format!("wps.row: {e}"))?;
        let state = parse_wp_state(&state_str);
        let mut wp = WorkPackage::new(feature_id, &title, sequence as i32, &acceptance_criteria);
        wp.id = id;
        wp.state = state;
        wp.file_scope = serde_json::from_str::<Vec<String>>(&file_scope)
            .unwrap_or_default();
        wp.agent_id = agent_id;
        wp.pr_url = pr_url;
        wp.worktree_path = worktree_path;
        wp.created_at = parse_iso_datetime(&created_at).unwrap_or_else(Utc::now);
        wp.updated_at = parse_iso_datetime(&updated_at).unwrap_or_else(Utc::now);
        store.work_packages.entry(feature_id).or_default().push(wp);
    }
    Ok(())
}

fn load_projects(conn: &Connection, store: &mut DashboardStore) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, slug, name, description, created_at, updated_at FROM projects ORDER BY id")
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
    for row in rows {
        let (id, slug, name, description, created_at, updated_at) =
            row.map_err(|e| format!("projects.row: {e}"))?;
        let mut p = match Project::new(name, slug) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(id = id, name = %name, error = ?e, "project.new failed, skipping");
                continue;
            }
        };
        p.id = id;
        if let Some(d) = description {
            p.description = Some(d);
        }
        if let Ok(ts) = parse_iso_datetime(&created_at) {
            p.created_at = ts;
        }
        if let Ok(ts) = parse_iso_datetime(&updated_at) {
            p.updated_at = ts;
        }
        store.projects.push(p);
    }
    Ok(())
}

fn load_modules(conn: &Connection, store: &mut DashboardStore) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, slug, friendly_name, parent_module_id, owner FROM modules ORDER BY id")
        .map_err(|e| format!("modules.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|e| format!("modules.query_map: {e}"))?;
    for row in rows {
        let (id, slug, friendly_name, _parent_module_id, _owner) =
            row.map_err(|e| format!("modules.row: {e}"))?;
        let mut m = crate::app_state::Module::new(&friendly_name, parent_module_id);
        m.id = id;
        store.modules.push(m);
    }
    Ok(())
}

fn load_cycles(conn: &Connection, store: &mut DashboardStore) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, state, start_date, end_date, module_scope_id              FROM cycles ORDER BY id",
        )
        .map_err(|e| format!("cycles.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<i64>>(5)?,
            ))
        })
        .map_err(|e| format!("cycles.query_map: {e}"))?;
    for row in rows {
        let (id, name, state_str, start_date, end_date, module_scope_id) =
            row.map_err(|e| format!("cycles.row: {e}"))?;
        let state = parse_cycle_state(&state_str);
        let start = parse_optional_date(start_date.as_deref())
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let end = parse_optional_date(end_date.as_deref())
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2024, 1, 8).unwrap());
        let mut c = match Cycle::new(name, start, end, module_scope_id) {
            Ok(c) => c,
            Err(_) => {
                // Invalid date range; try with end = start + 1
                Cycle::new(name, start, start + chrono::Duration::days(1), module_scope_id)
                    .unwrap_or_else(|_| {
                        // Last resort: build manually via empty struct
                        // Fall back to the empty case — log and skip
                        tracing::warn!(name = %name, "cycle has invalid date range, skipping");
                        continue;
                    })
            }
        };
        c.id = id;
        c.module_scope_id = module_scope_id;
        store.cycles.push(c);
    }
    Ok(())
}

fn load_cycle_features(conn: &Connection, store: &mut DashboardStore) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT cycle_id, feature_id FROM cycle_features ORDER BY cycle_id, feature_id")
        .map_err(|e| format!("cycle_features.prepare: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| format!("cycle_features.query_map: {e}"))?;
    for row in rows {
        let (cid, fid) = row.map_err(|e| format!("cycle_features.row: {e}"))?;
        store.cycle_features.entry(cid).or_default().push(fid);
    }
    Ok(())
}

fn parse_iso_datetime(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc());
    }
    None
}

fn parse_optional_date(s: Option<&str>) -> Option<NaiveDate> {
    let s = s?;
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.date_naive());
    }
    None
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

fn parse_cycle_state(s: &str) -> CycleState {
    use cycle_state::CycleState::*;
    match s.to_ascii_lowercase().as_str() {
        "draft" | "planned" => Draft,
        "active" => Active,
        "review" => Review,
        "shipped" | "completed" => Shipped,
        "archived" => Archived,
        _ => Draft,
    }
}

fn pad_spec_hash(b: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let take = b.len().min(32);
    let offset = 32 - take;
    out[offset..].copy_from_slice(&b[..take]);
    out
}
