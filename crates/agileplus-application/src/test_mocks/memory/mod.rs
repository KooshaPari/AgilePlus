// SPDX-License-Identifier: MIT OR Apache-2.0
//! Happy-path in-memory doubles for the application layer.
//!
//! These stores behave like a simple, correct backend: writes succeed and
//! reads return what was written. Each store also exposes `failing_*`
//! builders that flip a flag so one operation returns
//! [`agileplus_domain::error::DomainError::Storage`] — enough to drive a use
//! case down its error path without hand-writing a second port impl.
//!
//! - [`storage::InMemoryFeatureRepo`] — the `StoragePort` double
//! - [`ports::InMemoryStoryRepo`], [`ports::InMemoryEpicRepo`],
//!   [`ports::InMemoryWpRepo`] — the smaller repository ports
//! - [`ports::SpyPublisher`] — records emitted domain events

pub mod ports;
pub mod storage;

pub use ports::{InMemoryEpicRepo, InMemoryStoryRepo, InMemoryWpRepo, SpyPublisher, WpQuery};
pub use storage::InMemoryFeatureRepo;
