// SPDX-License-Identifier: MIT OR Apache-2.0
//! Evidence projection commands for the desktop client.
//!
//! Reads use a query-only SQLite connection. Evidence creation must use the
//! canonical CLI/MCP/device-service path so provenance, digests, execution
//! receipts, governance, and acceptance semantics cannot be bypassed.

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

fn mutation_disabled<T>(operation: &str) -> Result<T, String> {
    Err(crate::db::direct_mutation_disabled(operation))
}

#[tauri::command]
pub fn list_evidence(state: State<'_, AppState>, feature_id: i64) -> Result<Vec<Evidence>, String> {
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

    let evidence = stmt
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
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(evidence)
}

#[tauri::command]
pub fn create_evidence(
    _state: State<'_, AppState>,
    _wp_id: i64,
    _fr_id: String,
    _evidence_type: String,
    _artifact_path: String,
) -> Result<Evidence, String> {
    mutation_disabled("create_evidence")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_mutation_fails_closed() {
        let error = mutation_disabled::<Evidence>("create_evidence")
            .expect_err("direct evidence creation must be rejected");
        assert!(error.contains("create_evidence"));
        assert!(error.contains("evidence"));
        assert!(error.contains("acceptance"));
    }
}
