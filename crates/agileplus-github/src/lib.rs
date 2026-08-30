//! agileplus-github — GitHub integration via raw reqwest.
//!
//! # Modules
//! - `client` — rate-limited reqwest client for create/update/get issues
//! - `sync`   — conflict-aware sync adapter for backlog items

pub mod client;
pub mod map;
pub mod sync;
