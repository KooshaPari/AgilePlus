// SPDX-License-Identifier: MIT OR Apache-2.0
//! Device-local AgilePlus project registry.
//!
//! This crate is the first bounded slice of the future `agileplusd` device
//! supervisor. It persists only routing and lifecycle configuration. Governed
//! work packages, evidence, evaluation, audit, and acceptance remain in each
//! project's canonical AgilePlus core and project-local SQLite database.

mod model;
mod schema;
mod store;
mod support;

pub use model::{
    DesiredState, ObservedState, RegisteredProject, RegistryError, RegistrySummary,
};
pub use store::{DeviceRegistry, default_registry_path};

#[cfg(test)]
mod tests;
