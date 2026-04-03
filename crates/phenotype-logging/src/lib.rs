//! # Phenotype Logging
//!
//! Logging utilities for the Phenotype ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

/// Logging errors
#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Failed to initialize logger: {0}")]
    InitFailed(String),

    #[error("Failed to write log: {0}")]
    WriteFailed(String),
}

/// Result type for logging operations
pub type LoggingResult<T> = Result<T, LoggingError>;

// =============================================================================
// Log Levels
// =============================================================================

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level - most verbose
    Trace = 0,
    /// Debug level
    Debug = 1,
    /// Info level
    Info = 2,
    /// Warning level
    Warn = 3,
    /// Error level
    Error = 4,
    /// Fatal level - least verbose
    Fatal = 5,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

// =============================================================================
// Log Entry
// =============================================================================

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: i64,
    /// Log level
    pub level: LogLevel,
    /// Target/module
    pub target: String,
    /// Message
    pub message: String,
    /// Structured fields
    pub fields: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    /// Create a new log entry
    #[must_use]
    pub fn new(level: LogLevel, target: String, message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp_millis(),
            level,
            target,
            message,
            fields: HashMap::new(),
        }
    }

    /// Add a field to the log entry
    #[must_use]
    pub fn field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }
}

// =============================================================================
// Logger Configuration
// =============================================================================

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Minimum log level
    pub level: LogLevel,
    /// Enable ANSI colors
    pub ansi: bool,
    /// Enable JSON output
    pub json: bool,
    /// Include timestamp
    pub timestamp: bool,
    /// Include target/module
    pub include_target: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            ansi: true,
            json: false,
            timestamp: true,
            include_target: true,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            LogLevel::Info,
            "test_module".to_string(),
            "Test message".to_string(),
        );

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.target, "test_module");
        assert_eq!(entry.message, "Test message");
        assert!(entry.fields.is_empty());
    }

    #[test]
    fn test_log_entry_with_field() {
        let entry = LogEntry::new(
            LogLevel::Debug,
            "test".to_string(),
            "Debug info".to_string(),
        )
        .field("user_id", serde_json::json!("123"))
        .field("count", serde_json::json!(42));

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields["user_id"], "123");
        assert_eq!(entry.fields["count"], 42);
    }

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.ansi);
        assert!(!config.json);
    }
}
