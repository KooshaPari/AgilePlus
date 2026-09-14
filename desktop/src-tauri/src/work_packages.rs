// SPDX-License-Identifier: MIT OR Apache-2.0
//! Work package CRUD commands — reads from CLI's .agileplus schema.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub state: String,
    pub sequence: i64,
    pub acceptance_criteria: String,
    pub pr_url: Option<String>,
    pub pr_state: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn list_work_packages(
    state: State<'_, AppState>,
    feature_id: i64,
) -> Result<Vec<WorkPackage>, String> {
    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, title, state, sequence, acceptance_criteria,
                    pr_url, pr_state, created_at, updated_at
             FROM work_packages
             WHERE feature_id = ?1
             ORDER BY sequence ASC",
        )
        .map_err(|e| e.to_string())?;

    let packages = stmt
        .query_map([feature_id], |row| {
            Ok(WorkPackage {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                title: row.get(2)?,
                state: row.get(3)?,
                sequence: row.get(4)?,
                acceptance_criteria: row.get(5)?,
                pr_url: row.get(6)?,
                pr_state: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
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
    feature_id: i64,
    title: String,
    acceptance_criteria: Option<String>,
) -> Result<WorkPackage, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let ac = acceptance_criteria.unwrap_or_default();

    let id: i64 = {
        let conn_guard = state.db_connection()?;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        conn.execute(
            "INSERT INTO work_packages \
             (feature_id, title, state, sequence, acceptance_criteria, created_at, updated_at) \
             VALUES (?1, ?2, 'planned', 0, ?3, ?4, ?5)",
            rusqlite::params![feature_id, title, ac, now, now],
        )
        .map_err(|e| e.to_string())?;
        conn.last_insert_rowid()
    };

    Ok(WorkPackage {
        id,
        feature_id,
        title,
        state: "planned".to_string(),
        sequence: 0,
        acceptance_criteria: ac,
        pr_url: None,
        pr_state: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_work_package_state(
    state: State<'_, AppState>,
    id: i64,
    new_state: String,
) -> Result<WorkPackage, String> {
    let now = chrono::Utc::now().to_rfc3339();

    {
        let conn_guard = state.db_connection()?;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        conn.execute(
            "UPDATE work_packages SET state = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_state, now, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    conn.query_row(
        "SELECT id, feature_id, title, state, sequence, acceptance_criteria,
                pr_url, pr_state, created_at, updated_at
         FROM work_packages WHERE id = ?1",
        [id],
        |row| {
            Ok(WorkPackage {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                title: row.get(2)?,
                state: row.get(3)?,
                sequence: row.get(4)?,
                acceptance_criteria: row.get(5)?,
                pr_url: row.get(6)?,
                pr_state: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}
