//! AgilePlus protobuf/tonic generated types.
//!
//! When `protoc` is available at build time `build.rs` compiles the `.proto`
//! files and the output is included via the `include!` macro below.  When
//! `protoc` is absent (CI check-only, dev machines without the compiler)
//! `build.rs` emits `cargo:rustc-cfg=agileplus_proto_stubs` and the
//! hand-written stubs module is compiled instead so `cargo check --workspace`
//! stays green.
//!
//! Traceability: FR-AGP-011

pub mod agileplus {
    pub mod v1 {
        // Include protoc output when available …
        #[cfg(not(agileplus_proto_stubs))]
        include!(concat!(env!("OUT_DIR"), "/agileplus.v1.rs"));

        // … otherwise use the hand-written stubs.
        #[cfg(agileplus_proto_stubs)]
        include!("stubs.rs");
    }
}
