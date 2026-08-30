use std::path::{Path, PathBuf};

pub fn which_protoc() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PROTOC").map(PathBuf::from)
        && protoc_is_usable(&path)
    {
        return Some(path);
    }

    let name = if cfg!(windows) {
        "protoc.exe"
    } else {
        "protoc"
    };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            protoc_is_usable(&candidate).then_some(candidate)
        })
    })
}

pub fn protoc_is_usable(candidate: &Path) -> bool {
    std::process::Command::new(candidate)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
