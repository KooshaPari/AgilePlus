//! Focused atomic persistence boundary for terminal feature acceptance.

use async_trait::async_trait;

use crate::{domain::audit::AuditEntry, error::DomainError};

/// Fully prepared mutation set. Correctness/governance preflight happens before
/// this reaches persistence; the adapter commits these writes atomically.
#[derive(Debug, Clone)]
pub struct TerminalAcceptanceMutation {
    pub feature_id: i64,
    pub wp_ids_to_complete: Vec<i64>,
    pub audit_entries: Vec<AuditEntry>,
}

#[async_trait]
pub trait TerminalAcceptancePort: Send + Sync {
    async fn commit_terminal_acceptance(
        &self,
        mutation: &TerminalAcceptanceMutation,
    ) -> Result<(), DomainError>;
}
