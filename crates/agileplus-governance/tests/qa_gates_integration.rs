use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn governance_qa_gates_accept_valid_fixture() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = match manifest_dir.parent().and_then(Path::parent) {
        Some(path) => path,
        None => {
            assert!(false, "crate should live under crates/");
            return;
        }
    };
    let temp = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(err) => {
            assert!(false, "tempdir failed: {err}");
            return;
        }
    };

    assert!(fs::write(
        temp.path().join("SPEC.md"),
        "# Spec\n\nFR-001: shipped behavior\nNFR-002: operational guardrail\n",
    )
    .is_ok());
    assert!(fs::write(
        temp.path().join("lib.rs"),
        "pub fn ok() -> Option<u8> { Some(1) }\n",
    )
    .is_ok());

    run_gate(
        repo_root.join("scripts/qa-gates/antipattern.sh"),
        temp.path(),
        &[],
    );
    run_gate(
        repo_root.join("scripts/qa-gates/spec-verify.sh"),
        temp.path(),
        &[("PR_BODY", "Implements FR-001 and NFR-002.")],
    );
    run_gate(
        repo_root.join("scripts/qa-gates/governance-spec-first.sh"),
        temp.path(),
        &[(
            "CHANGED_FILES",
            "CHANGELOG.md\ndocs/adr/0001-governance.md\ndocs/qa-matrix.md\n",
        )],
    );
}

fn run_gate(path: impl AsRef<Path>, workdir: &Path, envs: &[(&str, &str)]) {
    let mut command = Command::new("bash");
    command.arg(path.as_ref()).current_dir(workdir);
    for (key, value) in envs {
        command.env(key, value);
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            assert!(false, "gate failed to run: {err}");
            return;
        }
    };
    assert!(
        output.status.success(),
        "gate {:?} failed\nstdout:\n{}\nstderr:\n{}",
        path.as_ref(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
