//! Public CLI acceptance: persisted planning and executed evidence must not
//! manufacture CI or review approval. Uses an isolated repository, never agents.

#![cfg(feature = "full-deps")]

use std::path::Path;
use std::process::{Command, Output};

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;

const FEATURE: &str = "public-acceptance";

fn cli(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agileplus"))
        .arg("--repo")
        .arg(repo)
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run installed CLI")
}

fn success(output: Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn database(repo: &Path) -> Connection {
    Connection::open_with_flags(
        repo.join(".agileplus/agileplus.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("open persisted state read-only")
}

fn feature_state(repo: &Path) -> String {
    database(repo)
        .query_row(
            "SELECT state FROM features WHERE slug=?1",
            [FEATURE],
            |row| row.get(0),
        )
        .unwrap()
}

#[test]
fn public_cli_plans_exact_requirements_and_blocks_unreviewed_work() {
    let repo = tempfile::TempDir::new().unwrap();
    success(
        Command::new("git")
            .args(["init", "--initial-branch=main"])
            .arg(repo.path())
            .output()
            .unwrap(),
    );
    let spec = repo.path().join("input.md");
    std::fs::write(
        &spec,
        "# Public acceptance fixture\n\n## Scenarios\nGiven FR-001 is referenced here before its definition\n\n## Functional Requirements\n- **FR-001**: List the implementing feature from repository-local state.\n- **FR-002**: Preserve feature state when validation evidence is missing.\n- **FR-003**: Refuse work-package review without CI evidence.\n",
    )
    .unwrap();
    success(cli(
        repo.path(),
        &[
            "specify",
            "--feature",
            FEATURE,
            "--from-file",
            spec.to_str().unwrap(),
        ],
    ));
    success(cli(repo.path(), &["research", "--feature", FEATURE]));
    success(cli(repo.path(), &["plan", "--feature", FEATURE]));
    let db = database(repo.path());
    let (wp_id, criteria): (i64, String) = db
        .query_row(
            "SELECT id, acceptance_criteria FROM work_packages",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let count: i64 = db
        .query_row("SELECT count(*) FROM work_packages", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        criteria,
        "- FR-001 -- List the implementing feature from repository-local state.\n- FR-002 -- Preserve feature state when validation evidence is missing.\n- FR-003 -- Refuse work-package review without CI evidence."
    );
    drop(db);
    success(cli(
        repo.path(),
        &["implement", "--feature", FEATURE, "--manual"],
    ));
    assert_eq!(feature_state(repo.path()), "implementing");

    // Execute an actual public list command, independently asserting its output.
    // This is evidence for the fixture's FR-001 only, not product-wide acceptance.
    let receipt = repo.path().join("receipts/list.json");
    let probe = "import subprocess,sys; r=subprocess.run([sys.argv[1],'--repo',sys.argv[2],'list','--state','implementing'],capture_output=True,text=True); print(r.stdout,end=''); print(r.stderr,end='',file=sys.stderr); assert r.returncode==0 and sys.argv[3] in r.stdout";
    success(cli(
        repo.path(),
        &[
            "evidence",
            "run-test",
            "--feature",
            FEATURE,
            "--wp",
            "WP01",
            "--fr",
            "FR-001",
            "--artifact",
            receipt.to_str().unwrap(),
            "--",
            "python3",
            "-c",
            probe,
            env!("CARGO_BIN_EXE_agileplus"),
            repo.path().to_str().unwrap(),
            FEATURE,
        ],
    ));
    let result: Value = serde_json::from_slice(&std::fs::read(&receipt).unwrap()).unwrap();
    assert_eq!(result["wp_id"], wp_id);
    assert_eq!(result["fr_id"], "FR-001");
    assert_eq!(result["exit_code"], 0);
    assert_eq!(result["success"], true);

    let report = repo.path().join("validation.json");
    let validate = || {
        cli(
            repo.path(),
            &[
                "validate",
                "--feature",
                FEATURE,
                "--format",
                "json",
                "--output",
                report.to_str().unwrap(),
            ],
        )
    };
    assert!(!validate().status.success());
    let report_value: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert_eq!(report_value["overall_pass"], false);
    let evidence = report_value["evidence_results"].as_array().unwrap();
    assert!(
        evidence
            .iter()
            .any(|row| row["fr_id"] == "FR-001" && row["found"] == true)
    );
    for fr in ["FR-CI", "FR-REVIEW", "FR-002", "FR-003"] {
        assert!(
            evidence
                .iter()
                .any(|row| row["fr_id"] == fr && row["found"] == false)
        );
    }
    assert_eq!(feature_state(repo.path()), "implementing");

    let wp_id_arg = wp_id.to_string();
    success(cli(
        repo.path(),
        &["mvp", "transition", "--wp", &wp_id_arg, "--to", "doing"],
    ));
    let denied = cli(
        repo.path(),
        &["mvp", "transition", "--wp", &wp_id_arg, "--to", "review"],
    );
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("FR-CI"));
    let state: String = database(repo.path())
        .query_row(
            "SELECT state FROM work_packages WHERE id=?1",
            [wp_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(state, "doing");

    // An altered receipt cannot continue to satisfy the stored content digest.
    std::fs::write(&receipt, b"{}").unwrap();
    assert!(!validate().status.success());
    let altered: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert!(
        altered["evidence_results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["fr_id"] == "FR-001" && row["found"] == false)
    );
    assert_eq!(feature_state(repo.path()), "implementing");
    assert!(
        !cli(repo.path(), &["ship", "--feature", FEATURE, "--dry-run"])
            .status
            .success()
    );
}
