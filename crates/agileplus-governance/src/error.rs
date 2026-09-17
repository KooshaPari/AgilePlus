//! Error types for the AgilePlus governance system

use thiserror::Error;

/// Result type alias for governance operations
pub type Result<T> = std::result::Result<T, GovernanceError>;

/// Errors that can occur during governance operations
#[derive(Error, Debug)]
pub enum GovernanceError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Network error connecting to remote governance
    #[error("Network error: {0}")]
    Network(String),

    /// Policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid channel transition
    #[error("Invalid channel transition from {from} to {to}")]
    InvalidChannelTransition { from: String, to: String },

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Operation not allowed
    #[error("Operation not allowed: {0}")]
    NotAllowed(String),

    /// Sync error
    #[error("Sync error: {0}")]
    Sync(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Rubric catalog parse or validation error
    #[error("Rubric error: {0}")]
    Rubric(String),
}

impl GovernanceError {
    /// Check if this is a policy-related error
    pub fn is_policy_error(&self) -> bool {
        matches!(
            self,
            GovernanceError::PolicyViolation(_) | GovernanceError::NotAllowed(_)
        )
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limit_error(&self) -> bool {
        matches!(self, GovernanceError::RateLimitExceeded(_))
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            GovernanceError::Config(_) => 400,
            GovernanceError::Database(_) => 500,
            GovernanceError::Network(_) => 503,
            GovernanceError::PolicyViolation(_) => 403,
            GovernanceError::RateLimitExceeded(_) => 429,
            GovernanceError::Auth(_) => 401,
            GovernanceError::InvalidChannelTransition { .. } => 400,
            GovernanceError::NotFound(_) => 404,
            GovernanceError::NotAllowed(_) => 403,
            GovernanceError::Sync(_) => 500,
            GovernanceError::Internal(_) => 500,
            GovernanceError::Rubric(_) => 422,
        }
    }
}

impl From<rusqlite::Error> for GovernanceError {
    fn from(err: rusqlite::Error) -> Self {
        GovernanceError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for GovernanceError {
    fn from(err: serde_json::Error) -> Self {
        GovernanceError::Config(err.to_string())
    }
}

impl From<config::ConfigError> for GovernanceError {
    fn from(err: config::ConfigError) -> Self {
        GovernanceError::Config(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<std::sync::Arc<dyn std::any::Any + Send + Sync>>>
    for GovernanceError
{
    fn from(
        err: tokio::sync::mpsc::error::SendError<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Self {
        GovernanceError::Internal(err.to_string())
    }
}

impl From<std::io::Error> for GovernanceError {
    fn from(err: std::io::Error) -> Self {
        GovernanceError::Internal(err.to_string())
    }
}

impl From<toml::de::Error> for GovernanceError {
    fn from(err: toml::de::Error) -> Self {
        GovernanceError::Config(err.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let cases = vec![
            (GovernanceError::Config("bad config".into()), "Configuration error: bad config"),
            (GovernanceError::Database("db fail".into()), "Database error: db fail"),
            (GovernanceError::Network("timeout".into()), "Network error: timeout"),
            (GovernanceError::PolicyViolation("denied".into()), "Policy violation: denied"),
            (GovernanceError::RateLimitExceeded("too many".into()), "Rate limit exceeded: too many"),
            (GovernanceError::Auth("no token".into()), "Authentication error: no token"),
            (GovernanceError::NotFound("item".into()), "Resource not found: item"),
            (GovernanceError::NotAllowed("ops".into()), "Operation not allowed: ops"),
            (GovernanceError::Sync("drift".into()), "Sync error: drift"),
            (GovernanceError::Internal("crash".into()), "Internal error: crash"),
            (GovernanceError::Rubric("parse".into()), "Rubric error: parse"),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn error_invalid_channel_transition() {
        let err = GovernanceError::InvalidChannelTransition { from: "alpha".into(), to: "prod".into() };
        assert_eq!(err.to_string(), "Invalid channel transition from alpha to prod");
    }

    #[test]
    fn is_policy_error() {
        assert!(GovernanceError::PolicyViolation("x".into()).is_policy_error());
        assert!(GovernanceError::NotAllowed("x".into()).is_policy_error());
        assert!(!GovernanceError::Config("x".into()).is_policy_error());
        assert!(!GovernanceError::RateLimitExceeded("x".into()).is_policy_error());
    }

    #[test]
    fn is_rate_limit_error() {
        assert!(GovernanceError::RateLimitExceeded("x".into()).is_rate_limit_error());
        assert!(!GovernanceError::PolicyViolation("x".into()).is_rate_limit_error());
        assert!(!GovernanceError::Config("x".into()).is_rate_limit_error());
    }

    #[test]
    fn status_codes() {
        assert_eq!(GovernanceError::Config("x".into()).status_code(), 400);
        assert_eq!(GovernanceError::Database("x".into()).status_code(), 500);
        assert_eq!(GovernanceError::Network("x".into()).status_code(), 503);
        assert_eq!(GovernanceError::PolicyViolation("x".into()).status_code(), 403);
        assert_eq!(GovernanceError::RateLimitExceeded("x".into()).status_code(), 429);
        assert_eq!(GovernanceError::Auth("x".into()).status_code(), 401);
        assert_eq!(GovernanceError::InvalidChannelTransition { from: "a".into(), to: "b".into() }.status_code(), 400);
        assert_eq!(GovernanceError::NotFound("x".into()).status_code(), 404);
        assert_eq!(GovernanceError::NotAllowed("x".into()).status_code(), 403);
        assert_eq!(GovernanceError::Sync("x".into()).status_code(), 500);
        assert_eq!(GovernanceError::Internal("x".into()).status_code(), 500);
        assert_eq!(GovernanceError::Rubric("x".into()).status_code(), 422);
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let gov_err: GovernanceError = io_err.into();
        assert!(matches!(gov_err, GovernanceError::Internal(_)));
        assert!(gov_err.to_string().contains("file missing"));
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let gov_err: GovernanceError = json_err.into();
        assert!(matches!(gov_err, GovernanceError::Config(_)));
    }

    #[test]
    fn from_toml_error() {
        let toml_err = toml::from_str::<toml::Value>("bad = [unclosed").unwrap_err();
        let gov_err: GovernanceError = toml_err.into();
        assert!(matches!(gov_err, GovernanceError::Config(_)));
    }

    #[test]
    fn error_is_debug() {
        let err = GovernanceError::Internal("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Internal"));
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn config_status_is_bad_request() {
        assert_eq!(GovernanceError::Config("x".into()).status_code(), 400);
    }

    #[test]
    fn database_status_is_server_error() {
        assert_eq!(GovernanceError::Database("x".into()).status_code(), 500);
    }

    #[test]
    fn network_status_is_service_unavailable() {
        assert_eq!(GovernanceError::Network("x".into()).status_code(), 503);
    }

    #[test]
    fn auth_status_is_unauthorized() {
        assert_eq!(GovernanceError::Auth("x".into()).status_code(), 401);
    }

    #[test]
    fn not_found_status_is_404() {
        assert_eq!(GovernanceError::NotFound("x".into()).status_code(), 404);
    }

    #[test]
    fn policy_violation_status_is_forbidden() {
        assert_eq!(GovernanceError::PolicyViolation("x".into()).status_code(), 403);
    }

    #[test]
    fn rate_limit_status_is_429() {
        assert_eq!(GovernanceError::RateLimitExceeded("x".into()).status_code(), 429);
    }

    #[test]
    fn internal_and_sync_are_server_errors() {
        assert_eq!(GovernanceError::Internal("x".into()).status_code(), 500);
        assert_eq!(GovernanceError::Sync("x".into()).status_code(), 500);
    }

    #[test]
    fn rubric_status_is_unprocessable() {
        assert_eq!(GovernanceError::Rubric("x".into()).status_code(), 422);
    }

    #[test]
    fn invalid_channel_transition_status_is_bad_request() {
        let err = GovernanceError::InvalidChannelTransition {
            from: "alpha".into(),
            to: "prod".into(),
        };
        assert_eq!(err.status_code(), 400);
    }

    #[test]
    fn is_policy_error_matches_only_policy_variants() {
        assert!(GovernanceError::PolicyViolation("x".into()).is_policy_error());
        assert!(GovernanceError::NotAllowed("x".into()).is_policy_error());
        assert!(!GovernanceError::Auth("x".into()).is_policy_error());
        assert!(!GovernanceError::NotFound("x".into()).is_policy_error());
        assert!(!GovernanceError::Internal("x".into()).is_policy_error());
    }

    #[test]
    fn is_rate_limit_error_matches_only_rate_variant() {
        assert!(GovernanceError::RateLimitExceeded("x".into()).is_rate_limit_error());
        assert!(!GovernanceError::NotAllowed("x".into()).is_rate_limit_error());
        assert!(!GovernanceError::Internal("x".into()).is_rate_limit_error());
    }

    #[test]
    fn from_json_error_preserves_message() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad").unwrap_err();
        let expected = json_err.to_string();
        let gov_err: GovernanceError = json_err.into();
        assert!(matches!(gov_err, GovernanceError::Config(_)));
        assert!(gov_err.to_string().contains(&expected));
    }

    #[test]
    fn from_toml_error_is_config() {
        let toml_err = toml::from_str::<toml::Value>("x = [").unwrap_err();
        let gov_err: GovernanceError = toml_err.into();
        assert!(matches!(gov_err, GovernanceError::Config(_)));
    }

    #[test]
    fn from_io_error_is_internal() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let gov_err: GovernanceError = io_err.into();
        assert!(matches!(gov_err, GovernanceError::Internal(_)));
        assert!(gov_err.to_string().contains("denied"));
    }

    #[test]
    fn error_implements_std_error_trait() {
        fn as_dyn(e: &GovernanceError) -> &dyn std::error::Error {
            e
        }
        let err = GovernanceError::Internal("boom".into());
        assert!(as_dyn(&err).to_string().contains("boom"));
    }

    #[test]
    fn result_alias_holds_ok_and_err() {
        let ok: Result<u32> = Ok(3);
        let err: Result<u32> = Err(GovernanceError::NotFound("thing".into()));
        assert_eq!(ok.unwrap(), 3);
        assert!(err.is_err());
    }

    #[test]
    fn error_debug_contains_variant_name() {
        assert!(format!("{:?}", GovernanceError::Config("x".into())).contains("Config"));
        assert!(format!("{:?}", GovernanceError::Rubric("x".into())).contains("Rubric"));
    }
}
