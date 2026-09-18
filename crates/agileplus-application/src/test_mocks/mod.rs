// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-memory test doubles for the application layer.
//!
//! Re-exported flat from this module, so tests can write
//! `use crate::test_mocks::*;` and get every double.
//!
//! - [`memory`] — in-memory stores (happy path + injectable faults)
//! - [`failing`] — doubles whose port calls always fail

pub mod failing;
pub mod memory;

pub use failing::FailingPublisher;
pub use memory::{
    InMemoryEpicRepo, InMemoryFeatureRepo, InMemoryStoryRepo, InMemoryWpRepo, SpyPublisher, WpQuery,
};
