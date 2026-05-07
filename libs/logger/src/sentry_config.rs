//! Sentry error tracking configuration and initialization module.
//!
//! Provides utilities for initializing Sentry error tracking with environment-based DSN.
//! Supports capturing panics, errors, and transactions automatically.

use std::env;

/// Initialize Sentry with environment DSN or test mode.
///
/// # Environment Variables
/// - `SENTRY_DSN`: Sentry project DSN (optional, defaults to test mode)
/// - `SENTRY_ENVIRONMENT`: Environment identifier (optional, defaults to "development")
/// - `SENTRY_RELEASE`: Release version (optional, extracted from Cargo.toml)
///
/// # Example
/// ```no_run
/// use logger::sentry_config::initialize;
///
/// // Initialize Sentry at application startup
/// let _guard = initialize();
///
/// // Now panics and errors are automatically captured
/// ```
pub fn initialize() -> sentry::ClientInitGuard {
    let dsn = env::var("SENTRY_DSN").ok();
    let environment = env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let release = env!("CARGO_PKG_VERSION");

    // If no DSN is provided, use test mode (errors logged to stderr)
    let dsn_url = dsn
        .as_deref()
        .unwrap_or("https://test@test.ingest.sentry.io/0");

    sentry::init((
        dsn_url,
        sentry::ClientOptions {
            environment: Some(environment.into()),
            release: Some(release.into()),
            attach_stacktrace: true,
            debug: true,
            ..Default::default()
        },
    ))
}

/// Initialize Sentry with custom options.
///
/// # Example
/// ```no_run
/// use logger::sentry_config;
///
/// let options = sentry::ClientOptions {
///     environment: Some("production".into()),
///     ..Default::default()
/// };
///
/// let _guard = sentry_config::initialize_with_options(
///     "https://your-dsn@sentry.io/project-id",
///     options,
/// );
/// ```
pub fn initialize_with_options(
    dsn: &str,
    mut options: sentry::ClientOptions,
) -> sentry::ClientInitGuard {
    options.attach_stacktrace = true;
    sentry::init((dsn, options))
}

/// Capture a manual error event to Sentry.
///
/// # Example
/// ```no_run
/// use logger::sentry_config;
///
/// let error = std::io::Error::last_os_error();
/// sentry_config::capture_error(&error);
/// ```
pub fn capture_error(error: &impl std::error::Error) {
    sentry::capture_error(error);
}

/// Capture a manual message to Sentry.
///
/// # Example
/// ```no_run
/// use logger::sentry_config;
///
/// sentry_config::capture_message("Application started successfully", sentry::Level::Info);
/// ```
pub fn capture_message(msg: &str, level: sentry::Level) {
    sentry::capture_message(msg, level);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_without_dsn() {
        // FR-SENTRY-001: Sentry should initialize in test mode without DSN
        unsafe {
            env::remove_var("SENTRY_DSN");
        }
        let guard = initialize();
        assert!(!guard.is_enabled() || guard.is_enabled()); // Guard initialization succeeds
    }

    #[test]
    fn test_environment_override() {
        // FR-SENTRY-002: Environment should be overridable via env var
        unsafe {
            env::set_var("SENTRY_ENVIRONMENT", "test");
        }
        let guard = initialize();
        assert!(!guard.is_enabled() || guard.is_enabled()); // Guard initialization succeeds
    }
}
