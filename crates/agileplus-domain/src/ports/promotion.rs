//! Capability-separated persistence and Git publication for resumable promotion.
use crate::{
    domain::promotion::{PromotionJournal, PromotionReceipt, PromotionStep},
    error::DomainError,
};
#[async_trait::async_trait]
pub trait PromotionPort: Send + Sync {
    async fn get_promotion(&self, feature_id: i64)
    -> Result<Option<PromotionJournal>, DomainError>;
    async fn begin_promotion(
        &self,
        journal: &PromotionJournal,
    ) -> Result<PromotionJournal, DomainError>;
    async fn prepare_promotion_step(
        &self,
        feature_id: i64,
        index: usize,
        step: &PromotionStep,
    ) -> Result<(), DomainError>;
    async fn confirm_promotion_step(
        &self,
        feature_id: i64,
        index: usize,
    ) -> Result<(), DomainError>;
    /// Commit Shipped, audit, event and receipt together, or none of them.
    async fn finalize_promotion(&self, feature_id: i64) -> Result<PromotionReceipt, DomainError>;
    async fn complete_promotion_cleanup(&self, feature_id: i64) -> Result<(), DomainError>;
}
#[async_trait::async_trait]
pub trait PromotionVcsPort: Send + Sync {
    async fn promotion_target(&self, target: &str) -> Result<String, DomainError>;
    /// Create immutable merge objects only. No branch, index or worktree mutation.
    async fn prepare_promotion_merge(
        &self,
        source: &str,
        expected_target: &str,
    ) -> Result<String, DomainError>;
    /// CAS the exact target ref, or recognize an exact previous publication.
    /// Any other target movement fails closed; never infer success from a branch name.
    async fn publish_promotion_merge(
        &self,
        target: &str,
        expected: &str,
        result: &str,
    ) -> Result<(), DomainError>;
}
