//! HTTP-based analytics tracker using phenotype-http-client

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::{error::Result, event::Event, track::Tracker};

pub use phenotype_http_client::adapters::ReqwestAdapter;
pub use phenotype_http_client::ports::HttpClientPort;
pub use phenotype_http_client::types::{Body, Method, Request, Uri};

#[derive(Debug, Clone)]
pub struct HttpTrackerConfig {
    pub endpoint: String,
    pub api_key: String,
}

impl HttpTrackerConfig {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
        }
    }
}

pub struct HttpTracker {
    client: Arc<dyn HttpClientPort>,
    endpoint: String,
    api_key: String,
}

impl HttpTracker {
    pub fn new(config: HttpTrackerConfig) -> Self {
        Self {
            client: Arc::new(ReqwestAdapter::new()),
            endpoint: config.endpoint,
            api_key: config.api_key,
        }
    }

    pub fn with_client<C: HttpClientPort + 'static>(client: C, config: HttpTrackerConfig) -> Self {
        Self {
            client: Arc::new(client),
            endpoint: config.endpoint,
            api_key: config.api_key,
        }
    }

    pub async fn send_batch(&self, events: &[Event]) -> Result<()> {
        let uri = Uri::parse(&self.endpoint)
            .map_err(|e| crate::AnalyticsError::TrackingFailed(e.to_string()))?;

        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .body(Body::from_json(&serde_json::json!({
                "events": events,
                "sent_at": Utc::now().to_rfc3339()
            })).map_err(|e| crate::AnalyticsError::TrackingFailed(e.to_string()))?)
            .build()
            .map_err(|e| crate::AnalyticsError::TrackingFailed(e.to_string()))?;

        let response = self.client.execute(request).await
            .map_err(|e| crate::AnalyticsError::TrackingFailed(e.to_string()))?;

        if !response.is_success() {
            return Err(crate::AnalyticsError::TrackingFailed(
                format!("HTTP {}", response.status)
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl Tracker for HttpTracker {
    fn track(&self, event: Event) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.send_batch(&[event]))
    }
}
