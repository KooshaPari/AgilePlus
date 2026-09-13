// SPDX-License-Identifier: MIT OR Apache-2.0
//! Work package CRUD commands.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: String,
    pub feature_id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn list_work_packages(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<WorkPackage>, String> {
    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, name, description, state, created_at, updated_at
             FROM work_packages
             WHERE feature_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let packages = stmt
        .query_map([feature_id], |row| {
            Ok(WorkPackage {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                state: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(packages)
}

#[tauri::command]
pub fn create_work_package(
    state: State<'_, AppState>,
    feature_id: String,
    name: String,
    description: Option<String>,
) -> Result<WorkPackage, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    {
        let conn_guard = state.db_connection()?;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        conn.execute(
            "INSERT INTO work_packages \
             (id, feature_id, name, description, state, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6)",
            rusqlite::params![id, feature_id, name, description, now, now],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(WorkPackage {
        id,
        feature_id,
        name,
        description,
        state: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_work_package_state(
    state: State<'_, AppState>,
    id: String,
    new_state: String,
) -> Result<WorkPackage, String> {
    {
        let conn_guard = state.db_connection()?;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE work_packages SET state = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_state, now, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, name, description, state, created_at, updated_at
             FROM work_packages WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], |row| {
        Ok(WorkPackage {
            id: row.get(0)?,
            feature_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            state: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })
    .map_err(|e| e.to_string())
}
