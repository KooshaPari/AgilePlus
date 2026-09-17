use crate::error::ConnectionError;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Snapshot store error: {0}")]
    SnapshotStore(String),

    #[error("Device store error: {0}")]
    DeviceStore(#[from] ConnectionError),

    #[error("Sync store error: {0}")]
    SyncStore(String),
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn export_error_io_display() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "disk full");
        let e: ExportError = io.into();
        assert!(e.to_string().contains("IO error"));
    }

    #[test]
    fn export_error_serialization_display() {
        let err = serde_json::from_str::<serde_json::Value>("x").unwrap_err();
        let e: ExportError = err.into();
        assert!(e.to_string().contains("Serialization error"));
    }

    #[test]
    fn export_error_event_store_display() {
        let e = ExportError::EventStore("es".into());
        assert_eq!(e.to_string(), "Event store error: es");
    }

    #[test]
    fn export_error_snapshot_store_display() {
        let e = ExportError::SnapshotStore("ss".into());
        assert_eq!(e.to_string(), "Snapshot store error: ss");
    }

    #[test]
    fn export_error_device_store_from() {
        let e: ExportError = ConnectionError::ConflictingRegistration.into();
        assert!(matches!(e, ExportError::DeviceStore(_)));
        assert!(e.to_string().contains("Device store error"));
    }

    #[test]
    fn export_error_sync_store_display() {
        let e = ExportError::SyncStore("sync".into());
        assert_eq!(e.to_string(), "Sync store error: sync");
    }

    #[test]
    fn export_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ExportError>();
    }
}
