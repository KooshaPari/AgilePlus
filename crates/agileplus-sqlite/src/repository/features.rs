// SPDX-License-Identifier: MIT OR Apache-2.0
//! Feature repository â€” CRUD operations for the `features` table.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::{feature::Feature, state_machine::FeatureState},
    error::DomainError,
};

pub(crate) fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn state_str(s: FeatureState) -> &'static str {
    match s {
        FeatureState::Created => "created",
        FeatureState::Specified => "specified",
        FeatureState::Researched => "researched",
        FeatureState::Planned => "planned",
        FeatureState::Implementing => "implementing",
        FeatureState::Validated => "validated",
        FeatureState::Shipped => "shipped",
        FeatureState::Retrospected => "retrospected",
    }
}

fn labels_to_json(labels: &[String]) -> String {
    serde_json::to_string(labels).unwrap_or_else(|_| "[]".to_owned())
}

fn labels_from_json(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

fn row_to_feature(row: &Row<'_>) -> rusqlite::Result<Feature> {
    let id: i64 = row.get(0)?;
    let slug: String = row.get(1)?;
    let friendly_name: String = row.get(2)?;
    let state_str: String = row.get(3)?;
    let spec_hash_bytes: Vec<u8> = row.get(4)?;
    let target_branch: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;
    // module_id column added by migration 015 -- may be NULL.
    let module_id: Option<i64> = row.get(8).unwrap_or(None);
    // labels column added by migration 026 -- defaults to '[]'.
    let labels_json: String = row.get(9).unwrap_or_else(|_| "[]".to_owned());

    let state = state_str.parse::<FeatureState>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let mut spec_hash = [0u8; 32];
    if spec_hash_bytes.len() == 32 {
        spec_hash.copy_from_slice(&spec_hash_bytes);
    }

    let created_at = created_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let updated_at = updated_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    Ok(Feature {
        id,
        slug,
        friendly_name,
        state,
        spec_hash,
        target_branch,
        plane_issue_id: None,
        plane_state_id: None,
        labels: labels_from_json(&labels_json),
        module_id,
        project_id: None,
        created_at,
        updated_at,
        created_at_commit: None,
        last_modified_commit: None,
    })
}

pub fn create_feature(conn: &Connection, feature: &Feature) -> Result<i64, DomainError> {
    conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, labels, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            feature.slug,
            feature.friendly_name,
            state_str(feature.state),
            feature.spec_hash.as_slice(),
            feature.target_branch,
            labels_to_json(&feature.labels),
            feature.created_at.to_rfc3339(),
            feature.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_feature_by_slug(conn: &Connection, slug: &str) -> Result<Option<Feature>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at,
                module_id, labels
         FROM features WHERE slug = ?1",
        params![slug],
        row_to_feature,
    )
    .optional()
    .map_err(map_err)
}

pub fn get_feature_by_id(conn: &Connection, id: i64) -> Result<Option<Feature>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at,
                module_id, labels
         FROM features WHERE id = ?1",
        params![id],
        row_to_feature,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_feature_state(
    conn: &Connection,
    id: i64,
    state: FeatureState,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET state = ?1, updated_at = ?2 WHERE id = ?3",
        params![state_str(state), now, id],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn update_feature(conn: &Connection, feature: &Feature) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET slug = ?1, friendly_name = ?2, state = ?3, labels = ?4,
                target_branch = ?5, spec_hash = ?6, updated_at = ?7 WHERE id = ?8",
        params![
            feature.slug,
            feature.friendly_name,
            state_str(feature.state),
            labels_to_json(&feature.labels),
            feature.target_branch,
            feature.spec_hash.as_slice(),
            now,
            feature.id
        ],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn list_features_by_state(
    conn: &Connection,
    state: FeatureState,
) -> Result<Vec<Feature>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at,
                    module_id, labels
             FROM features WHERE state = ?1 ORDER BY created_at",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![state_str(state)], row_to_feature)
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_all_features(conn: &Connection) -> Result<Vec<Feature>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at,
                    module_id, labels
             FROM features ORDER BY created_at DESC",
        )
        .map_err(map_err)?;

    let rows = stmt.query_map([], row_to_feature).map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_features_by_label(conn: &Connection, label: &str) -> Result<Vec<Feature>, DomainError> {
    // labels is stored as a JSON array: ["foo","bar"]. Use json_each to filter in SQL.
    let pattern = format!("%\"{}\"%", label.replace('"', "\\\""));
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at,
                    module_id, labels
             FROM features WHERE labels LIKE ?1 ORDER BY created_at DESC",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![pattern], row_to_feature)
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
    use agileplus_domain::domain::feature::Feature;

    fn sample_feature(slug: &str) -> Feature {
        let now = chrono::Utc::now();
        Feature {
            id: 0,
            slug: slug.to_string(),
            friendly_name: format!("Feature {slug}"),
            state: FeatureState::Created,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at: now,
            updated_at: now,
            created_at_commit: None,
            last_modified_commit: None,
        }
    }

    #[test]
    fn create_and_get_feature_by_slug() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut f = sample_feature("auth-flow");
        f.labels = vec!["security".into()];
        let id = create_feature(&conn, &f).unwrap();
        assert!(id > 0);
        let fetched = get_feature_by_slug(&conn, "auth-flow").unwrap().unwrap();
        assert_eq!(fetched.slug, "auth-flow");
        assert_eq!(fetched.state, FeatureState::Created);
        assert_eq!(fetched.labels, vec!["security"]);
        assert_eq!(fetched.target_branch, "main");
    }

    #[test]
    fn get_feature_by_slug_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_feature_by_slug(&conn, "nope").unwrap().is_none());
    }

    #[test]
    fn get_feature_by_id_works() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_feature(&conn, &sample_feature("feat-a")).unwrap();
        let fetched = get_feature_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.id, id);
        assert_eq!(fetched.slug, "feat-a");
    }

    #[test]
    fn get_feature_by_id_nonexistent_returns_none() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        assert!(get_feature_by_id(&conn, 99999).unwrap().is_none());
    }

    #[test]
    fn update_feature_state_changes_state() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_feature(&conn, &sample_feature("feat-b")).unwrap();
        update_feature_state(&conn, id, FeatureState::Implementing).unwrap();
        let fetched = get_feature_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.state, FeatureState::Implementing);
    }

    #[test]
    fn update_feature_modifies_fields() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let id = create_feature(&conn, &sample_feature("old-slug")).unwrap();
        let mut f = sample_feature("new-slug");
        f.id = id;
        f.friendly_name = "Updated Name".to_string();
        f.state = FeatureState::Planned;
        f.labels = vec!["urgent".into(), "backend".into()];
        f.target_branch = "develop".to_string();
        update_feature(&conn, &f).unwrap();
        let fetched = get_feature_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(fetched.slug, "new-slug");
        assert_eq!(fetched.friendly_name, "Updated Name");
        assert_eq!(fetched.state, FeatureState::Planned);
        assert_eq!(fetched.labels, vec!["urgent", "backend"]);
        assert_eq!(fetched.target_branch, "develop");
    }

    #[test]
    fn list_features_by_state_filters() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_feature(&conn, &sample_feature("a")).unwrap();
        let mut planned = sample_feature("b");
        planned.state = FeatureState::Planned;
        create_feature(&conn, &planned).unwrap();

        let created = list_features_by_state(&conn, FeatureState::Created).unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].slug, "a");

        let planned_list = list_features_by_state(&conn, FeatureState::Planned).unwrap();
        assert_eq!(planned_list.len(), 1);
        assert_eq!(planned_list[0].slug, "b");
    }

    #[test]
    fn list_all_features_returns_all() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        create_feature(&conn, &sample_feature("a")).unwrap();
        create_feature(&conn, &sample_feature("b")).unwrap();
        let all = list_all_features(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn list_features_by_label_filters() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let mut f1 = sample_feature("tagged");
        f1.labels = vec!["security".into()];
        create_feature(&conn, &f1).unwrap();
        create_feature(&conn, &sample_feature("untagged")).unwrap();

        let result = list_features_by_label(&conn, "security").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].slug, "tagged");
    }

    #[test]
    fn state_str_roundtrips() {
        for s in [
            "created",
            "specified",
            "researched",
            "planned",
            "implementing",
            "validated",
            "shipped",
            "retrospected",
        ] {
            let state: FeatureState = s.parse().unwrap();
            assert_eq!(state_str(state), s);
        }
    }

    #[test]
    fn labels_json_roundtrips() {
        let labels = vec!["a".to_string(), "b".to_string()];
        let json = labels_to_json(&labels);
        let back = labels_from_json(&json);
        assert_eq!(back, labels);
    }

    #[test]
    fn labels_from_json_invalid_returns_empty() {
        assert!(labels_from_json("not-json").is_empty());
    }

    /// Insert a feature row directly so a corrupted column can be read back.
    fn insert_raw_feature(
        conn: &Connection,
        slug: &str,
        state: &str,
        created_at: &str,
        updated_at: &str,
    ) {
        conn.execute(
            "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
             VALUES (?1, ?2, ?3, X'00', 'main', ?4, ?5)",
            params![slug, format!("Feature {slug}"), state, created_at, updated_at],
        )
        .expect("raw feature insert");
    }

    #[test]
    fn short_spec_hash_reads_as_zeroed_32_bytes() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        insert_raw_feature(&conn, "short-hash", "created", &now, &now);

        let feature = get_feature_by_slug(&conn, "short-hash").unwrap().unwrap();
        assert_eq!(feature.spec_hash, [0u8; 32]);
    }

    #[test]
    fn corrupt_state_column_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        // Bypass the state CHECK constraint to simulate a row written by an
        // older or mismatched schema.
        conn.execute_batch("PRAGMA ignore_check_constraints=ON;")
            .unwrap();
        insert_raw_feature(&conn, "bad-state", "bogus", &now, &now);

        let err = get_feature_by_slug(&conn, "bad-state").unwrap_err();
        assert!(
            matches!(err, DomainError::Storage(_)),
            "expected Storage error, got {err:?}"
        );
    }

    #[test]
    fn corrupt_created_at_column_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        insert_raw_feature(&conn, "bad-created", "created", "not-a-timestamp", &now);

        let err = get_feature_by_slug(&conn, "bad-created").unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
    }

    #[test]
    fn corrupt_updated_at_column_is_storage_error() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        insert_raw_feature(&conn, "bad-updated", "created", &now, "not-a-timestamp");

        let err = list_all_features(&conn).unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
    }

    #[test]
    fn module_id_and_labels_columns_are_read_back() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let conn = adapter.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO modules (id, slug, friendly_name, created_at, updated_at)
             VALUES (7, 'mod-7', 'Module 7', ?1, ?1)",
            params![now],
        )
        .unwrap();
        let id = create_feature(&conn, &sample_feature("modular")).unwrap();
        conn.execute(
            "UPDATE features SET module_id = 7, labels = '[\"a\",\"b\"]' WHERE id = ?1",
            params![id],
        )
        .unwrap();

        let feature = get_feature_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(feature.module_id, Some(7));
        assert_eq!(feature.labels, vec!["a".to_string(), "b".to_string()]);
    }
}
