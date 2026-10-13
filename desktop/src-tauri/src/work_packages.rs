// SPDX-License-Identifier: MIT OR Apache-2.0
//! Work package projection commands for the desktop client.
//!
//! Reads use a query-only SQLite connection. Mutations remain registered for
//! IPC compatibility but fail closed until they are routed through the
//! canonical AgilePlus application/device-service boundary.

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

fn mutation_disabled<T>(operation: &str) -> Result<T, String> {
    Err(crate::db::direct_mutation_disabled(operation))
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
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(packages)
}

#[tauri::command]
pub fn create_work_package(
    _state: State<'_, AppState>,
    _feature_id: i64,
    _title: String,
    _acceptance_criteria: Option<String>,
) -> Result<WorkPackage, String> {
    mutation_disabled("create_work_package")
}

#[tauri::command]
pub fn update_work_package_state(
    _state: State<'_, AppState>,
    _id: i64,
    _new_state: String,
) -> Result<WorkPackage, String> {
    mutation_disabled("update_work_package_state")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_package_mutations_fail_closed() {
        let create = mutation_disabled::<WorkPackage>("create_work_package")
            .expect_err("direct create must be rejected");
        assert!(create.contains("create_work_package"));
        assert!(create.contains("device service"));

        let update = mutation_disabled::<WorkPackage>("update_work_package_state")
            .expect_err("direct state update must be rejected");
        assert!(update.contains("update_work_package_state"));
        assert!(update.contains("governance"));
    }
}
