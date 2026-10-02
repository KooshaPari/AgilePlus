// SPDX-License-Identifier: MIT OR Apache-2.0
//! Terminal acceptance delegates to one focused transactional capability.
//! Callers cannot supply a boolean governance pass or separately write terminal state.

pub use agileplus_domain::domain::acceptance::{
    AcceptFeatureCommand, AcceptanceOutcome, FeatureAcceptanceReceipt,
};
use agileplus_domain::ports::execution::AtomicAcceptancePort;
use crate::error::AppError;

pub async fn accept_feature<S: AtomicAcceptancePort + ?Sized>(
    storage: &S,
    command: &AcceptFeatureCommand,
) -> Result<AcceptanceOutcome, AppError> {
    command.validate()?;
    Ok(storage.accept_feature_atomic(command).await?)
}
