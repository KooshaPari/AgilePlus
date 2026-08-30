#[path = "../agileplus-proto/protoc.rs"]
mod protoc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rustc-check-cfg=cfg(agileplus_proto_stubs)");
    println!("cargo:rerun-if-env-changed=SKIP_PROTO_BUILD");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PATH");
    let use_proto_stubs = should_use_proto_stubs(
        std::env::var_os("SKIP_PROTO_BUILD").is_some(),
        protoc::which_protoc(),
    );
    if use_proto_stubs {
        println!("cargo:rustc-cfg=agileplus_proto_stubs");
    }

    let protos = &[
        "../../proto/agileplus/v1/core.proto",
        "../../proto/agileplus/v1/agents.proto",
        "../../proto/agileplus/v1/common.proto",
        "../../proto/agileplus/v1/integrations.proto",
    ];

    let includes = &["../../proto"];

    if !use_proto_stubs {
        tonic_prost_build::configure()
            .build_server(true)
            .build_client(true)
            .compile_protos(protos, includes)?;
    }
    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }
    Ok(())
}

pub fn should_use_proto_stubs(skip_proto_build: bool, protoc: Option<std::path::PathBuf>) -> bool {
    skip_proto_build || protoc.is_none()
}
