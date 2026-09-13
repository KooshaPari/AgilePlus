use serde::{Deserialize, Serialize};
use tauri::State;

use crate::DatabaseState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_features: i64,
    pub features_by_state: std::collections::HashMap<String, i64>,
    pub total_work_packages: i64,
    pub work_packages_by_state: std::collections::HashMap<String, i64>,
}

#[tauri::command]
pub fn list_features(state: State<'_, DatabaseState>) -> Result<Vec<Feature>, String> {
    let conn = state.0.lock().unwrap();
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT id, name, description, state, created_at, updated_at FROM features ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let features = stmt
        .query_map([], |row| {
            Ok(Feature {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                state: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(features)
}

#[tauri::command]
pub fn get_feature(state: State<'_, DatabaseState>, id: String) -> Result<Option<Feature>, String> {
    let conn = state.0.lock().unwrap();
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT id, name, description, state, created_at, updated_at FROM features WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut features = stmt
        .query_map([id], |row| {
            Ok(Feature {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                state: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    features.next().transpose().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_feature(
    state: State<'_, DatabaseState>,
    name: String,
    description: Option<String>,
) -> Result<Feature, String> {
    let conn = state.0.lock().unwrap();
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO features (id, name, description, state, created_at, updated_at) VALUES (?1, ?2, ?3, 'created', ?4, ?5)",
        rusqlite::params![id, name, description, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Feature {
        id,
        name,
        description,
        state: "created".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_feature_state(
    state: State<'_, DatabaseState>,
    id: String,
    new_state: String,
) -> Result<Feature, String> {
    {
        let conn = state.0.lock().unwrap();
        let conn = conn.as_ref().ok_or("Database not initialized")?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE features SET state = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_state, now, id],
        )
        .map_err(|e| e.to_string())?;
    }
    // conn dropped here, releasing the borrow on state

    // Return updated feature
    get_feature(state, id)?.ok_or_else(|| "Feature not found after update".to_string())
}

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, DatabaseState>) -> Result<DashboardStats, String> {
    let conn = state.0.lock().unwrap();
    let conn = conn.as_ref().ok_or("Database not initialized")?;

    // Count features by state
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

    // Count work packages by state
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
