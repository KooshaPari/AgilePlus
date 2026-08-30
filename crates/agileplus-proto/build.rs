// SPDX-License-Identifier: MIT OR Apache-2.0
mod protoc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Declare the custom cfg flag so rustc doesn't warn about it.
    println!("cargo::rustc-check-cfg=cfg(agileplus_proto_stubs)");

    // Skip proto compilation when protoc is unavailable (e.g. CI check-only runs).
    println!("cargo:rerun-if-env-changed=SKIP_PROTO_BUILD");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PATH");
    if std::env::var("SKIP_PROTO_BUILD").is_ok() || protoc::which_protoc().is_none() {
        if std::env::var("SKIP_PROTO_BUILD").is_ok() {
            println!("cargo:warning=SKIP_PROTO_BUILD set — skipping protoc codegen");
        } else {
            println!(
                "cargo:warning=protoc not found on PATH — using hand-written stubs. \
                Install protobuf-compiler or set PROTOC env var for a full build."
            );
        }
        // Signal to lib.rs that we are in stub mode.
        println!("cargo:rustc-cfg=agileplus_proto_stubs");
        return Ok(());
    }

    let protos = &[
        "../../agileplus-agents/proto/agileplus/v1/common.proto",
        "../../agileplus-agents/proto/agileplus/v1/core.proto",
        "../../agileplus-agents/proto/agileplus/v1/agents.proto",
        "../../agileplus-agents/proto/agileplus/v1/integrations.proto",
        "../../agileplus-agents/proto/agileplus/v1/work_items.proto",
    ];
    let includes = &["../../agileplus-agents/proto"];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}
