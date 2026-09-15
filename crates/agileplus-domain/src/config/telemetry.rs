use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub log_file: Option<PathBuf>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: None,
            log_level: default_log_level(),
            log_file: None,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_telemetry_config() {
        let c = TelemetryConfig::default();
        assert!(!c.enabled);
        assert!(c.otlp_endpoint.is_none());
        assert_eq!(c.log_level, "info");
        assert!(c.log_file.is_none());
    }

    #[test]
    fn serde_roundtrip() {
        let c = TelemetryConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: TelemetryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.log_level, "info");
        assert!(!back.enabled);
    }

    #[test]
    fn custom_telemetry() {
        let c = TelemetryConfig {
            enabled: true,
            otlp_endpoint: Some("http://localhost:4317".into()),
            log_level: "debug".into(),
            log_file: Some(PathBuf::from("/tmp/app.log")),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: TelemetryConfig = serde_json::from_str(&json).unwrap();
        assert!(back.enabled);
        assert_eq!(back.otlp_endpoint.unwrap(), "http://localhost:4317");
    }
}
