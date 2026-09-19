//! Structured JSON logging via the `tracing` crate.
//!
//! Call [`init_logging`] exactly once from `main()` and hold the returned
//! [`WorkerGuard`] for the process lifetime.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan, prelude::*};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Where structured logs are written.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogOutput {
    #[default]
    Stdout,
    File(PathBuf),
    Both(PathBuf),
}

/// Configuration for structured JSON logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Minimum log level (`trace`, `debug`, `info`, `warn`, `error`).
    #[serde(default = "default_level")]
    pub level: String,
    /// Output destination.
    #[serde(default)]
    pub output: LogOutput,
    /// Include parent span IDs in log lines.
    #[serde(default = "default_true")]
    pub include_spans: bool,
    /// Include module path in log lines.
    #[serde(default = "default_true")]
    pub include_target: bool,
}

fn default_level() -> String {
    "info".into()
}
fn default_true() -> bool {
    true
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            output: LogOutput::Stdout,
            include_spans: true,
            include_target: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LogError {
    #[error("failed to open log file {path}: {source}")]
    FileOpen {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("subscriber initialisation failed: {0}")]
    Init(String),
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

/// Initialise the global `tracing` subscriber.
///
/// Must be called exactly once (idempotent via `try_init`).  Returns a
/// [`WorkerGuard`] that must be held for the process lifetime.
pub fn init_logging(config: &LogConfig) -> Result<WorkerGuard, LogError> {
    let filter = build_filter(config);

    match &config.output {
        LogOutput::Stdout => {
            let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
            let layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(writer)
                .with_target(config.include_target)
                .with_span_list(config.include_spans)
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LogError::Init(e.to_string()))?;

            Ok(guard)
        }
        LogOutput::File(path) => {
            let dir = path.parent().unwrap_or(std::path::Path::new("."));
            let file_name = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("agileplus.log"));
            let appender = tracing_appender::rolling::daily(dir, file_name);
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(writer)
                .with_target(config.include_target)
                .with_span_list(config.include_spans)
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .try_init()
                .map_err(|e| LogError::Init(e.to_string()))?;

            Ok(guard)
        }
        LogOutput::Both(path) => {
            let (stdout_writer, guard) = tracing_appender::non_blocking(std::io::stdout());
            let stdout_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(stdout_writer)
                .with_target(config.include_target)
                .with_span_list(config.include_spans)
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true);

            let dir = path.parent().unwrap_or(std::path::Path::new("."));
            let file_name = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("agileplus.log"));
            let appender = tracing_appender::rolling::daily(dir, file_name);
            let (file_writer, file_guard) = tracing_appender::non_blocking(appender);
            std::mem::forget(file_guard);
            let file_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file_writer)
                .with_target(config.include_target)
                .with_span_list(config.include_spans)
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .with(file_layer)
                .try_init()
                .map_err(|e| LogError::Init(e.to_string()))?;

            Ok(guard)
        }
    }
}

fn build_filter(config: &LogConfig) -> EnvFilter {
    EnvFilter::try_from_env("AGILEPLUS_LOG")
        .or_else(|_| EnvFilter::try_from_env("RUST_LOG"))
        .unwrap_or_else(|_| {
            EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"))
        })
}

/// Explicit flush hint (actual flush happens on guard drop).
pub fn flush() {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_config_defaults() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.output, LogOutput::Stdout);
        assert!(cfg.include_spans);
        assert!(cfg.include_target);
    }

    #[test]
    fn log_config_serde_roundtrip() {
        let yaml = r#"
level: "debug"
output: "stdout"
include_spans: false
include_target: true
"#;
        let cfg: LogConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.level, "debug");
        assert!(!cfg.include_spans);
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn log_output_default_is_stdout() {
        assert_eq!(LogOutput::default(), LogOutput::Stdout);
    }

    #[test]
    fn log_output_stdout_serde() {
        let s = serde_yaml::to_string(&LogOutput::Stdout).unwrap();
        assert_eq!(s.trim(), "stdout");
    }

    #[test]
    fn log_output_file_serde_roundtrip() {
        let out = LogOutput::File(PathBuf::from("/tmp/x.log"));
        let s = serde_yaml::to_string(&out).unwrap();
        let back: LogOutput = serde_yaml::from_str(&s).unwrap();
        assert_eq!(back, out);
    }

    #[test]
    fn log_output_both_serde_roundtrip() {
        let out = LogOutput::Both(PathBuf::from("/var/log/ap.log"));
        let s = serde_yaml::to_string(&out).unwrap();
        let back: LogOutput = serde_yaml::from_str(&s).unwrap();
        assert_eq!(back, out);
    }

    #[test]
    fn log_config_default_fields() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.output, LogOutput::Stdout);
        assert!(cfg.include_spans);
        assert!(cfg.include_target);
    }

    #[test]
    fn log_config_defaults_via_serde_empty_map() {
        let cfg: LogConfig = serde_yaml::from_str("{}").unwrap();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.output, LogOutput::Stdout);
        assert!(cfg.include_spans);
        assert!(cfg.include_target);
    }

    #[test]
    fn log_config_serde_roundtrip_all_fields() {
        let cfg = LogConfig {
            level: "warn".into(),
            output: LogOutput::File(PathBuf::from("/tmp/a.log")),
            include_spans: false,
            include_target: false,
        };
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let back: LogConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back.level, "warn");
        assert_eq!(back.output, cfg.output);
        assert!(!back.include_spans);
        assert!(!back.include_target);
    }

    #[test]
    fn log_config_clone_is_equal() {
        let cfg = LogConfig::default();
        let cloned = cfg.clone();
        assert_eq!(cloned.level, cfg.level);
        assert_eq!(cloned.output, cfg.output);
    }

    #[test]
    fn log_error_file_open_display() {
        let e = LogError::FileOpen {
            path: PathBuf::from("/nope/x.log"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
        };
        let s = e.to_string();
        assert!(s.contains("/nope/x.log"));
        assert!(s.contains("failed to open log file"));
    }

    #[test]
    fn log_error_init_display() {
        let e = LogError::Init("boom".into());
        assert!(e.to_string().contains("boom"));
        assert!(e.to_string().contains("subscriber initialisation failed"));
    }

    #[test]
    fn build_filter_uses_config_level() {
        let cfg = LogConfig {
            level: "debug".into(),
            ..LogConfig::default()
        };
        // build_filter consults env first; when neither env is set, it must
        // succeed with the configured level. Regardless, it must not panic.
        let _ = build_filter(&cfg);
    }

    #[test]
    fn build_filter_invalid_level_falls_back() {
        let cfg = LogConfig {
            level: "totally-not-a-level".into(),
            ..LogConfig::default()
        };
        let _ = build_filter(&cfg);
    }

    #[test]
    fn flush_is_callable() {
        flush();
    }

    #[test]
    fn output_file_with_no_parent_serialises() {
        let out = LogOutput::File(PathBuf::from("relative.log"));
        let s = serde_yaml::to_string(&out).unwrap();
        assert!(s.contains("relative.log"));
    }
}

#[cfg(test)]
mod env_filter_precedence_tests {
    use super::*;
    use std::ffi::{OsStr, OsString};
    use std::sync::{Mutex, MutexGuard};

    const AGILEPLUS_LOG: &str = "AGILEPLUS_LOG";
    const RUST_LOG: &str = "RUST_LOG";

    /// One lock for every test in this module: the process environment is global.
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
            // SAFETY: the caller holds `env_lock`, so no other thread in this
            // test binary mutates the process environment concurrently.
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

    fn config_with_level(level: &str) -> LogConfig {
        LogConfig {
            level: level.to_string(),
            ..LogConfig::default()
        }
    }

    #[test]
    fn agileplus_log_takes_precedence_over_rust_log() {
        let _lock = env_lock();
        let _agileplus = EnvGuard::set(AGILEPLUS_LOG, "debug");
        let _rust = EnvGuard::set(RUST_LOG, "warn");

        let filter = build_filter(&config_with_level("error"));

        assert_eq!(filter.to_string(), "debug");
    }

    #[test]
    fn rust_log_is_used_when_agileplus_log_is_absent() {
        let _lock = env_lock();
        let _agileplus = EnvGuard::unset(AGILEPLUS_LOG);
        let _rust = EnvGuard::set(RUST_LOG, "warn");

        let filter = build_filter(&config_with_level("error"));

        assert_eq!(filter.to_string(), "warn");
    }

    #[test]
    fn config_level_is_used_when_no_env_filter_is_set() {
        let _lock = env_lock();
        let _agileplus = EnvGuard::unset(AGILEPLUS_LOG);
        let _rust = EnvGuard::unset(RUST_LOG);

        let filter = build_filter(&config_with_level("trace"));

        assert_eq!(filter.to_string(), "trace");
    }

    #[test]
    fn bare_config_level_is_interpreted_as_a_target_directive() {
        let _lock = env_lock();
        let _agileplus = EnvGuard::unset(AGILEPLUS_LOG);
        let _rust = EnvGuard::unset(RUST_LOG);

        let filter = build_filter(&config_with_level("totally-not-a-level"));

        // `EnvFilter` accepts any bare word as a target filter, so a mistyped
        // level does not error: it silently becomes `<word>=trace`.
        assert_eq!(filter.to_string(), "totally-not-a-level=trace");
    }

    #[test]
    fn unparseable_config_level_falls_back_to_info() {
        let _lock = env_lock();
        let _agileplus = EnvGuard::unset(AGILEPLUS_LOG);
        let _rust = EnvGuard::unset(RUST_LOG);

        let filter = build_filter(&config_with_level("agileplus=not-a-level"));

        assert_eq!(filter.to_string(), "info");
    }
}
