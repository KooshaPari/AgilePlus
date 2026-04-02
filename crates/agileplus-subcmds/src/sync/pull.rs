use std::time::Instant;

use anyhow::{Context, Result};
use agileplus_plane::{PlaneClient, PlaneConfig, sync::SyncEngine};
use agileplus_sqlite::Database;

use super::{
    args::SyncPullArgs,
    helpers::{outcome_icon, outcome_verb, SyncDirection},
    types::{SyncItemOutcome, SyncReport, SyncReportEntry},
};

/// Run `agileplus sync pull`.
pub async fn run_sync_pull(args: SyncPullArgs) -> Result<()> {
    let start = Instant::now();

    // Initialize Plane.so client
    let config = PlaneConfig::from_env()
        .context("Failed to load Plane.so configuration. Ensure PLANE_API_KEY and PLANE_WORKSPACE_SLUG are set.")?;
    let client = PlaneClient::new(config)?;

    // Initialize local database
    let db = Database::open_default()
        .context("Failed to open AgilePlus database")?;

    // Create sync engine for bidirectional sync
    let sync_engine = SyncEngine::new(client, db);

    if args.dry_run {
        println!("[dry-run] Inspecting Plane.so for inbound changes...");
        // List what would be pulled without applying changes
        let pending = sync_engine.list_pending_inbound_changes().await
            .context("Failed to list pending changes from Plane.so")?;

        let mut report = SyncReport::new(SyncDirection::Pull);
        for change in pending {
            report.add(SyncReportEntry {
                entity_kind: change.entity_type.clone(),
                entity_name: change.name.clone(),
                outcome: SyncItemOutcome::Updated,
                plane_id: Some(change.plane_id),
                message: Some(format!("would {} from state '{}'",
                    change.operation, change.remote_state)),
            });
        }

        report.duration_ms = start.elapsed().as_millis() as u64;
        print_pull_report(&report, true);
        return Ok(());
    }

    println!("Pulling changes from Plane.so...");
    let mut report = SyncReport::new(SyncDirection::Pull);

    // Sync specific feature or all features
    if let Some(ref slug) = args.feature {
        // Pull single feature by slug
        println!("  Syncing feature '{}'...", slug);
        match sync_engine.pull_feature_by_slug(slug).await {
            Ok(result) => {
                report.add(SyncReportEntry {
                    entity_kind: "feature".to_string(),
                    entity_name: slug.clone(),
                    outcome: if result.created {
                        SyncItemOutcome::Imported
                    } else if result.updated {
                        SyncItemOutcome::Updated
                    } else {
                        SyncItemOutcome::NoChange
                    },
                    plane_id: Some(result.plane_id),
                    message: result.message,
                });
            }
            Err(e) => {
                report.add(SyncReportEntry {
                    entity_kind: "feature".to_string(),
                    entity_name: slug.clone(),
                    outcome: SyncItemOutcome::Error,
                    plane_id: None,
                    message: Some(format!("Failed to pull: {}", e)),
                });
            }
        }
    } else {
        // Full sync - pull all features
        println!("  Scanning all features from Plane.so...");
        match sync_engine.pull_all_features().await {
            Ok(results) => {
                for result in results {
                    report.add(SyncReportEntry {
                        entity_kind: "feature".to_string(),
                        entity_name: result.name.clone(),
                        outcome: if result.created {
                            SyncItemOutcome::Imported
                        } else if result.updated {
                            SyncItemOutcome::Updated
                        } else {
                            SyncItemOutcome::NoChange
                        },
                        plane_id: Some(result.plane_id),
                        message: result.message,
                    });
                }
            }
            Err(e) => {
                return Err(e).context("Failed to pull features from Plane.so");
            }
        }

        // Sync work packages
        println!("  Scanning work packages from Plane.so...");
        match sync_engine.pull_all_work_packages().await {
            Ok(results) => {
                for result in results {
                    report.add(SyncReportEntry {
                        entity_kind: "work_package".to_string(),
                        entity_name: result.name.clone(),
                        outcome: if result.created {
                            SyncItemOutcome::Imported
                        } else if result.updated {
                            SyncItemOutcome::Updated
                        } else {
                            SyncItemOutcome::NoChange
                        },
                        plane_id: Some(result.plane_id),
                        message: result.message,
                    });
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to sync work packages: {}", e);
            }
        }
    }

    // Detect conflicts
    let conflicts = sync_engine.detect_conflicts().await
        .context("Failed to detect sync conflicts")?;

    for conflict in conflicts {
        report.add(SyncReportEntry {
            entity_kind: conflict.entity_type,
            entity_name: conflict.name,
            outcome: SyncItemOutcome::Conflict,
            plane_id: Some(conflict.plane_id),
            message: Some(conflict.description),
        });
    }

    report.duration_ms = start.elapsed().as_millis() as u64;
    print_pull_report(&report, false);

    // Print summary
    let imported = report.entries.iter().filter(|e| matches!(e.outcome, SyncItemOutcome::Imported)).count();
    let updated = report.entries.iter().filter(|e| matches!(e.outcome, SyncItemOutcome::Updated)).count();
    let conflicts = report.entries.iter().filter(|e| matches!(e.outcome, SyncItemOutcome::Conflict)).count();

    if conflicts > 0 {
        println!("\n⚠️  {} conflict(s) detected. Run 'agileplus sync resolve' to reconcile.", conflicts);
    }

    println!("\nSummary: {} imported, {} updated", imported, updated);

    Ok(())
}

fn print_pull_report(report: &SyncReport, dry_run: bool) {
    for entry in &report.entries {
        let icon = outcome_icon(&entry.outcome);
        let kind_label = format!("{} '{}'", entry.entity_kind, entry.entity_name);
        let suffix = match &entry.message {
            Some(msg) => format!(" ({})", msg),
            None => match &entry.plane_id {
                Some(id) => format!(" (plane_id: {})", id),
                None => String::new(),
            },
        };
        let verb = if dry_run {
            format!("[dry-run] {}", outcome_verb(&entry.outcome))
        } else {
            outcome_verb(&entry.outcome).to_string()
        };
        println!("{} {} {}{}", icon, kind_label, verb, suffix);
    }
    println!("Duration: {:.1}s", report.duration_ms as f64 / 1000.0);
}
