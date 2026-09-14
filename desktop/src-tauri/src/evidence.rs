// SPDX-License-Identifier: MIT OR Apache-2.0
//! Evidence CRUD commands — reads from CLI's .agileplus schema.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub id: i64,
    pub wp_id: i64,
    pub fr_id: String,
    pub evidence_type: String,
    pub artifact_path: String,
    pub created_at: String,
}

#[tauri::command]
pub fn list_evidence(
    state: State<'_, AppState>,
    feature_id: i64,
) -> Result<Vec<Evidence>, String> {
    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.wp_id, e.fr_id, e.evidence_type, e.artifact_path, e.created_at
             FROM evidence e
             INNER JOIN work_packages wp ON e.wp_id = wp.id
             WHERE wp.feature_id = ?1
             ORDER BY e.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([feature_id], |row| {
            Ok(Evidence {
                id: row.get(0)?,
                wp_id: row.get(1)?,
                fr_id: row.get(2)?,
                evidence_type: row.get(3)?,
                artifact_path: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

#[tauri::command]
pub fn create_evidence(
    state: State<'_, AppState>,
    wp_id: i64,
    fr_id: String,
    evidence_type: String,
    artifact_path: String,
) -> Result<Evidence, String> {
    let now = chrono::Utc::now().to_rfc3339();

    let id: i64 = {
        let conn_guard = state.db_connection()?;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        conn.execute(
            "INSERT INTO evidence (wp_id, fr_id, evidence_type, artifact_path, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![wp_id, fr_id, evidence_type, artifact_path, now],
        )
        .map_err(|e| e.to_string())?;
        conn.last_insert_rowid()
    };

    Ok(Evidence {
        id,
        wp_id,
        fr_id,
        evidence_type,
        artifact_path,
        created_at: now,
    })
}
