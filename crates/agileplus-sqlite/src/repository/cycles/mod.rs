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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use chrono::NaiveDate;

    fn make_cycle(name: &str) -> Cycle {
        Cycle::new(
            name,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 14).unwrap(),
            None,
        )
        .unwrap()
    }

    fn seed_feature(conn: &Connection, id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)              VALUES (?1, ?2, ?3, 'created', X'00', 'main', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            rusqlite::params![id, format!("feat-{id}"), format!("Feature {id}")],
        )
        .unwrap();
    }

    #[test]
    fn test_create_and_get_cycle() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let c = make_cycle("Sprint 1");
        let id = create_cycle(&conn, &c).unwrap();
        assert!(id > 0);
        let got = get_cycle(&conn, id).unwrap().unwrap();
        assert_eq!(got.name, "Sprint 1");
        assert_eq!(got.state, CycleState::Draft);
    }

    #[test]
    fn test_get_cycle_nonexistent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_cycle(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn test_update_cycle_state() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_cycle(&conn, &make_cycle("S2")).unwrap();
        update_cycle_state(&conn, id, CycleState::Active).unwrap();
        let got = get_cycle(&conn, id).unwrap().unwrap();
        assert_eq!(got.state, CycleState::Active);
    }

    #[test]
    fn test_list_all_cycles() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_cycle(&conn, &make_cycle("C1")).unwrap();
        create_cycle(&conn, &make_cycle("C2")).unwrap();
        let all = list_all_cycles(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_by_state() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_cycle(&conn, &make_cycle("Draft")).unwrap();
        update_cycle_state(&conn, id, CycleState::Active).unwrap();
        create_cycle(&conn, &make_cycle("StillDraft")).unwrap();
        let active = list_cycles_by_state(&conn, CycleState::Active).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Draft");
    }

    #[test]
    fn test_add_and_remove_feature() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 10);
        let cycle_id = create_cycle(&conn, &make_cycle("S")).unwrap();
        let entry = CycleFeature::new(cycle_id, 10);
        add_feature_to_cycle(&conn, &entry).unwrap();
        let cwf = get_cycle_with_features(&conn, cycle_id).unwrap().unwrap();
        assert_eq!(cwf.features.len(), 1);
        remove_feature_from_cycle(&conn, cycle_id, 10).unwrap();
        let cwf2 = get_cycle_with_features(&conn, cycle_id).unwrap().unwrap();
        assert!(cwf2.features.is_empty());
    }

    #[test]
    fn test_cycle_not_found_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(update_cycle_state(&conn, 999, CycleState::Active).is_err());
    }

    #[test]
    fn wp_progress_buckets_doing_review_and_blocked() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let cycle_id = create_cycle(&conn, &make_cycle("Buckets")).unwrap();
        let feature_id = 42;
        seed_feature(&conn, feature_id);
        add_feature_to_cycle(&conn, &CycleFeature::new(cycle_id, feature_id)).unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        for (sequence, state) in [(1, "doing"), (2, "review"), (3, "blocked"), (4, "planned")] {
            conn.execute(
                "INSERT INTO work_packages
                 (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, '[]', '', ?5, ?5)",
                rusqlite::params![feature_id, format!("WP {state}"), state, sequence, now],
            )
            .unwrap();
        }

        let view = get_cycle_with_features(&conn, cycle_id)
            .unwrap()
            .expect("cycle view");
        let progress = view.wp_progress;
        assert_eq!(progress.total, 4);
        assert_eq!(
            progress.in_progress, 2,
            "doing and review both count as in progress"
        );
        assert_eq!(progress.blocked, 1);
        assert_eq!(progress.planned, 1);
        assert_eq!(progress.done, 0);
    }

    #[test]
    fn wp_progress_ignores_work_packages_of_other_cycles() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let in_scope = create_cycle(&conn, &make_cycle("InScope")).unwrap();
        let out_of_scope = create_cycle(&conn, &make_cycle("OutOfScope")).unwrap();
        let scoped_feature = 7;
        let other_feature = 8;
        seed_feature(&conn, scoped_feature);
        seed_feature(&conn, other_feature);
        add_feature_to_cycle(&conn, &CycleFeature::new(in_scope, scoped_feature)).unwrap();
        add_feature_to_cycle(&conn, &CycleFeature::new(out_of_scope, other_feature)).unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        for (feature_id, sequence) in [(scoped_feature, 1), (other_feature, 2)] {
            conn.execute(
                "INSERT INTO work_packages
                 (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
                 VALUES (?1, 'WP', 'done', ?2, '[]', '', ?3, ?3)",
                rusqlite::params![feature_id, sequence, now],
            )
            .unwrap();
        }

        let view = get_cycle_with_features(&conn, in_scope)
            .unwrap()
            .expect("cycle view");
        assert_eq!(view.wp_progress.total, 1);
        assert_eq!(view.wp_progress.done, 1);
    }

    /// Insert a cycle row verbatim so a column can hold a value the repository
    /// writer would never produce (only the date ordering is constrained).
    fn insert_raw_cycle(
        conn: &Connection,
        name: &str,
        state: &str,
        start_date: &str,
        end_date: &str,
        created_at: &str,
        updated_at: &str,
    ) -> i64 {
        conn.execute(
            "INSERT INTO cycles
             (name, description, state, start_date, end_date, module_scope_id, created_at, updated_at)
             VALUES (?1, '', ?2, ?3, ?4, NULL, ?5, ?6)",
            rusqlite::params![name, state, start_date, end_date, created_at, updated_at],
        )
        .expect("raw cycle insert");
        conn.last_insert_rowid()
    }

    fn raw_cycle_err(conn: &Connection, id: i64) -> DomainError {
        let err = get_cycle(conn, id).unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
        err
    }

    #[test]
    fn cycle_with_unknown_state_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let id = insert_raw_cycle(
            &conn,
            "raw-state",
            "Sideways",
            "2025-01-01",
            "2025-01-14",
            &now,
            &now,
        );

        let err = raw_cycle_err(&conn, id);
        assert!(
            err.to_string().contains("Sideways"),
            "the unparsable state must surface in the error: {err}"
        );
    }

    #[test]
    fn cycle_with_unparseable_start_date_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        // "0001-00-00" is not a valid date (month 00), so reading it must error,
        // yet it sorts before "2025-01-14" bytewise, keeping the
        // CHECK (end_date > start_date) satisfied so the raw insert is allowed.
        let id = insert_raw_cycle(
            &conn,
            "raw-start",
            "Draft",
            "0001-00-00",
            "2025-01-14",
            &now,
            &now,
        );

        raw_cycle_err(&conn, id);
    }

    #[test]
    fn cycle_with_unparseable_end_date_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let id = insert_raw_cycle(
            &conn,
            "raw-end",
            "Draft",
            "2025-01-01",
            "not-a-date",
            &now,
            &now,
        );

        raw_cycle_err(&conn, id);
    }

    #[test]
    fn cycle_with_corrupt_created_at_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let id = insert_raw_cycle(
            &conn,
            "raw-created",
            "Draft",
            "2025-01-01",
            "2025-01-14",
            "not-a-timestamp",
            &now,
        );

        raw_cycle_err(&conn, id);
    }

    #[test]
    fn cycle_with_corrupt_updated_at_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let id = insert_raw_cycle(
            &conn,
            "raw-updated",
            "Draft",
            "2025-01-01",
            "2025-01-14",
            &now,
            "not-a-timestamp",
        );

        raw_cycle_err(&conn, id);
    }

    /// Assign a feature to a fresh cycle, then corrupt a feature column and read
    /// the cycle view back, which maps features through `row_to_feature`.
    fn corrupt_cycle_feature(conn: &Connection, label: &str, sql: &str) -> DomainError {
        let feature_id = label.len() as i64 + 1000;
        seed_feature(conn, feature_id);
        let cycle_id = create_cycle(conn, &make_cycle(label)).unwrap();
        add_feature_to_cycle(conn, &CycleFeature::new(cycle_id, feature_id)).unwrap();
        conn.execute(
            &sql.replace("?feature", &feature_id.to_string()),
            rusqlite::params![],
        )
        .expect("corrupt feature");

        let err = get_cycle_with_features(conn, cycle_id).unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
        err
    }

    #[test]
    fn cycle_feature_with_corrupt_state_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        conn.execute_batch("PRAGMA ignore_check_constraints=ON;")
            .unwrap();

        let err = corrupt_cycle_feature(
            &conn,
            "FeatureState",
            "UPDATE features SET state = 'bogus' WHERE id = ?feature",
        );
        assert!(
            err.to_string().contains("bogus"),
            "the unparsable feature state must surface: {err}"
        );
    }

    #[test]
    fn cycle_feature_with_corrupt_created_at_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();

        corrupt_cycle_feature(
            &conn,
            "FeatureCreatedAt",
            "UPDATE features SET created_at = 'not-a-timestamp' WHERE id = ?feature",
        );
    }

    #[test]
    fn cycle_feature_with_corrupt_updated_at_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();

        corrupt_cycle_feature(
            &conn,
            "FeatureUpdatedAt",
            "UPDATE features SET updated_at = 'not-a-timestamp' WHERE id = ?feature",
        );
    }

    #[test]
    fn add_feature_to_unknown_cycle_is_cycle_not_found() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 5);

        let err = add_feature_to_cycle(&conn, &CycleFeature::new(4242, 5)).unwrap_err();
        assert!(
            matches!(err, DomainError::CycleNotFound(ref id) if id == "4242"),
            "got {err:?}"
        );
    }
}
