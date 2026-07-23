//! Service health check handlers and related types.
//!
//! Provides HTTP endpoints for health monitoring, service status, configuration,
//! and restart operations. Includes both HTML (partial + full page) and JSON responses.

use std::path::Path as FilePath;

use agileplus_domain::credentials::CredentialStore;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use super::settings::{Config, ServiceConfig, default_service_enabled};
use crate::app_state::{ServiceHealth, SharedState};
use crate::templates::{HealthPage, HealthPanelPartial, ServiceHealthView, ToastPartial};

use chrono::Utc;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

/// JSON response for GET /api/dashboard/health (service health status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub services: Vec<ServiceHealthJson>,
    pub timestamp: String,
    pub all_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthJson {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check: String,
}

/// Form submission for PATCH /api/dashboard/services/:name/config
#[derive(Debug, Deserialize)]
pub struct ServiceConfigForm {
    pub endpoint_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
}

/// Request body for POST /api/dashboard/services/:name/toggle
#[derive(Debug, Deserialize)]
pub struct ServiceToggleBody {
    pub enabled: Option<bool>,
}

fn apply_service_config(
    config: &mut Config,
    name: &str,
    endpoint_url: Option<String>,
    timeout_ms: Option<u64>,
    max_retries: Option<u32>,
) {
    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|service| service.name == name) {
        if let Some(url) = endpoint_url.filter(|url| !url.trim().is_empty()) {
            entry.endpoint_url = url;
        }
    } else if let Some(url) = endpoint_url.filter(|url| !url.trim().is_empty()) {
        services.push(ServiceConfig {
            name: name.to_string(),
            endpoint_url: url,
            enabled: default_service_enabled(),
            timeout_ms,
            max_retries,
        });
    }
}

/// Testable health-route persistence path. Its config load is the canonical
/// settings loader, so legacy Plane secrets are secured before a health save.
fn save_service_config_at_path_with_credentials(
    config_path: &FilePath,
    credentials: &dyn CredentialStore,
    name: &str,
    endpoint_url: Option<String>,
    timeout_ms: Option<u64>,
    max_retries: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load_from_path_with_credentials(config_path, credentials)?;
    apply_service_config(&mut config, name, endpoint_url, timeout_ms, max_retries);
    config.save_to_path(config_path)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Generic template renderer: converts Askama templates to HTML responses.
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

const ALLOWED_RESTART_PROGRAMS: [&str; 4] = ["systemctl", "docker", "process-compose", "echo"];

/// Check if a program name is in the approved restart command registry.
fn is_restart_command_allowed(program: &str) -> bool {
    ALLOWED_RESTART_PROGRAMS.contains(&program)
}

/// Validate a restart command by checking the program name against the allowlist.
fn validate_restart_command(cmd_line: &str) -> Result<(), String> {
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

/// Build a safe std::process::Command from a validated command line string.
fn build_restart_command(cmd_line: &str) -> Result<std::process::Command, String> {
    validate_restart_command(cmd_line)?;

    let mut parts: Vec<&str> = cmd_line.split_whitespace().collect();
    let program = parts.remove(0);

    let mut cmd = std::process::Command::new(program);
    if !parts.is_empty() {
        cmd.args(parts);
    }
    Ok(cmd)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/dashboard/health (HTML partial)
/// Returns the health panel partial showing current service status.
pub async fn health_panel(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    render(HealthPanelPartial {
        services: store.health.clone(),
    })
}

/// GET /api/dashboard/health (JSON)
/// Returns service health status as JSON. Runs real health checks and polls every 10s from dashboard.
pub async fn health_json(State(state): State<SharedState>) -> impl IntoResponse {
    // Run real health checks and update the store.
    let real_health = crate::health::run_health_checks();

    let mut store = state.write().await;
    store.health = real_health;

    let services: Vec<ServiceHealthJson> = store
        .health
        .iter()
        .map(|service| ServiceHealthJson {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check: service
                .last_check
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        })
        .collect();

    let all_healthy = services.iter().all(|s| s.healthy && !s.degraded);

    axum::Json(HealthStatus {
        services,
        timestamp: Utc::now().to_rfc3339(),
        all_healthy,
    })
}

/// GET /health (full page)
/// Returns the full health monitoring page with all services and their status.
pub async fn health_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let services: Vec<ServiceHealthView> = store
        .health
        .iter()
        .map(|service| ServiceHealthView {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check_str: service
                .last_check
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        })
        .collect();
    let healthy_count = services
        .iter()
        .filter(|service| service.healthy && !service.degraded)
        .count();
    let degraded_count = services.iter().filter(|service| service.degraded).count();
    let unhealthy_count = services
        .iter()
        .filter(|service| !service.healthy && !service.degraded)
        .count();

    render(HealthPage {
        services,
        healthy_count,
        degraded_count,
        unhealthy_count,
    })
}

/// POST /api/dashboard/services/:name/restart
/// Restart a service using the configured restart command template.
/// The template is read from AGILEPLUS_SERVICE_RESTART_CMD env var and must contain "{}".
pub async fn restart_service(
    State(_state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let template = std::env::var("AGILEPLUS_SERVICE_RESTART_CMD")
        .unwrap_or_else(|_| "systemctl restart {}".to_string());

    if !template.contains("{}") {
        return axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "error": "AGILEPLUS_SERVICE_RESTART_CMD must include '{}' placeholder",
        }));
    }

    let command_str = template.replace("{}", &name);

    let mut command = match build_restart_command(&command_str) {
        Ok(c) => c,
        Err(err) => {
            return axum::Json(serde_json::json!({
                "status": "error",
                "service": name,
                "command": command_str,
                "error": err,
            }));
        }
    };

    match command.output() {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            axum::Json(serde_json::json!({
                "status": if success { "ok" } else { "error" },
                "service": name,
                "command": command_str,
                "stdout": stdout,
                "stderr": stderr,
            }))
        }
        Err(err) => axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "command": command_str,
            "error": err.to_string(),
        })),
    }
}

/// PATCH /api/dashboard/services/:name/config
/// Update endpoint URL, timeout, and retry configuration for a service.
pub async fn patch_service_config(
    Path(name): Path<String>,
    axum::Form(form): axum::Form<ServiceConfigForm>,
) -> impl IntoResponse {
    let mut config = match Config::load() {
        Ok(config) => config,
        Err(error) => {
            return render(ToastPartial {
                message: format!("Failed to load settings safely: {error}"),
                success: false,
            });
        }
    };

    apply_service_config(
        &mut config,
        &name,
        form.endpoint_url,
        form.timeout_ms,
        form.max_retries,
    );

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: format!("Service '{name}' configuration saved"),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save: {e}"),
            success: false,
        }),
    }
}

/// POST /api/dashboard/services/:name/toggle
/// Enable or disable a service. Persists state in config and updates in-memory health status.
pub async fn toggle_service(
    State(state): State<SharedState>,
    Path(name): Path<String>,
    axum::Json(body): axum::Json<ServiceToggleBody>,
) -> impl IntoResponse {
    let enabled = body.enabled.unwrap_or(true);

    // Persist state in config file
    let mut config = match Config::load() {
        Ok(config) => config,
        Err(error) => {
            return axum::Json(serde_json::json!({
                "status": "error",
                "service": name,
                "enabled": enabled,
                "error": format!("Failed to load config safely: {error}"),
            }));
        }
    };

    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|s| s.name == name) {
        entry.enabled = enabled;
    } else {
        services.push(ServiceConfig {
            name: name.clone(),
            endpoint_url: String::new(),
            enabled,
            timeout_ms: None,
            max_retries: None,
        });
    }

    if let Err(err) = config.save() {
        return axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "enabled": enabled,
            "error": format!("Failed to save config: {err}"),
        }));
    }

    // Update in-memory health status for UI
    {
        let mut store = state.write().await;
        if let Some(item) = store.health.iter_mut().find(|s| s.name == name) {
            item.healthy = enabled;
            item.degraded = !enabled;
            item.last_check = Utc::now();
        } else {
            store.health.push(ServiceHealth {
                name: name.clone(),
                healthy: enabled,
                degraded: !enabled,
                latency_ms: None,
                last_check: Utc::now(),
            });
        }
    }

    axum::Json(serde_json::json!({
        "status": "ok",
        "service": name,
        "enabled": enabled,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::credentials::{CredentialError, InMemoryCredentialStore, PLANESO_KEY};
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn health_style_save_migrates_and_scrubs_legacy_plane_credential() {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "agileplus-health-config-{}",
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let config_path = directory.join(".agileplus/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(
            &config_path,
            concat!(
                "[plane]\n",
                "api_url = 'https://plane.example'\n",
                "api_key = 'health-route-legacy-secret'\n",
                "workspace_slug = 'workspace'\n",
                "project_slug = 'project'\n"
            ),
        )
        .unwrap();
        let credentials = InMemoryCredentialStore::new();

        save_service_config_at_path_with_credentials(
            &config_path,
            &credentials,
            "API",
            Some("http://127.0.0.1:3000".to_string()),
            Some(1_000),
            Some(2),
        )
        .unwrap();

        assert_eq!(
            credentials.get("agileplus", PLANESO_KEY).unwrap(),
            "health-route-legacy-secret"
        );
        let persisted = std::fs::read_to_string(config_path).unwrap();
        assert!(persisted.contains("api_key_ref"));
        assert!(!persisted.contains("health-route-legacy-secret"));
        assert!(persisted.contains("https://plane.example"));
        assert!(persisted.contains("workspace"));
        assert!(persisted.contains("endpoint_url = \"http://127.0.0.1:3000\""));
        std::fs::remove_dir_all(directory).unwrap();
    }

    struct RejectingCredentialStore;

    impl CredentialStore for RejectingCredentialStore {
        fn get(&self, _service: &str, _key: &str) -> Result<String, CredentialError> {
            Err(CredentialError::BackendError("unavailable".to_string()))
        }

        fn set(&self, _service: &str, _key: &str, _value: &str) -> Result<(), CredentialError> {
            Err(CredentialError::BackendError("unavailable".to_string()))
        }

        fn delete(&self, _service: &str, _key: &str) -> Result<(), CredentialError> {
            Err(CredentialError::BackendError("unavailable".to_string()))
        }

        fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
            Err(CredentialError::BackendError("unavailable".to_string()))
        }
    }

    #[test]
    fn health_style_save_returns_error_and_preserves_legacy_config_on_migration_failure() {
        let directory = std::env::temp_dir().join("agileplus-health-rejecting-store");
        let config_path = directory.join(".agileplus/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        let legacy = concat!(
            "[plane]\n",
            "api_url = 'https://plane.example'\n",
            "api_key = 'health-route-rejected-secret'\n",
            "workspace_slug = 'workspace'\n",
            "project_slug = 'project'\n"
        );
        std::fs::write(&config_path, legacy).unwrap();

        let error = save_service_config_at_path_with_credentials(
            &config_path,
            &RejectingCredentialStore,
            "API",
            Some("http://127.0.0.1:3000".to_string()),
            None,
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("unavailable"));
        assert_eq!(std::fs::read_to_string(&config_path).unwrap(), legacy);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
