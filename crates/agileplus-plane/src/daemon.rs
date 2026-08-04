//! Plane sync daemon — background polling loop that pushes AgilePlus state to Plane.so
//!
// Wraps the existing `runtime::maybe_sync_*_from_env()` per-entity hooks in a
//! `tokio::spawn` loop with configurable interval. Reuses all existing push/outbound
//! code; no new transport or sync logic required.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{error, info, warn};

use agileplus_plane::runtime::{self, PlaneSyncOutcome};

/// Configuration for the Plane sync daemon.
#[derive(Debug, Clone)]
pub struct PlaneDaemonConfig {
    /// How often the daemon polls and pushes state to Plane.
    pub interval: Duration,
    /// Module/project slug to push (e.g. "AGP"). If `None`, the daemon
    /// reads `PLANE_PROJECT` from env.
    pub project_slug: Option<String>,
}

impl Default for PlaneDaemonConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(60),
            project_slug: None,
        }
    }
}

/// Handle to a running Plane sync daemon. Dropping it without calling
/// `stop()` does not stop the task — call `stop()` explicitly for graceful
/// shutdown.
pub struct PlaneSyncDaemon {
    handle: JoinHandle<()>,
    stop_notify: Arc<Notify>,
    config: PlaneDaemonConfig,
}

impl PlaneSyncDaemon {
    /// Spawn the daemon. Returns immediately. The daemon runs until `stop()`
    /// is called or the process exits.
    pub fn spawn(config: PlaneDaemonConfig) -> Self {
        let stop_notify = Arc::new(Notify::new());
        let stop_signal = Arc::clone(&stop_notify);
        let cfg = config.clone();

        let handle = tokio::spawn(async move {
            run_loop(cfg, stop_signal).await;
        });

        Self {
            handle,
            stop_notify,
            config,
        }
    }

    /// Stop the daemon. Waits for the current tick to finish (up to ~one
    /// interval). Idempotent.
    pub async fn stop(self) {
        self.stop_notify.notify_one();
        // Best-effort join with a short timeout
        let _ = tokio::time::timeout(
            self.config.interval + Duration::from_secs(5),
            self.handle,
        )
        .await;
        info!("PlaneSyncDaemon stopped");
    }

    /// Returns the current configuration (useful for diagnostics).
    pub fn config(&self) -> &PlaneDaemonConfig {
        &self.config
    }
}

async fn run_loop(config: PlaneDaemonConfig, stop: Arc<Notify>) {
    info!(
        interval_secs = config.interval.as_secs(),
        "PlaneSyncDaemon started"
    );

    let mut ticker = interval(config.interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                tick_once(&config).await;
            }
            _ = stop.notified() => {
                info!("PlaneSyncDaemon received stop signal");
                break;
            }
        }
    }
}

async fn tick_once(config: &PlaneDaemonConfig) {
    let project = config
        .project_slug
        .clone()
        .or_else(|| std::env::var("PLANE_PROJECT").ok())
        .unwrap_or_else(|| "AGP".to_string());

    let outcomes = sync_all(&project).await;
    let mut pushed = 0usize;
    let mut failed = 0usize;
    for (label, outcome) in outcomes {
        match outcome {
            PlaneSyncOutcome::Pushed(n) => {
                pushed += n;
                info!(target: "plane.daemon", entity = %label, count = n, "pushed");
            }
            PlaneSyncOutcome::Skipped(reason) => {
                debug_skip(&label, &reason);
            }
            PlaneSyncOutcome::Failed(err) => {
                failed += 1;
                warn!(target: "plane.daemon", entity = %label, error = %err, "sync failed");
            }
        }
    }
    if failed > 0 {
        error!(
            target: "plane.daemon",
            pushed,
            failed,
            "Plane sync tick had failures"
        );
    } else {
        info!(
            target: "plane.daemon",
            pushed,
            "Plane sync tick complete"
        );
    }
}

/// Top-level sync helper — fan out to all per-entity hooks. Returns a vec
/// of `(label, outcome)` so the caller can log per-entity results.
async fn sync_all(project: &str) -> Vec<(String, PlaneSyncOutcome)> {
    let mut out = Vec::with_capacity(4);

    // Sync modules first (parents), then cycles, then features
    let module = runtime::maybe_sync_module_from_env(project).await;
    out.push(("module".to_string(), module));

    let cycles = runtime::maybe_sync_cycle_from_env(project).await;
    out.push(("cycle".to_string(), cycles));

    let features = runtime::maybe_sync_feature_from_env(project).await;
    out.push(("feature".to_string(), features));

    // If the work-packages sync exists, push it too
    let work_packages = runtime::maybe_sync_work_package_from_env(project).await;
    out.push(("work_package".to_string(), work_packages));

    out
}

fn debug_skip(label: &str, reason: &str) {
    if reason.is_empty() {
        info!(target: "plane.daemon", entity = %label, "skipped");
    } else {
        info!(target: "plane.daemon", entity = %label, reason = %reason, "skipped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_one_minute_interval() {
        let cfg = PlaneDaemonConfig::default();
        assert_eq!(cfg.interval, Duration::from_secs(60));
        assert!(cfg.project_slug.is_none());
    }

    #[test]
    fn config_is_cloneable() {
        let cfg = PlaneDaemonConfig {
            interval: Duration::from_secs(5),
            project_slug: Some("TEST".to_string()),
        };
        let _ = cfg.clone();
    }
}
