//! Epic repository functions.
//!
//! Traceability: FR-STORE-EPIC

use chrono::DateTime;
use rusqlite::{params, Connection};

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::error::DomainError;

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
        requirement_id: row.get(8).unwrap_or(None),
        created_at: parse_dt(&created_at),
        updated_at: parse_dt(&updated_at),
    })
}

/// Create an epic and return its new row ID.
pub fn create_epic(conn: &Connection, epic: &Epic) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO epics (project_id, title, description, status, owner_id, requirement_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            epic.project_id,
            epic.title,
            epic.description,
            epic.status.to_string(),
            epic.owner_id,
            epic.requirement_id,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

/// Upsert an epic by `requirement_id` — creates if absent, updates title/description/status if present.
/// Returns the row ID (new or existing).
///
/// Uses manual get-then-insert/update because `requirement_id` is added via ALTER TABLE
/// (no inline UNIQUE constraint); SQLite upsert `ON CONFLICT(col)` requires a declared
/// table-level constraint that ALTER TABLE cannot add.
pub fn upsert_epic_by_requirement_id(conn: &Connection, epic: &Epic) -> Result<i64, DomainError> {
    let req_id = epic.requirement_id.as_deref().ok_or_else(|| {
        DomainError::Validation("upsert_epic_by_requirement_id requires requirement_id".to_string())
    })?;
    let now = chrono::Utc::now().to_rfc3339();
    // Check if a row with this requirement_id already exists.
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM epics WHERE requirement_id = ?1",
            params![req_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(map_err)?;

    if let Some(id) = existing_id {
        // Update title, description, status.
        conn.execute(
            "UPDATE epics SET title = ?1, description = ?2, status = ?3, updated_at = ?4 WHERE id = ?5",
            params![
                epic.title,
                epic.description,
                epic.status.to_string(),
                now,
                id,
            ],
        )
        .map_err(map_err)?;
        Ok(id)
    } else {
        create_epic(conn, epic)
    }
}

/// Look up an epic by `requirement_id`. Returns `None` if not found.
pub fn get_epic_by_requirement_id(
    conn: &Connection,
    req_id: &str,
) -> Result<Option<Epic>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, description, status, owner_id, created_at, updated_at, requirement_id \
             FROM epics WHERE requirement_id = ?1",
        )
        .map_err(map_err)?;
    match stmt.query_row(params![req_id], row_to_epic) {
        Ok(e) => Ok(Some(e)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Look up an epic by ID. Returns `None` if not found.
pub fn get_epic_by_id(conn: &Connection, id: i64) -> Result<Option<Epic>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, title, description, status, owner_id, created_at, updated_at, requirement_id \
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
            "SELECT id, project_id, title, description, status, owner_id, created_at, updated_at, requirement_id \
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
