// SPDX-License-Identifier: MIT OR Apache-2.0
//! Declares the intentional `integration` cfg used to gate dormant live-stack
//! integration tests under `tests/`.
//!
//! Those tests require a running `process-compose` stack and therefore remain
//! disabled by default. They cannot be promoted to a real Cargo feature yet
//! (that would require adding `anyhow`/`tracing-subscriber` dev-dependencies
//! and would make `--all-features` builds compile them). Declaring the cfg
//! here keeps `-D warnings` builds clean without fabricating a broken feature,
//! matching the existing pattern in `crates/agileplus-grpc/build.rs`.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"integration\"))");
}
