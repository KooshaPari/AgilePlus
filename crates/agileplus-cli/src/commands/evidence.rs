//! Attach a local, content-bound result to a planned work package.

use std::path::{Path, PathBuf};

use agileplus_domain::domain::governance::{Evidence, EvidenceType};
use agileplus_domain::ports::StoragePort;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::Subcommand;
use sha2::{Digest, Sha256};

#[derive(Debug, clap::Args)]
pub struct EvidenceArgs {
    #[command(subcommand)]
    pub command: EvidenceCommand,
}

#[derive(Debug, Subcommand)]
pub enum EvidenceCommand {
    /// Attach a test, CI, review, or other result file to a work package.
    Attach(AttachArgs),
    /// Execute a local test program and attach its captured result.
    RunTest(RunTestArgs),
}

#[derive(Debug, clap::Args)]
pub struct RunTestArgs {
    #[arg(long)]
    pub feature: String,
    #[arg(long)]
    pub wp: String,
    #[arg(long)]
    pub fr: String,
    /// File to receive the command, exit status, stdout, and stderr.
    #[arg(long)]
    pub artifact: PathBuf,
    /// Executable and arguments after `--`; no shell interpolation occurs.
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

#[derive(Debug, clap::Args)]
pub struct AttachArgs {
    #[arg(long)]
    pub feature: String,
    /// Work-package sequence (WP01) or database ID.
    #[arg(long)]
    pub wp: String,
    #[arg(long)]
    pub fr: String,
    #[arg(long = "type")]
    pub evidence_type: String,
    #[arg(long)]
    pub artifact: PathBuf,
    /// Recorded outcome: pass or fail. A failed result remains visible but cannot satisfy validation.
    #[arg(long)]
    pub result: String,
}

pub fn digest_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading evidence artifact {}", path.display()))?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct RunProvenance {
    run_id: String,
    wp_id: i64,
    output_sha256: String,
}

pub fn matches_requirement(raw: &str, evidence: &Evidence) -> bool {
    let (fr, expected_type) = raw
        .split_once(':')
        .map_or((raw, None), |(fr, ty)| (fr, Some(ty)));
    evidence.fr_id == fr
        && expected_type.is_none_or(|ty| evidence.evidence_type.as_str() == ty)
        && valid_artifact(evidence)
}

pub fn valid_artifact(evidence: &Evidence) -> bool {
    let Some(metadata) = &evidence.metadata else {
        return false;
    };
    if metadata.get("result").and_then(|v| v.as_str()) != Some("pass") {
        return false;
    }
    let Some(expected) = metadata.get("sha256").and_then(|v| v.as_str()) else {
        return false;
    };
    if expected.len() != 64 {
        return false;
    }
    let path = Path::new(&evidence.artifact_path);
    if !path.is_file() || !digest_file(path).is_ok_and(|actual| actual == expected) {
        return false;
    }
    if evidence.evidence_type == EvidenceType::TestResult {
        if metadata.get("origin").and_then(|v| v.as_str()) != Some("agileplus-local-exec") {
            return false;
        }
        let Ok(bytes) = std::fs::read(path) else {
            return false;
        };
        let Ok(receipt) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            return false;
        };
        let Some(run_id) = metadata.get("run_id").and_then(|v| v.as_str()) else {
            return false;
        };
        let Some(output_sha256) = metadata.get("output_sha256").and_then(|v| v.as_str()) else {
            return false;
        };
        let Some(stdout) = receipt.get("stdout").and_then(|v| v.as_str()) else {
            return false;
        };
        let Some(stderr) = receipt.get("stderr").and_then(|v| v.as_str()) else {
            return false;
        };
        let actual_output_sha256 = digest_bytes(format!("{stdout}\0{stderr}").as_bytes());
        return uuid::Uuid::parse_str(run_id).is_ok()
            && receipt.get("runner").and_then(|v| v.as_str()) == Some("agileplus-local-exec")
            && receipt.get("success").and_then(|v| v.as_bool()) == Some(true)
            && receipt.get("exit_code").and_then(|v| v.as_i64()) == Some(0)
            && receipt.get("run_id").and_then(|v| v.as_str()) == Some(run_id)
            && receipt.get("wp_id").and_then(|v| v.as_i64()) == Some(evidence.wp_id)
            && metadata.get("wp_id").and_then(|v| v.as_i64()) == Some(evidence.wp_id)
            && receipt.get("fr_id").and_then(|v| v.as_str()) == Some(evidence.fr_id.as_str())
            && receipt.get("output_sha256").and_then(|v| v.as_str()) == Some(output_sha256)
            && actual_output_sha256 == output_sha256;
    }
    true
}

pub async fn run_evidence<S: StoragePort>(args: EvidenceArgs, storage: &S) -> Result<()> {
    match args.command {
        EvidenceCommand::Attach(args) => run_attach(args, storage, None).await,
        EvidenceCommand::RunTest(args) => run_test(args, storage).await,
    }
}

async fn run_test<S: StoragePort>(args: RunTestArgs, storage: &S) -> Result<()> {
    let feature = storage
        .get_feature_by_slug(&args.feature)
        .await
        .context("loading feature")?
        .ok_or_else(|| anyhow::anyhow!("Feature '{}' not found", args.feature))?;
    let wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("loading work packages")?;
    let wp_ref = args.wp.to_ascii_uppercase();
    let wp = wps
        .iter()
        .find(|wp| {
            wp_ref
                .strip_prefix("WP")
                .and_then(|n| n.parse::<i32>().ok())
                == Some(wp.sequence)
                || args.wp.parse::<i64>().ok() == Some(wp.id)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("WP '{}' not found for feature '{}'", args.wp, args.feature)
        })?;
    if !wp.acceptance_criteria.lines().any(|line| {
        line.trim_start_matches([' ', '-'])
            .trim_start()
            .split_whitespace()
            .next()
            == Some(args.fr.as_str())
    }) {
        bail!(
            "Requirement '{}' is not assigned to WP{:02}",
            args.fr,
            wp.sequence
        );
    }
    let wp_id = wp.id;
    let run_id = uuid::Uuid::new_v4().to_string();
    let program = &args.command[0];
    let output = std::process::Command::new(program)
        .args(&args.command[1..])
        .output()
        .with_context(|| format!("executing local test program {program}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let output_sha256 = digest_bytes(format!("{stdout}\0{stderr}").as_bytes());
    let receipt = serde_json::json!({
        "runner": "agileplus-local-exec",
        "run_id": run_id,
        "wp_id": wp_id,
        "fr_id": args.fr,
        "command": args.command,
        "exit_code": output.status.code(),
        "success": output.status.success(),
        "stdout": stdout,
        "stderr": stderr,
        "output_sha256": output_sha256,
    });
    if let Some(parent) = args.artifact.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating receipt directory {}", parent.display()))?;
    }
    std::fs::write(&args.artifact, serde_json::to_vec_pretty(&receipt)?)
        .with_context(|| format!("writing test receipt {}", args.artifact.display()))?;
    run_attach(
        AttachArgs {
            feature: args.feature,
            wp: args.wp,
            fr: args.fr,
            evidence_type: "test_result".into(),
            artifact: args.artifact,
            result: if output.status.success() {
                "pass"
            } else {
                "fail"
            }
            .into(),
        },
        storage,
        Some(RunProvenance {
            run_id,
            wp_id,
            output_sha256,
        }),
    )
    .await?;
    if !output.status.success() {
        bail!(
            "Local test exited with {:?}; failed evidence was attached",
            output.status.code()
        );
    }
    Ok(())
}

async fn run_attach<S: StoragePort>(
    args: AttachArgs,
    storage: &S,
    provenance: Option<RunProvenance>,
) -> Result<()> {
    let feature = storage
        .get_feature_by_slug(&args.feature)
        .await
        .context("loading feature")?
        .ok_or_else(|| anyhow::anyhow!("Feature '{}' not found", args.feature))?;
    let wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("loading work packages")?;
    let wp_ref = args.wp.to_ascii_uppercase();
    let wp = wps
        .iter()
        .find(|wp| {
            wp_ref
                .strip_prefix("WP")
                .and_then(|n| n.parse::<i32>().ok())
                == Some(wp.sequence)
                || args.wp.parse::<i64>().ok() == Some(wp.id)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("WP '{}' not found for feature '{}'", args.wp, args.feature)
        })?;
    if !args.fr.starts_with("FR-") {
        bail!("Evidence requirement must start with FR-");
    }
    if args.fr != "FR-CI"
        && args.fr != "FR-REVIEW"
        && !wp.acceptance_criteria.lines().any(|line| {
            line.trim_start_matches([' ', '-'])
                .trim_start()
                .split_whitespace()
                .next()
                == Some(args.fr.as_str())
        })
    {
        bail!(
            "Requirement '{}' is not assigned to WP{:02}",
            args.fr,
            wp.sequence
        );
    }
    let evidence_type = match args.evidence_type.as_str() {
        "test_result" => EvidenceType::TestResult,
        "ci_output" => EvidenceType::CiOutput,
        "review_approval" => EvidenceType::ReviewApproval,
        "security_scan" => EvidenceType::SecurityScan,
        "lint_result" => EvidenceType::LintResult,
        "manual_attestation" => EvidenceType::ManualAttestation,
        other => bail!("Unsupported evidence type '{other}'"),
    };
    if evidence_type == EvidenceType::TestResult && provenance.is_none() {
        bail!("Passing FR test evidence must be captured with `agileplus evidence run-test`");
    }
    if provenance.as_ref().is_some_and(|run| run.wp_id != wp.id) {
        bail!("Executed test receipt is bound to another work package");
    }
    if args.result != "pass" && args.result != "fail" {
        bail!("--result must be pass or fail");
    }
    let artifact = args
        .artifact
        .canonicalize()
        .with_context(|| format!("locating artifact {}", args.artifact.display()))?;
    if !artifact.is_file() {
        bail!("Evidence artifact must be a regular file");
    }
    let digest = digest_file(&artifact)?;
    let mut metadata = serde_json::json!({
        "result": args.result,
        "sha256": digest,
        "origin": if provenance.is_some() { "agileplus-local-exec" } else { "operator-declared" },
    });
    if let Some(run) = provenance {
        metadata["run_id"] = run.run_id.into();
        metadata["wp_id"] = run.wp_id.into();
        metadata["output_sha256"] = run.output_sha256.into();
    }
    let evidence = Evidence {
        id: 0,
        wp_id: wp.id,
        fr_id: args.fr,
        evidence_type,
        artifact_path: artifact.display().to_string(),
        metadata: Some(metadata),
        created_at: Utc::now(),
    };
    let id = storage
        .create_evidence(&evidence)
        .await
        .context("attaching evidence")?;
    println!(
        "Attached evidence #{id} to WP{:02}: {} {} ({})",
        wp.sequence,
        evidence.fr_id,
        evidence_type.as_str(),
        evidence.artifact_path
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locally_executed_receipt_cannot_claim_pass_when_file_records_failure() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let run_id = uuid::Uuid::new_v4().to_string();
        let output_sha256 = digest_bytes(b"test output\0");
        let mut receipt = serde_json::json!({
            "runner":"agileplus-local-exec", "run_id":run_id, "wp_id":1,
            "fr_id":"FR-001", "success":false, "exit_code":2,
            "stdout":"test output", "stderr":"", "output_sha256":output_sha256,
        });
        std::fs::write(file.path(), serde_json::to_vec(&receipt).unwrap()).unwrap();
        let digest = digest_file(file.path()).unwrap();
        let mut item = Evidence {
            id: 0,
            wp_id: 1,
            fr_id: "FR-001".into(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: file.path().display().to_string(),
            metadata: Some(serde_json::json!({
                "result":"pass", "sha256":digest, "origin":"agileplus-local-exec",
                "run_id":run_id, "wp_id":1, "output_sha256":output_sha256,
            })),
            created_at: Utc::now(),
        };
        assert!(!valid_artifact(&item));
        receipt["success"] = true.into();
        receipt["exit_code"] = 0.into();
        std::fs::write(file.path(), serde_json::to_vec(&receipt).unwrap()).unwrap();
        assert!(
            !valid_artifact(&item),
            "changed bytes must fail the stored digest"
        );
        item.metadata.as_mut().unwrap()["sha256"] = digest_file(file.path()).unwrap().into();
        assert!(valid_artifact(&item));
        item.metadata.as_mut().unwrap()["origin"] = "operator-declared".into();
        assert!(
            !valid_artifact(&item),
            "plain attachments cannot satisfy FR test results"
        );
    }
}
