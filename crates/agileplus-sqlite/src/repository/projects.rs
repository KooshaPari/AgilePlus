// SPDX-License-Identifier: MIT OR Apache-2.0
//! Project repository functions.
//!
//! Traceability: FR-STORE-PROJECT

use chrono::DateTime;
use rusqlite::{Connection, params};

use agileplus_domain::domain::project::Project;
use agileplus_domain::error::DomainError;

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn parse_dt(s: &str) -> DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let created_at: String = row.get(4)?;
    let updated_at: String = row.get(5)?;
    let description: String = row.get(3)?;
    Ok(Project {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        created_at: parse_dt(&created_at),
        updated_at: parse_dt(&updated_at),
    })
}

/// Create a project and return its new row ID.
pub fn create_project(conn: &Connection, project: &Project) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let description = project.description.as_deref().unwrap_or("");
    conn.execute(
        "INSERT INTO projects (slug, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![project.slug, project.name, description, now, now],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

/// Look up a project by its slug. Returns None if not found.
pub fn get_project_by_slug(conn: &Connection, slug: &str) -> Result<Option<Project>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, name, description, created_at, updated_at \
             FROM projects WHERE slug = ?1",
        )
        .map_err(map_err)?;

    match stmt.query_row(params![slug], row_to_project) {
        Ok(project) => Ok(Some(project)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Look up a project by its ID. Returns `None` if not found.
pub fn get_project_by_id(conn: &Connection, id: i64) -> Result<Option<Project>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, name, description, created_at, updated_at \
             FROM projects WHERE id = ?1",
        )
        .map_err(map_err)?;

    match stmt.query_row(params![id], row_to_project) {
        Ok(project) => Ok(Some(project)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// List all projects ordered by creation time.
pub fn list_all_projects(conn: &Connection) -> Result<Vec<Project>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, name, description, created_at, updated_at \
             FROM projects ORDER BY created_at ASC",
        )
        .map_err(map_err)?;

    let projects = stmt
        .query_map([], row_to_project)
        .map_err(map_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_err)?;
    Ok(projects)
}

/// Delete a project by ID. Will fail if epics reference it (FK constraint).
pub fn delete_project(conn: &Connection, id: i64) -> Result<(), DomainError> {
    let rows = conn
        .execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::NotFound(format!("project {id}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;

    fn make_project(slug: &str, name: &str) -> Project {
        Project {
            id: 0,
            slug: slug.to_string(),
            name: name.to_string(),
            description: Some("desc".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_create_and_get_project() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let p = make_project("proj-1", "Project One");
        let id = create_project(&conn, &p).unwrap();
        assert!(id > 0);
        let got = get_project_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(got.slug, "proj-1");
        assert_eq!(got.name, "Project One");
    }

    #[test]
    fn test_get_by_slug() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let p = make_project("slug-test", "Slug Test");
        create_project(&conn, &p).unwrap();
        let got = get_project_by_slug(&conn, "slug-test").unwrap().unwrap();
        assert_eq!(got.name, "Slug Test");
    }

    #[test]
    fn test_get_by_slug_nonexistent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let got = get_project_by_slug(&conn, "nope").unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn test_list_all_projects() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_project(&conn, &make_project("p1", "P1")).unwrap();
        create_project(&conn, &make_project("p2", "P2")).unwrap();
        let all = list_all_projects(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_project() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_project(&conn, &make_project("del", "Del")).unwrap();
        delete_project(&conn, id).unwrap();
        assert!(get_project_by_id(&conn, id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(delete_project(&conn, 999).is_err());
    }
}
