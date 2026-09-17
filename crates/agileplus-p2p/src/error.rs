//! Error types for agileplus-p2p.

use thiserror::Error;

/// Errors that occur during peer discovery via Tailscale.
#[derive(Debug, Error)]
pub enum PeerDiscoveryError {
    #[error("Tailscale local API unavailable: {0}")]
    ApiUnavailable(String),

    #[error("HTTP error querying Tailscale: {0}")]
    HttpError(String),

    #[error("Failed to parse Tailscale response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported platform")]
    UnsupportedPlatform,
}

/// Errors during P2P synchronization.
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("NATS connection failed for peer {peer_id}: {reason}")]
    ConnectionFailed { peer_id: String, reason: String },

    #[error("NATS publish failed: {0}")]
    PublishFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Peer discovery error: {0}")]
    Discovery(#[from] PeerDiscoveryError),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Timeout connecting to peer {peer_id}")]
    Timeout { peer_id: String },
}

impl From<async_nats::Error> for SyncError {
    fn from(e: async_nats::Error) -> Self {
        SyncError::Nats(e.to_string())
    }
}

impl From<async_nats::ConnectError> for SyncError {
    fn from(e: async_nats::ConnectError) -> Self {
        SyncError::Nats(e.to_string())
    }
}

/// Errors during device registration.
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Could not connect to device registry: {0}")]
    RegistryUnavailable(String),

    #[error("Device already registered with different ID")]
    ConflictingRegistration,

    #[error("SQLite error: {0}")]
    Database(String),

    #[error("Tailscale query failed: {0}")]
    TailscaleQuery(#[from] PeerDiscoveryError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn peer_discovery_api_unavailable_display() {
        let e = PeerDiscoveryError::ApiUnavailable("no socket".into());
        assert_eq!(e.to_string(), "Tailscale local API unavailable: no socket");
    }

    #[test]
    fn peer_discovery_http_error_display() {
        let e = PeerDiscoveryError::HttpError("boom".into());
        assert!(e.to_string().contains("HTTP error"));
        assert!(e.to_string().contains("boom"));
    }

    #[test]
    fn peer_discovery_parse_error_from_serde() {
        let err = serde_json::from_str::<serde_json::Value>("{not json")
            .map(|_| ())
            .unwrap_err();
        let e: PeerDiscoveryError = err.into();
        assert!(matches!(e, PeerDiscoveryError::ParseError(_)));
        assert!(e.to_string().contains("parse Tailscale response"));
    }

    #[test]
    fn peer_discovery_io_error_from() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let e: PeerDiscoveryError = io.into();
        assert!(matches!(e, PeerDiscoveryError::Io(_)));
    }

    #[test]
    fn peer_discovery_unsupported_platform_display() {
        let e = PeerDiscoveryError::UnsupportedPlatform;
        assert_eq!(e.to_string(), "Unsupported platform");
    }

    #[test]
    fn peer_discovery_error_debug_nonempty() {
        let e = PeerDiscoveryError::UnsupportedPlatform;
        assert!(!format!("{e:?}").is_empty());
    }

    #[test]
    fn sync_error_connection_failed_display() {
        let e = SyncError::ConnectionFailed {
            peer_id: "dev-1".into(),
            reason: "refused".into(),
        };
        let s = e.to_string();
        assert!(s.contains("dev-1"));
        assert!(s.contains("refused"));
    }

    #[test]
    fn sync_error_publish_failed_display() {
        let e = SyncError::PublishFailed("nope".into());
        assert_eq!(e.to_string(), "NATS publish failed: nope");
    }

    #[test]
    fn sync_error_serialization_from() {
        let err = serde_json::from_str::<serde_json::Value>("x").unwrap_err();
        let e: SyncError = err.into();
        assert!(matches!(e, SyncError::Serialization(_)));
    }

    #[test]
    fn sync_error_event_store_display() {
        let e = SyncError::EventStore("db down".into());
        assert!(e.to_string().contains("db down"));
    }

    #[test]
    fn sync_error_discovery_from() {
        let d = PeerDiscoveryError::UnsupportedPlatform;
        let e: SyncError = d.into();
        assert!(matches!(e, SyncError::Discovery(_)));
    }

    #[test]
    fn sync_error_nats_display() {
        let e = SyncError::Nats("nats err".into());
        assert_eq!(e.to_string(), "NATS error: nats err");
    }

    #[test]
    fn sync_error_timeout_display() {
        let e = SyncError::Timeout {
            peer_id: "dev-9".into(),
        };
        assert!(e.to_string().contains("dev-9"));
        assert!(e.to_string().contains("Timeout"));
    }

    #[test]
    fn sync_error_debug() {
        let e = SyncError::Nats("x".into());
        assert!(format!("{e:?}").contains("Nats"));
    }

    #[test]
    fn connection_error_registry_unavailable_display() {
        let e = ConnectionError::RegistryUnavailable("down".into());
        assert!(e.to_string().contains("device registry"));
        assert!(e.to_string().contains("down"));
    }

    #[test]
    fn connection_error_conflicting_registration_display() {
        let e = ConnectionError::ConflictingRegistration;
        assert_eq!(
            e.to_string(),
            "Device already registered with different ID"
        );
    }

    #[test]
    fn connection_error_database_display() {
        let e = ConnectionError::Database("locked".into());
        assert!(e.to_string().contains("SQLite error"));
    }

    #[test]
    fn connection_error_tailscale_from() {
        let d = PeerDiscoveryError::ApiUnavailable("x".into());
        let e: ConnectionError = d.into();
        assert!(matches!(e, ConnectionError::TailscaleQuery(_)));
    }

    #[test]
    fn connection_error_io_from() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no");
        let e: ConnectionError = io.into();
        assert!(matches!(e, ConnectionError::Io(_)));
    }

    #[test]
    fn connection_error_debug() {
        let e = ConnectionError::Database("x".into());
        assert!(format!("{e:?}").contains("Database"));
    }

    #[test]
    fn all_errors_implement_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<PeerDiscoveryError>();
        assert_error::<SyncError>();
        assert_error::<ConnectionError>();
    }
}
