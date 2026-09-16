//! Integration tests for daemon module.
//!
//! Covers: PlaneDaemonConfig, SyncState serialization, env-based config.

use std::sync::Mutex;
use std::time::Duration;

use agileplus_plane::daemon::{PlaneDaemonConfig, SyncState};

// Serialize all env-var tests to avoid parallel interference
static ENV_MUTEX: Mutex<()> = Mutex::new(());

// ── PlaneDaemonConfig defaults ──────────────────────────────

#[test]
fn config_default_interval() {
    let cfg = PlaneDaemonConfig::default();
    assert_eq!(cfg.interval, Duration::from_secs(300));
}

#[test]
fn config_default_batch_size() {
    let cfg = PlaneDaemonConfig::default();
    assert_eq!(cfg.batch_size, 25);
}

#[test]
fn config_default_dry_run() {
    let cfg = PlaneDaemonConfig::default();
    assert!(!cfg.dry_run);
}

// ── PlaneDaemonConfig construction ──────────────────────────

#[test]
fn config_custom_values() {
    let cfg = PlaneDaemonConfig {
        interval: Duration::from_secs(60),
        batch_size: 50,
        dry_run: true,
    };
    assert_eq!(cfg.interval, Duration::from_secs(60));
    assert_eq!(cfg.batch_size, 50);
    assert!(cfg.dry_run);
}

#[test]
fn config_clone() {
    let cfg = PlaneDaemonConfig {
        interval: Duration::from_secs(120),
        batch_size: 10,
        dry_run: true,
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.interval, Duration::from_secs(120));
    assert_eq!(cloned.batch_size, 10);
    assert!(cloned.dry_run);
}

#[test]
fn config_debug_format() {
    let cfg = PlaneDaemonConfig::default();
    let debug = format!("{:?}", cfg);
    assert!(debug.contains("PlaneDaemonConfig"));
    assert!(debug.contains("batch_size"));
    assert!(debug.contains("dry_run"));
}

// ── PlaneDaemonConfig serialization ─────────────────────────

#[test]
fn config_serialization_roundtrip() {
    let cfg = PlaneDaemonConfig {
        interval: Duration::from_secs(120),
        batch_size: 50,
        dry_run: true,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let restored: PlaneDaemonConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.interval, Duration::from_secs(120));
    assert_eq!(restored.batch_size, 50);
    assert!(restored.dry_run);
}

#[test]
fn config_serialization_contains_expected_keys() {
    let cfg = PlaneDaemonConfig::default();
    let json = serde_json::to_string(&cfg).unwrap();
    assert!(json.contains("interval"));
    assert!(json.contains("batch_size"));
    assert!(json.contains("dry_run"));
}

#[test]
fn config_deserialization_with_null_interval() {
    let json = r#"{
        "interval": {"secs": 60, "nanos": 0},
        "batch_size": 10,
        "dry_run": false
    }"#;
    let cfg: PlaneDaemonConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.interval, Duration::from_secs(60));
    assert_eq!(cfg.batch_size, 10);
}

// ── PlaneDaemonConfig from_env ──────────────────────────────
// All env-based tests acquire the mutex to prevent parallel interference.

#[test]
fn config_from_env_defaults_when_unset() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert_eq!(cfg.interval, Duration::from_secs(300));
    assert_eq!(cfg.batch_size, 25);
    assert!(!cfg.dry_run);
}

#[test]
fn config_from_env_custom_values() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "30");
        std::env::set_var("PLANE_DAEMON_BATCH_SIZE", "100");
        std::env::set_var("PLANE_DAEMON_DRY_RUN", "1");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert_eq!(cfg.interval, Duration::from_secs(30));
    assert_eq!(cfg.batch_size, 100);
    assert!(cfg.dry_run);
    // Cleanup
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }
}

#[test]
fn config_from_env_dry_run_true_string() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::set_var("PLANE_DAEMON_DRY_RUN", "true");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert!(cfg.dry_run);
    unsafe {
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }
}

#[test]
fn config_from_env_dry_run_true_uppercase() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::set_var("PLANE_DAEMON_DRY_RUN", "TRUE");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert!(cfg.dry_run);
    unsafe {
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
    }
}

#[test]
fn config_from_env_invalid_interval_falls_back() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
        std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "not_a_number");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert_eq!(cfg.interval, Duration::from_secs(300));
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
    }
}

#[test]
fn config_from_env_invalid_batch_size_falls_back() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
        std::env::set_var("PLANE_DAEMON_BATCH_SIZE", "abc");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert_eq!(cfg.batch_size, 25);
    unsafe {
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
    }
}

#[test]
fn config_from_env_zero_interval() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
        std::env::remove_var("PLANE_DAEMON_DRY_RUN");
        std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "0");
    }
    let cfg = PlaneDaemonConfig::from_env();
    assert_eq!(cfg.interval, Duration::from_secs(0));
    unsafe {
        std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
    }
}

// ── SyncState ───────────────────────────────────────────────

#[test]
fn sync_state_default() {
    let state = SyncState::default();
    assert!(!state.running);
    assert!(state.last_tick_at.is_none());
    assert_eq!(state.last_tick_duration_ms, 0);
    assert_eq!(state.modules_synced, 0);
    assert_eq!(state.cycles_synced, 0);
    assert_eq!(state.errors, 0);
}

#[test]
fn sync_state_clone() {
    let state = SyncState {
        running: true,
        last_tick_at: Some(chrono::Utc::now()),
        last_tick_duration_ms: 42,
        modules_synced: 5,
        cycles_synced: 3,
        errors: 1,
    };
    let cloned = state.clone();
    assert_eq!(cloned.running, true);
    assert_eq!(cloned.modules_synced, 5);
    assert_eq!(cloned.cycles_synced, 3);
    assert_eq!(cloned.errors, 1);
}

#[test]
fn sync_state_debug_format() {
    let state = SyncState::default();
    let debug = format!("{:?}", state);
    assert!(debug.contains("SyncState"));
}

#[test]
fn sync_state_serialization_roundtrip() {
    let state = SyncState {
        running: true,
        last_tick_at: Some(chrono::Utc::now()),
        last_tick_duration_ms: 42,
        modules_synced: 10,
        cycles_synced: 7,
        errors: 2,
    };
    let json = serde_json::to_string(&state).unwrap();
    let restored: SyncState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.running, true);
    assert_eq!(restored.modules_synced, 10);
    assert_eq!(restored.cycles_synced, 7);
    assert_eq!(restored.errors, 2);
}

#[test]
fn sync_state_serialization_json_structure() {
    let state = SyncState::default();
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("running"));
    assert!(json.contains("last_tick_at"));
    assert!(json.contains("modules_synced"));
    assert!(json.contains("cycles_synced"));
    assert!(json.contains("errors"));
}

#[test]
fn sync_state_deserialization_null_tick() {
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
    assert!(state.last_tick_at.is_none());
}
