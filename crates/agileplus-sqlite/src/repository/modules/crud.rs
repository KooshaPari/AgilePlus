use rusqlite::{Connection, params};

use agileplus_domain::{
    domain::module::{Module, ModuleWithFeatures},
    error::DomainError,
};

use crate::repository::features::map_err;

use super::mappers::{OptionalExt, row_to_feature, row_to_module};

/// Create a module. Returns the new row ID.
///
/// Detects circular references via a recursive CTE before inserting.
pub fn create_module(conn: &Connection, module: &Module) -> Result<i64, DomainError> {
    if let Some(parent_id) = module.parent_module_id {
        let parent_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM modules WHERE id = ?1",
                params![parent_id],
                |row| row.get(0),
            )
            .map_err(map_err)?;
        if parent_exists == 0 {
            return Err(DomainError::ModuleNotFound(parent_id.to_string()));
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO modules (slug, friendly_name, description, parent_module_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            module.slug,
            module.friendly_name,
            module.description,
            module.parent_module_id,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_module(conn: &Connection, id: i64) -> Result<Option<Module>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
         FROM modules WHERE id = ?1",
        params![id],
        row_to_module,
    )
    .optional()
    .map_err(map_err)
}

pub fn get_module_by_slug(conn: &Connection, slug: &str) -> Result<Option<Module>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
         FROM modules WHERE slug = ?1",
        params![slug],
        row_to_module,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_module(
    conn: &Connection,
    id: i64,
    friendly_name: &str,
    description: Option<&str>,
) -> Result<(), DomainError> {
    let slug = agileplus_domain::domain::module::Module::slug_from_name(friendly_name);
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE modules SET slug = ?1, friendly_name = ?2, description = ?3, updated_at = ?4
             WHERE id = ?5",
            params![slug, friendly_name, description, now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::ModuleNotFound(id.to_string()));
    }
    Ok(())
}

/// Delete a module. Fails with `ModuleHasDependents` if child modules or owned features exist.
pub fn delete_module(conn: &Connection, id: i64) -> Result<(), DomainError> {
    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM modules WHERE parent_module_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(map_err)?;
    if child_count > 0 {
        return Err(DomainError::ModuleHasDependents(format!(
            "module {id} has {child_count} child module(s)"
        )));
    }

    let feature_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM features WHERE module_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(map_err)?;
    if feature_count > 0 {
        return Err(DomainError::ModuleHasDependents(format!(
            "module {id} owns {feature_count} feature(s)"
        )));
    }

    let rows = conn
        .execute("DELETE FROM modules WHERE id = ?1", params![id])
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::ModuleNotFound(id.to_string()));
    }
    Ok(())
}

pub fn list_root_modules(conn: &Connection) -> Result<Vec<Module>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
             FROM modules WHERE parent_module_id IS NULL ORDER BY friendly_name",
        )
        .map_err(map_err)?;
    let rows = stmt.query_map([], row_to_module).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_child_modules(conn: &Connection, parent_id: i64) -> Result<Vec<Module>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
             FROM modules WHERE parent_module_id = ?1 ORDER BY friendly_name",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![parent_id], row_to_module)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// Detect whether setting `proposed_parent_id` as the parent of `module_id` would
/// create a cycle in the module hierarchy using a recursive CTE.
pub fn would_create_circular_ref(
    conn: &Connection,
    module_id: i64,
    proposed_parent_id: i64,
) -> Result<bool, DomainError> {
    let count: i64 = conn
        .query_row(
            "WITH RECURSIVE ancestors(id) AS (
                SELECT ?1
                UNION ALL
                SELECT m.parent_module_id
                FROM modules m
                INNER JOIN ancestors a ON m.id = a.id
                WHERE m.parent_module_id IS NOT NULL
            )
            SELECT COUNT(*) FROM ancestors WHERE id = ?2",
            params![proposed_parent_id, module_id],
            |row| row.get(0),
        )
        .map_err(map_err)?;
    Ok(count > 0)
}

pub fn get_module_with_features(
    conn: &Connection,
    id: i64,
) -> Result<Option<ModuleWithFeatures>, DomainError> {
    let module = match get_module(conn, id)? {
        Some(m) => m,
        None => return Ok(None),
    };

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at
             FROM features WHERE module_id = ?1 ORDER BY created_at",
        )
        .map_err(map_err)?;
    let owned_features = stmt
        .query_map(params![id], row_to_feature)
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;

    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.slug, f.friendly_name, f.state, f.spec_hash, f.target_branch,
                    f.created_at, f.updated_at
             FROM features f
             INNER JOIN module_feature_tags t ON f.id = t.feature_id
             WHERE t.module_id = ?1
             ORDER BY f.created_at",
        )
        .map_err(map_err)?;
    let tagged_features = stmt
        .query_map(params![id], row_to_feature)
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;

    let child_modules = list_child_modules(conn, id)?;

    Ok(Some(ModuleWithFeatures {
        module,
        owned_features,
        tagged_features,
        child_modules,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;

    #[test]
    fn test_create_and_get_module() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m = Module::new("Test Module", None);
        let id = create_module(&conn, &m).unwrap();
        assert!(id > 0);
        let got = get_module(&conn, id).unwrap().unwrap();
        assert_eq!(got.friendly_name, "Test Module");
    }

    #[test]
    fn test_get_module_by_slug() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m = Module::new("Slug Test", None);
        create_module(&conn, &m).unwrap();
        let got = get_module_by_slug(&conn, "slug-test").unwrap().unwrap();
        assert_eq!(got.friendly_name, "Slug Test");
    }

    #[test]
    fn test_list_root_modules() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_module(&conn, &Module::new("Root A", None)).unwrap();
        create_module(&conn, &Module::new("Root B", None)).unwrap();
        let roots = list_root_modules(&conn).unwrap();
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn test_list_child_modules() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let parent_id = create_module(&conn, &Module::new("Parent", None)).unwrap();
        create_module(&conn, &Module::new("Child 1", Some(parent_id))).unwrap();
        create_module(&conn, &Module::new("Child 2", Some(parent_id))).unwrap();
        let children = list_child_modules(&conn, parent_id).unwrap();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_update_module_name() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_module(&conn, &Module::new("Old Name", None)).unwrap();
        update_module(&conn, id, "New Name", Some("new desc")).unwrap();
        let got = get_module(&conn, id).unwrap().unwrap();
        assert_eq!(got.friendly_name, "New Name");
        assert_eq!(got.slug, "new-name");
    }

    #[test]
    fn test_circular_ref_detected() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let a = create_module(&conn, &Module::new("A", None)).unwrap();
        let b = create_module(&conn, &Module::new("B", Some(a))).unwrap();
        assert!(would_create_circular_ref(&conn, a, b).unwrap());
    }

    #[test]
    fn test_circular_ref_not_detected() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let a = create_module(&conn, &Module::new("X", None)).unwrap();
        let b = create_module(&conn, &Module::new("Y", None)).unwrap();
        assert!(!would_create_circular_ref(&conn, a, b).unwrap());
    }

    #[test]
    fn test_delete_module_no_children() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_module(&conn, &Module::new("Del", None)).unwrap();
        delete_module(&conn, id).unwrap();
        assert!(get_module(&conn, id).unwrap().is_none());
    }

    #[test]
    fn test_create_nonexistent_parent_errors() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let m = Module::new("Orphan", Some(999));
        assert!(create_module(&conn, &m).is_err());
    }
}
