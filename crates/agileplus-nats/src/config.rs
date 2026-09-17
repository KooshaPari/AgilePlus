//! Configuration for the NATS event bus connection.

/// Configuration for connecting to a NATS server.
#[derive(Clone, Debug)]
pub struct NatsConfig {
    /// NATS server URL (e.g. `nats://localhost:4222`).
    pub url: String,
    /// Optional authentication token.
    pub auth_token: Option<String>,
    /// Subject prefix for all AgilePlus messages.
    pub subject_prefix: String,
    /// Maximum payload size in bytes (NATS default is 1 MiB).
    pub max_payload: usize,
}

impl NatsConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            auth_token: None,
            subject_prefix: "agileplus".to_string(),
            max_payload: 1_048_576,
        }
    }

    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.subject_prefix = prefix.into();
        self
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self::new("nats://localhost:4222")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_defaults() {
        let cfg = NatsConfig::new("nats://broker:4222");
        assert_eq!(cfg.url, "nats://broker:4222");
        assert_eq!(cfg.auth_token, None);
        assert_eq!(cfg.subject_prefix, "agileplus");
        assert_eq!(cfg.max_payload, 1_048_576);
    }

    #[test]
    fn default_points_to_localhost() {
        let cfg = NatsConfig::default();
        assert_eq!(cfg.url, "nats://localhost:4222");
        assert_eq!(cfg.auth_token, None);
        assert_eq!(cfg.subject_prefix, "agileplus");
        assert_eq!(cfg.max_payload, 1_048_576);
    }

    #[test]
    fn with_auth_sets_token() {
        let cfg =
            NatsConfig::new("nats://broker:4222").with_auth("secret-token-123");
        assert_eq!(cfg.auth_token.as_deref(), Some("secret-token-123"));
    }

    #[test]
    fn with_prefix_overrides_default() {
        let cfg =
            NatsConfig::new("nats://broker:4222").with_prefix("myorg");
        assert_eq!(cfg.subject_prefix, "myorg");
    }

    #[test]
    fn builder_chain_auth_and_prefix() {
        let cfg = NatsConfig::new("nats://broker:4222")
            .with_auth("tok")
            .with_prefix("pfx");
        assert_eq!(cfg.auth_token.as_deref(), Some("tok"));
        assert_eq!(cfg.subject_prefix, "pfx");
    }

    #[test]
    fn config_is_clone() {
        let cfg =
            NatsConfig::new("nats://broker:4222").with_auth("x");
        let cfg2 = cfg.clone();
        assert_eq!(cfg.url, cfg2.url);
        assert_eq!(cfg.auth_token, cfg2.auth_token);
        assert_eq!(cfg.subject_prefix, cfg2.subject_prefix);
        assert_eq!(cfg.max_payload, cfg2.max_payload);
    }

    #[test]
    fn new_accepts_string_ref() {
        let url = String::from("nats://custom:9999");
        let cfg = NatsConfig::new(&url);
        assert_eq!(cfg.url, "nats://custom:9999");
    }

    #[test]
    fn with_auth_accepts_string_ref() {
        let token = String::from("bearer-xyz");
        let cfg =
            NatsConfig::new("nats://broker:4222").with_auth(&token);
        assert_eq!(cfg.auth_token.as_deref(), Some("bearer-xyz"));
    }

    #[test]
    fn with_prefix_accepts_string_ref() {
        let prefix = String::from("custom_prefix");
        let cfg =
            NatsConfig::new("nats://broker:4222").with_prefix(&prefix);
        assert_eq!(cfg.subject_prefix, "custom_prefix");
    }

    #[test]
    fn default_max_payload_is_1_mib() {
        let cfg = NatsConfig::default();
        assert_eq!(cfg.max_payload, 1024 * 1024);
    }

    #[test]
    fn debug_impl_works() {
        let cfg = NatsConfig::new("nats://broker:4222");
        let debug_str = format!("{cfg:?}");
        assert!(debug_str.contains("NatsConfig"));
        assert!(debug_str.contains("nats://broker:4222"));
    }

    #[test]
    fn with_auth_overwrites_token() {
        let cfg = NatsConfig::new("nats://broker:4222")
            .with_auth("first")
            .with_auth("second");
        assert_eq!(cfg.auth_token.as_deref(), Some("second"));
    }

    #[test]
    fn with_prefix_overwrites_prefix() {
        let cfg = NatsConfig::default()
            .with_prefix("p1")
            .with_prefix("p2");
        assert_eq!(cfg.subject_prefix, "p2");
    }

    #[test]
    fn new_accepts_arbitrary_url() {
        let cfg = NatsConfig::new("nats://user:pass@host:4222");
        assert_eq!(cfg.url, "nats://user:pass@host:4222");
        assert!(cfg.auth_token.is_none());
    }

    #[test]
    fn default_clone_preserves_all_fields() {
        let original = NatsConfig::default();
        let clone = original.clone();
        assert_eq!(clone.url, original.url);
        assert_eq!(clone.subject_prefix, original.subject_prefix);
        assert_eq!(clone.max_payload, original.max_payload);
    }
}
