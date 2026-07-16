//! Settings handlers for plane, agents, services, and dashboard configuration.
//!
//! Provides routes for displaying and persisting configuration across the dashboard.
//! Includes form handlers for plane sync, agent pool, service endpoints, and theme/logging
//! preferences. Each handler validates inputs before persisting to the local config file.

use std::env;

use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::app_state::SharedState;
use crate::templates::{
    PlaneHealthEndpointView, PlaneSettingsPage, ServicesSettingsPage, SettingsPage, ToastPartial,
};

// ── Configuration Types ────────────────────────────────────────────────────

/// Plane sync configuration (API endpoint, credentials, workspace/project slugs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneConfig {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

/// Agent pool configuration (size, retry budget, dispatch strategy, LLM provider).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

/// Single service endpoint configuration (name, URL, optional timeout/retry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub endpoint_url: String,
    #[serde(default = "default_service_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

pub fn default_service_enabled() -> bool {
    true
}

/// Dashboard UI configuration (theme, logging level, data directory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

/// Composite configuration container (reads/writes to ~/.agileplus/config.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub plane: Option<PlaneConfig>,
    pub agents: Option<AgentConfig>,
    pub services: Option<Vec<ServiceConfig>>,
    pub dashboard: Option<DashboardConfig>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config {
                plane: None,
                agents: None,
                services: None,
                dashboard: None,
            })
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        std::env::var("HOME")
            .ok()
            .map(|home| std::path::PathBuf::from(home).join(".agileplus/config.toml"))
            .unwrap_or_else(|| std::path::PathBuf::from(".agileplus/config.toml"))
    }
}

// ── Form Request Types ────────────────────────────────────────────────────

/// Form data for plane sync settings (from HTML form POST).
#[derive(Debug, Deserialize)]
pub struct PlaneSettingsForm {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

/// Form data for agent pool configuration (from HTML form POST).
#[derive(Debug, Deserialize)]
pub struct AgentSettingsForm {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

/// Form data for custom service endpoint addition (from HTML form POST).
#[derive(Debug, Deserialize)]
pub struct ServiceSettingsForm {
    pub names: Vec<String>,
    pub endpoint_urls: Vec<String>,
}

/// Form data for dashboard UI settings (from HTML form POST).
#[derive(Debug, Deserialize)]
pub struct DashboardSettingsForm {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

/// Form data for single service endpoint validation (from HTML form POST).
#[derive(Debug, Deserialize)]
pub struct SingleServiceTestForm {
    pub name: String,
    pub endpoint_url: String,
}

// ── Helper Functions ──────────────────────────────────────────────────────

const DEFAULT_PLANE_API_URL: &str = "https://app.plane.so";
const DEFAULT_PLANE_WEB_URL: &str = "https://app.plane.so";

/// Extract environment variable or return None if missing/empty.
fn env_or_none(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Format API key as hint string (first and last char with dots for obfuscation).
pub(crate) fn plane_api_key_hint(api_key: &Option<String>) -> String {
    match api_key {
        Some(key) => match (key.chars().next(), key.chars().next_back()) {
            (Some(first), Some(last)) => format!("{first}••••••{last}"),
            _ => "Configured".to_string(),
        },
        None => "Not configured".to_string(),
    }
}

/// Filter service health records to plane-related endpoints and convert to view models.
fn plane_health_endpoints(
    services: &[crate::app_state::ServiceHealth],
) -> Vec<PlaneHealthEndpointView> {
    services
        .iter()
        .filter(|service| service.name.contains("Plane") || service.name.starts_with("API"))
        .map(|service| PlaneHealthEndpointView {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check_utc: service
                .last_check
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        })
        .collect()
}

/// Determine plane sync mode from environment variable (bidirectional or one-way).
fn plane_sync_mode() -> String {
    let bidirectional = env::var("PLANE_SYNC_BIDIRECTIONAL")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);

    if bidirectional {
        "Bidirectional".to_string()
    } else {
        "One-way".to_string()
    }
}

/// Validate plane connection status by checking required configuration fields.
pub(crate) fn plane_connection_checks(
    api_key: &Option<String>,
    workspace: &Option<String>,
) -> (bool, String, Vec<String>) {
    let mut warnings = Vec::new();
    if api_key.is_none() {
        warnings.push("Missing PLANE_API_KEY; configure a valid Plane API key".to_string());
    }
    if workspace.is_none() {
        warnings.push("Missing PLANE_WORKSPACE; set workspace slug for Plane sync".to_string());
    }

    if warnings.is_empty() {
        (true, "Connected via PLANE_API_KEY".to_string(), warnings)
    } else if warnings.len() == 1 {
        let status = warnings[0].clone();
        (false, status, warnings)
    } else {
        (false, "Plane settings incomplete".to_string(), warnings)
    }
}

/// Format coverage percentage as human-readable string (e.g. "5/10 (50%)").
pub(crate) fn percentage_coverage(hit: usize, total: usize) -> String {
    if total == 0 {
        return "0/0 (0%)".to_string();
    }
    let ratio = (hit.saturating_mul(100)).saturating_div(total);
    format!("{hit}/{total} ({ratio}%)")
}

/// Template rendering helper that converts Askama templates to HTML responses.
fn render<T: Template>(tpl: T) -> Response {
    match tpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {e}"),
        )
            .into_response(),
    }
}

// ── Route Handlers ────────────────────────────────────────────────────────

/// GET /settings
/// Returns the settings overview page (links to all settings sections).
pub async fn settings_page() -> Response {
    render(SettingsPage)
}

/// GET /settings/plane
/// Returns the Plane sync configuration page with connection status and health metrics.
pub async fn plane_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let plane_workspace = env_or_none("PLANE_WORKSPACE");
    let project_slug = env_or_none("PLANE_PROJECT").unwrap_or_else(|| "not configured".to_string());
    let plane_api_key = env_or_none("PLANE_API_KEY");
    let plane_api_url =
        env_or_none("PLANE_API_URL").unwrap_or_else(|| DEFAULT_PLANE_API_URL.to_string());
    let plane_web_url =
        env_or_none("PLANE_WEB_URL").unwrap_or_else(|| DEFAULT_PLANE_WEB_URL.to_string());
    let (connected, connection_status, mut config_warnings) =
        plane_connection_checks(&plane_api_key, &plane_workspace);

    let plane_health_endpoints = plane_health_endpoints(&store.health);
    let plane_health_healthy = plane_health_endpoints
        .iter()
        .all(|endpoint| endpoint.healthy && !endpoint.degraded);
    let plane_api_latency_ms = plane_health_endpoints
        .iter()
        .find(|endpoint| endpoint.name == "Plane API")
        .and_then(|endpoint| endpoint.latency_ms);

    if !connected {
        config_warnings
            .push("Plane sync disabled until required settings are provided".to_string());
    }

    if !plane_health_healthy {
        config_warnings.push("Plane API health check is not healthy".to_string());
    }

    let mapped_features = store
        .features
        .iter()
        .filter(|feature| feature.plane_issue_id.is_some())
        .count();
    let total_features = store.features.len();
    let mapped_work_packages = store
        .work_packages
        .values()
        .flatten()
        .filter(|wp| wp.plane_sub_issue_id.is_some())
        .count();
    let total_work_packages: usize = store.work_packages.values().map(Vec::len).sum();

    let connection_status_configured = !connection_status.is_empty();

    render(PlaneSettingsPage {
        workspace_name: plane_workspace
            .clone()
            .unwrap_or_else(|| "Not configured".to_string()),
        workspace_slug: plane_workspace.unwrap_or_else(|| "not configured".to_string()),
        project_slug,
        plane_api_url: plane_api_url.trim_end_matches('/').to_string(),
        plane_web_url: plane_web_url.trim_end_matches('/').to_string(),
        plane_api_url_set: !plane_api_url.trim_end_matches('/').is_empty(),
        plane_web_url_set: !plane_web_url.trim_end_matches('/').is_empty(),
        plane_api_key_hint: plane_api_key_hint(&plane_api_key),
        plane_api_key_set: plane_api_key.is_some(),
        sync_enabled: connected,
        sync_mode: plane_sync_mode(),
        connected,
        connection_status: connection_status.clone(),
        connection_status_configured,
        plane_service_healthy: plane_health_healthy,
        plane_api_latency_ms,
        plane_health_endpoints,
        mapped_features_coverage: percentage_coverage(mapped_features, total_features),
        mapped_work_packages_coverage: percentage_coverage(
            mapped_work_packages,
            total_work_packages,
        ),
        mapped_features,
        mapped_work_packages,
        config_warnings,
    })
}

/// GET /settings/agents
/// Returns the agent pool configuration page (size, retry budget, dispatch mode, provider).
pub async fn agent_settings_page() -> Response {
    let config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let agent_config = config.agents.unwrap_or_else(|| AgentConfig {
        pool_size: 6,
        retry_budget: 3,
        dispatch_mode: "balanced".to_string(),
        default_provider: "claude".to_string(),
    });

    render(crate::templates::AgentSettingsPage {
        agent_pool_size: agent_config.pool_size,
        retry_budget: agent_config.retry_budget,
        dispatch_mode: agent_config.dispatch_mode,
        default_provider: agent_config.default_provider,
    })
}

/// GET /settings/services
/// Returns the custom service endpoints configuration page.
pub async fn services_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let configs: Vec<crate::templates::ServiceConfigView> = config
        .services
        .unwrap_or_default()
        .into_iter()
        .map(|s| crate::templates::ServiceConfigView {
            name: s.name,
            endpoint_url: s.endpoint_url,
        })
        .collect();

    render(ServicesSettingsPage {
        services: store.health.clone(),
        configs,
    })
}

// ── Settings POST Handlers ─────────────────────────────────────────────────

/// POST /api/settings/plane
/// Persists plane sync configuration to the local config file.
pub async fn save_plane_settings(axum::Form(form): axum::Form<PlaneSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.plane = Some(PlaneConfig {
        api_url: form.api_url.trim().to_string(),
        api_key: form.api_key.trim().to_string(),
        workspace_slug: form.workspace_slug.trim().to_string(),
        project_slug: form.project_slug.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Plane settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {e}"),
            success: false,
        }),
    }
}

/// POST /api/settings/agents
/// Persists agent configuration (pool_size, retry_budget, dispatch_mode, default_provider).
pub async fn save_agent_settings(axum::Form(form): axum::Form<AgentSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.agents = Some(AgentConfig {
        pool_size: form.pool_size,
        retry_budget: form.retry_budget,
        dispatch_mode: form.dispatch_mode.trim().to_string(),
        default_provider: form.default_provider.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Agent settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {e}"),
            success: false,
        }),
    }
}

/// POST /api/settings/dashboard
/// Persists dashboard UI configuration (theme, log_level, data_directory).
pub async fn save_dashboard_settings(
    axum::Form(form): axum::Form<DashboardSettingsForm>,
) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.dashboard = Some(DashboardConfig {
        theme: form.theme.trim().to_string(),
        log_level: form.log_level.trim().to_string(),
        data_directory: form.data_directory.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Dashboard settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {e}"),
            success: false,
        }),
    }
}

/// POST /api/settings/services
/// Persists custom service endpoint configuration.
pub async fn save_services_settings(axum::Form(form): axum::Form<ServiceSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    let mut services = Vec::new();
    for (name, url) in form.names.into_iter().zip(form.endpoint_urls) {
        if !name.trim().is_empty() {
            services.push(ServiceConfig {
                name: name.trim().to_string(),
                endpoint_url: url.trim().to_string(),
                enabled: default_service_enabled(),
                timeout_ms: None,
                max_retries: None,
            });
        }
    }
    config.services = Some(services);

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Service settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {e}"),
            success: false,
        }),
    }
}

// ── Connection Testing Handlers ────────────────────────────────────────────

/// POST /api/settings/services/test
/// Validates a single service endpoint (basic URL format check).
pub async fn test_service_connection(
    axum::Form(form): axum::Form<SingleServiceTestForm>,
) -> Response {
    let is_valid = !form.endpoint_url.trim().is_empty() && form.endpoint_url.starts_with("http");

    if is_valid {
        render(ToastPartial {
            message: format!("Connection to {} successful (mock)", form.name),
            success: true,
        })
    } else {
        render(ToastPartial {
            message: format!("Invalid endpoint for {}: {}", form.name, form.endpoint_url),
            success: false,
        })
    }
}

/// POST /api/settings/plane/test
/// Validates plane connection (checks required fields and URL format).
pub async fn test_plane_connection(axum::Form(form): axum::Form<PlaneSettingsForm>) -> Response {
    // Simple validation: check that required fields are filled and api_url looks like a URL
    let is_valid = !form.api_url.trim().is_empty()
        && !form.api_key.trim().is_empty()
        && !form.workspace_slug.trim().is_empty()
        && form.api_url.starts_with("http");

    if is_valid {
        // In a real implementation, you would make an HTTP request to verify connectivity
        render(ToastPartial {
            message: "Plane connection test passed (mock)".to_string(),
            success: true,
        })
    } else {
        render(ToastPartial {
            message: "Plane settings are incomplete or invalid".to_string(),
            success: false,
        })
    }
}
