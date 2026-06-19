//! agileplus-cli worklog subcommand.
//!
//! Provides `agileplus worklog {validate,convert,schema,list}` for
//! managing the canonical 8-field worklog schema across repos.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

const CANONICAL_FIELDS: &[&str] = &[
    "status",
    "task_id",
    "agent_id",
    "files_changed",
    "commit_sha",
    "verification_result",
    "started_at",
    "completed_at",
];

/// Top-level worklog command.
#[derive(Debug, Args)]
pub struct WorklogArgs {
    /// Working directory (defaults to current directory).
    #[arg(long, short = 'd', default_value = ".")]
    pub dir: PathBuf,

    #[command(subcommand)]
    pub action: WorklogAction,
}

#[derive(Debug, Subcommand)]
pub enum WorklogAction {
    /// Validate worklog JSON files in DIR against the canonical schema.
    Validate,
    /// Convert worklog JSON files to canonical schema. Writes
    /// `worklog-*-canonical.json` next to each source file unless
    /// `--in-place` is passed.
    Convert {
        /// Replace original files with converted content.
        #[arg(long)]
        in_place: bool,
    },
    /// Print the canonical 8-field schema.
    Schema,
    /// List all worklog files (raw + canonical) in DIR.
    List,
}

#[derive(Debug, Serialize, Deserialize)]
struct CanonicalWorklog {
    status: String,
    task_id: String,
    agent_id: String,
    files_changed: Vec<String>,
    commit_sha: String,
    verification_result: VerificationResult,
    started_at: Option<String>,
    completed_at: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VerificationResult {
    #[serde(default)]
    status: String,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    notes: String,
}

pub fn run(args: &WorklogArgs) -> anyhow::Result<()> {
    match &args.action {
        WorklogAction::Schema => {
            println!("Canonical worklog schema (8 fields):");
            for (i, f) in CANONICAL_FIELDS.iter().enumerate() {
                println!("  {}. {}", i + 1, f);
            }
            Ok(())
        }
        WorklogAction::List => list(&args.dir),
        WorklogAction::Validate => validate(&args.dir),
        WorklogAction::Convert { in_place } => convert(&args.dir, *in_place),
    }
}

fn list(dir: &Path) -> anyhow::Result<()> {
    let raw = find_worklogs(dir, false)?;
    let canonical = find_worklogs(dir, true)?;
    println!("Raw worklogs in {}:", dir.display());
    for p in raw {
        println!("  {}", p.display());
    }
    println!("\nCanonical worklogs:");
    for p in canonical {
        println!("  {}", p.display());
    }
    Ok(())
}

fn validate(dir: &Path) -> anyhow::Result<()> {
    let files = find_worklogs(dir, false)?;
    println!(
        "Validating {} worklog(s) in {} against canonical schema...",
        files.len(),
        dir.display()
    );
    let mut ok = 0;
    let mut err = 0;
    for path in &files {
        match std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        {
            Some(v) => {
                if let Some(obj) = v.as_object() {
                    let missing: Vec<&&str> = CANONICAL_FIELDS
                        .iter()
                        .filter(|f| !obj.contains_key(**f))
                        .collect();
                    if missing.is_empty() {
                        println!("OK   {}", path.display());
                        ok += 1;
                    } else {
                        println!(
                            "FAIL {}: missing {}",
                            path.display(),
                            missing
                                .iter()
                                .map(|s| **s)
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        err += 1;
                    }
                } else {
                    println!("FAIL {}: not a JSON object", path.display());
                    err += 1;
                }
            }
            None => {
                println!("FAIL {}: invalid JSON", path.display());
                err += 1;
            }
        }
    }
    println!("\nResult: {} OK, {} FAIL", ok, err);
    if err > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn convert(dir: &Path, in_place: bool) -> anyhow::Result<()> {
    let files = find_worklogs(dir, false)?;
    println!(
        "Converting {} worklog(s) in {} (in_place={})",
        files.len(),
        dir.display(),
        in_place
    );
    for path in &files {
        let content = std::fs::read_to_string(path)?;
        let raw: serde_json::Value = serde_json::from_str(&content)?;
        let canonical = to_canonical(&raw);
        let pretty = serde_json::to_string_pretty(&canonical)?;
        let target = if in_place {
            path.clone()
        } else {
            path_with_suffix(path, "-canonical.json")
        };
        std::fs::write(&target, pretty + "\n")?;
        println!("OK   {} -> {}", path.display(), target.display());
    }
    Ok(())
}

fn to_canonical(raw: &serde_json::Value) -> CanonicalWorklog {
    let obj = raw.as_object();
    let get_str = |k: &str| {
        obj.and_then(|o| o.get(k))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    let get_arr = |k: &str| {
        obj.and_then(|o| o.get(k))
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };
    let verification = obj
        .and_then(|o| o.get("verification_result").cloned().or_else(|| o.get("verification").cloned()))
        .map(|v| {
            let status = v
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("not_run")
                .to_string();
            let commands = v
                .get("commands")
                .and_then(|c| c.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let notes = v
                .get("notes")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            VerificationResult {
                status,
                commands,
                notes,
            }
        })
        .unwrap_or_default();

    CanonicalWorklog {
        status: get_str("status").unwrap_or_else(|| "completed".to_string()),
        task_id: get_str("task_id")
            .or_else(|| get_str("task"))
            .unwrap_or_else(|| "unknown".to_string()),
        agent_id: get_str("agent_id").unwrap_or_else(|| "codex-exec".to_string()),
        files_changed: {
            let v: Vec<String> = get_arr("files_changed");
            if v.is_empty() {
                get_arr("files")
            } else {
                v
            }
        },
        commit_sha: get_str("commit_sha")
            .or_else(|| get_str("branch"))
            .or_else(|| get_str("merge_commit"))
            .unwrap_or_else(|| "unknown".to_string()),
        verification_result: verification,
        started_at: get_str("started_at"),
        completed_at: get_str("completed_at").or_else(|| get_str("date")),
    }
}

fn find_worklogs(dir: &Path, canonical_only: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let is_worklog = name_str.starts_with("worklog-") && name_str.ends_with(".json");
        let is_canonical = name_str.contains("-canonical.json");
        if is_worklog && (canonical_only == is_canonical || (!canonical_only && is_canonical)) {
            out.push(entry.path());
        }
    }
    out.sort();
    Ok(out)
}

fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_suffix(".json") {
        PathBuf::from(format!("{stripped}{suffix}"))
    } else {
        path.with_extension(suffix)
    }
}
