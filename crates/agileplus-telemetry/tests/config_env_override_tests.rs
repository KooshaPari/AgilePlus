//! Environment-override tests for `agileplus_telemetry::config`.
//!
//! `TelemetryConfig::load_from` layers `AGILEPLUS_LOG_LEVEL` and
//! `AGILEPLUS_OTLP_ENDPOINT` over the YAML file, and `TelemetryConfig::load`
//! does the same on top of defaults when the canonical file is missing. The
//! crate's existing tests explicitly skipped these branches ("apply_env_overrides
//! is private"); these tests drive them through the public loaders.
//!
//! Every test in this file mutates process-global environment variables, so all
//! tests hold one file-wide lock for the whole body and restore what they changed
//! through a `Drop` guard, even on panic.

use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::Path,
    sync::{Mutex, MutexGuard},
};

use agileplus_telemetry::config::{ConfigError, OtlpProtocol, TelemetryConfig};

const LOG_LEVEL: &str = "AGILEPLUS_LOG_LEVEL";
const OTLP_ENDPOINT: &str = "AGILEPLUS_OTLP_ENDPOINT";
const HOME: &str = "HOME";

// ---------------------------------------------------------------------------
// Environment guard
// ---------------------------------------------------------------------------

/// One lock for the whole file: the process environment is global state.
fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Restores one environment variable on drop, even if the test panics.
struct EnvGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvGuard {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            original: std::env::var_os(key),
        }
    }

    /// Capture and set `key`. Caller must hold [`env_lock`].
    fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
        let guard = Self::capture(key);
        // SAFETY: the caller holds `env_lock`, so no other thread in this test
        // binary reads or writes the process environment concurrently.
        unsafe { std::env::set_var(key, value) };
        guard
    }

    /// Capture and remove `key`. Caller must hold [`env_lock`].
    fn unset(key: &'static str) -> Self {
        let guard = Self::capture(key);
        // SAFETY: as above.
        unsafe { std::env::remove_var(key) };
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.original.take() {
            // SAFETY: as above; the lock is still held by the caller's guard.
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

/// Load a config from a temporary YAML file. Caller must hold [`env_lock`].
fn load_yaml(yaml: &str) -> Result<TelemetryConfig, ConfigError> {
    let mut file = tempfile::NamedTempFile::new().expect("create temp config file");
    write!(file, "{yaml}").expect("write temp config file");
    file.flush().expect("flush temp config file");
    TelemetryConfig::load_from(file.path())
}

/// Write `~/.agileplus/otel-config.yaml` under `home`. Caller must hold [`env_lock`].
fn write_home_config(home: &Path, yaml: &str) {
    let dir = home.join(".agileplus");
    std::fs::create_dir_all(&dir).expect("create .agileplus dir");
    std::fs::write(dir.join("otel-config.yaml"), yaml).expect("write home config");
}

// ---------------------------------------------------------------------------
// AGILEPLUS_LOG_LEVEL
// ---------------------------------------------------------------------------

#[test]
fn log_level_env_overrides_the_yaml_value() {
    let _lock = env_lock();
    let _level = EnvGuard::set(LOG_LEVEL, "warn");

    let config = load_yaml("logging:\n  level: \"debug\"\n").expect("load config");

    assert_eq!(config.logging.level, "warn");
}

#[test]
fn log_level_env_value_is_kept_verbatim_even_when_invalid() {
    let _lock = env_lock();
    let _level = EnvGuard::set(LOG_LEVEL, "not-a-level");

    let config = load_yaml("logging:\n  level: \"debug\"\n").expect("load config");

    // Config validation does not police log levels; the filter builder does.
    assert_eq!(config.logging.level, "not-a-level");
}

#[test]
fn yaml_log_level_is_kept_when_the_env_var_is_absent() {
    let _lock = env_lock();
    let _level = EnvGuard::unset(LOG_LEVEL);

    let config = load_yaml("logging:\n  level: \"error\"\n").expect("load config");

    assert_eq!(config.logging.level, "error");
}

// ---------------------------------------------------------------------------
// AGILEPLUS_OTLP_ENDPOINT
// ---------------------------------------------------------------------------

#[test]
fn otlp_endpoint_env_overrides_only_the_endpoint_of_an_existing_block() {
    let _lock = env_lock();
    let _endpoint = EnvGuard::set(OTLP_ENDPOINT, "http://collector.internal:4318");

    let config = load_yaml(
        r#"
otlp:
  endpoint: "http://localhost:4317"
  protocol: http
  headers:
    x-tenant: "acme"
  timeout_ms: 1234
  export_interval_ms: 4321
"#,
    )
    .expect("load config");

    let otlp = config.otlp.expect("otlp block");
    assert_eq!(otlp.endpoint, "http://collector.internal:4318");
    assert_eq!(
        otlp.protocol,
        OtlpProtocol::Http,
        "other fields must survive"
    );
    assert_eq!(
        otlp.headers.get("x-tenant").map(String::as_str),
        Some("acme")
    );
    assert_eq!(otlp.timeout_ms, 1234);
    assert_eq!(otlp.export_interval_ms, 4321);
}

#[test]
fn otlp_endpoint_env_synthesises_a_block_with_defaults_when_yaml_has_none() {
    let _lock = env_lock();
    let _endpoint = EnvGuard::set(OTLP_ENDPOINT, "http://env-collector:4317");

    let config = load_yaml("logging:\n  level: \"info\"\n").expect("load config");

    let otlp = config.otlp.expect("env endpoint must create an otlp block");
    assert_eq!(otlp.endpoint, "http://env-collector:4317");
    assert_eq!(otlp.protocol, OtlpProtocol::Grpc);
    assert_eq!(otlp.timeout_ms, 5_000);
    assert_eq!(otlp.export_interval_ms, 60_000);
    assert!(otlp.headers.is_empty());
}

#[test]
fn otlp_block_stays_absent_without_a_yaml_block_or_env_endpoint() {
    let _lock = env_lock();
    let _endpoint = EnvGuard::unset(OTLP_ENDPOINT);

    let config = load_yaml("logging:\n  level: \"info\"\n").expect("load config");

    assert!(config.otlp.is_none());
}

#[test]
fn otlp_endpoint_from_env_is_validated_like_a_yaml_endpoint() {
    let _lock = env_lock();
    let _endpoint = EnvGuard::set(OTLP_ENDPOINT, "definitely not a url");

    let error = load_yaml("logging:\n  level: \"info\"\n").expect_err("invalid env URL must fail");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("not a valid URL"), "message: {message}");
        }
        other => panic!("expected a validation error, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Canonical path loading (~/.agileplus/otel-config.yaml)
// ---------------------------------------------------------------------------

#[test]
fn load_reads_the_canonical_config_file_from_home() {
    let _lock = env_lock();
    let home = tempfile::tempdir().expect("temp home");
    write_home_config(
        home.path(),
        "otlp:\n  endpoint: \"http://from-home-config:4317\"\nlogging:\n  level: \"trace\"\n",
    );
    let _home = EnvGuard::set(HOME, home.path().as_os_str());
    let _level = EnvGuard::unset(LOG_LEVEL);
    let _endpoint = EnvGuard::unset(OTLP_ENDPOINT);

    let config = TelemetryConfig::load().expect("load canonical config");

    assert_eq!(config.logging.level, "trace");
    assert_eq!(
        config.otlp.expect("otlp block").endpoint,
        "http://from-home-config:4317"
    );
}

#[test]
fn load_returns_defaults_plus_env_overrides_when_the_file_is_missing() {
    let _lock = env_lock();
    let home = tempfile::tempdir().expect("temp home");
    let _home = EnvGuard::set(HOME, home.path().as_os_str());
    let _level = EnvGuard::unset(LOG_LEVEL);
    let _endpoint = EnvGuard::set(OTLP_ENDPOINT, "http://env-only:4317");

    let config = TelemetryConfig::load().expect("load defaults");

    assert_eq!(config.logging.level, "info");
    assert_eq!(config.sampling.trace_ratio, 1.0);
    let otlp = config.otlp.expect("env endpoint must create an otlp block");
    assert_eq!(otlp.endpoint, "http://env-only:4317");
    assert_eq!(otlp.timeout_ms, 5_000);
}

#[test]
fn load_without_env_overrides_yields_defaults_when_the_file_is_missing() {
    let _lock = env_lock();
    let home = tempfile::tempdir().expect("temp home");
    let _home = EnvGuard::set(HOME, home.path().as_os_str());
    let _level = EnvGuard::unset(LOG_LEVEL);
    let _endpoint = EnvGuard::unset(OTLP_ENDPOINT);

    let config = TelemetryConfig::load().expect("load defaults");

    assert!(config.otlp.is_none());
    assert_eq!(config.logging.level, "info");
    assert!(config.logging.include_spans);
    assert_eq!(config.sampling.trace_ratio, 1.0);
}
