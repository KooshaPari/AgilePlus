fn main() {
    println!("cargo::rustc-check-cfg=cfg(agileplus_proto_stubs)");
    println!("cargo:rerun-if-env-changed=SKIP_PROTO_BUILD");
    println!("cargo:rerun-if-env-changed=PROTOC");
    let specified = std::env::var_os("PROTOC")
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists());
    let name = if cfg!(windows) {
        "protoc.exe"
    } else {
        "protoc"
    };
    let on_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(name))
            .find(|p| p.is_file())
    });
    if std::env::var_os("SKIP_PROTO_BUILD").is_some() || specified.or(on_path).is_none() {
        println!("cargo:rustc-cfg=agileplus_proto_stubs");
    }

    let protos = &[
        "../../proto/agileplus/v1/core.proto",
        "../../proto/agileplus/v1/agents.proto",
        "../../proto/agileplus/v1/common.proto",
        "../../proto/agileplus/v1/integrations.proto",
    ];

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }
}
