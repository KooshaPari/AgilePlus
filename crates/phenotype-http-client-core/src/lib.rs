//! # Phenotype HTTP Client Core
//!
//! Unified HTTP client patterns and pooling strategies for Phenotype ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

/// HTTP client errors
#[derive(Error, Debug)]
pub enum HttpClientError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Response parse error: {0}")]
    ParseError(String),

    #[error("Status code error: {status} - {body}")]
    StatusError { status: u16, body: String },
}

/// Result type for HTTP operations
pub type HttpResult<T> = Result<T, HttpClientError>;

// =============================================================================
// HTTP Methods
// =============================================================================

/// Supported HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// PATCH request
    Patch,
    /// DELETE request
    Delete,
    /// HEAD request
    Head,
    /// OPTIONS request
    Options,
}

// =============================================================================
// Request/Response
// =============================================================================

/// HTTP request builder
#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u64>,
}

impl Request {
    /// Create a new GET request
    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            url: url.into(),
            headers: vec![],
            body: None,
            timeout_ms: None,
        }
    }

    /// Create a new POST request
    #[must_use]
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Post,
            url: url.into(),
            headers: vec![],
            body: None,
            timeout_ms: None,
        }
    }

    /// Add a header to the request
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Set request body
    #[must_use]
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set timeout in milliseconds
    #[must_use]
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    /// Check if status is success (2xx)
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if status is client error (4xx)
    #[must_use]
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if status is server error (5xx)
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Parse body as UTF-8 string
    pub fn text(&self) -> HttpResult<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    /// Parse body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> HttpResult<T> {
        serde_json::from_slice(&self.body)
            .map_err(|e| HttpClientError::ParseError(e.to_string()))
    }
}

// =============================================================================
// HTTP Client Trait
// =============================================================================

/// HTTP client abstraction
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Execute a request and return the response
    async fn execute(&self, request: Request) -> HttpResult<Response>;

    /// Execute a GET request
    async fn get(&self, url: &str) -> HttpResult<Response> {
        self.execute(Request::get(url)).await
    }

    /// Execute a POST request with JSON body
    async fn post_json<T: serde::Serialize>(&self, url: &str, body: &T) -> HttpResult<Response> {
        let json = serde_json::to_vec(body)
            .map_err(|e| HttpClientError::ParseError(e.to_string()))?;

        self.execute(
            Request::post(url)
                .header("Content-Type", "application/json")
                .body(json),
        )
        .await
    }
}

// =============================================================================
// Connection Pool
// =============================================================================

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum idle connections per host
    pub max_idle_per_host: usize,
    /// Maximum total idle connections
    pub max_idle_total: usize,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 5,
            max_idle_total: 10,
            connect_timeout_secs: 30,
            idle_timeout_secs: 90,
        }
    }
}

// =============================================================================
// Retry Configuration
// =============================================================================

/// Retry configuration for failed requests
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// HTTP status codes that should be retried
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            retry_on_status: vec![429, 500, 502, 503, 504],
        }
    }
}

impl RetryConfig {
    /// Check if a status code should trigger a retry
    #[must_use]
    pub fn should_retry(&self, status: u16) -> bool {
        self.retry_on_status.contains(&status)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let request = Request::get("https://api.example.com/users")
            .header("Authorization", "Bearer token")
            .header("Accept", "application/json")
            .timeout(5000);

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, "https://api.example.com/users");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.timeout_ms, Some(5000));
    }

    #[test]
    fn test_request_with_body() {
        let body = b"Hello World".to_vec();
        let request = Request::post("https://api.example.com/data")
            .header("Content-Type", "text/plain")
            .body(body.clone());

        assert_eq!(request.body, Some(body));
    }

    #[test]
    fn test_response_status_checks() {
        let success = Response {
            status: 200,
            headers: vec![],
            body: vec![],
        };
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let client_err = Response {
            status: 404,
            headers: vec![],
            body: vec![],
        };
        assert!(!client_err.is_success());
        assert!(client_err.is_client_error());

        let server_err = Response {
            status: 500,
            headers: vec![],
            body: vec![],
        };
        assert!(!server_err.is_success());
        assert!(server_err.is_server_error());
    }

    #[test]
    fn test_response_text() {
        let response = Response {
            status: 200,
            headers: vec![],
            body: b"Hello World".to_vec(),
        };

        assert_eq!(response.text().unwrap(), "Hello World");
    }

    #[test]
    fn test_response_json() {
        let response = Response {
            status: 200,
            headers: vec![],
            body: b"{\"key\": \"value\"}".to_vec(),
        };

        let json: serde_json::Value = response.json().unwrap();
        assert_eq!(json["key"], "value");
    }

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_idle_per_host, 5);
        assert_eq!(config.max_idle_total, 10);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.should_retry(500));
        assert!(config.should_retry(503));
        assert!(!config.should_retry(200));
        assert!(!config.should_retry(404));
    }
}
