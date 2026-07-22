use anyhow::Result;

use crate::platform::args::PlatformStatusArgs;
use crate::platform::health::{fetch_platform_health, print_status_table};
use crate::platform::types::{OverallStatus, ServiceStatus};

/// Display platform service health.
pub fn run_platform_status(args: PlatformStatusArgs) -> Result<()> {
    let api_url = resolved_health_url(
        &args.api_url,
        std::env::var("AGILEPLUS_API_URL").ok().as_deref(),
    );
    let health = fetch_platform_health(&api_url);
    print_status_table(&health.services);
    println!();

    let api_up = health
        .services
        .iter()
        .any(|s| s.name == "API" && s.status == ServiceStatus::Healthy);
    let down_names: Vec<&str> = health
        .services
        .iter()
        .filter(|s| {
            matches!(
                s.status,
                ServiceStatus::Unknown | ServiceStatus::Unhealthy
            )
        })
        .map(|s| s.name.as_str())
        .collect();
    let degraded_names: Vec<&str> = health
        .services
        .iter()
        .filter(|s| s.status == ServiceStatus::Degraded)
        .map(|s| s.name.as_str())
        .collect();

    let overall_msg = match &health.overall {
        OverallStatus::Healthy => "HEALTHY".to_string(),
        OverallStatus::Degraded => {
            let mut parts = Vec::new();
            if api_up {
                parts.push("API up".to_string());
            }
            if !degraded_names.is_empty() {
                parts.push(format!("slow: {}", degraded_names.join(", ")));
            }
            if !down_names.is_empty() {
                parts.push(format!("down: {}", down_names.join(", ")));
            }
            if parts.is_empty() {
                "DEGRADED".to_string()
            } else {
                format!("DEGRADED ({})", parts.join("; "))
            }
        }
        OverallStatus::Down => {
            if down_names.is_empty() {
                "DOWN".to_string()
            } else {
                format!("DOWN ({})", down_names.join(", "))
            }
        }
    };
    println!("Overall Status: {overall_msg}");
    Ok(())
}

/// Keep the explicit CLI flag authoritative, but let the local runtime resolver
/// provide the status endpoint when the user did not supply one.
pub(crate) fn resolved_health_url(explicit_url: &str, runtime_url: Option<&str>) -> String {
    if explicit_url == "http://127.0.0.1:3000" {
        if let Some(runtime_url) = runtime_url.filter(|url| !url.trim().is_empty()) {
            return runtime_url.to_owned();
        }
    }
    explicit_url.to_owned()
}
