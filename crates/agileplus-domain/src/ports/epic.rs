// SPDX-License-Identifier: MIT OR Apache-2.0
//! Epic repository port.

use crate::domain::epic::{Epic, EpicStatus};
use crate::error::DomainError;

/// Repository port for Epic aggregates.
pub trait EpicRepository: Send + Sync {
    fn create(&self, epic: &Epic) -> Result<i64, DomainError>;
    fn get_by_id(&self, id: i64) -> Result<Option<Epic>, DomainError>;
    fn update_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError>;
    fn list_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError>;
}
