use std::time::Instant;

use anyhow::{Context, Result};
use agileplus_plane::{PlaneClient, PlaneConfig, PushOptions, SyncStatus};
use tracing::{debug, info, warn};

use crate::sync::helpers::{format_duration, print_sync_summary, SyncDirection};
use crate::sync::types::{SyncMetrics, SyncResult};

/// Execute push command - sync local SQLite data to Plane.so
pub async fn execute_push(
    api_key: Option<String>,
    dry_run: bool,
    force: bool,
    include_completed: bool,
) -> Result<SyncResult> {
    let start = Instant::now();
    info!("Starting push sync to Plane.so...");

    // Initialize Plane.so client
    let config = PlaneConfig {
        api_key: api_key.or_else(|| std::env::var("PLANE_API_KEY").ok()),
        base_url: std::env::var("PLANE_BASE_URL")
            .unwrap_or_else(|_| "https://api.plane.so".to_string()),
        timeout_secs: 30,
    };

    let client = PlaneClient::new(config)
        .context("Failed to initialize Plane.so client. Check PLANE_API_KEY.")?;

    // Verify connectivity
    match client.health_check().await {
        Ok(true) => debug!("Plane.so API connectivity verified"),
        Ok(false) => {
            warn!("Plane.so API reports unhealthy status");
            if !force {
                anyhow::bail!("Plane.so API is unhealthy. Use --force to sync anyway.");
            }
        }
        Err(e) => {
            warn!("Cannot reach Plane.so API: {}", e);
            if !force {
                anyhow::bail!("Cannot connect to Plane.so API. Use --force to sync anyway.");
            }
        }
    }

    // Execute sync
    let options = PushOptions {
        dry_run,
        include_completed,
        batch_size: 100,
    };

    let sync_result = match client.push_local_to_remote(options).await {
        Ok(result) => {
            info!(
                "Push sync completed: {} features, {} work packages synced",
                result.features_pushed, result.work_packages_pushed
            );
            result
        }
        Err(e) => {
            anyhow::bail!("Push sync failed: {}", e);
        }
    };

    let duration = start.elapsed();

    // Build result
    let result = SyncResult {
        direction: SyncDirection::Push,
        duration_ms: duration.as_millis() as u64,
        features_synced: sync_result.features_pushed,
        work_packages_synced: sync_result.work_packages_pushed,
        errors: sync_result.conflicts.len(),
        metrics: SyncMetrics {
            total_api_calls: sync_result.api_calls,
            total_records_processed: sync_result.features_pushed + sync_result.work_packages_pushed,
        },
    };

    // Print summary
    print_sync_summary(&result);

    if dry_run {
        info!("Push completed in dry-run mode. No changes were made to Plane.so.");
        info!("Run without --dry-run to apply changes.");
    } else {
        info!("Push completed successfully in {}", format_duration(duration));
    }

    Ok(result)
}

/// Push only features (without work packages)
pub async fn execute_push_features(
    api_key: Option<String>,
    dry_run: bool,
) -> Result<SyncResult> {
    let start = Instant::now();
    info!("Starting features-only push sync...");

    let config = PlaneConfig {
        api_key: api_key.or_else(|| std::env::var("PLANE_API_KEY").ok()),
        base_url: std::env::var("PLANE_BASE_URL")
            .unwrap_or_else(|_| "https://api.plane.so".to_string()),
        timeout_secs: 30,
    };

    let client = PlaneClient::new(config)
        .context("Failed to initialize Plane.so client")?;

    let result = client.push_features_only(dry_run).await
        .context("Failed to push features to Plane.so")?;

    let duration = start.elapsed();

    info!(
        "Pushed {} features to Plane.so in {}",
        result.features_pushed,
        format_duration(duration)
    );

    Ok(SyncResult {
        direction: SyncDirection::Push,
        duration_ms: duration.as_millis() as u64,
        features_synced: result.features_pushed,
        work_packages_synced: 0,
        errors: 0,
        metrics: SyncMetrics {
            total_api_calls: result.api_calls,
            total_records_processed: result.features_pushed,
        },
    })
}

/// Push only work packages (without features)
pub async fn execute_push_work_packages(
    api_key: Option<String>,
    dry_run: bool,
) -> Result<SyncResult> {
    let start = Instant::now();
    info!("Starting work-packages-only push sync...");

    let config = PlaneConfig {
        api_key: api_key.or_else(|| std::env::var("PLANE_API_KEY").ok()),
        base_url: std::env::var("PLANE_BASE_URL")
            .unwrap_or_else(|_| "https://api.plane.so".to_string()),
        timeout_secs: 30,
    };

    let client = PlaneClient::new(config)
        .context("Failed to initialize Plane.so client")?;

    let result = client.push_work_packages_only(dry_run).await
        .context("Failed to push work packages to Plane.so")?;

    let duration = start.elapsed();

    info!(
        "Pushed {} work packages to Plane.so in {}",
        result.work_packages_pushed,
        format_duration(duration)
    );

    Ok(SyncResult {
        direction: SyncDirection::Push,
        duration_ms: duration.as_millis() as u64,
        features_synced: 0,
        work_packages_synced: result.work_packages_pushed,
        errors: 0,
        metrics: SyncMetrics {
            total_api_calls: result.api_calls,
            total_records_processed: result.work_packages_pushed,
        },
    })
}
