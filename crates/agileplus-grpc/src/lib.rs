//! agileplus-grpc — tonic gRPC adapter layer.
//!
//! This crate wires the AgilePlus domain/application ports to gRPC services
//! via tonic.  All proto types are sourced from the `agileplus-proto` crate.
//!
//! Services implemented:
//!  - `AgilePlusCoreService`  — feature lifecycle, work-packages, governance, audit
//!  - `WorkItemsService`      — projects, epics, stories, GitHub sync (FR-AGP-011)
//!
//! Hexagonal adapter: every RPC delegates to a domain port (`StoragePort`,
//! `VcsPort`, etc.) with no business logic inside the adapter layer.
//!
//! Traceability: FR-AGP-011, WP14-T079, T080, T083

pub mod conversions;
pub mod event_bus;
pub mod proxy;
pub mod server;
pub mod streaming;
pub mod work_items;

// Re-export the public entry points callers need.
pub use server::domain_error_to_status;
pub use server::AgilePlusCoreServer;
#[cfg(not(agileplus_proto_stubs))]
pub use server::start_server;
