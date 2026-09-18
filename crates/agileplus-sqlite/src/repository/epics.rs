//! Epic repository functions.
//!
//! Traceability: FR-STORE-EPIC

use chrono::DateTime;
use rusqlite::{Connection, params};

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::error::DomainError;

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn parse_dt(s: &str) -> DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

fn row_to_epic(row: &rusqlite::Row<'_>) -> rusqlite::Result<Epic> {
    let status_str: String = row.get(4)?;
    let created_at: String = row.get(6)?;
    let updated_at: String = row.get(7)?;
    Ok(Epic {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        status: status_str.parse().unwrap_or(EpicStatus::Backlog),
        owner_id: row.get(5)?,
        requirement_id: None,
        created_at: parse_dt(&created_at),
        updated_at: parse_dt(&updated_at),
    })
}

/// Create an epic and return its new row ID.
pub fn create_epic(conn: &Connection, epic: &Epic) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO epics (project_id, title, description, status, owner_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            epic.project_id,
            epic.title,
            epic.description,
            epic.status.to_string(),
            epic.owner_id,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_epic_by_requirement_id(
    conn: &Connection,
    requirement_id: &str,
) -> Result<Option<Epic>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, description, status, owner_id, created_at, updated_at \
             FROM epics WHERE requirement_id = ?1",
        )
        .map_err(map_err)?;

    match stmt.query_row(params![requirement_id], row_to_epic) {
        Ok(mut epic) => {
            epic.requirement_id = Some(requirement_id.to_string());
            Ok(Some(epic))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

pub fn upsert_epic_by_requirement_id(conn: &Connection, epic: &Epic) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let req_id_owned = epic.requirement_id.clone();
    let Some(requirement_id) = req_id_owned.as_deref() else {
        return create_epic(conn, epic);
    };

    if let Some(existing) = get_epic_by_requirement_id(conn, requirement_id)? {
        return Ok(existing.id);
    }

    // Insert path: create_epic ignores requirement_id (a pre-existing bug). Insert
    // here directly so the row is keyed on requirement_id and get-by-requirement-id
    // can find it on the next call.
    conn.execute(
        "INSERT INTO epics (project_id, title, description, status, owner_id, requirement_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            epic.project_id,
            epic.title,
            epic.description,
            epic.status.to_string(),
            epic.owner_id,
            requirement_id,
            &now,
            &now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

/// Look up an epic by ID. Returns `None` if not found.
pub fn get_epic_by_id(conn: &Connection, id: i64) -> Result<Option<Epic>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, description, status, owner_id, created_at, updated_at \
             FROM epics WHERE id = ?1",
        )
        .map_err(map_err)?;

    match stmt.query_row(params![id], row_to_epic) {
        Ok(e) => Ok(Some(e)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Update the status of an epic.
pub fn update_epic_status(
    conn: &Connection,
    id: i64,
    status: EpicStatus,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE epics SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.to_string(), now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::NotFound(format!("epic {id}")));
    }
    Ok(())
}

/// List all epics for a given project, ordered by creation time.
pub fn list_epics_by_project(conn: &Connection, project_id: i64) -> Result<Vec<Epic>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, description, status, owner_id, created_at, updated_at \
             FROM epics WHERE project_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(map_err)?;

    let epics = stmt
        .query_map(params![project_id], row_to_epic)
        .map_err(map_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_err)?;
    Ok(epics)
}

/// Delete an epic by ID. Will fail if stories reference it (FK constraint).
pub fn delete_epic(conn: &Connection, id: i64) -> Result<(), DomainError> {
    let rows = conn
        .execute("DELETE FROM epics WHERE id = ?1", params![id])
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::NotFound(format!("epic {id}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;

    fn setup_project(conn: &Connection, project_id: i64) {
        conn.execute(
            "INSERT INTO projects (id, slug, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, '', ?4, ?4)",
            params![project_id, format!("proj-{project_id}"), format!("Project {project_id}"), chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();
    }

    fn sample_epic(project_id: i64, title: &str) -> Epic {
        Epic {
            id: 0,
            project_id,
            title: title.to_string(),
            description: Some(format!("Desc for {title}")),
            status: EpicStatus::Backlog,
            owner_id: None,
            requirement_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn create_and_get_epic_by_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let id = create_epic(&conn, &sample_epic(1, "Auth System")).unwrap();
        assert!(id > 0);
        let fetched = get_epic_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.title, "Auth System");
        assert_eq!(fetched.status, EpicStatus::Backlog);
        assert_eq!(fetched.project_id, 1);
    }

    #[test]
    fn get_epic_by_id_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_epic_by_id(&conn, 99999).unwrap().is_none());
    }

    #[test]
    fn update_epic_status_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let id = create_epic(&conn, &sample_epic(1, "Epic A")).unwrap();
        update_epic_status(&conn, id, EpicStatus::Active).unwrap();
        let fetched = get_epic_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.status, EpicStatus::Active);
    }

    #[test]
    fn update_epic_status_nonexistent_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let err = update_epic_status(&conn, 99999, EpicStatus::Done).unwrap_err();
        assert!(format!("{err}").contains("99999"));
    }

    #[test]
    fn list_epics_by_project_filters() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        setup_project(&conn, 2);
        create_epic(&conn, &sample_epic(1, "P1 Epic")).unwrap();
        create_epic(&conn, &sample_epic(1, "P1 Epic 2")).unwrap();
        create_epic(&conn, &sample_epic(2, "P2 Epic")).unwrap();

        let p1 = list_epics_by_project(&conn, 1).unwrap();
        assert_eq!(p1.len(), 2);
        let p2 = list_epics_by_project(&conn, 2).unwrap();
        assert_eq!(p2.len(), 1);
    }

    #[test]
    fn list_epics_by_project_empty() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let list = list_epics_by_project(&conn, 999).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn delete_epic_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let id = create_epic(&conn, &sample_epic(1, "To Delete")).unwrap();
        delete_epic(&conn, id).unwrap();
        assert!(get_epic_by_id(&conn, id).unwrap().is_none());
    }

    #[test]
    fn delete_epic_nonexistent_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let err = delete_epic(&conn, 99999).unwrap_err();
        assert!(format!("{err}").contains("99999"));
    }

    #[test]
    fn upsert_epic_creates_when_no_requirement_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let epic = sample_epic(1, "No Req");
        let id = upsert_epic_by_requirement_id(&conn, &epic).unwrap();
        let fetched = get_epic_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.title, "No Req");
    }

    #[test]
    fn upsert_epic_by_requirement_id_creates_new() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let mut epic = sample_epic(1, "FR Epic");
        epic.requirement_id = Some("FR-001".to_string());
        let id = upsert_epic_by_requirement_id(&conn, &epic).unwrap();
        let fetched = get_epic_by_requirement_id(&conn, "FR-001")
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.requirement_id.as_deref(), Some("FR-001"));
    }

    #[test]
    fn upsert_epic_by_requirement_id_is_idempotent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let mut epic = sample_epic(1, "FR Epic");
        epic.requirement_id = Some("FR-001".to_string());
        let id1 = upsert_epic_by_requirement_id(&conn, &epic).unwrap();
        let id2 = upsert_epic_by_requirement_id(&conn, &epic).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn get_epic_by_requirement_id_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(
            get_epic_by_requirement_id(&conn, "FR-999")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn unknown_status_and_timestamp_fall_back_to_defaults() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        setup_project(&conn, 1);
        let before = chrono::Utc::now();
        conn.execute(
            "INSERT INTO epics (id, project_id, title, description, status, owner_id, created_at, updated_at)
             VALUES (1, 1, 'Legacy epic', NULL, 'not-a-status', NULL, 'not-a-timestamp', 'not-a-timestamp')",
            [],
        )
        .unwrap();

        let epic = get_epic_by_id(&conn, 1).unwrap().unwrap();
        assert_eq!(epic.status, EpicStatus::Backlog);
        assert!(
            epic.created_at >= before,
            "unparseable timestamp falls back to now"
        );
    }
}
