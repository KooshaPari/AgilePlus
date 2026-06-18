use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("failed to walk {path}: {source}")]
    Walk {
        path: PathBuf,
        source: walkdir::Error,
    },
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceDocument {
    pub id: String,
    pub kind: TraceDocumentKind,
    pub refs: Vec<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceDocumentKind {
    FunctionalRequirement,
    NonFunctionalRequirement,
    Test,
    Code,
}

#[derive(Debug, Deserialize)]
struct RawTraceDocument {
    id: Option<String>,
    #[serde(default, alias = "kind")]
    r#type: Option<String>,
    #[serde(default, alias = "trace", alias = "traces", alias = "references")]
    refs: Vec<String>,
}

pub fn load_trace_documents(root: &Path) -> Result<Vec<TraceDocument>, LoadError> {
    let mut documents = Vec::new();

    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|source| LoadError::Walk {
            path: root.to_path_buf(),
            source,
        })?;
        if !entry.file_type().is_file() || !is_trace_candidate(entry.path()) {
            continue;
        }

        if let Some(document) = load_trace_document(root, entry.path())? {
            documents.push(document);
        }
    }

    documents.sort_by(|left, right| left.id.cmp(&right.id).then(left.path.cmp(&right.path)));
    Ok(documents)
}

fn load_trace_document(root: &Path, path: &Path) -> Result<Option<TraceDocument>, LoadError> {
    let body = fs::read_to_string(path).map_err(|source| LoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    let raw = parse_raw_document(path, &body)?;
    let Some(id) = raw.id.filter(|id| !id.trim().is_empty()) else {
        return Ok(None);
    };

    let relative = path.strip_prefix(root).unwrap_or(path);
    let Some(kind) = raw
        .r#type
        .as_deref()
        .and_then(parse_kind)
        .or_else(|| infer_kind(&id, relative))
    else {
        return Ok(None);
    };

    Ok(Some(TraceDocument {
        id,
        kind,
        refs: raw.refs,
        path: relative.to_path_buf(),
    }))
}

fn parse_raw_document(path: &Path, body: &str) -> Result<RawTraceDocument, LoadError> {
    match extension(path) {
        Some("json") => serde_json::from_str(body).map_err(|source| LoadError::Parse {
            path: path.to_path_buf(),
            message: source.to_string(),
        }),
        Some("yaml" | "yml") => serde_yaml::from_str(body).map_err(|source| LoadError::Parse {
            path: path.to_path_buf(),
            message: source.to_string(),
        }),
        _ => unreachable!("candidate extensions are filtered before parsing"),
    }
}

fn is_trace_candidate(path: &Path) -> bool {
    matches!(extension(path), Some("json" | "yaml" | "yml"))
}

fn extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|ext| ext.to_str())
}

fn parse_kind(kind: &str) -> Option<TraceDocumentKind> {
    match kind.to_ascii_lowercase().as_str() {
        "fr" | "functional_requirement" | "functional-requirement" => {
            Some(TraceDocumentKind::FunctionalRequirement)
        }
        "nfr" | "non_functional_requirement" | "non-functional-requirement" => {
            Some(TraceDocumentKind::NonFunctionalRequirement)
        }
        "test" | "tests" => Some(TraceDocumentKind::Test),
        "code" | "source" | "implementation" => Some(TraceDocumentKind::Code),
        _ => None,
    }
}

fn infer_kind(id: &str, path: &Path) -> Option<TraceDocumentKind> {
    let text = format!("{} {}", id, path.display()).to_ascii_lowercase();
    let patterns = [
        (r"\bfr[-_]", TraceDocumentKind::FunctionalRequirement),
        (r"\bnfr[-_]", TraceDocumentKind::NonFunctionalRequirement),
        (r"\btest[-_/]|tests?/", TraceDocumentKind::Test),
        (r"\bcode[-_/]|src/|source/", TraceDocumentKind::Code),
    ];

    patterns.iter().find_map(|(pattern, kind)| {
        Regex::new(pattern)
            .expect("trace-kind regex compiles")
            .is_match(&text)
            .then_some(*kind)
    })
}
