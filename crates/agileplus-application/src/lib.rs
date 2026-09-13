// SPDX-License-Identifier: MIT OR Apache-2.0
//! `agileplus-application` — hexagonal application / use-case layer.
//!
//! Each use case is a struct holding `Arc<dyn Port + Send + Sync>` deps wired
//! explicitly at call-site. No DI frameworks; no axum/sqlx types.
//!
//! ## Sub-modules
//!
//! - [`dto`]       — Data Transfer Objects
//! - [`use_cases`] — Use case implementations
//!
//! # Structure
//! - `use_cases/` — one module per use case (incl. `triage` for the CLI
//!   triage subcommands backed by `agileplus-triage` + `agileplus-graph`)
//! - `dto/`       — command/output data transfer objects
//! - `error.rs`   — `AppError` (thiserror; never leaks storage details)
//! - `events.rs`  — re-exports `DomainEvent` / `DomainEventPublisher`

pub mod dto;
pub mod error;
pub mod events;
pub mod use_cases;

#[cfg(test)]
mod test_cases;
#[cfg(test)]
pub mod test_mocks;
