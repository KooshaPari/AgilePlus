//! Configuration for the NATS event bus connection.

use agileplus_config::config_builder;

config_builder! {
    /// Configuration for connecting to a NATS server.
    #[derive(Clone, Debug)]
    pub struct NatsConfig {
        /// NATS server URL (e.g. `nats://localhost:4222`).
        (str)     pub url: String = "nats://localhost:4222".to_string(),
        /// Optional authentication token.
        (opt_str) pub auth_token: Option<String> = None,
        /// Subject prefix for all AgilePlus messages.
        (str)     pub subject_prefix: String = "agileplus".to_string(),
        /// Maximum payload size in bytes (NATS default is 1 MiB).
        (val)     pub max_payload: usize = 1_048_576,
    }
}

impl NatsConfig {
    /// Construct with explicit server URL; all other fields use defaults.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Self::default()
        }
    }

    /// Back-compat alias: set auth token (wraps [`with_auth_token`]).
    #[inline]
    pub fn with_auth(self, token: impl Into<String>) -> Self {
        self.with_auth_token(token)
    }

    /// Back-compat alias: set subject prefix (wraps [`with_subject_prefix`]).
    #[inline]
    pub fn with_prefix(self, prefix: impl Into<String>) -> Self {
        self.with_subject_prefix(prefix)
    }
}
