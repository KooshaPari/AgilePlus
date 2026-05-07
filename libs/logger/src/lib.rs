//! Logging infrastructure for AgilePlus
//!
//! Provides a trait-based logging abstraction with multiple implementations.

pub mod error;
pub mod sentry_config;

pub use error::{LogError, Result};
pub use sentry_config::{capture_error, capture_message, initialize, initialize_with_options};

/// Log severity levels.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Check if this level should be logged given a minimum level.
    pub fn should_log(self, min_level: LogLevel) -> bool {
        self >= min_level
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// Logger trait for logging operations.
pub trait Logger: Send + Sync {
    /// Log a debug message.
    fn debug(&self, msg: &str) -> Result<()>;

    /// Log an info message.
    fn info(&self, msg: &str) -> Result<()>;

    /// Log a warning message.
    fn warn(&self, msg: &str) -> Result<()>;

    /// Log an error message.
    fn error(&self, msg: &str) -> Result<()>;

    /// Get the current log level.
    fn level(&self) -> LogLevel;
}

/// JSON-structured logger implementation.
#[derive(Debug, Clone)]
pub struct JsonLogger {
    level: LogLevel,
}

impl JsonLogger {
    /// Create a new JSON logger with the specified level.
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    /// Create a new JSON logger with default level.
    pub fn default_logger() -> Self {
        Self::new(LogLevel::default())
    }
}

impl Logger for JsonLogger {
    fn debug(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Debug) {
            return Ok(());
        }
        let entry = serde_json::json!({
            "level": "debug",
            "message": msg,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", entry);
        Ok(())
    }

    fn info(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Info) {
            return Ok(());
        }
        let entry = serde_json::json!({
            "level": "info",
            "message": msg,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", entry);
        Ok(())
    }

    fn warn(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Warn) {
            return Ok(());
        }
        let entry = serde_json::json!({
            "level": "warn",
            "message": msg,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", entry);
        Ok(())
    }

    fn error(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Error) {
            return Ok(());
        }
        let entry = serde_json::json!({
            "level": "error",
            "message": msg,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        println!("{}", entry);
        Ok(())
    }

    fn level(&self) -> LogLevel {
        self.level
    }
}

/// Console logger with human-readable output.
#[derive(Debug, Clone)]
pub struct ConsoleLogger {
    level: LogLevel,
}

impl ConsoleLogger {
    /// Create a new console logger with the specified level.
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    /// Create a new console logger with default level.
    pub fn default_logger() -> Self {
        Self::new(LogLevel::default())
    }
}

impl Logger for ConsoleLogger {
    fn debug(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Debug) {
            return Ok(());
        }
        println!(
            "[{}] DEBUG: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
        Ok(())
    }

    fn info(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Info) {
            return Ok(());
        }
        println!(
            "[{}] INFO: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
        Ok(())
    }

    fn warn(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Warn) {
            return Ok(());
        }
        println!(
            "[{}] WARN: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
        Ok(())
    }

    fn error(&self, msg: &str) -> Result<()> {
        if !self.level.should_log(LogLevel::Error) {
            return Ok(());
        }
        println!(
            "[{}] ERROR: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
        Ok(())
    }

    fn level(&self) -> LogLevel {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
    }

    #[test]
    fn test_log_level_should_log() {
        assert!(LogLevel::Debug.should_log(LogLevel::Debug));
        assert!(LogLevel::Info.should_log(LogLevel::Debug));
        assert!(!LogLevel::Debug.should_log(LogLevel::Info));
        assert!(LogLevel::Error.should_log(LogLevel::Warn));
    }

    #[test]
    fn test_console_logger_creation() {
        let logger = ConsoleLogger::new(LogLevel::Debug);
        assert_eq!(logger.level(), LogLevel::Debug);
    }

    #[test]
    fn test_json_logger_creation() {
        let logger = JsonLogger::new(LogLevel::Info);
        assert_eq!(logger.level(), LogLevel::Info);
    }

    #[test]
    fn test_console_logger_filtering() {
        let logger = ConsoleLogger::new(LogLevel::Warn);
        assert!(!LogLevel::Debug.should_log(logger.level()));
        assert!(!LogLevel::Info.should_log(logger.level()));
        assert!(LogLevel::Warn.should_log(logger.level()));
        assert!(LogLevel::Error.should_log(logger.level()));
    }
}
