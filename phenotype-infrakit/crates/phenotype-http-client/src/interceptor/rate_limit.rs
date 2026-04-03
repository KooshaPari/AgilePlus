//! Rate limiting interceptor
//!
//! Uses phenotype-rate-limiter to enforce rate limits on HTTP requests.

use crate::error::{Error, HttpError, Result};
use crate::ports::InterceptorPort;
use crate::types::{Request, Response};
use async_trait::async_trait;
use std::sync::Arc;

pub struct RateLimitInterceptor {
    limiter: Arc<dyn RateLimiterPort>,
    key_fn: Arc<dyn Fn(&Request) -> String + Send + Sync>,
}

#[cfg(feature = "rate-limiter")]
use phenotype_rate_limiter::{RateLimiter, RateLimitError, RateLimitResult};

#[cfg(feature = "rate-limiter")]
pub trait RateLimiterPort: Send + Sync {
    fn check(&self, key: &str) -> impl std::future::Future<Output = RateLimitResult<()>> + Send + Sync
    where
        Self: Sized;
}

#[cfg(not(feature = "rate-limiter"))]
pub trait RateLimiterPort: Send + Sync {
    fn check(&self, key: &str) -> impl std::future::Future<Output = Result<()>> + Send + Sync
    where
        Self: Sized;
}

#[cfg(feature = "rate-limiter")]
#[async_trait]
impl RateLimiter for Arc<dyn RateLimiterPort> {
    async fn try_acquire(&self) -> RateLimitResult<()> {
        Ok(())
    }

    fn retry_after(&self) -> std::time::Duration {
        std::time::Duration::from_millis(1000)
    }

    fn available_permits(&self) -> u64 {
        100
    }
}

impl RateLimitInterceptor {
    pub fn new(limiter: Arc<dyn RateLimiterPort>) -> Self {
        Self {
            limiter,
            key_fn: Arc::new(|req: &Request| req.uri.host().unwrap_or("default").to_string()),
        }
    }

    pub fn with_key_fn<F>(limiter: Arc<dyn RateLimiterPort>, key_fn: F) -> Self
    where
        F: Fn(&Request) -> String + Send + Sync + 'static,
    {
        Self {
            limiter,
            key_fn: Arc::new(key_fn),
        }
    }
}

#[async_trait]
impl InterceptorPort for RateLimitInterceptor {
    type Error = Error;

    async fn intercept_request(&self, request: Request) -> std::result::Result<Request, Self::Error> {
        let key = (self.key_fn)(&request);
        self.limiter
            .check(&key)
            .await
            .map_err(|e| {
                if let Some(retry_after_ms) = e.retry_after_ms() {
                    Error::RateLimited {
                        retry_after_ms,
                    }
                } else {
                    Error::CircuitBreakerOpen(e.to_string())
                }
            })?;
        Ok(request)
    }

    async fn intercept_response(&self, response: Response) -> std::result::Result<Response, Self::Error> {
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Method, Uri};

    struct MockLimiter {
        allowed: bool,
        retry_after_ms: u64,
    }

    impl MockLimiter {
        fn new(allowed: bool, retry_after_ms: u64) -> Self {
            Self { allowed, retry_after_ms }
        }
    }

    #[cfg(not(feature = "rate-limiter"))]
    impl RateLimiterPort for MockLimiter {
        async fn check(&self, _key: &str) -> Result<()> {
            if self.allowed {
                Ok(())
            } else {
                Err(Error::RateLimited {
                    retry_after_ms: self.retry_after_ms,
                })
            }
        }
    }

    #[cfg(feature = "rate-limiter")]
    impl RateLimiterPort for MockLimiter {
        async fn check(&self, _key: &str) -> RateLimitResult<()> {
            if self.allowed {
                Ok(())
            } else {
                Err(RateLimitError::rate_limited(self.retry_after_ms))
            }
        }
    }

    #[test]
    fn test_rate_limit_interceptor_construction() {
        let limiter = Arc::new(MockLimiter::new(true, 1000));
        let _interceptor = RateLimitInterceptor::new(limiter);
    }

    #[test]
    fn test_rate_limit_interceptor_with_key_fn() {
        let limiter = Arc::new(MockLimiter::new(true, 1000));
        let interceptor = RateLimitInterceptor::with_key_fn(limiter, |req| {
            req.uri.host().unwrap_or("default").to_string()
        });
        assert_eq!(
            (interceptor.key_fn)(&Request::builder()
                .method(Method::GET)
                .uri("https://api.example.com")
                .build()
                .unwrap()),
            "api.example.com"
        );
    }

    #[tokio::test]
    async fn test_rate_limit_interceptor_allows_request() {
        let limiter = Arc::new(MockLimiter::new(true, 1000));
        let interceptor = RateLimitInterceptor::new(limiter);

        let request = Request::builder()
            .method(Method::GET)
            .uri("https://api.example.com")
            .build()
            .unwrap();

        let result = interceptor.intercept_request(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_interceptor_blocks_request() {
        let limiter = Arc::new(MockLimiter::new(false, 2000));
        let interceptor = RateLimitInterceptor::new(limiter);

        let request = Request::builder()
            .method(Method::GET)
            .uri("https://api.example.com")
            .build()
            .unwrap();

        let result = interceptor.intercept_request(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_interceptor_response_passthrough() {
        let limiter = Arc::new(MockLimiter::new(true, 1000));
        let interceptor = RateLimitInterceptor::new(limiter);

        let response = Response {
            status: 200,
            headers: crate::types::Headers::new(),
            body: crate::types::Body::empty(),
            duration_ms: 100,
        };

        let result = interceptor.intercept_response(response.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, 200);
    }
}
