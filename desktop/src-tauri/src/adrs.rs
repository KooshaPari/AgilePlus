// SPDX-License-Identifier: MIT OR Apache-2.0
//! ADR (Architecture Decision Record) filesystem commands.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdrSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub path: String,
}

#[tauri::command]
pub fn list_adrs(state: State<'_, AppState>) -> Result<Vec<AdrSummary>, String> {
    let repo_path = state.repo_path();
    let adr_dir = PathBuf::from(&repo_path).join("docs").join("adr");

    if !adr_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<AdrSummary> = Vec::new();

    let dir_entries =
        fs::read_dir(&adr_dir).map_err(|e| format!("Failed to read ADR directory: {e}"))?;

    for entry in dir_entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Parse "NNNN-title" filename pattern
        let (id, title) = match file_name.split_once('-') {
            Some((num, rest)) if num.chars().all(|c| c.is_ascii_digit()) => {
                (num.to_string(), rest.replace('-', " "))
            }
            _ => (file_name.clone(), file_name.clone()),
        };

        let status = extract_adr_status(&path).unwrap_or_else(|| "unknown".to_string());

        entries.push(AdrSummary {
            id,
            title,
            status,
            path: path.to_string_lossy().to_string(),
        });
    }

    // Sort by id descending (newest first)
    entries.sort_by(|a, b| b.id.cmp(&a.id));

    Ok(entries)
}

#[tauri::command]
pub fn read_adr(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let repo_path = state.repo_path();
    let adr_dir = PathBuf::from(&repo_path).join("docs").join("adr");

    if !adr_dir.exists() {
        return Err(format!("ADR directory not found: {}", adr_dir.display()));
    }

    // Find file matching the id prefix
    let entries =
        fs::read_dir(&adr_dir).map_err(|e| format!("Failed to read ADR directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        if let Some((num, _)) = file_name.split_once('-') {
            if num == id {
                return fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read ADR file: {e}"));
            }
        }
    }

    Err(format!("ADR with id '{id}' not found"))
}

/// Extract the ADR status from the file content by scanning for a "Status:" line.
fn extract_adr_status(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Status:") {
            return Some(value.trim().to_string());
        }
    }

    None
}
