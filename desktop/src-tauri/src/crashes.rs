//! Crash log management for the desktop app.
//!
//! Provides Tauri commands to list and clear locally-stored crash reports.
//! Crash files are written by the panic hook in `lib.rs` and stored under
//! the platform data directory at `com.phenotype.agileplus-desktop/crashes/`.

use serde::{Deserialize, Serialize};

/// A single crash report extracted from a `.log` file.
#[derive(Debug, Serialize, Deserialize)]
pub struct CrashReport {
    pub filename: String,
    pub timestamp: String,
    pub payload: String,
    pub location: String,
}

/// Return the path to the crash logs directory.
fn crash_dir() -> Option<std::path::PathBuf> {
    dirs_next::data_dir().map(|d| d.join("com.phenotype.agileplus-desktop").join("crashes"))
}

/// Tauri command: list all crash log files, newest first.
#[tauri::command]
pub fn list_crash_logs() -> Result<Vec<CrashReport>, String> {
    let crash_dir = crash_dir().ok_or("Could not find data directory")?;

    if !crash_dir.exists() {
        return Ok(vec![]);
    }

    let mut reports = Vec::new();
    for entry in
        std::fs::read_dir(&crash_dir).map_err(|e| format!("Failed to read crash dir: {e}"))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let filename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let timestamp = extract_field(&content, "Time:").unwrap_or_default();
                let payload = extract_field(&content, "Payload:").unwrap_or_default();
                let location = extract_field(&content, "Location:").unwrap_or_default();

                reports.push(CrashReport {
                    filename,
                    timestamp,
                    payload,
                    location,
                });
            }
        }
    }

    reports.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(reports)
}

/// Tauri command: delete all crash log files.
#[tauri::command]
pub fn clear_crash_logs() -> Result<(), String> {
    let crash_dir = crash_dir().ok_or("Could not find data directory")?;

    if crash_dir.exists() {
        std::fs::remove_dir_all(&crash_dir)
            .map_err(|e| format!("Failed to clear crash logs: {e}"))?;
    }
    Ok(())
}

/// Parse a `Field: value` line from a crash log.
fn extract_field(content: &str, field: &str) -> Option<String> {
    content
        .lines()
        .find(|line| line.contains(field))
        .map(|line| line.trim_start_matches(field).trim().to_string())
}
