// Proto compilation is handled by agileplus-proto crate.
// This build.rs propagates the agileplus_proto_stubs cfg flag so that
// server/mod.rs can gate the transport-dependent start_server function.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(agileplus_proto_stubs)");

    // Mirror the protoc availability check: if protoc is absent we're in stub mode.
    if std::env::var("SKIP_PROTO_BUILD").is_ok() || which_protoc().is_none() {
        println!("cargo:rustc-cfg=agileplus_proto_stubs");
    }
}

fn which_protoc() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("PROTOC") {
        let path = std::path::PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    let name = if cfg!(windows) {
        "protoc.exe"
    } else {
        "protoc"
    };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            candidate.is_file().then_some(candidate)
        })
    })
}
