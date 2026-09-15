// SPDX-License-Identifier: MIT OR Apache-2.0
use std::collections::HashMap;
use std::env;

use agileplus_domain::domain::{
    feature::Feature, state_machine::FeatureState, work_package::WpState,
};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};

use crate::app_state::DashboardStore;
use crate::templates::{FeatureView, ProjectSummaryView, ProjectView, all_feature_states};

#[allow(dead_code)] // reserved for Plane.so API integration
pub(super) const DEFAULT_PLANE_API_URL: &str = "https://app.plane.so";
#[allow(dead_code)] // reserved for Plane.so API integration
pub(super) const DEFAULT_PLANE_WEB_URL: &str = "https://app.plane.so";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DashboardFilter {
    All,
    Active,
    Blocked,
    Shipped,
}

pub(super) fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub(super) fn render<T: Template>(tpl: T) -> Response {
    match tpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {e}"),
        )
            .into_response(),
    }
}

pub(super) fn load_projects(store: &DashboardStore) -> (Vec<ProjectView>, Option<ProjectView>) {
    let projects: Vec<ProjectView> = store
        .projects
        .iter()
        .map(|p| ProjectView {
            id: p.id,
            slug: p.slug.clone(),
            name: p.name.clone(),
            description: p.description.clone().unwrap_or_default(),
        })
        .collect();
    let active_project = store.active_project().map(|p| ProjectView {
        id: p.id,
        slug: p.slug.clone(),
        name: p.name.clone(),
        description: p.description.clone().unwrap_or_default(),
    });
    (projects, active_project)
}

pub(super) fn build_project_summaries(store: &DashboardStore) -> Vec<ProjectSummaryView> {
    store
        .projects
        .iter()
        .map(|project| {
            let (feature_count, active_count, shipped_count) =
                store.feature_counts_for_project(project.id);
            ProjectSummaryView {
                project: ProjectView {
                    id: project.id,
                    slug: project.slug.clone(),
                    name: project.name.clone(),
                    description: project.description.clone().unwrap_or_default(),
                },
                feature_count,
                active_count,
                shipped_count,
            }
        })
        .collect()
}

#[allow(dead_code)] // utility - reserved for future env-based configuration
pub(super) fn env_or_none(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[allow(dead_code)] // utility - reserved for future env-based configuration
pub(super) fn parse_bool_env(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

pub(super) fn dashboard_filter_from_query(query: &HashMap<String, String>) -> DashboardFilter {
    match query.get("filter").map(|value| value.as_str()) {
        Some("active") => DashboardFilter::Active,
        Some("blocked") => DashboardFilter::Blocked,
        Some("shipped") => DashboardFilter::Shipped,
        _ => DashboardFilter::All,
    }
}

pub(super) fn feature_matches_filter(
    store: &DashboardStore,
    feature: &Feature,
    filter: DashboardFilter,
) -> bool {
    let is_blocked = store
        .work_packages
        .get(&feature.id)
        .map(|workpackages| workpackages.iter().any(|wp| wp.state == WpState::Blocked))
        .unwrap_or(false);

    match filter {
        DashboardFilter::All => true,
        DashboardFilter::Active => !matches!(
            feature.state,
            FeatureState::Shipped | FeatureState::Retrospected
        ),
        DashboardFilter::Blocked => is_blocked,
        DashboardFilter::Shipped => {
            matches!(
                feature.state,
                FeatureState::Shipped | FeatureState::Retrospected
            )
        }
    }
}

pub(super) fn build_kanban_cards(
    store: &DashboardStore,
    filter: DashboardFilter,
) -> HashMap<String, Vec<FeatureView>> {
    let states = all_feature_states();
    let mut cards: HashMap<String, Vec<FeatureView>> = HashMap::new();
    for s in &states {
        cards.insert(s.clone(), vec![]);
    }
    for feature in store.features_for_active_project() {
        if !feature_matches_filter(store, feature, filter) {
            continue;
        }
        let state_key = feature.state.to_string();
        let view = FeatureView::from_feature(feature);
        cards.entry(state_key).or_default().push(view);
    }
    cards
}

pub(super) fn event_view(
    id: impl Into<String>,
    kind: impl Into<String>,
    description: impl Into<String>,
    timestamp: impl Into<String>,
) -> crate::templates::EventView {
    crate::templates::EventView {
        id: id.into(),
        kind: kind.into(),
        description: description.into(),
        timestamp: timestamp.into(),
        agent_name: None,
        agent_link: None,
        wp_id: None,
        wp_link: None,
        commit_sha: None,
        commit_link: None,
        ci_run_id: None,
        ci_run_link: None,
    }
}

pub(super) fn agent_view(
    name: impl Into<String>,
    status: impl Into<String>,
    current_task: impl Into<String>,
    last_action: impl Into<String>,
) -> crate::templates::AgentView {
    crate::templates::AgentView {
        name: name.into(),
        status: status.into(),
        current_task: current_task.into(),
        last_action: last_action.into(),
        pid: None,
        started_at: None,
        worktree: String::new(),
        worktree_label: String::new(),
        is_live: false,
    }
}

pub(super) fn sample_events() -> Vec<crate::templates::EventView> {
    vec![
        event_view(
            "evt-1",
            "system",
            "Dashboard booted with native Plane surface",
            "just now",
        ),
        event_view(
            "evt-2",
            "agent_action",
            "Planner synced feature ownership metadata",
            "2m ago",
        ),
        event_view(
            "evt-3",
            "state_change",
            "Feature moved from researched to planned",
            "9m ago",
        ),
    ]
}

// ── HTML and URL utilities ─────────────────────────────────────────────────

/// Minimal HTML entity escaping for embedding text content in HTML attributes/elements.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Classify a file extension into a broad artifact type for display purposes.
#[allow(dead_code)] // utility - reserved for future artifact display features
pub fn artifact_type_for_ext(ext: &str) -> &'static str {
    match ext {
        "lcov" | "coverage" | "cov" => "coverage",
        "xml" | "junit" | "tap" => "test-results",
        "json" | "sarif" => "report",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image",
        "md" | "txt" | "log" => "text",
        _ => "artifact",
    }
}

/// Percent-encode path segments so they are safe to embed in URLs.
///
/// Only encodes characters that are not allowed unencoded in URL path segments:
/// spaces, `#`, `?`, `%`, and `+`.
#[allow(dead_code)] // utility - reserved for future artifact display features
pub fn percent_encode_path(path: &str) -> String {
    path.chars()
        .flat_map(|c| match c {
            ' ' => vec!['%', '2', '0'],
            '#' => vec!['%', '2', '3'],
            '?' => vec!['%', '3', 'F'],
            '%' => vec!['%', '2', '5'],
            '+' => vec!['%', '2', 'B'],
            other => vec![other],
        })
        .collect()
}

// ── Service restart command validation ─────────────────────────────────────

const ALLOWED_RESTART_PROGRAMS: [&str; 4] = ["systemctl", "docker", "process-compose", "echo"];

pub fn is_restart_command_allowed(program: &str) -> bool {
    ALLOWED_RESTART_PROGRAMS.contains(&program)
}

pub fn validate_restart_command(cmd_line: &str) -> Result<(), String> {
    let mut parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty restart command".into());
    }

    let program = parts.remove(0);
    if !is_restart_command_allowed(program) {
        return Err(format!(
            "command '{program}' is not in approved restart command registry: {ALLOWED_RESTART_PROGRAMS:?}"
        ));
    }

    Ok(())
}

pub fn build_restart_command(cmd_line: &str) -> Result<std::process::Command, String> {
    validate_restart_command(cmd_line)?;

    let mut parts: Vec<&str> = cmd_line.split_whitespace().collect();
    let program = parts.remove(0);

    let mut cmd = std::process::Command::new(program);
    if !parts.is_empty() {
        cmd.args(parts);
    }
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── is_htmx ─────────────────────────────────────────────────────────

    #[test]
    fn test_is_htmx_true() {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Request", "true".parse().unwrap());
        assert!(is_htmx(&headers));
    }

    #[test]
    fn test_is_htmx_false_value() {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Request", "false".parse().unwrap());
        assert!(!is_htmx(&headers));
    }

    #[test]
    fn test_is_htmx_missing_header() {
        let headers = HeaderMap::new();
        assert!(!is_htmx(&headers));
    }

    // ── dashboard_filter_from_query ──────────────────────────────────────

    #[test]
    fn test_filter_all_default() {
        let query = HashMap::new();
        assert_eq!(
            dashboard_filter_from_query(&query),
            DashboardFilter::All
        );
    }

    #[test]
    fn test_filter_active() {
        let mut query = HashMap::new();
        query.insert("filter".to_string(), "active".to_string());
        assert_eq!(
            dashboard_filter_from_query(&query),
            DashboardFilter::Active
        );
    }

    #[test]
    fn test_filter_blocked() {
        let mut query = HashMap::new();
        query.insert("filter".to_string(), "blocked".to_string());
        assert_eq!(
            dashboard_filter_from_query(&query),
            DashboardFilter::Blocked
        );
    }

    #[test]
    fn test_filter_shipped() {
        let mut query = HashMap::new();
        query.insert("filter".to_string(), "shipped".to_string());
        assert_eq!(
            dashboard_filter_from_query(&query),
            DashboardFilter::Shipped
        );
    }

    #[test]
    fn test_filter_unknown_falls_back_to_all() {
        let mut query = HashMap::new();
        query.insert("filter".to_string(), "unknown".to_string());
        assert_eq!(
            dashboard_filter_from_query(&query),
            DashboardFilter::All
        );
    }

    // ── html_escape ──────────────────────────────────────────────────────

    #[test]
    fn test_html_escape_plain_text() {
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn test_html_escape_all_entities() {
        assert_eq!(
            html_escape("a & b < c > d \"e\" f'g"),
            "a &amp; b &lt; c &gt; d &quot;e&quot; f&#39;g"
        );
    }

    #[test]
    fn test_html_escape_empty_string() {
        assert_eq!(html_escape(""), "");
    }

    // ── artifact_type_for_ext ────────────────────────────────────────────

    #[test]
    fn test_artifact_type_coverage() {
        assert_eq!(artifact_type_for_ext("lcov"), "coverage");
        assert_eq!(artifact_type_for_ext("coverage"), "coverage");
        assert_eq!(artifact_type_for_ext("cov"), "coverage");
    }

    #[test]
    fn test_artifact_type_test_results() {
        assert_eq!(artifact_type_for_ext("xml"), "test-results");
        assert_eq!(artifact_type_for_ext("junit"), "test-results");
        assert_eq!(artifact_type_for_ext("tap"), "test-results");
    }

    #[test]
    fn test_artifact_type_report() {
        assert_eq!(artifact_type_for_ext("json"), "report");
        assert_eq!(artifact_type_for_ext("sarif"), "report");
    }

    #[test]
    fn test_artifact_type_image() {
        assert_eq!(artifact_type_for_ext("png"), "image");
        assert_eq!(artifact_type_for_ext("jpg"), "image");
        assert_eq!(artifact_type_for_ext("jpeg"), "image");
        assert_eq!(artifact_type_for_ext("gif"), "image");
        assert_eq!(artifact_type_for_ext("svg"), "image");
        assert_eq!(artifact_type_for_ext("webp"), "image");
    }

    #[test]
    fn test_artifact_type_text() {
        assert_eq!(artifact_type_for_ext("md"), "text");
        assert_eq!(artifact_type_for_ext("txt"), "text");
        assert_eq!(artifact_type_for_ext("log"), "text");
    }

    #[test]
    fn test_artifact_type_unknown() {
        assert_eq!(artifact_type_for_ext("rs"), "artifact");
        assert_eq!(artifact_type_for_ext("unknown"), "artifact");
    }

    // ── percent_encode_path ──────────────────────────────────────────────

    #[test]
    fn test_percent_encode_no_special_chars() {
        assert_eq!(percent_encode_path("hello/world.txt"), "hello/world.txt");
    }

    #[test]
    fn test_percent_encode_space() {
        assert_eq!(percent_encode_path("hello world"), "hello%20world");
    }

    #[test]
    fn test_percent_encode_all_special() {
        assert_eq!(
            percent_encode_path("a#?%+b"),
            "a%23%3F%25%2Bb"
        );
    }

    #[test]
    fn test_percent_encode_empty() {
        assert_eq!(percent_encode_path(""), "");
    }

    // ── is_restart_command_allowed ───────────────────────────────────────

    #[test]
    fn test_restart_allowed_systemctl() {
        assert!(is_restart_command_allowed("systemctl"));
    }

    #[test]
    fn test_restart_allowed_docker() {
        assert!(is_restart_command_allowed("docker"));
    }

    #[test]
    fn test_restart_allowed_process_compose() {
        assert!(is_restart_command_allowed("process-compose"));
    }

    #[test]
    fn test_restart_allowed_echo() {
        assert!(is_restart_command_allowed("echo"));
    }

    #[test]
    fn test_restart_not_allowed() {
        assert!(!is_restart_command_allowed("rm"));
        assert!(!is_restart_command_allowed("bash"));
        assert!(!is_restart_command_allowed(""));
    }

    // ── validate_restart_command ─────────────────────────────────────────

    #[test]
    fn test_validate_restart_empty_command() {
        assert!(validate_restart_command("").is_err());
    }

    #[test]
    fn test_validate_restart_allowed_command() {
        assert!(validate_restart_command("systemctl restart nginx").is_ok());
    }

    #[test]
    fn test_validate_restart_disallowed_command() {
        let err = validate_restart_command("rm -rf /").unwrap_err();
        assert!(err.contains("not in approved"));
    }

    // ── build_restart_command ────────────────────────────────────────────

    #[test]
    fn test_build_restart_command_valid() {
        let cmd = build_restart_command("echo hello");
        assert!(cmd.is_ok());
    }

    #[test]
    fn test_build_restart_command_invalid() {
        assert!(build_restart_command("malware").is_err());
    }

    // ── event_view ───────────────────────────────────────────────────────

    #[test]
    fn test_event_view_creation() {
        let ev = event_view("evt-1", "system", "booted", "just now");
        assert_eq!(ev.id, "evt-1");
        assert_eq!(ev.kind, "system");
        assert_eq!(ev.description, "booted");
        assert_eq!(ev.timestamp, "just now");
        assert!(ev.agent_name.is_none());
        assert!(ev.wp_id.is_none());
    }

    // ── agent_view ───────────────────────────────────────────────────────

    #[test]
    fn test_agent_view_creation() {
        let av = agent_view("spec-agent", "idle", "WP10", "2m ago");
        assert_eq!(av.name, "spec-agent");
        assert_eq!(av.status, "idle");
        assert_eq!(av.current_task, "WP10");
        assert_eq!(av.last_action, "2m ago");
        assert!(!av.is_live);
        assert!(av.pid.is_none());
    }

    // ── sample_events ────────────────────────────────────────────────────

    #[test]
    fn test_sample_events_returns_three() {
        let events = sample_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].kind, "system");
        assert_eq!(events[1].kind, "agent_action");
        assert_eq!(events[2].kind, "state_change");
    }

    // ── DashboardFilter equality ─────────────────────────────────────────

    #[test]
    fn test_dashboard_filter_variants() {
        assert_eq!(DashboardFilter::All, DashboardFilter::All);
        assert_ne!(DashboardFilter::All, DashboardFilter::Active);
        assert_ne!(DashboardFilter::Blocked, DashboardFilter::Shipped);
    }
}
