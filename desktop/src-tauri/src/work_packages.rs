// SPDX-License-Identifier: MIT OR Apache-2.0
//! Work package CRUD commands — reads from CLI's .agileplus schema.

use rusqlite::OptionalExtension;
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
        update_work_package_state_in(conn, id, &new_state, &now)?;
    }

    let conn_guard = state.db_connection()?;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

    query_work_package(conn, id)
}

pub fn update_work_package_state_in(
    conn: &rusqlite::Connection,
    id: i64,
    new_state: &str,
    now: &str,
) -> Result<(), String> {
    // Terminal and unknown requests must go through the canonical lifecycle
    // writer. The desktop command is deliberately not an alternate acceptor.
    const ALLOWED_NON_TERMINAL_STATES: [&str; 3] = ["planned", "doing", "review"];
    if !ALLOWED_NON_TERMINAL_STATES.contains(&new_state) {
        return Err(format!(
            "Desktop state '{new_state}' is terminal or unsupported; use the canonical lifecycle command"
        ));
    }

    // An accepted terminal work package is immutable from the desktop. The
    // guard belongs on the write itself, not on a prior SELECT: canonical
    // acceptance can commit `done` between a read and a write, and an
    // unconditional UPDATE would then reopen the accepted package.
    //
    // `blocked` is deliberately NOT terminal here: `implement` assigns it when
    // a work package exhausts its review cycles, and an operator retrying that
    // work needs the desktop to move it back to `planned`/`doing`. Only `done`
    // carries an acceptance receipt that a desktop write would invalidate.
    const TERMINAL_STATES: [&str; 1] = ["done"];
    let updated = conn
        .execute(
            "UPDATE work_packages
             SET state = ?1, updated_at = ?2
             WHERE id = ?3 AND state NOT IN ('done')",
            rusqlite::params![new_state, now, id],
        )
        .map_err(|e| e.to_string())?;
    if updated > 0 {
        return Ok(());
    }

    // Nothing was written: distinguish "no such package" from "terminal state".
    let current_state: Option<String> = conn
        .query_row(
            "SELECT state FROM work_packages WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    match current_state.as_deref() {
        None => Err(format!("Work package {id} not found")),
        Some(state) if TERMINAL_STATES.contains(&state) => Err(format!(
            "Work package {id} is in terminal state '{state}' and cannot be mutated from the desktop"
        )),
        Some(state) => Err(format!(
            "Work package {id} state changed concurrently (now '{state}'); retry"
        )),
    }
}

fn query_work_package(conn: &rusqlite::Connection, id: i64) -> Result<WorkPackage, String> {
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
