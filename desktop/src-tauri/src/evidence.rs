// SPDX-License-Identifier: MIT OR Apache-2.0
//! Evidence CRUD commands.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub feature_id: String,
    pub work_package_id: Option<String>,
    pub evidence_type: String,
    pub content: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub fn list_evidence(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<Evidence>, String> {
    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, work_package_id, evidence_type, content, created_at
             FROM evidence
             WHERE feature_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([feature_id], |row| {
            Ok(Evidence {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                work_package_id: row.get(2)?,
                evidence_type: row.get(3)?,
                content: row.get(4)?,
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
    feature_id: String,
    work_package_id: Option<String>,
    evidence_type: String,
    content: Option<String>,
) -> Result<Evidence, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    {
        let conn_guard = state.db_connection()?;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        conn.execute(
            "INSERT INTO evidence \
             (id, feature_id, work_package_id, evidence_type, content, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, feature_id, work_package_id, evidence_type, content, now],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(Evidence {
        id,
        feature_id,
        work_package_id,
        evidence_type,
        content,
        created_at: now,
    })
}
