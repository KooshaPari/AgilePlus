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
}

impl Default for InterceptorChain {
    fn default() -> Self {
        Self::new()
    }
}
