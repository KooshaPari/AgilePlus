use rusqlite::Row;

use agileplus_domain::domain::{feature::Feature, module::Module, state_machine::FeatureState};

pub(super) fn row_to_module(row: &Row<'_>) -> rusqlite::Result<Module> {
    let id: i64 = row.get(0)?;
    let slug: String = row.get(1)?;
    let friendly_name: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let parent_module_id: Option<i64> = row.get(4)?;
    let created_at_str: String = row.get(5)?;
    let updated_at_str: String = row.get(6)?;

    let created_at = created_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
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
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    Ok(Module {
        id,
        slug,
        friendly_name,
        description,
        parent_module_id,
        created_at,
        updated_at,
    })
}

pub(super) fn row_to_feature(row: &Row<'_>) -> rusqlite::Result<Feature> {
    let id: i64 = row.get(0)?;
    let slug: String = row.get(1)?;
    let friendly_name: String = row.get(2)?;
    let state_str: String = row.get(3)?;
    let spec_hash_bytes: Vec<u8> = row.get(4)?;
    let target_branch: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;

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
        labels: Vec::new(),
        module_id: None,
        project_id: None,
        created_at_commit: None,
        last_modified_commit: None,
        created_at,
        updated_at,
    })
}

pub(super) trait OptionalExt<T> {
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
    use rusqlite::params;

    use agileplus_domain::error::DomainError;

    use crate::SqliteStorageAdapter;
    use crate::repository::modules::{get_module, get_module_with_features};

    fn adapter() -> SqliteStorageAdapter {
        SqliteStorageAdapter::in_memory().expect("in-memory adapter")
    }

    #[test]
    fn module_with_corrupt_created_at_is_storage_error() {
        let a = adapter();
        let conn = a.conn_for_bench().unwrap();
        conn.execute(
            "INSERT INTO modules (id, slug, friendly_name, created_at, updated_at)
             VALUES (1, 'm', 'M', 'not-a-timestamp', ?1)",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();

        let err = get_module(&conn, 1).unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
    }

    #[test]
    fn module_with_corrupt_updated_at_is_storage_error() {
        let a = adapter();
        let conn = a.conn_for_bench().unwrap();
        conn.execute(
            "INSERT INTO modules (id, slug, friendly_name, created_at, updated_at)
             VALUES (1, 'm', 'M', ?1, 'not-a-timestamp')",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();

        let err = get_module(&conn, 1).unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
    }

    #[test]
    fn module_feature_with_short_spec_hash_reads_as_zeroed_bytes() {
        let a = adapter();
        let conn = a.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO modules (id, slug, friendly_name, created_at, updated_at)
             VALUES (1, 'm', 'M', ?1, ?1)",
            params![now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, module_id, created_at, updated_at)
             VALUES (1, 'owned', 'Owned', 'created', X'00', 'main', 1, ?1, ?1)",
            params![now],
        )
        .unwrap();

        let view = get_module_with_features(&conn, 1).unwrap().unwrap();
        assert_eq!(view.owned_features.len(), 1);
        assert_eq!(view.owned_features[0].spec_hash, [0u8; 32]);
        assert!(view.tagged_features.is_empty());
    }

    /// Seed a module owning one feature, corrupt a feature column, and read the
    /// module view back so `row_to_feature` is exercised on the bad row.
    ///
    /// `ignore_check_constraints` is applied on the same connection before the
    /// corruption, because the pragma is connection scoped.
    fn module_with_corrupt_feature(sql: &str, ignore_check_constraints: bool) -> DomainError {
        let a = adapter();
        let conn = a.conn_for_bench().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO modules (id, slug, friendly_name, created_at, updated_at)
             VALUES (1, 'm', 'M', ?1, ?1)",
            params![now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, module_id, created_at, updated_at)
             VALUES (1, 'owned', 'Owned', 'created', X'00', 'main', 1, ?1, ?1)",
            params![now],
        )
        .unwrap();
        if ignore_check_constraints {
            conn.execute_batch("PRAGMA ignore_check_constraints=ON;")
                .unwrap();
        }
        conn.execute(sql, params![]).expect("corrupt feature");

        get_module_with_features(&conn, 1).unwrap_err()
    }

    #[test]
    fn module_feature_with_corrupt_state_is_storage_error() {
        let err = module_with_corrupt_feature(
            "UPDATE features SET state = 'bogus' WHERE id = 1",
            true,
        );
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
        assert!(
            err.to_string().contains("bogus"),
            "the unparsable feature state must surface: {err}"
        );
    }

    #[test]
    fn module_feature_with_corrupt_created_at_is_storage_error() {
        let err = module_with_corrupt_feature(
            "UPDATE features SET created_at = 'not-a-timestamp' WHERE id = 1",
            false,
        );
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
    }

    #[test]
    fn module_feature_with_corrupt_updated_at_is_storage_error() {
        let err = module_with_corrupt_feature(
            "UPDATE features SET updated_at = 'not-a-timestamp' WHERE id = 1",
            false,
        );
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
    }
}
