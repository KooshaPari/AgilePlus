//! Durable Git/SQLite promotion saga. Prepared Git object identities precede ref mutation.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionPlanEntry {
    pub wp_id: i64,
    pub wp_sequence: i32,
    pub wp_label: String,
    pub title: String,
    pub branch: String,
    pub accepted_candidate_ref: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionPlan {
    pub feature_id: i64,
    pub feature_slug: String,
    pub target_branch: String,
    pub entries: Vec<PromotionPlanEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionStep {
    pub expected_target: String,
    pub resulting_commit: String,
    pub confirmed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionReceipt {
    pub feature_id: i64,
    pub acceptance_request_id: String,
    pub target_branch: String,
    pub resulting_commit: String,
    pub accepted_candidates: Vec<String>,
    pub audit_id: i64,
    pub event_id: i64,
    pub committed_at: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionJournal {
    pub plan: PromotionPlan,
    pub acceptance_request_id: String,
    pub initial_target: String,
    pub steps: Vec<PromotionStep>,
    pub receipt: Option<PromotionReceipt>,
    pub cleanup_complete: bool,
}
