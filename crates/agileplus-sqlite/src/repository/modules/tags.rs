use rusqlite::{Connection, params};

use agileplus_domain::domain::module::ModuleFeatureTag;
use agileplus_domain::error::DomainError;

use crate::repository::features::map_err;

/// Tag a feature to a module. Idempotent (INSERT OR IGNORE).
pub fn tag_feature_to_module(conn: &Connection, tag: &ModuleFeatureTag) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO module_feature_tags (module_id, feature_id, created_at)
         VALUES (?1, ?2, ?3)",
        params![tag.module_id, tag.feature_id, now],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn untag_feature_from_module(
    conn: &Connection,
    module_id: i64,
    feature_id: i64,
) -> Result<(), DomainError> {
    conn.execute(
        "DELETE FROM module_feature_tags WHERE module_id = ?1 AND feature_id = ?2",
        params![module_id, feature_id],
    )
    .map_err(map_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use agileplus_domain::domain::module::ModuleFeatureTag;

    fn seed_feature(conn: &Connection, feature_id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)              VALUES (?1, ?2, ?3, 'created', X'00', 'main', datetime('now'), datetime('now'))",
            params![feature_id, format!("feat-{feature_id}"), format!("Feature {feature_id}")],
        )
        .unwrap();
    }

    fn seed_module(conn: &Connection, module_id: i64) {
        conn.execute(
            "INSERT OR IGNORE INTO modules (id, slug, friendly_name, created_at, updated_at)              VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
            params![module_id, format!("mod-{module_id}"), format!("Module {module_id}")],
        )
        .unwrap();
    }

    #[test]
    fn tag_and_untag_feature() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 10);
        seed_module(&conn, 20);
        let tag = ModuleFeatureTag::new(20, 10);
        tag_feature_to_module(&conn, &tag).unwrap();
        untag_feature_from_module(&conn, 20, 10).unwrap();
    }

    #[test]
    fn tag_is_idempotent() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        seed_feature(&conn, 10);
        seed_module(&conn, 20);
        let tag = ModuleFeatureTag::new(20, 10);
        tag_feature_to_module(&conn, &tag).unwrap();
        tag_feature_to_module(&conn, &tag).unwrap();
    }

    #[test]
    fn untag_nonexistent_is_ok() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        untag_feature_from_module(&conn, 999, 999).unwrap();
    }
}
