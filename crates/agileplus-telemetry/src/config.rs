//! OTLP and telemetry configuration loader.
//!
//! Reads `~/.agileplus/otel-config.yaml`. Missing file returns defaults (stdout
//! only, no OTLP export).  Environment variables override YAML values.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::logs::LogConfig;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error reading telemetry config: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error in telemetry config: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid config: {0}")]
    Validation(String),
}

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

/// OTLP export protocol selection.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OtlpProtocol {
    #[default]
    Grpc,
    Http,
}

/// OTLP exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// gRPC/HTTP endpoint, e.g. `"http://localhost:4317"`.
    pub endpoint: String,
    /// Wire protocol — `grpc` (default) or `http`.
    #[serde(default)]
    pub protocol: OtlpProtocol,
    /// Additional headers (auth tokens, etc.).
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Connection timeout in milliseconds (default: 5 000).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Metrics export interval in milliseconds (default: 60 000).
    #[serde(default = "default_export_interval_ms")]
    pub export_interval_ms: u64,
}

fn default_timeout_ms() -> u64 {
    5_000
}
fn default_export_interval_ms() -> u64 {
    60_000
}

/// Trace sampling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Fraction of traces to sample — 0.0 (none) to 1.0 (all).
    #[serde(default = "default_trace_ratio")]
    pub trace_ratio: f64,
}

fn default_trace_ratio() -> f64 {
    1.0
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self { trace_ratio: 1.0 }
    }
}

/// Top-level telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelemetryConfig {
    /// OTLP export — absent means no export (console only).
    pub otlp: Option<OtlpConfig>,
    /// Structured logging configuration.
    #[serde(default)]
    pub logging: LogConfig,
    /// Sampling configuration.
    #[serde(default)]
    pub sampling: SamplingConfig,
}

// ---------------------------------------------------------------------------
// Default YAML template
// ---------------------------------------------------------------------------

/// Default YAML config content, suitable for `agileplus init` to write.
pub const DEFAULT_CONFIG_YAML: &str = r#"# AgilePlus OpenTelemetry Configuration
# Remove or comment out the `otlp` section to disable OTLP export.
otlp:
  endpoint: "http://localhost:4317"
  protocol: grpc
  headers: {}
  timeout_ms: 5000
  export_interval_ms: 60000
logging:
  level: "info"
  output: "stdout"
  include_spans: true
  include_target: true
sampling:
  trace_ratio: 1.0
"#;

// ---------------------------------------------------------------------------
// Loaders
// ---------------------------------------------------------------------------

impl TelemetryConfig {
    /// Load from the canonical path `~/.agileplus/otel-config.yaml`.
    ///
    /// Returns `TelemetryConfig::default()` when the file does not exist.
    pub fn load() -> Result<Self, ConfigError> {
        let path = default_config_path();
        if !path.exists() {
            return Ok(Self::default_with_env_overrides());
        }
        Self::load_from(&path)
    }

    /// Load from an explicit path (useful for testing).
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let mut cfg: TelemetryConfig = serde_yaml::from_str(&content)?;
        cfg.apply_env_overrides();
        cfg.validate()?;
        Ok(cfg)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn default_with_env_overrides() -> Self {
        let mut cfg = TelemetryConfig::default();
        cfg.apply_env_overrides();
        cfg
    }

    /// Apply environment variable overrides on top of YAML values.
    fn apply_env_overrides(&mut self) {
        if let Ok(level) = std::env::var("AGILEPLUS_LOG_LEVEL") {
            self.logging.level = level;
        }
        if let Ok(endpoint) = std::env::var("AGILEPLUS_OTLP_ENDPOINT") {
            match &mut self.otlp {
                Some(o) => o.endpoint = endpoint,
                None => {
                    self.otlp = Some(OtlpConfig {
                        endpoint,
                        protocol: OtlpProtocol::Grpc,
                        headers: HashMap::new(),
                        timeout_ms: default_timeout_ms(),
                        export_interval_ms: default_export_interval_ms(),
                    });
                }
            }
        }
    }

    /// Validate field invariants.
    fn validate(&self) -> Result<(), ConfigError> {
        let r = self.sampling.trace_ratio;
        if !(0.0..=1.0).contains(&r) {
            return Err(ConfigError::Validation(format!(
                "sampling.trace_ratio must be 0.0–1.0, got {r}"
            )));
        }
        if let Some(otlp) = &self.otlp {
            if otlp.timeout_ms == 0 {
                return Err(ConfigError::Validation(
                    "otlp.timeout_ms must be > 0".into(),
                ));
            }
            // Basic URL check.
            let _ = otlp.endpoint.parse::<url::Url>().map_err(|_| {
                ConfigError::Validation(format!(
                    "otlp.endpoint '{}' is not a valid URL",
                    otlp.endpoint
                ))
            })?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agileplus")
        .join("otel-config.yaml")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_file_returns_defaults() {
        let cfg = TelemetryConfig::load_from(Path::new("/nonexistent/path/otel.yaml"));
        // Should error with Io not panic.
        assert!(cfg.is_err());
    }

    #[test]
    fn valid_yaml_parses() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{}", DEFAULT_CONFIG_YAML).unwrap();
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert!(cfg.otlp.is_some());
        assert_eq!(cfg.sampling.trace_ratio, 1.0);
    }

    #[test]
    fn invalid_trace_ratio_errors() {
        let yaml = r#"
sampling:
  trace_ratio: 2.5
"#;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{yaml}").unwrap();
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn malformed_yaml_errors() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, ": !! bad yaml {{{{").unwrap();
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Yaml(_)));
    }

    #[test]
    fn missing_otlp_defaults_to_none() {
        let yaml = r#"
logging:
  level: "debug"
"#;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{yaml}").unwrap();
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert!(cfg.otlp.is_none());
    }
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use std::io::Write;

    fn write_yaml(yaml: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{yaml}").unwrap();
        f.flush().unwrap();
        f
    }

    // ── OtlpProtocol serde ──────────────────────────────────────────────────

    #[test]
    fn otlp_protocol_default_is_grpc() {
        assert_eq!(OtlpProtocol::default(), OtlpProtocol::Grpc);
    }

    #[test]
    fn otlp_protocol_serde_roundtrip_grpc() {
        let s = serde_yaml::to_string(&OtlpProtocol::Grpc).unwrap();
        assert_eq!(s.trim(), "grpc");
        let p: OtlpProtocol = serde_yaml::from_str("grpc").unwrap();
        assert_eq!(p, OtlpProtocol::Grpc);
    }

    #[test]
    fn otlp_protocol_serde_roundtrip_http() {
        let s = serde_yaml::to_string(&OtlpProtocol::Http).unwrap();
        assert_eq!(s.trim(), "http");
        let p: OtlpProtocol = serde_yaml::from_str("http").unwrap();
        assert_eq!(p, OtlpProtocol::Http);
    }

    #[test]
    fn otlp_config_defaults_applied() {
        let yaml = r#"
endpoint: "http://localhost:4317"
"#;
        let otlp: OtlpConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(otlp.protocol, OtlpProtocol::Grpc);
        assert_eq!(otlp.timeout_ms, 5_000);
        assert_eq!(otlp.export_interval_ms, 60_000);
        assert!(otlp.headers.is_empty());
    }

    #[test]
    fn otlp_config_headers_parse() {
        let yaml = r#"
endpoint: "http://localhost:4317"
headers:
  authorization: "Bearer xyz"
  x-tenant: "acme"
"#;
        let otlp: OtlpConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(otlp.headers.get("authorization").unwrap(), "Bearer xyz");
        assert_eq!(otlp.headers.get("x-tenant").unwrap(), "acme");
        assert_eq!(otlp.headers.len(), 2);
    }

    #[test]
    fn otlp_config_timeout_override() {
        let yaml = r#"
endpoint: "http://localhost:4317"
timeout_ms: 123
export_interval_ms: 456
"#;
        let otlp: OtlpConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(otlp.timeout_ms, 123);
        assert_eq!(otlp.export_interval_ms, 456);
    }

    // ── SamplingConfig ──────────────────────────────────────────────────────

    #[test]
    fn sampling_default_ratio_is_one() {
        assert_eq!(SamplingConfig::default().trace_ratio, 1.0);
    }

    #[test]
    fn sampling_ratio_zero_valid() {
        let f = write_yaml("sampling:\n  trace_ratio: 0.0\n");
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert_eq!(cfg.sampling.trace_ratio, 0.0);
    }

    #[test]
    fn sampling_ratio_one_valid() {
        let f = write_yaml("sampling:\n  trace_ratio: 1.0\n");
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert_eq!(cfg.sampling.trace_ratio, 1.0);
    }

    #[test]
    fn sampling_ratio_slightly_above_one_invalid() {
        let f = write_yaml("sampling:\n  trace_ratio: 1.0001\n");
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn sampling_ratio_negative_invalid() {
        let f = write_yaml("sampling:\n  trace_ratio: -0.1\n");
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn sampling_ratio_midpoint_valid() {
        let f = write_yaml("sampling:\n  trace_ratio: 0.5\n");
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert!((cfg.sampling.trace_ratio - 0.5).abs() < f64::EPSILON);
    }

    // ── OTLP validation ─────────────────────────────────────────────────────

    #[test]
    fn otlp_invalid_url_rejected() {
        let f = write_yaml("otlp:\n  endpoint: \"not a url\"\n");
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn otlp_empty_endpoint_rejected() {
        let f = write_yaml("otlp:\n  endpoint: \"\"\n");
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn otlp_zero_timeout_rejected() {
        let f = write_yaml("otlp:\n  endpoint: \"http://localhost:4317\"\n  timeout_ms: 0\n");
        let err = TelemetryConfig::load_from(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn otlp_https_endpoint_accepted() {
        let f = write_yaml("otlp:\n  endpoint: \"https://otlp.example.com:4317\"\n");
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert!(cfg.otlp.is_some());
    }

    #[test]
    fn default_config_yaml_roundtrips_and_validates() {
        let f = write_yaml(DEFAULT_CONFIG_YAML);
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert!(cfg.otlp.is_some());
        let otlp = cfg.otlp.unwrap();
        assert_eq!(otlp.endpoint, "http://localhost:4317");
        assert_eq!(otlp.protocol, OtlpProtocol::Grpc);
        assert!(otlp.headers.is_empty());
        assert_eq!(otlp.timeout_ms, 5_000);
        assert_eq!(otlp.export_interval_ms, 60_000);
        assert_eq!(cfg.sampling.trace_ratio, 1.0);
    }

    #[test]
    fn default_config_yaml_includes_logging_section() {
        let f = write_yaml(DEFAULT_CONFIG_YAML);
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert_eq!(cfg.logging.level, "info");
        assert!(cfg.logging.include_spans);
        assert!(cfg.logging.include_target);
    }

    #[test]
    fn empty_yaml_uses_defaults() {
        let f = write_yaml("{}\n");
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert!(cfg.otlp.is_none());
        assert_eq!(cfg.sampling.trace_ratio, 1.0);
        assert_eq!(cfg.logging.level, "info");
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let f = write_yaml("some_future_field: 42\nlogging:\n  level: debug\n");
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert_eq!(cfg.logging.level, "debug");
    }

    #[test]
    fn config_serde_roundtrip() {
        let cfg = TelemetryConfig {
            otlp: Some(OtlpConfig {
                endpoint: "http://localhost:4317".into(),
                protocol: OtlpProtocol::Http,
                headers: HashMap::new(),
                timeout_ms: 111,
                export_interval_ms: 222,
            }),
            logging: LogConfig::default(),
            sampling: SamplingConfig { trace_ratio: 0.25 },
        };
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let back: TelemetryConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back.sampling.trace_ratio, 0.25);
        assert_eq!(back.otlp.as_ref().unwrap().timeout_ms, 111);
        assert_eq!(back.otlp.unwrap().protocol, OtlpProtocol::Http);
    }

    #[test]
    fn telemetry_config_default_has_no_otlp() {
        let cfg = TelemetryConfig::default();
        assert!(cfg.otlp.is_none());
        assert_eq!(cfg.sampling.trace_ratio, 1.0);
    }

    // ── ConfigError display ─────────────────────────────────────────────────

    #[test]
    fn config_error_validation_display() {
        let e = ConfigError::Validation("bad".into());
        assert!(e.to_string().contains("bad"));
        assert!(e.to_string().contains("invalid config"));
    }

    #[test]
    fn config_error_io_from_missing_file() {
        let e = TelemetryConfig::load_from(Path::new("/definitely/not/here/otel.yaml")).unwrap_err();
        assert!(matches!(e, ConfigError::Io(_)));
        assert!(e.to_string().contains("IO error"));
    }

    #[test]
    fn config_error_yaml_display() {
        let e = ConfigError::Yaml(serde_yaml::from_str::<TelemetryConfig>(": bad {").unwrap_err());
        assert!(e.to_string().contains("YAML parse error"));
    }

    #[test]
    fn default_config_path_ends_with_expected_suffix() {
        // default_config_path is private but reachable from this module.
        let p = default_config_path();
        assert!(p.ends_with(".agileplus/otel-config.yaml"));
    }

    #[test]
    fn load_missing_default_path_returns_ok_or_err_without_panic() {
        // We can't control the home dir; just ensure it does not panic and
        // returns a Result.
        let _ = TelemetryConfig::load();
    }

    #[test]
    fn logs_section_parses_with_file_output() {
        let yaml = r#"
logging:
  level: "trace"
  output: !file /tmp/agileplus-test.log
  include_spans: false
  include_target: false
"#;
        let f = write_yaml(yaml);
        let cfg = TelemetryConfig::load_from(f.path()).unwrap();
        assert_eq!(cfg.logging.level, "trace");
        assert!(!cfg.logging.include_spans);
        assert!(!cfg.logging.include_target);
    }

    #[test]
    fn otlp_headers_serde_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("a".to_string(), "1".to_string());
        let otlp = OtlpConfig {
            endpoint: "http://localhost:4317".into(),
            protocol: OtlpProtocol::Grpc,
            headers,
            timeout_ms: 5,
            export_interval_ms: 6,
        };
        let yaml = serde_yaml::to_string(&otlp).unwrap();
        let back: OtlpConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back.headers.get("a").unwrap(), "1");
    }

    #[test]
    fn timeout_defaults_helpers() {
        assert_eq!(default_timeout_ms(), 5_000);
        assert_eq!(default_export_interval_ms(), 60_000);
        assert_eq!(default_trace_ratio(), 1.0);
    }

    #[test]
    fn otlp_protocol_lowercase_serde_rejects_uppercase() {
        let r: Result<OtlpProtocol, _> = serde_yaml::from_str("GRPC");
        assert!(r.is_err());
    }
}
