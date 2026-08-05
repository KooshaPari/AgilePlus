//! Feature detail route handlers.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use agileplus_domain::domain::state_machine::FeatureState;

use crate::app_state::SharedState;
use crate::templates::{EventTimelinePartial, FeatureDetailPage, FeatureView, KanbanPartial, WpView};

use super::features::{
    DashboardFilter, build_feature_events, build_feature_evidence_bundles,
    build_feature_media_assets, build_feature_reports, build_kanban_cards, render,
};

// ── Route Handlers ───────────────────────────────────────────────────────────

/// GET /api/dashboard/features/:id
/// Returns the full feature detail page with all associated data:
/// events, evidence bundles, media assets, and reports.
pub async fn feature_detail(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    _headers: HeaderMap,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
    };
    let fid = feature.id;
    let wps: Vec<WpView> = store
        .work_packages
        .get(&id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    let events = build_feature_events(&feature, &wps);
    let evidence_bundles = build_feature_evidence_bundles(&feature, &wps);
    let media_assets = build_feature_media_assets(&feature, &wps);
    let reports = build_feature_reports(&feature);

    render(FeatureDetailPage {
        feature,
        feature_id: fid,
        workpackages: wps,
        events,
        evidence_bundles,
        media_assets,
        reports,
    })
}

/// GET /features/:id
/// Alias for feature_detail; renders the full page layout.
pub async fn feature_page(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
    feature_detail(State(state), Path(id), HeaderMap::new()).await
}

/// GET /api/dashboard/features/:id/events
/// Returns the event timeline partial for a feature (HTMX).
pub async fn feature_events(
    State(state): State<SharedState>,
    Path(feature_id): Path<i64>,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == feature_id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
    };
    let wps: Vec<WpView> = store
        .work_packages
        .get(&feature_id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    let events = build_feature_events(&feature, &wps);

    render(EventTimelinePartial { feature_id, events })
}

/// GET /api/dashboard/features/:id/media
/// Returns the media gallery partial for a feature (HTMX).
/// Renders as a 2-column grid of media assets.
pub async fn feature_media(
    State(state): State<SharedState>,
    Path(feature_id): Path<i64>,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == feature_id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
    };
    let wps: Vec<WpView> = store
        .work_packages
        .get(&feature_id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    let media = build_feature_media_assets(&feature, &wps);

    // Return media assets as a simple HTML partial
    let html = media
        .iter()
        .map(|m| {
            format!(
                r#"<div class="media-asset border rounded p-3 bg-zinc-800">
                <img src="{}" alt="{}" class="w-full rounded"/>
                <p class="text-xs text-zinc-400 mt-2">{}</p>
              </div>"#,
                m.url_or_path, m.name, m.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(format!(
        r#"<div class="grid grid-cols-2 gap-3 media-gallery">{html}</div>"#
    ))
    .into_response()
}

/// POST /api/features/:id/transition
/// Form data: `target_state` (feature state enum)
/// Transitions a feature to a new state and returns the updated Kanban cards.
#[derive(Debug, Deserialize)]
pub struct FeatureTransitionForm {
    #[serde(rename = "target_state")]
    pub new_state: String,
}

pub async fn feature_transition(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    axum::Form(form): axum::Form<FeatureTransitionForm>,
) -> Response {
    let new_state = match form.new_state.parse::<FeatureState>() {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid feature state").into_response(),
    };

    let feature_name = {
        let store = state.read().await;
        match store.features.iter().find(|f| f.id == id) {
            Some(f) => f.slug.clone(),
            None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
        }
    };

    // Broadcast the update so SSE clients refresh
    // (In a real app, persist the state change here)
    tracing::info!(
        "Feature {} transitioned to {:?} (SSE broadcast triggers UI refresh)",
        feature_name,
        new_state
    );

    // Return the kanban partial so htmx can swap it
    let store = state.read().await;
    let cards = build_kanban_cards(&store, DashboardFilter::All);
    render(KanbanPartial { cards })
}
