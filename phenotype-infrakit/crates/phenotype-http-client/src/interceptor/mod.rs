//! Request/response interceptors
//!
//! Interceptors allow modifying requests before they're sent and
//! responses after they're received.

#[cfg(feature = "rate-limiter")]
pub mod rate_limit;

use crate::{
    error::{Error, Result},
    ports::InterceptorPort,
    types::{Request, Response},
};
use async_trait::async_trait;
use std::sync::Arc;

pub use crate::error::Error as InterceptorError;

#[cfg(feature = "rate-limiter")]
pub use rate_limit::RateLimitInterceptor;

#[cfg(not(feature = "rate-limiter"))]
use crate::error::HttpError;

#[cfg(not(feature = "rate-limiter"))]
impl From<crate::error::HttpError> for Error {
    fn from(e: HttpError) -> Self {
        Error::Unknown(e.to_string())
    }
}

/// Chain of interceptors
pub struct InterceptorChain {
    interceptors: Vec<Arc<dyn InterceptorPort<Error = Error> + Send + Sync>>,
}

impl InterceptorChain {
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    pub fn add(&mut self, interceptor: impl InterceptorPort<Error = Error> + 'static) {
        self.interceptors.push(Arc::new(interceptor));
    }

    pub async fn intercept_request(&self, mut request: Request) -> Result<Request> {
        for interceptor in &self.interceptors {
            request = interceptor.intercept_request(request).await?;
        }
        Ok(request)
    }

    pub async fn intercept_response(&self, mut response: Response) -> Result<Response> {
        for interceptor in self.interceptors.iter().rev() {
            response = interceptor.intercept_response(response).await?;
        }
        Ok(response)
    }

    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.interceptors.len()
    }
}

impl Default for InterceptorChain {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoggingInterceptor {
    log_bodies: bool,
}

impl LoggingInterceptor {
    pub fn new() -> Self {
        Self { log_bodies: false }
    }

    pub fn with_bodies(mut self) -> Self {
        self.log_bodies = true;
        self
    }
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InterceptorPort for LoggingInterceptor {
    type Error = Error;

    async fn intercept_request(&self, request: Request) -> Result<Request> {
        tracing::info!(
            method = %request.method.as_str(),
            uri = %request.uri,
            "HTTP Request"
        );

        if self.log_bodies && !request.body.is_empty() {
            if let Ok(text) = request.body.text() {
                tracing::debug!(body = %text, "Request body");
            }
        }

        Ok(request)
    }

    async fn intercept_response(&self, response: Response) -> Result<Response> {
        let status_class = if response.is_success() {
            "success"
        } else if response.is_client_error() {
            "client_error"
        } else if response.is_server_error() {
            "server_error"
        } else {
            "other"
        };

        tracing::info!(
            status = response.status,
            status_class = status_class,
            duration_ms = response.duration_ms,
            "HTTP Response"
        );

        Ok(response)
    }
}

pub struct TimingInterceptor;

impl TimingInterceptor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimingInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InterceptorPort for TimingInterceptor {
    type Error = Error;

    async fn intercept_request(&self, request: Request) -> Result<Request> {
        Ok(request)
    }

    async fn intercept_response(&self, response: Response) -> Result<Response> {
        Ok(response)
    }
}

pub struct AuthInterceptor {
    token: String,
    header_name: String,
}

impl AuthInterceptor {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            header_name: "authorization".to_string(),
        }
    }

    pub fn api_key(key: impl Into<String>, header_name: impl Into<String>) -> Self {
        Self {
            token: key.into(),
            header_name: header_name.into(),
        }
    }
}

#[async_trait]
impl InterceptorPort for AuthInterceptor {
    type Error = Error;

    async fn intercept_request(&self, mut request: Request) -> Result<Request> {
        let value = if self.header_name == "authorization" {
            format!("Bearer {}", self.token)
        } else {
            self.token.clone()
        };

        request.headers.insert(&self.header_name, value);
        Ok(request)
    }

    async fn intercept_response(&self, response: Response) -> Result<Response> {
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Method;

    #[test]
    fn test_interceptor_chain() {
        let mut chain = InterceptorChain::new();
        assert!(chain.is_empty());

        chain.add(LoggingInterceptor::new());
        assert_eq!(chain.len(), 1);
    }

    #[tokio::test]
    async fn test_auth_interceptor() {
        let interceptor = AuthInterceptor::bearer("test-token");

        let request = Request::builder()
            .method(Method::GET)
            .uri("https://api.example.com")
            .build()
            .unwrap();

        let modified = interceptor.intercept_request(request).await.unwrap();
        assert_eq!(
            modified.headers.get("authorization"),
            Some(&"Bearer test-token".to_string())
        );
    }
}
