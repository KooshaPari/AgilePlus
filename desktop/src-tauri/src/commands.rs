use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

// ── Models matching CLI's .agileplus schema ──────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub state: String,
    pub target_branch: String,
    pub created_at: String,
    pub updated_at: String,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub id: i64,
    pub wp_id: i64,
    pub fr_id: String,
    pub evidence_type: String,
    pub artifact_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureWithDetails {
    pub feature: Feature,
    pub work_packages: Vec<WorkPackage>,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_features: i64,
    pub features_by_state: std::collections::HashMap<String, i64>,
    pub total_work_packages: i64,
    pub work_packages_by_state: std::collections::HashMap<String, i64>,
}

// ── Feature commands ─────────────────────────────────────────────────

#[tauri::command]
pub fn list_features(state: State<'_, AppState>) -> Result<Vec<Feature>, String> {
    let conn = state.db_connection()?;
    let conn = conn.as_ref().ok_or("Database not initialized. Open a project first.")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, target_branch, created_at, updated_at
             FROM features
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let features = stmt
        .query_map([], |row| {
            Ok(Feature {
                id: row.get(0)?,
                slug: row.get(1)?,
                friendly_name: row.get(2)?,
                state: row.get(3)?,
                target_branch: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(features)
}

#[tauri::command]
pub fn get_feature(state: State<'_, AppState>, id: i64) -> Result<Option<Feature>, String> {
    let conn = state.db_connection()?;
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, target_branch, created_at, updated_at
             FROM features WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut features = stmt
        .query_map([id], |row| {
            Ok(Feature {
                id: row.get(0)?,
                slug: row.get(1)?,
                friendly_name: row.get(2)?,
                state: row.get(3)?,
                target_branch: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    features.next().transpose().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_feature_with_details(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<FeatureWithDetails>, String> {
    let feature = get_feature(state.clone(), id)?
        .ok_or_else(|| format!("Feature '{id}' not found"))?;

    let conn = state.db_connection()?;
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    // Fetch work packages
    let mut wp_stmt = conn
        .prepare(
            "SELECT id, feature_id, title, state, sequence, acceptance_criteria,
                    pr_url, pr_state, created_at, updated_at
             FROM work_packages WHERE feature_id = ?1 ORDER BY sequence ASC",
        )
        .map_err(|e| e.to_string())?;

    let work_packages: Vec<WorkPackage> = wp_stmt
        .query_map([id], |row| {
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

    // Fetch evidence (through work packages)
    let mut ev_stmt = conn
        .prepare(
            "SELECT e.id, e.wp_id, e.fr_id, e.evidence_type, e.artifact_path, e.created_at
             FROM evidence e
             INNER JOIN work_packages wp ON e.wp_id = wp.id
             WHERE wp.feature_id = ?1
             ORDER BY e.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let evidence: Vec<Evidence> = ev_stmt
        .query_map([id], |row| {
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

    Ok(Some(FeatureWithDetails {
        feature,
        work_packages,
        evidence,
    }))
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let conn = state.db_connection()?;
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT state, COUNT(*) FROM features GROUP BY state")
        .map_err(|e| e.to_string())?;

    let features_by_state: std::collections::HashMap<String, i64> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let total_features: i64 = features_by_state.values().sum();

    let mut stmt = conn
        .prepare("SELECT state, COUNT(*) FROM work_packages GROUP BY state")
        .map_err(|e| e.to_string())?;

    let work_packages_by_state: std::collections::HashMap<String, i64> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let total_work_packages: i64 = work_packages_by_state.values().sum();

    Ok(DashboardStats {
        total_features,
        features_by_state,
        total_work_packages,
        work_packages_by_state,
    })
}

// ── Repo/project commands ────────────────────────────────────────────

#[tauri::command]
pub fn set_repo_path(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state.set_repo_path(path);
    Ok(())
}

#[tauri::command]
pub fn get_repo_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.repo_path())
}

/// Open a project directory: find .agileplus/agileplus.db and connect.
#[tauri::command]
pub fn open_project(state: State<'_, AppState>, path: String) -> Result<Feature, String> {
    let project_root = std::path::PathBuf::from(&path);

    // Check for .agileplus/agileplus.db
    let db_path = project_root.join(".agileplus").join("agileplus.db");
    if !db_path.exists() {
        return Err(format!(
            "No .agileplus/agileplus.db found in {path}. Is this an AgilePlus project?"
        ));
    }

    // Open the connection
    let conn = crate::db::open_project_db(&project_root)?;

    // Store in state
    {
        let mut db_guard = state
            .db_connection()
            .map_err(|e| e.to_string())?;
        *db_guard = Some(conn);
    }
    state.set_repo_path(path);

    // Return stats as proof of life
    Ok(Feature {
        id: 0,
        slug: "connected".into(),
        friendly_name: "Project connected".into(),
        state: "shipped".into(),
        target_branch: "main".into(),
        created_at: String::new(),
        updated_at: String::new(),
    })
}
