// SPDX-License-Identifier: MIT OR Apache-2.0
//! Trace filesystem commands for browsing trace files.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceSummary {
    pub name: String,
    pub file_type: String,
    pub path: String,
}

#[tauri::command]
pub fn list_traces(state: State<'_, AppState>) -> Result<Vec<TraceSummary>, String> {
    let repo_path = state.repo_path();
    let traces_dir = PathBuf::from(&repo_path).join("traces");

    if !traces_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<TraceSummary> = Vec::new();

    let dir_entries =
        fs::read_dir(&traces_dir).map_err(|e| format!("Failed to read traces directory: {e}"))?;

    for entry in dir_entries.flatten() {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Only include .jsonl and .md files
        if ext != "jsonl" && ext != "md" {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        entries.push(TraceSummary {
            name,
            file_type: ext,
            path: path.to_string_lossy().to_string(),
        });
    }

    // Sort by name descending
    entries.sort_by(|a, b| b.name.cmp(&a.name));

    Ok(entries)
}

#[tauri::command]
pub fn read_trace(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let repo_path = state.repo_path();
    let traces_dir = PathBuf::from(&repo_path).join("traces");

    if !traces_dir.exists() {
        return Err(format!(
            "Traces directory not found: {}",
            traces_dir.display()
        ));
    }

    // Try .jsonl first, then .md
    let jsonl_path = traces_dir.join(format!("{name}.jsonl"));
    let md_path = traces_dir.join(format!("{name}.md"));

    if jsonl_path.exists() {
        return fs::read_to_string(&jsonl_path)
            .map_err(|e| format!("Failed to read trace file: {e}"));
    }

    if md_path.exists() {
        return fs::read_to_string(&md_path).map_err(|e| format!("Failed to read trace file: {e}"));
    }

    Err(format!("Trace '{name}' not found"))
}
