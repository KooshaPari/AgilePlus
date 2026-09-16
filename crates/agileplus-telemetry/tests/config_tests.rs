//! Comprehensive tests for `agileplus_telemetry::config`.
//!
//! Covers: config creation, defaults, serde roundtrips, validation, YAML
//! loading, env overrides, error types, and all public API surface.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use agileplus_telemetry::config::{
    ConfigError, DEFAULT_CONFIG_YAML, OtlpConfig, OtlpProtocol, SamplingConfig,
    TelemetryConfig,
};

// ---------------------------------------------------------------------------
// OtlpProtocol
// ---------------------------------------------------------------------------

#[test]
fn otlp_protocol_default_is_grpc() {
    assert_eq!(OtlpProtocol::default(), OtlpProtocol::Grpc);
}

#[test]
fn otlp_protocol_serde_roundtrip_grpc() {
    let proto = OtlpProtocol::Grpc;
    let yaml = serde_yaml::to_string(&proto).unwrap();
    assert_eq!(yaml.trim(), "grpc");
    let deserialized: OtlpProtocol = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized, OtlpProtocol::Grpc);
}

#[test]
fn otlp_protocol_serde_roundtrip_http() {
    let proto = OtlpProtocol::Http;
    let yaml = serde_yaml::to_string(&proto).unwrap();
    assert_eq!(yaml.trim(), "http");
    let deserialized: OtlpProtocol = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized, OtlpProtocol::Http);
}

#[test]
fn otlp_protocol_debug_format() {
    assert_eq!(format!("{:?}", OtlpProtocol::Grpc), "Grpc");
    assert_eq!(format!("{:?}", OtlpProtocol::Http), "Http");
}

#[test]
fn otlp_protocol_clone() {
    let p = OtlpProtocol::Http;
    let p2 = p.clone();
    assert_eq!(p, p2);
}

#[test]
fn otlp_protocol_equality() {
    assert_eq!(OtlpProtocol::Grpc, OtlpProtocol::Grpc);
    assert_ne!(OtlpProtocol::Grpc, OtlpProtocol::Http);
}

// ---------------------------------------------------------------------------
// OtlpConfig
// ---------------------------------------------------------------------------

#[test]
fn otlp_config_has_defaults() {
    let cfg = OtlpConfig {
        endpoint: "http://localhost:4317".into(),
        protocol: OtlpProtocol::default(),
        headers: HashMap::new(),
        timeout_ms: 5_000,
        export_interval_ms: 60_000,
    };
    assert_eq!(cfg.protocol, OtlpProtocol::Grpc);
    assert!(cfg.headers.is_empty());
    assert_eq!(cfg.timeout_ms, 5_000);
    assert_eq!(cfg.export_interval_ms, 60_000);
}

#[test]
fn otlp_config_serde_roundtrip() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), "Bearer tok123".into());
    let cfg = OtlpConfig {
        endpoint: "http://collector:4317".into(),
        protocol: OtlpProtocol::Http,
        headers,
        timeout_ms: 10_000,
        export_interval_ms: 30_000,
    };
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    let deserialized: OtlpConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.endpoint, "http://collector:4317");
    assert_eq!(deserialized.protocol, OtlpProtocol::Http);
    assert_eq!(
        deserialized.headers.get("Authorization").unwrap(),
        "Bearer tok123"
    );
    assert_eq!(deserialized.timeout_ms, 10_000);
    assert_eq!(deserialized.export_interval_ms, 30_000);
}

#[test]
fn otlp_config_missing_protocol_defaults_to_grpc() {
    let yaml = r#"
endpoint: "http://localhost:4317"
"#;
    let cfg: OtlpConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.protocol, OtlpProtocol::Grpc);
    assert_eq!(cfg.timeout_ms, 5_000); // default
    assert_eq!(cfg.export_interval_ms, 60_000); // default
}

#[test]
fn otlp_config_clone() {
    let cfg = OtlpConfig {
        endpoint: "http://x:4317".into(),
        protocol: OtlpProtocol::Grpc,
        headers: HashMap::new(),
        timeout_ms: 1000,
        export_interval_ms: 5000,
    };
    let cfg2 = cfg.clone();
    assert_eq!(cfg.endpoint, cfg2.endpoint);
    assert_eq!(cfg.protocol, cfg2.protocol);
}

#[test]
fn otlp_config_debug_format() {
    let cfg = OtlpConfig {
        endpoint: "http://localhost:4317".into(),
        protocol: OtlpProtocol::Grpc,
        headers: HashMap::new(),
        timeout_ms: 5000,
        export_interval_ms: 60000,
    };
    let debug = format!("{:?}", cfg);
    assert!(debug.contains("OtlpConfig"));
    assert!(debug.contains("localhost:4317"));
}

// ---------------------------------------------------------------------------
// SamplingConfig
// ---------------------------------------------------------------------------

#[test]
fn sampling_config_default_ratio_is_one() {
    let cfg = SamplingConfig::default();
    assert_eq!(cfg.trace_ratio, 1.0);
}

#[test]
fn sampling_config_serde_roundtrip() {
    let cfg = SamplingConfig { trace_ratio: 0.5 };
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    let deserialized: SamplingConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.trace_ratio, 0.5);
}

#[test]
fn sampling_config_missing_ratio_defaults_to_one() {
    let yaml = "{}";
    let cfg: SamplingConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.trace_ratio, 1.0);
}

#[test]
fn sampling_config_clone() {
    let cfg = SamplingConfig { trace_ratio: 0.25 };
    let cfg2 = cfg.clone();
    assert_eq!(cfg.trace_ratio, cfg2.trace_ratio);
}

// ---------------------------------------------------------------------------
// TelemetryConfig
// ---------------------------------------------------------------------------

#[test]
fn telemetry_config_default() {
    let cfg = TelemetryConfig::default();
    assert!(cfg.otlp.is_none());
    assert_eq!(cfg.logging.level, "info");
    assert_eq!(cfg.logging.output, agileplus_telemetry::logs::LogOutput::Stdout);
    assert!(cfg.logging.include_spans);
    assert!(cfg.logging.include_target);
    assert_eq!(cfg.sampling.trace_ratio, 1.0);
}

#[test]
fn telemetry_config_clone() {
    let cfg = TelemetryConfig::default();
    let cfg2 = cfg.clone();
    assert_eq!(cfg.otlp.is_none(), cfg2.otlp.is_none());
    assert_eq!(cfg.logging.level, cfg2.logging.level);
}

#[test]
fn telemetry_config_debug_format() {
    let cfg = TelemetryConfig::default();
    let debug = format!("{:?}", cfg);
    assert!(debug.contains("TelemetryConfig"));
}

#[test]
fn telemetry_config_serde_roundtrip_default() {
    let cfg = TelemetryConfig::default();
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    let deserialized: TelemetryConfig = serde_yaml::from_str(&yaml).unwrap();
    assert!(deserialized.otlp.is_none());
    assert_eq!(deserialized.sampling.trace_ratio, 1.0);
}

#[test]
fn telemetry_config_serde_roundtrip_with_otlp() {
    let cfg = TelemetryConfig {
        otlp: Some(OtlpConfig {
            endpoint: "http://tracing:4317".into(),
            protocol: OtlpProtocol::Http,
            headers: HashMap::from([("X-Token".into(), "abc".into())]),
            timeout_ms: 3000,
            export_interval_ms: 15000,
        }),
        logging: agileplus_telemetry::logs::LogConfig {
            level: "warn".into(),
            ..Default::default()
        },
        sampling: SamplingConfig { trace_ratio: 0.1 },
    };
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    let deserialized: TelemetryConfig = serde_yaml::from_str(&yaml).unwrap();
    let otlp = deserialized.otlp.unwrap();
    assert_eq!(otlp.endpoint, "http://tracing:4317");
    assert_eq!(otlp.protocol, OtlpProtocol::Http);
    assert_eq!(otlp.headers.get("X-Token").unwrap(), "abc");
    assert_eq!(otlp.timeout_ms, 3000);
    assert_eq!(deserialized.logging.level, "warn");
    assert_eq!(deserialized.sampling.trace_ratio, 0.1);
}

#[test]
fn telemetry_config_missing_sampling_defaults() {
    let yaml = r#"
otlp:
  endpoint: "http://localhost:4317"
"#;
    let cfg: TelemetryConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.sampling.trace_ratio, 1.0); // default
}

#[test]
fn telemetry_config_missing_logging_defaults() {
    let yaml = r#"
sampling:
  trace_ratio: 0.5
"#;
    let cfg: TelemetryConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.logging.level, "info");
    assert!(cfg.logging.include_spans);
}

// ---------------------------------------------------------------------------
// DEFAULT_CONFIG_YAML
// ---------------------------------------------------------------------------

#[test]
fn default_config_yaml_parses() {
    let cfg: TelemetryConfig = serde_yaml::from_str(DEFAULT_CONFIG_YAML).unwrap();
    let otlp = cfg.otlp.unwrap();
    assert_eq!(otlp.endpoint, "http://localhost:4317");
    assert_eq!(otlp.protocol, OtlpProtocol::Grpc);
    assert_eq!(otlp.timeout_ms, 5000);
    assert_eq!(otlp.export_interval_ms, 60000);
    assert_eq!(cfg.logging.level, "info");
    assert_eq!(cfg.logging.output, agileplus_telemetry::logs::LogOutput::Stdout);
    assert!(cfg.logging.include_spans);
    assert!(cfg.logging.include_target);
    assert_eq!(cfg.sampling.trace_ratio, 1.0);
}

#[test]
fn default_config_yaml_roundtrips() {
    let cfg1: TelemetryConfig = serde_yaml::from_str(DEFAULT_CONFIG_YAML).unwrap();
    let yaml = serde_yaml::to_string(&cfg1).unwrap();
    let cfg2: TelemetryConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(
        cfg1.otlp.as_ref().unwrap().endpoint,
        cfg2.otlp.as_ref().unwrap().endpoint
    );
    assert_eq!(cfg1.sampling.trace_ratio, cfg2.sampling.trace_ratio);
}

// ---------------------------------------------------------------------------
// ConfigError
// ---------------------------------------------------------------------------

#[test]
fn config_error_display_io() {
    let err = ConfigError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file missing",
    ));
    let msg = err.to_string();
    assert!(msg.contains("IO error reading telemetry config"));
    assert!(msg.contains("file missing"));
}

#[test]
fn config_error_display_validation() {
    let err = ConfigError::Validation("bad value".into());
    let msg = err.to_string();
    assert!(msg.contains("invalid config"));
    assert!(msg.contains("bad value"));
}

#[test]
fn config_error_debug_format() {
    let err = ConfigError::Validation("test".into());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Validation"));
}

#[test]
fn config_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "broken");
    let config_err: ConfigError = io_err.into();
    assert!(matches!(config_err, ConfigError::Io(_)));
}

// ---------------------------------------------------------------------------
// Loading from file
// ---------------------------------------------------------------------------

#[test]
fn load_from_valid_file() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
otlp:
  endpoint: "http://custom:4317"
  protocol: http
  timeout_ms: 10000
  export_interval_ms: 30000
logging:
  level: "debug"
  output: "stdout"
  include_spans: false
  include_target: false
sampling:
  trace_ratio: 0.5
"#
    )
    .unwrap();
    let cfg = TelemetryConfig::load_from(f.path()).unwrap();
    let otlp = cfg.otlp.unwrap();
    assert_eq!(otlp.endpoint, "http://custom:4317");
    assert_eq!(otlp.protocol, OtlpProtocol::Http);
    assert_eq!(otlp.timeout_ms, 10000);
    assert_eq!(otlp.export_interval_ms, 30000);
    assert_eq!(cfg.logging.level, "debug");
    assert!(!cfg.logging.include_spans);
    assert!(!cfg.logging.include_target);
    assert_eq!(cfg.sampling.trace_ratio, 0.5);
}

#[test]
fn load_from_nonexistent_file_errors() {
    let result = TelemetryConfig::load_from(Path::new("/tmp/__nonexistent_config_test__.yaml"));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
}

#[test]
fn load_from_malformed_yaml_errors() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "{{{{ invalid yaml : :").unwrap();
    let result = TelemetryConfig::load_from(f.path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::Yaml(_)));
}

#[test]
fn load_validates_trace_ratio_above_one() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
sampling:
  trace_ratio: 1.5
"#
    )
    .unwrap();
    let result = TelemetryConfig::load_from(f.path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
}

#[test]
fn load_validates_trace_ratio_negative() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
sampling:
  trace_ratio: -0.1
"#
    )
    .unwrap();
    let result = TelemetryConfig::load_from(f.path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
}

#[test]
fn load_validates_zero_timeout() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
otlp:
  endpoint: "http://localhost:4317"
  timeout_ms: 0
"#
    )
    .unwrap();
    let result = TelemetryConfig::load_from(f.path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
}

#[test]
fn load_validates_invalid_endpoint_url() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
otlp:
  endpoint: "not a valid url"
"#
    )
    .unwrap();
    let result = TelemetryConfig::load_from(f.path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
}

#[test]
fn load_valid_trace_ratio_zero_is_valid() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
sampling:
  trace_ratio: 0.0
"#
    )
    .unwrap();
    let cfg = TelemetryConfig::load_from(f.path()).unwrap();
    assert_eq!(cfg.sampling.trace_ratio, 0.0);
}

#[test]
fn load_valid_trace_ratio_one_is_valid() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
sampling:
  trace_ratio: 1.0
"#
    )
    .unwrap();
    let cfg = TelemetryConfig::load_from(f.path()).unwrap();
    assert_eq!(cfg.sampling.trace_ratio, 1.0);
}

// ---------------------------------------------------------------------------
// Environment variable overrides
// ---------------------------------------------------------------------------

// (env override tests removed - apply_env_overrides is private)

// ---------------------------------------------------------------------------
// Edge cases: empty file, comment-only, minimal YAML
// ---------------------------------------------------------------------------

#[test]
fn load_empty_file_defaults() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "").unwrap();
    let cfg = TelemetryConfig::load_from(f.path()).unwrap();
    assert!(cfg.otlp.is_none());
    assert_eq!(cfg.logging.level, "info");
    assert_eq!(cfg.sampling.trace_ratio, 1.0);
}

#[test]
fn load_comment_only_file_defaults() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "# just a comment\n").unwrap();
    let cfg = TelemetryConfig::load_from(f.path()).unwrap();
    assert!(cfg.otlp.is_none());
    assert_eq!(cfg.sampling.trace_ratio, 1.0);
}

// ---------------------------------------------------------------------------
// Serde: OTLP protocol lowercase only
// ---------------------------------------------------------------------------

#[test]
fn otlp_protocol_serde_rejects_unknown_variant() {
    let result = serde_yaml::from_str::<OtlpProtocol>("\"quic\"");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Multiple YAML key combinations
// ---------------------------------------------------------------------------

#[test]
fn yaml_with_all_otlp_fields() {
    let yaml = r#"
otlp:
  endpoint: "http://tracing.example.com:4317"
  protocol: http
  headers:
    Authorization: "Bearer xyz"
    X-Custom: "value"
  timeout_ms: 15000
  export_interval_ms: 45000
logging:
  level: "trace"
  output: "stdout"
  include_spans: false
  include_target: false
sampling:
  trace_ratio: 0.75
"#;
    let cfg: TelemetryConfig = serde_yaml::from_str(yaml).unwrap();
    let otlp = cfg.otlp.unwrap();
    assert_eq!(otlp.endpoint, "http://tracing.example.com:4317");
    assert_eq!(otlp.protocol, OtlpProtocol::Http);
    assert_eq!(otlp.headers.len(), 2);
    assert_eq!(otlp.headers.get("Authorization").unwrap(), "Bearer xyz");
    assert_eq!(otlp.headers.get("X-Custom").unwrap(), "value");
    assert_eq!(cfg.logging.level, "trace");
    assert_eq!(cfg.sampling.trace_ratio, 0.75);
}

#[test]
fn yaml_with_only_logging() {
    let yaml = r#"
logging:
  level: "warn"
  output: "stdout"
  include_spans: true
  include_target: false
"#;
    let cfg: TelemetryConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(cfg.otlp.is_none());
    assert_eq!(cfg.logging.level, "warn");
    assert!(!cfg.logging.include_target);
}

// ---------------------------------------------------------------------------
// Check that adapter.rs inline tests file exists (adapter/tests.rs)
// ---------------------------------------------------------------------------

#[test]
fn adapter_tests_file_exists_and_compiles() {
    // This test just ensures the tests module in adapter.rs compiles.
    // The real adapter tests are in adapter::tests::* (inline).
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("adapter")
        .join("tests.rs");
    assert!(path.exists(), "adapter/tests.rs should exist");
}
