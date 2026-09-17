// SPDX-License-Identifier: MIT OR Apache-2.0
//! Comprehensive contract tests for the protoc-generated `agileplus.v1` types.
//!
//! These tests only compile against the real prost/tonic codegen output; the
//! hand-written stubs used when `protoc` is unavailable do not implement
//! [`prost::Message`], so the module is gated on `not(agileplus_proto_stubs)`
//! (see the declaration in `lib.rs`).
//!
//! Coverage:
//! - binary encode/decode round-trips for every generated message type
//! - `encoded_len` agreement with the produced buffer
//! - proto3 defaults and field presence rules
//! - nested message / map / `bytes` / `repeated` handling
//! - merge (concatenated message) semantics
//! - decode validation (empty buffer, truncated buffer, unknown fields)
//! - tonic request/response/metadata conversions
//! - service `NamedService` wiring
//!
//! Modules are split by the `.proto` file that defines the messages under test.
//!
//! Traceability: FR-AGP-011

/// Assert that `$value` survives an encode/decode round-trip unchanged and that
/// `encoded_len` matches the bytes actually produced.
macro_rules! roundtrip {
    ($name:ident, $ty:ty, $value:expr) => {
        #[test]
        fn $name() {
            let original: $ty = $value;
            let mut buf = Vec::new();
            original.encode(&mut buf).expect("encode");
            assert_eq!(buf.len(), original.encoded_len(), "encoded_len mismatch");
            let decoded = <$ty as Message>::decode(&buf[..]).expect("decode");
            assert_eq!(original, decoded);
        }
    };
}

mod agents;
mod common;
mod core_service;
mod integrations;
mod semantics;
mod work_items;
