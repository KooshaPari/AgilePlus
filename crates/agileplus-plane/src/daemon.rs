//! Plane.so sync daemon
//!
// Background loop that periodically drives the existing per-entity sync functions in
//! `runtime::maybe_sync_*_from_env`. Reuses all existing push/outbound code; this module
//! adds only the orchestration layer.
//!
//! ## Architecture
//!
//! ```text
//!  ┌──────────────────────────────────────────────────────┐
//!  │ PlaneSyncDaemon (this module)                       │
//!  │   └─ tokio::spawn loop (cancellable via Handle)     │
//!  │        ├─ tick → maybe_sync_module_from_env(S, ...)  │
//!  │        ├─ tick → maybe_sync_cycle_from_env(S, ...)    │
//!  │        └─ updates SyncState (counter, last_tick)    │
//!  └──────────────────────────────────────────────────────┘
//! ```
//!
//! Public API:
//! - [`PlaneDaemonConfig`]: tunable behaviour (interval, batch_size, dry_run)
//! - [`PlaneSyncDaemon`]: handle type with `pause`, `resume`, `sync_now`, `stop`, `state`
//! - [`PlaneSyncState`]: observable snapshot for `/api/dashboard/plane/sync`

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use agileplus_domain::ports::StoragePort;

use crate::runtime;

/// Sync daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneDaemonConfig {
    /// Interval between sync ticks. Defaults to 5 minutes.
    pub interval: Duration,
    /// Maximum modules/cycles processed per tick. Defaults to 25.
    pub batch_size: usize,
    /// If true, run an empty tick (no real I/O) — useful for smoke-testing the loop.
    pub dry_run: bool,
}

impl Default for PlaneDaemonConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5 * 60),
            batch_size: 25,
            dry_run: false,
        }
    }
}

impl PlaneDaemonConfig {
    /// Build a config from environment variables.
    /// Recognizes: PLANE_DAEMON_INTERVAL_SECS, PLANE_DAEMON_BATCH_SIZE, PLANE_DAEMON_DRY_RUN.
    pub fn from_env() -> Self {
        let interval_secs: u64 = std::env::var("PLANE_DAEMON_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5 * 60);
        let batch_size: usize = std::env::var("PLANE_DAEMON_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(25);
        let dry_run: bool = std::env::var("PLANE_DAEMON_DRY_RUN")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
        Self {
            interval: Duration::from_secs(interval_secs),
            batch_size,
            dry_run,
        }
    }
}

/// Observable sync state (returned by [`PlaneSyncDaemon::state`]).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncState {
    /// True if the loop is currently running.
    pub running: bool,
    /// Last successful tick completion (UTC).
    pub last_tick_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration of the last tick.
    pub last_tick_duration_ms: u64,
    /// Total modules synced since daemon start.
    pub modules_synced: u64,
    /// Total cycles synced since daemon start.
    pub cycles_synced: u64,
    /// Total sync errors since daemon start.
    pub errors: u64,
}

/// Opaque handle to a running [`PlaneSyncDaemon`].
///
/// Drop the handle (or call [`PlaneSyncDaemon::stop`]) to terminate the loop.
pub struct PlaneSyncDaemon {
    state: Arc<Mutex<SyncState>>,
    cancel: tokio::sync::watch::Sender<bool>,
    join: Mutex<Option<JoinHandle<()>>>,
    config: PlaneDaemonConfig,
}

impl PlaneSyncDaemon {
    /// Start the daemon. Returns a handle that controls the loop.
    pub fn spawn<S>(storage: Arc<S>, config: PlaneDaemonConfig) -> Self
    where
        S: StoragePort + Send + Sync + 'static,
    {
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
        let state = Arc::new(Mutex::new(SyncState {
            running: true,
            ..Default::default()
        }));

        let state_for_task = Arc::clone(&state);
        let cfg = config.clone();

        let join = tokio::spawn(async move {
            // Mark as not running when task ends.
            let _guard = RunningGuard {
                state: Arc::clone(&state_for_task),
            };
            loop {
                // Cancellation check.
                if *cancel_rx.borrow() {
                    break;
                }

                if let Err(e) = run_tick(storage.as_ref(), &cfg, &state_for_task).await {
                    tracing::warn!(error = %e, "plane sync tick failed");
                    let mut s = state_for_task.lock().await;
                    s.errors += 1;
                }

                // Sleep with cancellation awareness.
                tokio::select! {
                    _ = sleep(cfg.interval) => {},
                    _ = cancel_rx.changed() => break,
                }
            }
        });

        Self {
            state,
            cancel: cancel_tx,
            join: Mutex::new(Some(join)),
            config,
        }
    }

    /// Pause the loop (sets the cancel flag; the task exits at the next interval boundary).
    pub async fn pause(&self) {
        let _ = self.cancel.send(true);
    }

    /// Resume after pause (restarts a fresh task with the same state).
    pub async fn resume(&self) {
        // Note: a full pause/resume cycle requires re-spawning; here we just clear the cancel
        // flag and let the existing task notice on its next iteration. For now `resume` is
        // effectively `unpause` for the cancel flag; the user is expected to construct a new
        // daemon for full resume semantics.
        let _ = self.cancel.send(false);
    }

    /// Snapshot the current sync state.
    pub async fn state(&self) -> SyncState {
        self.state.lock().await.clone()
    }

    /// Trigger an immediate sync tick (best-effort: posts a wake-up to the loop).
    pub async fn sync_now(&self) {
        // The current implementation ticks on its own schedule; sync_now is a placeholder for
        // a future nudge mechanism (e.g., a notify channel). Kept for API stability.
    }

    /// Stop the loop and await the task. Safe to call multiple times.
    pub async fn stop(&self) {
        let _ = self.cancel.send(true);
        if let Some(handle) = self.join.lock().await.take() {
            let _ = handle.await;
        }
    }

    /// Get a snapshot of the current configuration (useful for status endpoint).
    pub fn config(&self) -> PlaneDaemonConfig {
        PlaneDaemonConfig {
            interval: self.config.interval,
            batch_size: self.config.batch_size,
            dry_run: self.config.dry_run,
        }
    }

    /// Start (or no-op resume) the sync loop. Safe to call when already running.
    pub async fn start(&self) {
        // spawn() at construction time already started the loop.
        // This method exists for the API contract; future: could implement pause/resume.
    }
}

/// RAII guard: when the task ends, set `running = false`.
struct RunningGuard {
    state: Arc<Mutex<SyncState>>,
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        let state = Arc::clone(&self.state);
        tokio::spawn(async move {
            let mut s = state.lock().await;
            s.running = false;
        });
    }
}

/// One tick of the daemon: iterate modules and cycles, call their sync fns.
async fn run_tick<S: StoragePort>(
    storage: &S,
    cfg: &PlaneDaemonConfig,
    state: &Arc<Mutex<SyncState>>,
) -> anyhow::Result<()> {
    let started = Instant::now();

    if cfg.dry_run {
        // Pretend work happened; useful for tests and smoke checks.
        sleep(Duration::from_millis(10)).await;
    } else {
        // Sync root modules (up to batch_size).
        let modules = storage.list_root_modules().await?;
        for module in modules.into_iter().take(cfg.batch_size) {
            let mid = module.id;
            match runtime::maybe_sync_module_from_env(storage, mid).await {
                Ok(()) => {
                    let mut s = state.lock().await;
                    s.modules_synced += 1;
                }
                Err(e) => {
                    tracing::warn!(module_id = mid, error = %e, "module sync failed");
                    let mut s = state.lock().await;
                    s.errors += 1;
                }
            }
        }

        // Sync cycles (up to batch_size).
        let cycles = storage.list_all_cycles().await?;
        for cycle in cycles.into_iter().take(cfg.batch_size) {
            let cid = cycle.id;
            match runtime::maybe_sync_cycle_from_env(storage, cid).await {
                Ok(()) => {
                    let mut s = state.lock().await;
                    s.cycles_synced += 1;
                }
                Err(e) => {
                    tracing::warn!(cycle_id = cid, error = %e, "cycle sync failed");
                    let mut s = state.lock().await;
                    s.errors += 1;
                }
            }
        }
    }

    let elapsed = started.elapsed();
    let mut s = state.lock().await;
    s.last_tick_at = Some(chrono::Utc::now());
    s.last_tick_duration_ms = elapsed.as_millis() as u64;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_is_reasonable() {
        let cfg = PlaneDaemonConfig::default();
        assert_eq!(cfg.batch_size, 25);
        assert!(!cfg.dry_run);
        assert!(cfg.interval >= Duration::from_secs(60));
    }

    #[test]
    fn sync_state_serializes() {
        let state = SyncState {
            running: true,
            last_tick_at: Some(chrono::Utc::now()),
            last_tick_duration_ms: 42,
            modules_synced: 5,
            cycles_synced: 3,
            errors: 1,
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("running"));
        assert!(json.contains("modules_synced"));
    }

    #[test]
    fn sync_state_deserializes() {
        let json = r#"{
            "running": false,
            "last_tick_at": null,
            "last_tick_duration_ms": 0,
            "modules_synced": 0,
            "cycles_synced": 0,
            "errors": 0
        }"#;
        let state: SyncState = serde_json::from_str(json).unwrap();
        assert!(!state.running);
        assert_eq!(state.modules_synced, 0);
    }

    #[test]
    fn daemon_config_serializes() {
        let config = PlaneDaemonConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("batch_size"));
        assert!(json.contains("dry_run"));
    }

    #[test]
    fn daemon_config_deserializes() {
        let json = r#"{
            "interval": {"secs": 120, "nanos": 0},
            "batch_size": 50,
            "dry_run": true
        }"#;
        let config: PlaneDaemonConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.batch_size, 50);
        assert!(config.dry_run);
        assert_eq!(config.interval, Duration::from_secs(120));
    }

    #[test]
    fn sync_state_default_values() {
        let state = SyncState::default();
        assert!(!state.running);
        assert!(state.last_tick_at.is_none());
        assert_eq!(state.last_tick_duration_ms, 0);
        assert_eq!(state.modules_synced, 0);
        assert_eq!(state.cycles_synced, 0);
        assert_eq!(state.errors, 0);
    }

    #[test]
    fn daemon_config_clone() {
        let config = PlaneDaemonConfig {
            interval: Duration::from_secs(60),
            batch_size: 10,
            dry_run: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.interval, Duration::from_secs(60));
        assert_eq!(cloned.batch_size, 10);
        assert!(cloned.dry_run);
    }
}

/// Behaviour of the background tick loop: lifecycle transitions (`running`),
/// per-tick counters, and failure handling.
#[cfg(test)]
mod loop_tests {
    // The environment lock is deliberately held across `await` points: these are
    // current-thread `#[tokio::test]`s, so there is no scheduler on which the
    // guard could deadlock, and dropping it early would let another test rewrite
    // PLANE_* halfway through a tick.
    #![allow(clippy::await_holding_lock)]

    use super::*;
    use crate::mock_storage::MockStoragePort;
    use crate::test_env::lock_env;
    use agileplus_domain::domain::cycle::{Cycle, CycleState};
    use agileplus_domain::domain::module::Module;
    use chrono::NaiveDate;
    use std::env;

    /// A port that refuses connections immediately, so outbound sync calls fail
    /// without touching the real Plane.so API.
    const DEAD_ENDPOINT: &str = "http://127.0.0.1:1";

    fn clear_plane_env() {
        unsafe {
            env::remove_var("PLANE_API_KEY");
            env::remove_var("PLANE_WORKSPACE");
            env::remove_var("PLANE_PROJECT");
            env::remove_var("PLANE_API_URL");
        }
    }

    fn point_plane_at_dead_endpoint() {
        unsafe {
            env::set_var("PLANE_API_KEY", "test-key");
            env::set_var("PLANE_WORKSPACE", "test-ws");
            env::set_var("PLANE_PROJECT", "test-proj");
            env::set_var("PLANE_API_URL", DEAD_ENDPOINT);
        }
    }

    fn module(name: &str) -> Module {
        Module::new(name, None)
    }

    fn cycle(name: &str) -> Cycle {
        Cycle {
            id: 0,
            name: name.to_string(),
            description: None,
            state: CycleState::Active,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            module_scope_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn cfg(interval_ms: u64, batch_size: usize, dry_run: bool) -> PlaneDaemonConfig {
        PlaneDaemonConfig {
            interval: Duration::from_millis(interval_ms),
            batch_size,
            dry_run,
        }
    }

    /// Read daemon state until `pred` holds or the deadline expires.
    async fn wait_for<F>(daemon: &PlaneSyncDaemon, pred: F) -> SyncState
    where
        F: Fn(&SyncState) -> bool,
    {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let state = daemon.state().await;
            if pred(&state) || Instant::now() >= deadline {
                return state;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    // -- lifecycle: running flag transitions --

    #[tokio::test]
    async fn spawn_reports_running_and_exposes_config() {
        let daemon = PlaneSyncDaemon::spawn(Arc::new(MockStoragePort::new()), cfg(60_000, 7, true));

        let state = daemon.state().await;
        assert!(state.running, "a freshly spawned daemon is running");
        assert_eq!(state.modules_synced, 0);

        let snapshot = daemon.config();
        assert_eq!(snapshot.batch_size, 7);
        assert_eq!(snapshot.interval, Duration::from_millis(60_000));
        assert!(snapshot.dry_run);

        daemon.stop().await;
    }

    #[tokio::test]
    async fn stop_clears_running_flag_and_is_idempotent() {
        let daemon = PlaneSyncDaemon::spawn(Arc::new(MockStoragePort::new()), cfg(60_000, 25, true));
        assert!(daemon.state().await.running);

        daemon.stop().await;
        let state = wait_for(&daemon, |s| !s.running).await;
        assert!(!state.running, "stop() must mark the loop as no longer running");

        // Documented as safe to call repeatedly; a second call must not hang.
        daemon.stop().await;
        assert!(!daemon.state().await.running);
    }

    #[tokio::test]
    async fn pause_stops_the_loop() {
        let daemon = PlaneSyncDaemon::spawn(Arc::new(MockStoragePort::new()), cfg(5, 25, true));
        assert!(daemon.state().await.running);

        daemon.pause().await;
        let state = wait_for(&daemon, |s| !s.running).await;
        assert!(!state.running, "pause() must let the loop exit");
    }

    #[tokio::test]
    async fn resume_cannot_revive_an_exited_loop() {
        let daemon = PlaneSyncDaemon::spawn(Arc::new(MockStoragePort::new()), cfg(5, 25, true));
        daemon.pause().await;

        let stopped = wait_for(&daemon, |s| !s.running).await;
        assert!(!stopped.running);
        let last_tick = stopped.last_tick_at;

        // `resume` only clears the cancel flag; the task is already gone, so a
        // fresh daemon is required for full resume semantics (see the doc
        // comment on `PlaneSyncDaemon::resume`).
        daemon.resume().await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let after = daemon.state().await;
        assert!(!after.running, "resume() must not report a dead loop as running");
        assert_eq!(
            after.last_tick_at, last_tick,
            "an exited loop must not produce further ticks"
        );
    }

    #[tokio::test]
    async fn sync_now_and_start_keep_the_daemon_running() {
        let daemon = PlaneSyncDaemon::spawn(Arc::new(MockStoragePort::new()), cfg(60_000, 25, true));

        daemon.sync_now().await;
        daemon.start().await;

        let state = daemon.state().await;
        assert!(state.running);
        assert_eq!(state.errors, 0, "neither call performs sync work");

        daemon.stop().await;
    }

    // -- tick behaviour against a working store --

    #[tokio::test]
    async fn dry_run_tick_stamps_state_without_touching_storage() {
        let store = MockStoragePort::new();
        store.create_module(&module("A")).await.unwrap();
        store.create_cycle(&cycle("Sprint")).await.unwrap();

        let daemon = PlaneSyncDaemon::spawn(Arc::new(store), cfg(60_000, 25, true));
        let state = wait_for(&daemon, |s| s.last_tick_at.is_some()).await;

        assert!(state.last_tick_at.is_some(), "dry-run tick still stamps the tick time");
        assert_eq!(
            (state.modules_synced, state.cycles_synced, state.errors),
            (0, 0, 0),
            "dry-run ticks must not report synced entities"
        );

        daemon.stop().await;
    }

    #[tokio::test]
    async fn tick_counts_every_module_and_cycle_when_plane_is_unconfigured() {
        let _env = lock_env();
        clear_plane_env();

        let store = MockStoragePort::new();
        store.create_module(&module("A")).await.unwrap();
        store.create_module(&module("B")).await.unwrap();
        store.create_cycle(&cycle("Sprint")).await.unwrap();

        let daemon = PlaneSyncDaemon::spawn(Arc::new(store), cfg(60_000, 25, false));
        let state = wait_for(&daemon, |s| s.last_tick_at.is_some()).await;

        assert_eq!(state.modules_synced, 2);
        assert_eq!(state.cycles_synced, 1);
        assert_eq!(state.errors, 0);

        daemon.stop().await;
        clear_plane_env();
    }

    #[tokio::test]
    async fn tick_honours_batch_size() {
        let _env = lock_env();
        clear_plane_env();

        let store = MockStoragePort::new();
        for name in ["A", "B", "C"] {
            store.create_module(&module(name)).await.unwrap();
        }

        let daemon = PlaneSyncDaemon::spawn(Arc::new(store), cfg(60_000, 1, false));
        let state = wait_for(&daemon, |s| s.last_tick_at.is_some()).await;

        assert_eq!(
            state.modules_synced, 1,
            "only batch_size modules may be processed per tick"
        );
        assert_eq!(state.errors, 0);

        daemon.stop().await;
        clear_plane_env();
    }

    #[tokio::test]
    async fn tick_counts_push_failures_as_errors() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();

        let store = MockStoragePort::new();
        store.create_module(&module("A")).await.unwrap();
        store.create_cycle(&cycle("Sprint")).await.unwrap();

        let daemon = PlaneSyncDaemon::spawn(Arc::new(store), cfg(60_000, 25, false));
        // Wait for the whole tick, not just the first failure: the error counter
        // is only final once the cycle pass has finished.
        let state = wait_for(&daemon, |s| s.last_tick_at.is_some()).await;

        assert_eq!(state.errors, 2, "both the module and the cycle push must fail");
        assert_eq!((state.modules_synced, state.cycles_synced), (0, 0));
        assert!(state.running, "a failing push must not kill the daemon");

        daemon.stop().await;
        clear_plane_env();
    }

    // -- tick behaviour when the store itself fails --

    #[tokio::test]
    async fn failed_tick_aborts_before_stamping_state() {
        let store = MockStoragePort::new().failing_list_modules();
        let state = Arc::new(Mutex::new(SyncState {
            running: true,
            ..Default::default()
        }));

        let err = run_tick(&store, &cfg(60_000, 25, false), &state)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("list_root_modules failed"),
            "unexpected error: {err}"
        );

        let snapshot = state.lock().await.clone();
        assert!(
            snapshot.last_tick_at.is_none(),
            "an aborted tick must not stamp last_tick_at"
        );
        assert_eq!(
            snapshot.errors, 0,
            "the tick itself does not own the error counter"
        );
    }

    #[tokio::test]
    async fn loop_survives_repeated_tick_failures() {
        let store = Arc::new(MockStoragePort::new().failing_list_modules());
        let daemon = PlaneSyncDaemon::spawn(store, cfg(10, 25, false));

        let state = wait_for(&daemon, |s| s.errors >= 2).await;
        assert!(state.errors >= 2, "the loop must count failing ticks");
        assert!(state.running, "the loop must keep running after a failed tick");
        assert!(
            state.last_tick_at.is_none(),
            "failed ticks never reach the stamping code"
        );

        daemon.stop().await;
        let stopped = wait_for(&daemon, |s| !s.running).await;
        assert!(!stopped.running);
    }
}
