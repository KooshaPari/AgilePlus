#[path = "../crates/agileplus-proto/protoc.rs"]
mod protoc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rustc-check-cfg=cfg(agileplus_proto_stubs)");
    println!("cargo:rerun-if-env-changed=SKIP_PROTO_BUILD");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PATH");

    if std::env::var_os("SKIP_PROTO_BUILD").is_some() || protoc::which_protoc().is_none() {
        println!("cargo:rustc-cfg=agileplus_proto_stubs");
        return Ok(());
    }

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../proto/agileplus/v1/common.proto",
                "../proto/agileplus/v1/core.proto",
                "../proto/agileplus/v1/agents.proto",
                "../proto/agileplus/v1/integrations.proto",
            ],
            &["../proto"],
        )?;
    Ok(())
}
