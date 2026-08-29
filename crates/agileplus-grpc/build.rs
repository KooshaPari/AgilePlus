fn main() {
    println!("cargo::rustc-check-cfg=cfg(agileplus_proto_stubs)");
    println!("cargo:rerun-if-env-changed=SKIP_PROTO_BUILD");
    if std::env::var_os("SKIP_PROTO_BUILD").is_some() {
        println!("cargo:rustc-cfg=agileplus_proto_stubs");
    }

    let protos = &[
        "../../proto/agileplus/v1/core.proto",
        "../../proto/agileplus/v1/agents.proto",
        "../../proto/agileplus/v1/common.proto",
        "../../proto/agileplus/v1/integrations.proto",
    ];

    let includes = &["../../proto"];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }
}
