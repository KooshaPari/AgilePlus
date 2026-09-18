// SPDX-License-Identifier: MIT OR Apache-2.0
//! Doubles whose port calls always fail.
//!
//! Fault injection on the stores themselves lives with the stores (see
//! [`super::memory`]); this module holds the doubles whose *only* behaviour is
//! failure.

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::events::{DomainEvent, DomainEventPublisher};

/// Publisher that rejects every event.
///
/// Use it to assert what a use case does when persistence succeeded but the
/// event bus did not: the write stays committed and the error surfaces.
#[derive(Debug, Default, Clone, Copy)]
pub struct FailingPublisher;

impl FailingPublisher {
    /// The error every publish returns.
    pub fn error() -> DomainError {
        DomainError::Storage("event bus unavailable".to_string())
    }
}

impl DomainEventPublisher for FailingPublisher {
    fn publish(&self, _event: DomainEvent) -> Result<(), DomainError> {
        Err(Self::error())
    }
}
