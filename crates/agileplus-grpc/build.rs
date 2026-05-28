fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip proto compilation when protoc is unavailable (e.g. CI check-only runs).
    // Set SKIP_PROTO_BUILD=1 or ensure protoc is on PATH for a full codegen build.
    if std::env::var("SKIP_PROTO_BUILD").is_ok() {
        println!("cargo:warning=SKIP_PROTO_BUILD set — skipping protoc codegen");
        return Ok(());
    }

    // Also skip gracefully when protoc cannot be found on PATH.
    if which_protoc().is_none() {
        println!("cargo:warning=protoc not found on PATH — skipping proto codegen. \
            Install protobuf-compiler or set PROTOC env var for a full build.");
        return Ok(());
    }

    let protos = &[
        "../../proto/agileplus/v1/core.proto",
        "../../proto/agileplus/v1/agents.proto",
        "../../proto/agileplus/v1/common.proto",
        "../../proto/agileplus/v1/integrations.proto",
    ];

    let includes = &["../../proto"];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;

    for proto in protos {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}

fn which_protoc() -> Option<std::path::PathBuf> {
    // Honour explicit PROTOC env var first.
    if let Ok(p) = std::env::var("PROTOC") {
        let path = std::path::PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    // Fall back to PATH lookup.
    let name = if cfg!(windows) { "protoc.exe" } else { "protoc" };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            candidate.is_file().then_some(candidate)
        })
    })
}
