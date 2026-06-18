use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct TraceEntry {
    pub fr_id: String,
    pub spec_slug: String,
    pub spec_anchor: String,
    pub docs_pages: Vec<String>,
    pub tests: Vec<String>,
    pub code_modules: Vec<String>,
    pub journeys: Vec<String>,
}

#[derive(Debug)]
pub struct TraceValidation {
    pub root: PathBuf,
    pub traces: Vec<TraceEntry>,
    pub missing_functional_requirements: Vec<String>,
}

impl TraceValidation {
    pub fn trace_count(&self) -> usize {
        self.traces.len()
    }

    pub fn referenced_path_count(&self) -> usize {
        self.traces
            .iter()
            .map(|trace| {
                trace.docs_pages.len()
                    + trace.tests.len()
                    + trace.code_modules.len()
                    + trace.journeys.len()
            })
            .sum()
    }
}

pub fn validate_trace_path(path: impl AsRef<Path>) -> Result<TraceValidation> {
    let root = path.as_ref();
    let traces_dir = if root.file_name().is_some_and(|name| name == "traces") {
        root.to_path_buf()
    } else {
        root.join("traces")
    };

    if !traces_dir.is_dir() {
        return Err(anyhow!(
            "trace directory not found: {}",
            traces_dir.display()
        ));
    }

    let repo_root = traces_dir.parent().unwrap_or(root);
    let mut traces = Vec::new();
    let mut errors = Vec::new();

    for entry in fs::read_dir(&traces_dir)
        .with_context(|| format!("failed to read {}", traces_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        match read_trace(&path) {
            Ok(trace) => {
                validate_trace_paths(repo_root, &trace, &mut errors);
                traces.push(trace);
            }
            Err(err) => errors.push(format!("{}: {err}", path.display())),
        }
    }

    traces.sort_by(|a, b| a.fr_id.cmp(&b.fr_id));

    let missing_functional_requirements = missing_requirements(repo_root, &traces)?;
    for fr_id in &missing_functional_requirements {
        errors.push(format!("{fr_id}: missing traces/{fr_id}.json"));
    }

    if !errors.is_empty() {
        return Err(anyhow!(errors.join("\n")));
    }

    Ok(TraceValidation {
        root: repo_root.to_path_buf(),
        traces,
        missing_functional_requirements,
    })
}

fn read_trace(path: &Path) -> Result<TraceEntry> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read trace {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&source)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;

    for field in ["fr_id", "spec_slug", "spec_anchor"] {
        if !value.get(field).is_some_and(|field| field.is_string()) {
            return Err(anyhow!("missing or invalid string field {field}"));
        }
    }

    for field in ["docs_pages", "tests", "code_modules", "journeys"] {
        if !value.get(field).is_some_and(|field| field.is_array()) {
            return Err(anyhow!("missing or invalid list field {field}"));
        }
    }

    serde_json::from_value(value).context("trace schema mismatch")
}

fn validate_trace_paths(repo_root: &Path, trace: &TraceEntry, errors: &mut Vec<String>) {
    let paths = trace
        .docs_pages
        .iter()
        .chain(trace.code_modules.iter())
        .chain(trace.journeys.iter());

    for path in paths {
        if path.starts_with('/') || path.starts_with("~/") || path.contains('\\') {
            errors.push(format!("{}: malformed path {path}", trace.fr_id));
            continue;
        }

        let normalized = path.strip_prefix("AgilePlus/").unwrap_or(path);
        if !repo_root.join(normalized).exists() {
            errors.push(format!("{}: dangling path {path}", trace.fr_id));
        }
    }
}

fn missing_requirements(repo_root: &Path, traces: &[TraceEntry]) -> Result<Vec<String>> {
    let requirements = repo_root.join("FUNCTIONAL_REQUIREMENTS.md");
    if !requirements.exists() {
        return Ok(Vec::new());
    }

    let source = fs::read_to_string(&requirements)
        .with_context(|| format!("failed to read {}", requirements.display()))?;
    let existing: std::collections::HashSet<&str> =
        traces.iter().map(|trace| trace.fr_id.as_str()).collect();
    let mut missing = Vec::new();

    for token in source.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-')) {
        if token.starts_with("FR-") && token.len() > 3 && !existing.contains(token) {
            missing.push(token.to_string());
        }
    }

    missing.sort();
    missing.dedup();
    Ok(missing)
}
