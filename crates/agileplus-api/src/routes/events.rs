//! Event query route handlers.
//!
//! - GET /api/v1/events        → paginated, filtered list of domain events
//! - GET /api/v1/events/:id    → single event detail
//!
//! The StoragePort does not yet expose event querying directly — this
//! module uses the audit trail as the event source and applies in-memory
//! filters until a dedicated EventStore port is wired up.
//!
//! Traceability: WP11-T068

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::ports::{ObservabilityPort, StoragePort};

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(list_events::<S, V, O>))
        .route("/{id}", get(get_event::<S, V, O>))
}

/// Query params for the event list endpoint.
#[derive(Debug, Deserialize, IntoParams)]
pub struct EventListParams {
    /// Filter by entity type: "feature" or "work_package".
    pub entity_type: Option<String>,
    /// Filter by entity id.
    pub entity_id: Option<i64>,
    /// ISO-8601 datetime or relative duration like "1h", "24h".
    pub since: Option<String>,
    /// ISO-8601 datetime upper bound.
    pub until: Option<String>,
    /// Filter by event/transition type string.
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Filter by actor name.
    pub actor: Option<String>,
    /// Maximum number of results (default 100).
    pub limit: Option<usize>,
    /// Offset for pagination (default 0).
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub event_type: String,
    pub actor: String,
    pub timestamp: String,
    pub payload: serde_json::Value,
}

/// `GET /api/v1/events`
#[utoipa::path(
    get,
    path = "/api/v1/events",
    tag = "events",
    params(EventListParams),
    responses(
        (status = 200, description = "Paginated event list", body = [EventResponse]),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_events<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Query(params): Query<EventListParams>,
) -> Result<Json<Vec<EventResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    // Gather audit entries from all features as an event source.
    let all_features = app
        .storage
        .list_all_features()
        .await
        .map_err(ApiError::from)?;

    let mut events: Vec<EventResponse> = Vec::new();
    for feature in &all_features {
        let trail = app
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(ApiError::from)?;
        for entry in trail {
            events.push(EventResponse {
                id: entry.id,
                entity_type: "feature".to_string(),
                entity_id: entry.feature_id,
                event_type: entry.transition.clone(),
                actor: entry.actor.clone(),
                timestamp: entry.timestamp.to_rfc3339(),
                payload: serde_json::json!({
                    "wp_id": entry.wp_id,
                    "transition": entry.transition,
                }),
            });
        }
        // Include work-package events.
        let wps = app
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(ApiError::from)?;
        for wp in wps {
            events.push(EventResponse {
                id: wp.id,
                entity_type: "work_package".to_string(),
                entity_id: wp.id,
                event_type: format!("wp_{:?}", wp.state).to_lowercase(),
                actor: "system".to_string(),
                timestamp: wp.updated_at.to_rfc3339(),
                payload: serde_json::json!({
                    "title": wp.title,
                    "state": format!("{:?}", wp.state).to_lowercase(),
                }),
            });
        }
    }

    // Apply filters.
    let since_dt = params.since.as_deref().map(parse_since).transpose()?;
    let until_dt: Option<DateTime<Utc>> = params
        .until
        .as_deref()
        .map(|s| {
            s.parse::<DateTime<Utc>>()
                .map_err(|e| ApiError::BadRequest(format!("invalid until: {e}")))
        })
        .transpose()?;

    let filtered: Vec<EventResponse> = events
        .into_iter()
        .filter(|e| {
            if let Some(et) = &params.entity_type
                && &e.entity_type != et
            {
                return false;
            }
            if let Some(eid) = params.entity_id
                && e.entity_id != eid
            {
                return false;
            }
            if let Some(ev_type) = &params.event_type
                && &e.event_type != ev_type
            {
                return false;
            }
            if let Some(actor) = &params.actor
                && &e.actor != actor
            {
                return false;
            }
            if let Some(since) = since_dt
                && let Ok(ts) = e.timestamp.parse::<DateTime<Utc>>()
                && ts < since
            {
                return false;
            }
            if let Some(until) = until_dt
                && let Ok(ts) = e.timestamp.parse::<DateTime<Utc>>()
                && ts > until
            {
                return false;
            }
            true
        })
        .collect();

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100);

    let page: Vec<EventResponse> = filtered.into_iter().skip(offset).take(limit).collect();
    Ok(Json(page))
}

/// `GET /api/v1/events/:id`
pub async fn get_event<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<EventResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    // Scan audit entries across all features for the matching id.
    let all_features = app
        .storage
        .list_all_features()
        .await
        .map_err(ApiError::from)?;

    for feature in &all_features {
        let trail = app
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(ApiError::from)?;
        if let Some(entry) = trail.into_iter().find(|e| e.id == id) {
            return Ok(Json(EventResponse {
                id: entry.id,
                entity_type: "feature".to_string(),
                entity_id: entry.feature_id,
                event_type: entry.transition.clone(),
                actor: entry.actor.clone(),
                timestamp: entry.timestamp.to_rfc3339(),
                payload: serde_json::json!({
                    "wp_id": entry.wp_id,
                    "transition": entry.transition,
                    "evidence_refs": entry.evidence_refs,
                }),
            }));
        }
    }

    Err(ApiError::NotFound(format!("Event {id} not found")))
}

/// Parse a `since` parameter: ISO-8601 datetime or relative like "1h", "24h".
fn parse_since(s: &str) -> Result<DateTime<Utc>, ApiError> {
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }
    // Try relative durations like "1h", "24h", "30m".
    let parse_num = |n: &str| {
        n.parse::<i64>()
            .map_err(|_| ApiError::BadRequest(format!("invalid since: {s}")))
    };
    let duration = if let Some(n) = s.strip_suffix('h') {
        Duration::hours(parse_num(n)?)
    } else if let Some(n) = s.strip_suffix('m') {
        Duration::minutes(parse_num(n)?)
    } else {
        return Err(ApiError::BadRequest(format!("invalid since: {s}")));
    };
    Ok(Utc::now() - duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_iso8601() {
        use chrono::Datelike;
        let dt = parse_since("2025-01-15T12:00:00Z").unwrap();
        let naive = dt.naive_utc();
        assert_eq!(naive.date().year(), 2025);
        assert_eq!(naive.date().month(), 1);
        assert_eq!(naive.date().day(), 15);
    }

    #[test]
    fn parse_since_hours() {
        let before = Utc::now();
        let dt = parse_since("1h").unwrap();
        let after = Utc::now();
        // Should be roughly 1 hour ago
        assert!(dt >= before - Duration::hours(1) - Duration::seconds(5));
        assert!(dt <= after - Duration::hours(1) + Duration::seconds(5));
    }

    #[test]
    fn parse_since_24_hours() {
        let dt = parse_since("24h").unwrap();
        let now = Utc::now();
        let diff = now - dt;
        assert!(diff >= Duration::hours(23) && diff <= Duration::hours(25));
    }

    #[test]
    fn parse_since_minutes() {
        let dt = parse_since("30m").unwrap();
        let now = Utc::now();
        let diff = now - dt;
        assert!(diff >= Duration::minutes(29) && diff <= Duration::minutes(31));
    }

    #[test]
    fn parse_since_invalid_format() {
        let err = parse_since("invalid");
        assert!(err.is_err());
        match err.unwrap_err() {
            ApiError::BadRequest(msg) => assert!(msg.contains("invalid since")),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn parse_since_invalid_number() {
        let err = parse_since("abc_h");
        assert!(err.is_err());
    }

    #[test]
    fn parse_since_hours_zero() {
        let dt = parse_since("0h").unwrap();
        let now = Utc::now();
        let diff = (now - dt).num_seconds().abs();
        assert!(diff < 5);
    }

    #[test]
    fn event_list_params_defaults() {
        let params = EventListParams {
            entity_type: None,
            entity_id: None,
            since: None,
            until: None,
            event_type: None,
            actor: None,
            limit: None,
            offset: None,
        };
        assert!(params.entity_type.is_none());
        assert!(params.entity_id.is_none());
        assert!(params.since.is_none());
        assert!(params.limit.is_none());
    }

    #[test]
    fn event_response_serializes() {
        let resp = EventResponse {
            id: 1,
            entity_type: "feature".into(),
            entity_id: 10,
            event_type: "Created".into(),
            actor: "user".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
            payload: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["entity_type"], "feature");
        assert_eq!(json["entity_id"], 10);
        assert_eq!(json["actor"], "user");
        assert_eq!(json["payload"]["key"], "value");
    }

    #[test]
    fn event_response_wp_type() {
        let resp = EventResponse {
            id: 5,
            entity_type: "work_package".into(),
            entity_id: 5,
            event_type: "wp_planned".into(),
            actor: "system".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
            payload: serde_json::json!({"title": "Test WP"}),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["entity_type"], "work_package");
        assert_eq!(json["payload"]["title"], "Test WP");
    }
}
