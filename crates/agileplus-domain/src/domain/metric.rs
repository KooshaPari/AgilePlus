//! Metric type — telemetry attached to features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A recorded metric for a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: i64,
    pub feature_id: Option<i64>,
    pub command: String,
    pub duration_ms: i64,
    pub agent_runs: i32,
    pub review_cycles: i32,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}
