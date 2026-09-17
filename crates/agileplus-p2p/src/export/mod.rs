//! Git-backed state export — serialize SQLite state to deterministic files.
//!
//! Writes to `.agileplus/sync/` with the layout:
//!   events/{entity_type}/{id}.jsonl  — one JSON per line, ordered by sequence
//!   snapshots/{entity_type}/{id}.json — latest snapshot, pretty-printed sorted keys
//!   sync_state.json                  — SyncMapping entries and device sync vectors
//!   device.json                      — local DeviceNode info
//!
//! All JSON uses sorted keys, 2-space indent, UTF-8.
//!
//! Traceability: WP17 / T101

mod errors;
mod serialization;
mod types;
mod writer;

#[cfg(test)]
mod tests;

pub use errors::ExportError;
pub use types::{EntityRef, ExportStats};
pub use writer::export_state;
