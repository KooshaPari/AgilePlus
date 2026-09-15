//! User repository functions.
//!
//! Traceability: FR-STORE-USER

use chrono::DateTime;
use rusqlite::{Connection, params};

use agileplus_domain::domain::user::{User, UserRole, UserStatus};
use agileplus_domain::error::DomainError;

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn parse_dt(s: &str) -> DateTime<chrono::Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

fn row_to_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    let role_str: String = row.get(3)?;
    let status_str: String = row.get(4)?;
    let created_at: String = row.get(7)?;
    let updated_at: String = row.get(8)?;
    Ok(User {
        id: row.get(0)?,
        display_name: row.get(1)?,
        email: row.get(2)?,
        role: role_str.parse().unwrap_or(UserRole::Member),
        status: status_str.parse().unwrap_or(UserStatus::Active),
        avatar_url: row.get(5)?,
        github_login: row.get(6)?,
        created_at: parse_dt(&created_at),
        updated_at: parse_dt(&updated_at),
    })
}

/// Create a user and return its new row ID.
pub fn create_user(conn: &Connection, user: &User) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO users (display_name, email, role, status, avatar_url, github_login, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            user.display_name,
            user.email,
            user.role.to_string(),
            user.status.to_string(),
            user.avatar_url,
            user.github_login,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

/// Look up a user by ID. Returns `None` if not found.
pub fn get_user_by_id(conn: &Connection, id: i64) -> Result<Option<User>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, display_name, email, role, status, avatar_url, github_login, created_at, updated_at \
             FROM users WHERE id = ?1",
        )
        .map_err(map_err)?;

    match stmt.query_row(params![id], row_to_user) {
        Ok(u) => Ok(Some(u)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Look up a user by email. Returns `None` if not found.
pub fn get_user_by_email(conn: &Connection, email: &str) -> Result<Option<User>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, display_name, email, role, status, avatar_url, github_login, created_at, updated_at \
             FROM users WHERE email = ?1",
        )
        .map_err(map_err)?;

    match stmt.query_row(params![email], row_to_user) {
        Ok(u) => Ok(Some(u)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Update the status of a user.
pub fn update_user_status(
    conn: &Connection,
    id: i64,
    status: UserStatus,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE users SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.to_string(), now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::NotFound(format!("user {id}")));
    }
    Ok(())
}

/// Update the role of a user.
pub fn update_user_role(conn: &Connection, id: i64, role: UserRole) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE users SET role = ?1, updated_at = ?2 WHERE id = ?3",
            params![role.to_string(), now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::NotFound(format!("user {id}")));
    }
    Ok(())
}

/// List all users ordered by created_at.
pub fn list_all_users(conn: &Connection) -> Result<Vec<User>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, display_name, email, role, status, avatar_url, github_login, created_at, updated_at \
             FROM users ORDER BY created_at ASC",
        )
        .map_err(map_err)?;

    let users = stmt
        .query_map([], row_to_user)
        .map_err(map_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_err)?;
    Ok(users)
}

/// Delete a user by ID.
pub fn delete_user(conn: &Connection, id: i64) -> Result<(), DomainError> {
    let rows = conn
        .execute("DELETE FROM users WHERE id = ?1", params![id])
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::NotFound(format!("user {id}")));
    }
    Ok(())
}

mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;

    fn sample_user(name: &str) -> User {
        User {
            id: 0,
            display_name: name.to_string(),
            email: format!("{name}@test.com"),
            role: UserRole::Member,
            status: UserStatus::Active,
            avatar_url: None,
            github_login: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn create_and_get_user_by_id() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_user(&conn, &sample_user("Alice")).unwrap();
        assert!(id > 0);
        let u = get_user_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(u.display_name, "Alice");
    }

    #[test]
    fn get_by_id_nonexistent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_user_by_id(&conn, 999).unwrap().is_none());
    }

    #[test]
    fn get_by_email_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_user(&conn, &sample_user("Bob")).unwrap();
        assert!(get_user_by_email(&conn, "bob@test.com").unwrap().is_some());
    }

    #[test]
    fn list_all_users_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_user(&conn, &sample_user("A")).unwrap();
        create_user(&conn, &sample_user("B")).unwrap();
        assert_eq!(list_all_users(&conn).unwrap().len(), 2);
    }

    #[test]
    fn update_status() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_user(&conn, &sample_user("S")).unwrap();
        update_user_status(&conn, id, UserStatus::Suspended).unwrap();
        assert_eq!(get_user_by_id(&conn, id).unwrap().unwrap().status, UserStatus::Suspended);
    }

    #[test]
    fn update_role() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_user(&conn, &sample_user("R")).unwrap();
        update_user_role(&conn, id, UserRole::Admin).unwrap();
        assert_eq!(get_user_by_id(&conn, id).unwrap().unwrap().role, UserRole::Admin);
    }

    #[test]
    fn delete_user_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_user(&conn, &sample_user("Del")).unwrap();
        delete_user(&conn, id).unwrap();
        assert!(get_user_by_id(&conn, id).unwrap().is_none());
    }

    #[test]
    fn user_with_optional_fields() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut u = sample_user("Opt");
        u.avatar_url = Some("https://avatar.url".to_string());
        u.github_login = Some("optuser".to_string());
        let id = create_user(&conn, &u).unwrap();
        let fetched = get_user_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.avatar_url.as_deref(), Some("https://avatar.url"));
        assert_eq!(fetched.github_login.as_deref(), Some("optuser"));
    }
}
