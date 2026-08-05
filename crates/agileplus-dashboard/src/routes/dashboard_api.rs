//! Dashboard JSON and server-sent-event endpoints.

use std::{convert::Infallible, path::PathBuf};

use axum::{
    extract::State,
    response::{IntoResponse, sse::{Event, Sse}},
};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, interval};

use crate::app_state::SharedState;

/// JSON response for GET /api/dashboard/work-packages.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackageJson {
    pub id: String,
    pub feature_id: i64,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assignee: Option<String>,
}

async fn stream_payloads(state: &SharedState) -> (String, String) {
    let store = state.read().await;
    let heartbeat = serde_json::json!({
        "type": "heartbeat",
        "features": store.features.len(),
    })
    .to_string();
    let healthy_count = store.health.iter().filter(|service| service.healthy).count();
    let total_count = store.health.len();
    let health = serde_json::json!({
        "healthy": healthy_count,
        "total": total_count,
        "all_healthy": healthy_count == total_count,
    })
    .to_string();

    (heartbeat, health)
}

/// GET /api/stream.
///
/// Streams real-time feature and health updates, broadcasting a heartbeat every
/// five seconds.
pub async fn sse_stream(
    State(state): State<SharedState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let state = state.clone();
    let stream = async_stream::stream! {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            let (heartbeat, health) = stream_payloads(&state).await;
            yield Ok(Event::default().event("feature_updated").data(heartbeat));
            yield Ok(Event::default().event("health_changed").data(health));
        }
    };
    Sse::new(stream)
}

/// GET /api/dashboard/work-packages.json.
///
/// Returns all work packages across all features as a flat JSON array.
pub async fn all_work_packages_json(State(state): State<SharedState>) -> impl IntoResponse {
    let store = state.read().await;
    let work_packages: Vec<WorkPackageJson> = store
        .work_packages
        .iter()
        .flat_map(|(feature_id, work_packages)| {
            work_packages.iter().map(|work_package| {
                let status = match work_package.state {
                    agileplus_domain::domain::work_package::WpState::Planned => "planned",
                    agileplus_domain::domain::work_package::WpState::Doing
                    | agileplus_domain::domain::work_package::WpState::Review => "in_progress",
                    agileplus_domain::domain::work_package::WpState::Done => "completed",
                    agileplus_domain::domain::work_package::WpState::Blocked => "blocked",
                };
                WorkPackageJson {
                    id: work_package.id.to_string(),
                    feature_id: *feature_id,
                    title: work_package.title.clone(),
                    status: status.to_string(),
                    priority: "medium".to_string(),
                    assignee: work_package.agent_id.clone(),
                }
            })
        })
        .collect();

    axum::Json(serde_json::json!({
        "work_packages": work_packages,
        "count": work_packages.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// GET /api/dashboard/epics-stories.json.
///
/// Reads epics and stories from SQLite and returns them as a flat JSON payload.
pub async fn epics_stories_json() -> impl IntoResponse {
    let db_path: PathBuf = if let Ok(url) = std::env::var("DATABASE_URL") {
        url.strip_prefix("sqlite:").unwrap_or(&url).into()
    } else if let Ok(path) = std::env::var("DATABASE_PATH") {
        PathBuf::from(path)
    } else {
        PathBuf::from("agileplus.db")
    };

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(connection) => connection,
        Err(error) => {
            return axum::Json(serde_json::json!({
                "epics": [],
                "stories": [],
                "epic_count": 0,
                "story_count": 0,
                "error": format!("db open failed: {error}"),
            }));
        }
    };

    let epics: Vec<serde_json::Value> = {
        let mut statement = conn
            .prepare("SELECT id, title, status, requirement_id FROM epics ORDER BY id")
            .unwrap_or_else(|_| conn.prepare("SELECT 1 WHERE 0").unwrap());
        statement
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0).unwrap_or(0),
                    "title": row.get::<_, String>(1).unwrap_or_default(),
                    "status": row.get::<_, String>(2).unwrap_or_default(),
                    "requirement_id": row.get::<_, Option<String>>(3).unwrap_or(None),
                }))
            })
            .map(|rows| rows.filter_map(|row| row.ok()).collect())
            .unwrap_or_default()
    };

    let stories: Vec<serde_json::Value> = {
        let mut statement = conn
            .prepare("SELECT id, epic_id, title, status, requirement_id FROM stories ORDER BY id")
            .unwrap_or_else(|_| conn.prepare("SELECT 1 WHERE 0").unwrap());
        statement
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0).unwrap_or(0),
                    "epic_id": row.get::<_, Option<i64>>(1).unwrap_or(None),
                    "title": row.get::<_, String>(2).unwrap_or_default(),
                    "status": row.get::<_, String>(3).unwrap_or_default(),
                    "requirement_id": row.get::<_, Option<String>>(4).unwrap_or(None),
                }))
            })
            .map(|rows| rows.filter_map(|row| row.ok()).collect())
            .unwrap_or_default()
    };

    axum::Json(serde_json::json!({
        "epic_count": epics.len(),
        "story_count": stories.len(),
        "epics": epics,
        "stories": stories,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::to_bytes,
        extract::State,
        response::IntoResponse,
    };
    use tokio::sync::RwLock;

    use super::{all_work_packages_json, epics_stories_json, stream_payloads};
    use crate::app_state::{DashboardStore, SharedState};

    fn seeded_state() -> SharedState {
        Arc::new(RwLock::new(DashboardStore::seeded()))
    }

    #[tokio::test]
    async fn work_packages_json_reports_seeded_work_package_count() {
        let response = all_work_packages_json(State(seeded_state())).await.into_response();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let work_packages = body["work_packages"].as_array().unwrap();
        assert!(!work_packages.is_empty());
        assert_eq!(body["count"].as_u64(), Some(work_packages.len() as u64));
        assert!(work_packages.iter().all(|work_package| {
            work_package["id"].is_string()
                && work_package["feature_id"].is_i64()
                && work_package["status"].is_string()
        }));
    }

    #[tokio::test]
    async fn epics_stories_json_always_returns_collection_contract() {
        let response = epics_stories_json().await.into_response();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert!(body["epics"].is_array());
        assert!(body["stories"].is_array());
        assert!(body["epic_count"].is_u64());
        assert!(body["story_count"].is_u64());
        assert!(body["timestamp"].is_string());
    }

    #[tokio::test]
    async fn sse_payloads_report_seeded_feature_and_health_counts() {
        let state = seeded_state();
        let (expected_features, expected_total, expected_healthy) = {
            let store = state.read().await;
            (
                store.features.len() as u64,
                store.health.len() as u64,
                store.health.iter().filter(|service| service.healthy).count() as u64,
            )
        };
        let (heartbeat, health) = stream_payloads(&state).await;
        let heartbeat: serde_json::Value = serde_json::from_str(&heartbeat).unwrap();
        let health: serde_json::Value = serde_json::from_str(&health).unwrap();

        assert_eq!(heartbeat["type"], "heartbeat");
        assert_eq!(heartbeat["features"].as_u64(), Some(expected_features));
        assert_eq!(health["total"].as_u64(), Some(expected_total));
        assert_eq!(health["healthy"].as_u64(), Some(expected_healthy));
        assert_eq!(health["all_healthy"], expected_total == expected_healthy);
    }
}
