use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: default_api_port(),
            grpc_port: default_grpc_port(),
            cors_origins: Vec::new(),
        }
    }
}

fn default_api_port() -> u16 {
    3000
}

fn default_grpc_port() -> u16 {
    50051
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_api_config() {
        let c = ApiConfig::default();
        assert_eq!(c.port, 3000);
        assert_eq!(c.grpc_port, 50051);
        assert!(c.cors_origins.is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let c = ApiConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: ApiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.port, 3000);
        assert_eq!(back.grpc_port, 50051);
    }

    #[test]
    fn custom_cors() {
        let c = ApiConfig {
            port: 8080,
            grpc_port: 9090,
            cors_origins: vec!["http://localhost".into()],
        };
        assert_eq!(c.cors_origins.len(), 1);
    }
}
