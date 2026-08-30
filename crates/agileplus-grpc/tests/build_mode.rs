#[allow(dead_code)]
mod build_script {
    include!("../build.rs");
}

#[allow(dead_code)]
mod shared_protoc {
    include!("../../agileplus-proto/protoc.rs");
}

#[test]
fn missing_protoc_selects_stub_mode() {
    assert!(build_script::should_use_proto_stubs(false, None));
}

#[test]
fn explicit_skip_selects_stub_mode_even_with_protoc() {
    assert!(build_script::should_use_proto_stubs(
        true,
        Some(std::path::PathBuf::from("/usr/bin/protoc"))
    ));
}

#[test]
fn available_protoc_keeps_generated_mode() {
    assert!(!build_script::should_use_proto_stubs(
        false,
        Some(std::path::PathBuf::from("/usr/bin/protoc"))
    ));
}

#[cfg(unix)]
#[test]
fn candidates_must_execute_version_successfully_in_both_build_scripts() {
    use std::os::unix::fs::PermissionsExt;

    let test_dir = std::env::temp_dir().join(format!("agileplus-protoc-{}", std::process::id()));
    std::fs::create_dir_all(&test_dir).unwrap();
    let invalid = test_dir.join("invalid-protoc");
    std::fs::write(&invalid, "not executable").unwrap();
    assert!(!shared_protoc::protoc_is_usable(&invalid));

    let failing = test_dir.join("failing-protoc");
    std::fs::write(&failing, "#!/bin/sh\nexit 1\n").unwrap();
    let mut permissions = std::fs::metadata(&failing).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&failing, permissions).unwrap();
    assert!(!shared_protoc::protoc_is_usable(&failing));

    let valid = test_dir.join("valid-protoc");
    std::fs::write(&valid, "#!/bin/sh\nexit 0\n").unwrap();
    let mut permissions = std::fs::metadata(&valid).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&valid, permissions).unwrap();
    assert!(shared_protoc::protoc_is_usable(&valid));

    std::fs::remove_dir_all(test_dir).unwrap();
}
