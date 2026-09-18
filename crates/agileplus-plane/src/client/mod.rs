//! Plane.so REST API client with rate limiting.
//!
//! Traceability: WP18-T104

mod endpoints;
mod models;
mod rate_limit;
mod resources;
#[cfg(test)]
mod tests;
mod transport;

#[cfg(test)]
mod mock;

use std::sync::Arc;

use anyhow::Result;
use reqwest::Method;
use tokio::sync::Mutex;

pub use self::models::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};
pub use self::rate_limit::TokenBucket;

#[cfg(test)]
pub use mock::InMemoryPlaneClient;

/// Plane.so API client with token bucket rate limiter.
#[derive(Debug, Clone)]
pub struct PlaneClient {
    base_url: String,
    api_key: String,
    workspace_slug: String,
    project_id: String,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<TokenBucket>>,
}

impl PlaneClient {
    /// Create a new Plane.so client.
    /// Rate limited to 50 requests/minute.
    pub fn new(
        base_url: String,
        api_key: String,
        workspace_slug: String,
        project_id: String,
    ) -> Self {
        Self {
            base_url,
            api_key,
            workspace_slug,
            project_id,
            client: reqwest::Client::new(),
            // 50 req/min = 0.833 req/sec
            rate_limiter: Arc::new(Mutex::new(TokenBucket::new(50.0, 50.0 / 60.0))),
        }
    }

    /// Wait for rate limit token, then proceed.
    async fn acquire_token(&self) -> Result<()> {
        loop {
            let mut limiter = self.rate_limiter.lock().await;
            if limiter.try_acquire() {
                return Ok(());
            }
            let wait = limiter.time_until_available();
            drop(limiter);
            tokio::time::sleep(wait).await;
        }
    }

    async fn execute_request_json<T: serde::Serialize + ?Sized>(
        &self,
        method: Method,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        transport::request_json(&self.client, method, url, &self.api_key, body).await
    }

    async fn execute_request_without_body(
        &self,
        method: Method,
        url: &str,
    ) -> Result<reqwest::Response> {
        transport::request_without_body(&self.client, method, url, &self.api_key).await
    }
}

/// Rate-limiter behaviour observed through the public client API.
///
/// `TokenBucket` itself is unit-tested elsewhere; what matters here is that a
/// caller which has spent the bucket is actually made to wait for a refill
/// instead of being rejected.
#[cfg(test)]
mod rate_limit_backoff_tests {
    use super::*;
    use std::time::{Duration, Instant};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn work_item() -> PlaneWorkItem {
        PlaneWorkItem {
            id: None,
            name: "Item".to_string(),
            description_html: None,
            state: None,
            priority: None,
            parent: None,
            labels: vec![],
        }
    }

    #[tokio::test]
    async fn client_waits_for_a_refill_once_the_bucket_is_empty() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": "item-1", "name": "Item"})),
            )
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let item = work_item();

        // The client is constructed with a full 50-token bucket refilling at
        // 50/min, so the 51st call can only succeed after `acquire_token` has
        // slept for the refill.
        let started = Instant::now();
        for _ in 0..51 {
            client
                .create_work_item(&item)
                .await
                .expect("every call should eventually be admitted");
        }
        let elapsed = started.elapsed();

        assert!(
            elapsed >= Duration::from_millis(900),
            "the 51st request should have blocked for a refill, elapsed={elapsed:?}"
        );
    }
}
