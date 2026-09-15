//! Cycle repository -- CRUD operations for the `cycles` table and `cycle_features`.
//!
//! Traces to: FR-C01, FR-C02, FR-C03, FR-C04, FR-C05, FR-C07

use rusqlite::{Connection, OptionalExtension, params};

use agileplus_domain::{
    domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures},
    error::DomainError,
};

use crate::repository::features::map_err;

mod mappers;
mod progress;

use mappers::{row_to_cycle, row_to_feature};
use progress::compute_wp_progress;

// ---------------------------------------------------------------------------
// Cycle CRUD
// ---------------------------------------------------------------------------

pub fn create_cycle(conn: &Connection, cycle: &Cycle) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO cycles (name, description, state, start_date, end_date, module_scope_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            cycle.name,
            cycle.description,
            cycle.state.to_string(),
            cycle.start_date.format("%Y-%m-%d").to_string(),
            cycle.end_date.format("%Y-%m-%d").to_string(),
            cycle.module_scope_id,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_cycle(conn: &Connection, id: i64) -> Result<Option<Cycle>, DomainError> {
    conn.query_row(
        "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                created_at, updated_at
         FROM cycles WHERE id = ?1",
        params![id],
        row_to_cycle,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_cycle_state(
    conn: &Connection,
    id: i64,
    state: CycleState,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE cycles SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state.to_string(), now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::CycleNotFound(id.to_string()));
    }
    Ok(())
}

pub fn list_cycles_by_state(
    conn: &Connection,
    state: CycleState,
) -> Result<Vec<Cycle>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                    created_at, updated_at
             FROM cycles WHERE state = ?1 ORDER BY start_date",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![state.to_string()], row_to_cycle)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_cycles_by_module(conn: &Connection, module_id: i64) -> Result<Vec<Cycle>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                    created_at, updated_at
             FROM cycles WHERE module_scope_id = ?1 ORDER BY start_date",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![module_id], row_to_cycle)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// List every cycle regardless of state.
pub fn list_all_cycles(conn: &Connection) -> Result<Vec<Cycle>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                    created_at, updated_at
             FROM cycles ORDER BY start_date",
        )
        .map_err(map_err)?;
    let rows = stmt.query_map([], row_to_cycle).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// Load a cycle with its assigned features and compute a `WpProgressSummary`.
pub fn get_cycle_with_features(
    conn: &Connection,
    id: i64,
) -> Result<Option<CycleWithFeatures>, DomainError> {
    let cycle = match get_cycle(conn, id)? {
        Some(c) => c,
        None => return Ok(None),
    };

    // Load assigned features.
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.slug, f.friendly_name, f.state, f.spec_hash, f.target_branch,
                    f.created_at, f.updated_at
             FROM features f
             INNER JOIN cycle_features cf ON f.id = cf.feature_id
             WHERE cf.cycle_id = ?1
             ORDER BY f.created_at",
        )
        .map_err(map_err)?;
    let features = stmt
        .query_map(params![id], row_to_feature)
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;

    // Compute WP progress summary for all features in this cycle.
    let wp_progress = compute_wp_progress(conn, id)?;

    Ok(Some(CycleWithFeatures {
        cycle,
        features,
        wp_progress,
    }))
}

// ---------------------------------------------------------------------------
// Cycle-feature join ops
// ---------------------------------------------------------------------------

/// Add a feature to a cycle. Enforces module_scope_id validation if set.
/// Idempotent (INSERT OR IGNORE).
pub fn add_feature_to_cycle(conn: &Connection, entry: &CycleFeature) -> Result<(), DomainError> {
    // Check if the cycle has a module scope restriction.
    let module_scope_id: Option<i64> = conn
        .query_row(
            "SELECT module_scope_id FROM cycles WHERE id = ?1",
            params![entry.cycle_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_err)?
        .ok_or_else(|| DomainError::CycleNotFound(entry.cycle_id.to_string()))?;

    if let Some(scope_module_id) = module_scope_id {
        // Feature must be owned by or tagged to the scope module.
        let in_scope: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (
                    SELECT id FROM features WHERE id = ?1 AND module_id = ?2
                    UNION
                    SELECT feature_id FROM module_feature_tags WHERE feature_id = ?1 AND module_id = ?2
                )",
                params![entry.feature_id, scope_module_id],
                |row| row.get(0),
            )
            .map_err(map_err)?;

        if in_scope == 0 {
            // Load slugs for a good error message.
            let feature_slug: String = conn
                .query_row(
                    "SELECT slug FROM features WHERE id = ?1",
                    params![entry.feature_id],
                    |row| row.get(0),
                )
                .map_err(map_err)?;
            let module_slug: String = conn
                .query_row(
                    "SELECT slug FROM modules WHERE id = ?1",
                    params![scope_module_id],
                    |row| row.get(0),
                )
                .map_err(map_err)?;
            return Err(DomainError::FeatureNotInModuleScope {
                feature_slug,
                module_slug,
            });
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO cycle_features (cycle_id, feature_id, added_at)
         VALUES (?1, ?2, ?3)",
        params![entry.cycle_id, entry.feature_id, now],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn remove_feature_from_cycle(
    conn: &Connection,
    cycle_id: i64,
    feature_id: i64,
) -> Result<(), DomainError> {
    conn.execute(
        "DELETE FROM cycle_features WHERE cycle_id = ?1 AND feature_id = ?2",
        params![cycle_id, feature_id],
    )
    .map_err(map_err)?;
    Ok(())
}


mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
    use chrono::NaiveDate;

    fn seed_feature(conn: &Connection, feature_id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
            params![feature_id, format!("feat-{feature_id}"), format!("Feature {feature_id}")],
        )
        .unwrap();
    }

    fn sample_cycle(name: &str) -> Cycle {
        Cycle {
            id: 0,
            name: name.to_string(),
            description: Some(format!("Desc for {name}")),
            state: CycleState::Draft,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 1, 14).unwrap(),
            module_scope_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn create_and_get_cycle() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_cycle(&conn, &sample_cycle("Sprint 1")).unwrap();
        assert!(id > 0);
        let fetched = get_cycle(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.name, "Sprint 1");
        assert_eq!(fetched.state, CycleState::Draft);
    }

    #[test]
    fn get_cycle_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_cycle(&conn, 99999).unwrap().is_none());
    }

    #[test]
    fn update_cycle_state_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_cycle(&conn, &sample_cycle("C1")).unwrap();
        update_cycle_state(&conn, id, CycleState::Active).unwrap();
        let fetched = get_cycle(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.state, CycleState::Active);
    }

    #[test]
    fn update_cycle_state_nonexistent_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let err = update_cycle_state(&conn, 99999, CycleState::Active).unwrap_err();
        assert!(format!("{err}").contains("99999"));
    }

    #[test]
    fn list_cycles_by_state_filters() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_cycle(&conn, &sample_cycle("C1")).unwrap();
        let mut active = sample_cycle("C2");
        active.state = CycleState::Active;
        create_cycle(&conn, &active).unwrap();
        assert_eq!(list_cycles_by_state(&conn, CycleState::Draft).unwrap().len(), 1);
        assert_eq!(list_cycles_by_state(&conn, CycleState::Active).unwrap().len(), 1);
    }

    #[test]
    fn list_cycles_by_module_filters() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut c1 = sample_cycle("Mod1");
        c1.module_scope_id = Some(1);
        create_cycle(&conn, &c1).unwrap();
        let mut c2 = sample_cycle("Mod2");
        c2.module_scope_id = Some(2);
        create_cycle(&conn, &c2).unwrap();
        assert_eq!(list_cycles_by_module(&conn, 1).unwrap().len(), 1);
        assert_eq!(list_cycles_by_module(&conn, 2).unwrap().len(), 1);
    }

    #[test]
    fn list_all_cycles_returns_all() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_cycle(&conn, &sample_cycle("C1")).unwrap();
        create_cycle(&conn, &sample_cycle("C2")).unwrap();
        assert_eq!(list_all_cycles(&conn).unwrap().len(), 2);
    }

    #[test]
    fn add_and_remove_feature_from_cycle() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 42);
        let cycle_id = create_cycle(&conn, &sample_cycle("C1")).unwrap();
        let entry = CycleFeature::new(cycle_id, 42);
        add_feature_to_cycle(&conn, &entry).unwrap();
        let cwf = get_cycle_with_features(&conn, cycle_id).unwrap().unwrap();
        assert_eq!(cwf.features.len(), 1);
        remove_feature_from_cycle(&conn, cycle_id, 42).unwrap();
        let cwf = get_cycle_with_features(&conn, cycle_id).unwrap().unwrap();
        assert!(cwf.features.is_empty());
    }

    #[test]
    fn add_feature_to_nonexistent_cycle_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let entry = CycleFeature::new(99999, 1);
        assert!(add_feature_to_cycle(&conn, &entry).is_err());
    }

    #[test]
    fn get_cycle_with_features_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_cycle_with_features(&conn, 99999).unwrap().is_none());
    }
}
