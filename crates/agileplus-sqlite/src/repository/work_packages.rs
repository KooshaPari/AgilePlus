//! Work package repository — CRUD operations for `work_packages` and `wp_dependencies`.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::work_package::{DependencyType, PrState, WorkPackage, WpDependency, WpState},
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn wp_state_str(s: WpState) -> &'static str {
    match s {
        WpState::Planned => "planned",
        WpState::Doing => "doing",
        WpState::Review => "review",
        WpState::Done => "done",
        WpState::Blocked => "blocked",
    }
}

fn wp_state_from_str(s: &str) -> Result<WpState, DomainError> {
    match s {
        "planned" => Ok(WpState::Planned),
        "doing" => Ok(WpState::Doing),
        "review" => Ok(WpState::Review),
        "done" => Ok(WpState::Done),
        "blocked" => Ok(WpState::Blocked),
        _ => Err(DomainError::Storage(format!("invalid wp state: {s}"))),
    }
}

fn pr_state_str(s: PrState) -> &'static str {
    match s {
        PrState::Open => "open",
        PrState::Review => "review",
        PrState::ChangesRequested => "changes_requested",
        PrState::Approved => "approved",
        PrState::Merged => "merged",
    }
}

fn dep_type_str(d: DependencyType) -> &'static str {
    match d {
        DependencyType::Explicit => "explicit",
        DependencyType::FileOverlap => "file_overlap",
        DependencyType::Data => "data",
    }
}

fn dep_type_from_str(s: &str) -> Result<DependencyType, DomainError> {
    match s {
        "explicit" => Ok(DependencyType::Explicit),
        "file_overlap" => Ok(DependencyType::FileOverlap),
        "data" => Ok(DependencyType::Data),
        _ => Err(DomainError::Storage(format!("invalid dep type: {s}"))),
    }
}

fn row_to_wp(row: &Row<'_>) -> rusqlite::Result<WorkPackage> {
    let id: i64 = row.get(0)?;
    let feature_id: i64 = row.get(1)?;
    let title: String = row.get(2)?;
    let state_s: String = row.get(3)?;
    let sequence: i32 = row.get(4)?;
    let file_scope_json: String = row.get(5)?;
    let acceptance_criteria: String = row.get(6)?;
    let agent_id: Option<String> = row.get(7)?;
    let pr_url: Option<String> = row.get(8)?;
    let pr_state_s: Option<String> = row.get(9)?;
    let worktree_path: Option<String> = row.get(10)?;
    let created_at_s: String = row.get(11)?;
    let updated_at_s: String = row.get(12)?;

    let state = wp_state_from_str(&state_s).map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad wp state",
            )),
        )
    })?;

    // We use serde_json to parse, but can't use ? directly in rusqlite's Result<T>
    let file_scope: Vec<String> = serde_json::from_str(&file_scope_json).unwrap_or_default();

    let pr_state = pr_state_s.as_deref().map(|s| match s {
        "open" => PrState::Open,
        "review" => PrState::Review,
        "changes_requested" => PrState::ChangesRequested,
        "approved" => PrState::Approved,
        "merged" => PrState::Merged,
        _ => PrState::Open, // fallback
    });

    let created_at = created_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                11,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let updated_at = updated_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    Ok(WorkPackage {
        id,
        feature_id,
        title,
        state,
        sequence,
        file_scope,
        acceptance_criteria,
        agent_id,
        pr_url,
        pr_state,
        worktree_path,
        plane_sub_issue_id: None,
        created_at,
        updated_at,
        base_commit: None,
        head_commit: None,
    })
}

pub fn create_work_package(conn: &Connection, wp: &WorkPackage) -> Result<i64, DomainError> {
    let file_scope_json =
        serde_json::to_string(&wp.file_scope).map_err(|e| DomainError::Storage(e.to_string()))?;
    let pr_state_s = wp.pr_state.map(pr_state_str);

    conn.execute(
        "INSERT INTO work_packages
         (feature_id, title, state, sequence, file_scope, acceptance_criteria,
          agent_id, pr_url, pr_state, worktree_path, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            wp.feature_id,
            wp.title,
            wp_state_str(wp.state),
            wp.sequence,
            file_scope_json,
            wp.acceptance_criteria,
            wp.agent_id,
            wp.pr_url,
            pr_state_s,
            wp.worktree_path,
            wp.created_at.to_rfc3339(),
            wp.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_work_package(conn: &Connection, id: i64) -> Result<Option<WorkPackage>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,title,state,sequence,file_scope,acceptance_criteria,
                agent_id,pr_url,pr_state,worktree_path,created_at,updated_at
         FROM work_packages WHERE id = ?1",
        params![id],
        row_to_wp,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_wp_state(conn: &Connection, id: i64, state: WpState) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE work_packages SET state = ?1, updated_at = ?2 WHERE id = ?3",
        params![wp_state_str(state), now, id],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn update_work_package(conn: &Connection, wp: &WorkPackage) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let file_scope_json =
        serde_json::to_string(&wp.file_scope).map_err(|e| DomainError::Storage(e.to_string()))?;
    let pr_state_s = wp.pr_state.map(pr_state_str);

    conn.execute(
        "UPDATE work_packages SET title = ?1, state = ?2, sequence = ?3, file_scope = ?4, \
         acceptance_criteria = ?5, agent_id = ?6, pr_url = ?7, pr_state = ?8, worktree_path = ?9, \
         updated_at = ?10 WHERE id = ?11",
        params![
            wp.title,
            wp_state_str(wp.state),
            wp.sequence,
            file_scope_json,
            wp.acceptance_criteria,
            wp.agent_id,
            wp.pr_url,
            pr_state_s,
            wp.worktree_path,
            now,
            wp.id
        ],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn list_wps_by_feature(
    conn: &Connection,
    feature_id: i64,
) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,feature_id,title,state,sequence,file_scope,acceptance_criteria,
                    agent_id,pr_url,pr_state,worktree_path,created_at,updated_at
             FROM work_packages WHERE feature_id = ?1 ORDER BY sequence",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![feature_id], row_to_wp)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn create_work_package_for_story(
    conn: &Connection,
    story_id: i64,
    wp: &WorkPackage,
) -> Result<i64, DomainError> {
    let wp_id = create_work_package(conn, wp)?;
    conn.execute(
        "INSERT OR IGNORE INTO story_work_packages (story_id, work_package_id) VALUES (?1, ?2)",
        params![story_id, wp_id],
    )
    .map_err(map_err)?;
    Ok(wp_id)
}

pub fn list_wps_by_story(
    conn: &Connection,
    story_id: i64,
) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT wp.id,wp.feature_id,wp.title,wp.state,wp.sequence,wp.file_scope,
                    wp.acceptance_criteria,wp.agent_id,wp.pr_url,wp.pr_state,
                    wp.worktree_path,wp.created_at,wp.updated_at
             FROM work_packages wp
             JOIN story_work_packages swp ON swp.work_package_id = wp.id
             WHERE swp.story_id = ?1
             ORDER BY wp.sequence",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![story_id], row_to_wp)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_all_work_packages(conn: &Connection) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,feature_id,title,state,sequence,file_scope,acceptance_criteria,
                    agent_id,pr_url,pr_state,worktree_path,created_at,updated_at
             FROM work_packages ORDER BY feature_id, sequence",
        )
        .map_err(map_err)?;

    let rows = stmt.query_map([], row_to_wp).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn get_next_ready_wps(
    conn: &Connection,
    _cycle: Option<i64>,
) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT wp.id,wp.feature_id,wp.title,wp.state,wp.sequence,wp.file_scope,
                    wp.acceptance_criteria,wp.agent_id,wp.pr_url,wp.pr_state,
                    wp.worktree_path,wp.created_at,wp.updated_at
             FROM work_packages wp
             WHERE wp.state = 'planned'
               AND NOT EXISTS (
                   SELECT 1 FROM wp_dependencies d
                   JOIN work_packages dep ON dep.id = d.depends_on
                   WHERE d.wp_id = wp.id AND dep.state != 'done'
               )
             ORDER BY wp.sequence",
        )
        .map_err(map_err)?;

    let rows = stmt.query_map([], row_to_wp).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn add_wp_dependency(conn: &Connection, dep: &WpDependency) -> Result<(), DomainError> {
    conn.execute(
        "INSERT OR IGNORE INTO wp_dependencies (wp_id, depends_on, dep_type)
         VALUES (?1, ?2, ?3)",
        params![dep.wp_id, dep.depends_on, dep_type_str(dep.dep_type)],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn get_wp_dependencies(
    conn: &Connection,
    wp_id: i64,
) -> Result<Vec<WpDependency>, DomainError> {
    let mut stmt = conn
        .prepare("SELECT wp_id, depends_on, dep_type FROM wp_dependencies WHERE wp_id = ?1")
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![wp_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(map_err)?;

    let mut deps = Vec::new();
    for row in rows {
        let (wp_id, depends_on, dep_type_s) = row.map_err(map_err)?;
        let dep_type = dep_type_from_str(&dep_type_s)?;
        deps.push(WpDependency {
            wp_id,
            depends_on,
            dep_type,
        });
    }
    Ok(deps)
}

pub fn get_ready_wps(conn: &Connection, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT wp.id,wp.feature_id,wp.title,wp.state,wp.sequence,wp.file_scope,
                    wp.acceptance_criteria,wp.agent_id,wp.pr_url,wp.pr_state,
                    wp.worktree_path,wp.created_at,wp.updated_at
             FROM work_packages wp
             WHERE wp.feature_id = ?1 AND wp.state = 'planned'
               AND NOT EXISTS (
                   SELECT 1 FROM wp_dependencies d
                   JOIN work_packages dep ON dep.id = d.depends_on
                   WHERE d.wp_id = wp.id AND dep.state != 'done'
               )
             ORDER BY wp.sequence",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![feature_id], row_to_wp)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// Extension trait to add `.optional()` on rusqlite query results.
trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use chrono::Utc;

    fn seed_feature(conn: &Connection, feature_id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
            params![feature_id, format!("feat-{feature_id}"), format!("Feature {feature_id}")],
        )
        .unwrap();
    }

    fn wp(feature_id: i64, title: &str, seq: i32) -> WorkPackage {
        let now = Utc::now();
        WorkPackage {
            id: 0,
            feature_id,
            title: title.to_string(),
            state: WpState::Planned,
            sequence: seq,
            file_scope: vec!["src/main.rs".to_string()],
            acceptance_criteria: "tests pass".to_string(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
            base_commit: None,
            head_commit: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn create_and_get_work_package() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let mut w = wp(1, "WP01 - Auth", 1);
        let id = create_work_package(&conn, &mut w).unwrap();
        assert!(id > 0);
        let fetched = get_work_package(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.title, "WP01 - Auth");
        assert_eq!(fetched.state, WpState::Planned);
        assert_eq!(fetched.feature_id, 1);
        assert_eq!(fetched.sequence, 1);
        assert_eq!(fetched.file_scope, vec!["src/main.rs"]);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_work_package(&conn, 99999).unwrap().is_none());
    }

    #[test]
    fn update_wp_state_changes_state() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let mut w = wp(1, "WP01", 1);
        let id = create_work_package(&conn, &mut w).unwrap();
        update_wp_state(&conn, id, WpState::Doing).unwrap();
        let fetched = get_work_package(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.state, WpState::Doing);
    }

    #[test]
    fn update_work_package_modifies_fields() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let mut w = wp(1, "WP01", 1);
        let id = create_work_package(&conn, &mut w).unwrap();
        w.id = id;
        w.title = "WP01 Updated".to_string();
        w.state = WpState::Doing;
        w.agent_id = Some("agent-1".to_string());
        w.pr_url = Some("https://github.com/pr/1".to_string());
        w.pr_state = Some(PrState::Open);
        update_work_package(&conn, &w).unwrap();
        let fetched = get_work_package(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.title, "WP01 Updated");
        assert_eq!(fetched.state, WpState::Doing);
        assert_eq!(fetched.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(fetched.pr_url.as_deref(), Some("https://github.com/pr/1"));
    }

    #[test]
    fn list_wps_by_feature_filters_and_orders() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        seed_feature(&conn, 2);
        create_work_package(&conn, &mut wp(1, "A", 2)).unwrap();
        create_work_package(&conn, &mut wp(1, "B", 1)).unwrap();
        create_work_package(&conn, &mut wp(2, "C", 1)).unwrap();

        let list = list_wps_by_feature(&conn, 1).unwrap();
        assert_eq!(list.len(), 2);
        // Should be ordered by sequence
        assert_eq!(list[0].title, "B");
        assert_eq!(list[1].title, "A");
    }

    #[test]
    fn list_all_work_packages_returns_all() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        seed_feature(&conn, 2);
        create_work_package(&conn, &mut wp(1, "A", 1)).unwrap();
        create_work_package(&conn, &mut wp(2, "B", 1)).unwrap();

        let all = list_all_work_packages(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn add_and_get_wp_dependencies() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let id1 = create_work_package(&conn, &mut wp(1, "WP01", 1)).unwrap();
        let id2 = create_work_package(&conn, &mut wp(1, "WP02", 2)).unwrap();

        let dep = WpDependency {
            wp_id: id2,
            depends_on: id1,
            dep_type: DependencyType::Explicit,
        };
        add_wp_dependency(&conn, &dep).unwrap();

        let deps = get_wp_dependencies(&conn, id2).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].wp_id, id2);
        assert_eq!(deps[0].depends_on, id1);
        assert_eq!(deps[0].dep_type, DependencyType::Explicit);
    }

    #[test]
    fn get_wp_dependencies_empty_when_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let deps = get_wp_dependencies(&conn, 999).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn get_ready_wps_excludes_blocked_by_unfinished_dep() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let id1 = create_work_package(&conn, &mut wp(1, "WP01", 1)).unwrap();
        let id2 = create_work_package(&conn, &mut wp(1, "WP02", 2)).unwrap();

        // WP02 depends on WP01 which is still planned (not done)
        add_wp_dependency(
            &conn,
            &WpDependency {
                wp_id: id2,
                depends_on: id1,
                dep_type: DependencyType::Explicit,
            },
        )
        .unwrap();

        let ready = get_ready_wps(&conn, 1).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, id1); // WP01 is ready (no deps)
    }

    #[test]
    fn get_ready_wps_includes_when_dep_done() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let id1 = create_work_package(&conn, &mut wp(1, "WP01", 1)).unwrap();
        let id2 = create_work_package(&conn, &mut wp(1, "WP02", 2)).unwrap();

        // Mark WP01 as done
        update_wp_state(&conn, id1, WpState::Review).unwrap();
        update_wp_state(&conn, id1, WpState::Done).unwrap();

        add_wp_dependency(
            &conn,
            &WpDependency {
                wp_id: id2,
                depends_on: id1,
                dep_type: DependencyType::Explicit,
            },
        )
        .unwrap();

        let ready = get_ready_wps(&conn, 1).unwrap();
        assert_eq!(ready.len(), 1); // only WP02 is planned now
        assert_eq!(ready[0].id, id2);
    }

    #[test]
    fn list_wps_by_story_empty_when_no_link() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS story_work_packages (
                story_id INTEGER NOT NULL,
                work_package_id INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
                PRIMARY KEY (story_id, work_package_id)
            );",
        )
        .unwrap();
        let list = list_wps_by_story(&conn, 999).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn get_next_ready_wps_filters_correctly() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        create_work_package(&conn, &mut wp(1, "Ready", 1)).unwrap();
        let mut doing = wp(1, "In Progress", 2);
        doing.state = WpState::Doing;
        create_work_package(&conn, &mut doing).unwrap();

        let ready = get_next_ready_wps(&conn, None).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].title, "Ready");
    }

    #[test]
    fn dep_type_all_variants_roundtrip() {
        for dt in [
            DependencyType::Explicit,
            DependencyType::FileOverlap,
            DependencyType::Data,
        ] {
            let s = dep_type_str(dt);
            assert_eq!(dep_type_from_str(s).unwrap(), dt);
        }
    }

    #[test]
    fn dep_type_from_str_invalid_errors() {
        assert!(dep_type_from_str("invalid").is_err());
    }

    #[test]
    fn wp_state_all_variants_roundtrip() {
        for s in ["planned", "doing", "review", "done", "blocked"] {
            let state = wp_state_from_str(s).unwrap();
            assert_eq!(wp_state_str(state), s);
        }
    }

    #[test]
    fn wp_state_from_str_invalid_errors() {
        assert!(wp_state_from_str("invalid").is_err());
    }

    #[test]
    fn pr_state_all_variants() {
        let _ = pr_state_str(PrState::Open);
        let _ = pr_state_str(PrState::Review);
        let _ = pr_state_str(PrState::ChangesRequested);
        let _ = pr_state_str(PrState::Approved);
        let _ = pr_state_str(PrState::Merged);
    }

    #[test]
    fn file_scope_roundtrips_through_json() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let mut w = wp(1, "WP01", 1);
        w.file_scope = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
        let id = create_work_package(&conn, &mut w).unwrap();
        let fetched = get_work_package(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.file_scope, vec!["src/a.rs", "src/b.rs"]);
    }

    #[test]
    fn optional_extension_trait_on_none() {
        let result: rusqlite::Result<i64> = Err(rusqlite::Error::QueryReturnedNoRows);
        assert!(result.optional().unwrap().is_none());
    }

    #[test]
    fn optional_extension_trait_on_some() {
        let result: rusqlite::Result<i64> = Ok(42);
        assert_eq!(result.optional().unwrap(), Some(42));
    }

    #[test]
    fn optional_extension_trait_on_other_error() {
        let result: rusqlite::Result<i64> =
            Err(rusqlite::Error::InvalidParameterName("test".to_string()));
        assert!(result.optional().is_err());
    }

    #[test]
    fn wp_with_all_pr_states_roundtrips() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        for pr_state in [
            PrState::Open,
            PrState::Review,
            PrState::ChangesRequested,
            PrState::Approved,
            PrState::Merged,
        ] {
            let mut w = wp(1, "WP", 1);
            w.pr_state = Some(pr_state);
            let id = create_work_package(&conn, &mut w).unwrap();
            let fetched = get_work_package(&conn, id).unwrap().unwrap();
            assert_eq!(fetched.pr_state, Some(pr_state));
        }
    }

    #[test]
    fn update_wp_state_with_all_states() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let mut w = wp(1, "WP", 1);
        let id = create_work_package(&conn, &mut w).unwrap();
        for state in [WpState::Doing, WpState::Review, WpState::Done] {
            update_wp_state(&conn, id, state).unwrap();
            let fetched = get_work_package(&conn, id).unwrap().unwrap();
            assert_eq!(fetched.state, state);
        }
    }

    #[test]
    fn multiple_deps_for_one_wp() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 1);
        let id1 = create_work_package(&conn, &mut wp(1, "WP01", 1)).unwrap();
        let id2 = create_work_package(&conn, &mut wp(1, "WP02", 2)).unwrap();
        let id3 = create_work_package(&conn, &mut wp(1, "WP03", 3)).unwrap();

        add_wp_dependency(
            &conn,
            &WpDependency {
                wp_id: id3,
                depends_on: id1,
                dep_type: DependencyType::Explicit,
            },
        )
        .unwrap();
        add_wp_dependency(
            &conn,
            &WpDependency {
                wp_id: id3,
                depends_on: id2,
                dep_type: DependencyType::FileOverlap,
            },
        )
        .unwrap();

        let deps = get_wp_dependencies(&conn, id3).unwrap();
        assert_eq!(deps.len(), 2);
    }
}
