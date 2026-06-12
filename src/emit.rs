//! Emitter: produce a TRAC-aligned representation (NDJSON) and Markdown index.

use crate::WorkPackage;

/// Emit NDJSON: one WorkPackage per line.
pub fn emit_ndjson(pkgs: &[WorkPackage]) -> String {
    let mut out = String::new();
    for p in pkgs {
        let json = serde_json::to_string(p).unwrap_or_default();
        out.push_str(&json);
        out.push('\n');
    }
    out
}

/// Emit a Markdown index, grouped by source_format, with anchor links.
pub fn emit_markdown(pkgs: &[WorkPackage]) -> String {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<&WorkPackage>> = BTreeMap::new();
    for p in pkgs {
        groups.entry(p.source_format.clone()).or_default().push(p);
    }
    let mut out = String::new();
    out.push_str("# Harmonized Work Packages\n\n");
    out.push_str(&format!("Total: **{}** packages across **{}** formats.\n\n",
        pkgs.len(), groups.len()));
    for (fmt, group) in &groups {
        out.push_str(&format!("## {}\n\n", fmt));
        out.push_str("| ID | Title | Acceptance |\n|---|---|---|\n");
        for p in group {
            let acc = p.acceptance.len();
            out.push_str(&format!("| `{}` | {} | {} |\n", p.id, p.title, acc));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkPackage;

    fn pkg(anchor: &str, fmt: &str) -> WorkPackage {
        WorkPackage {
            id: format!("{}-{}", fmt, anchor),
            title: format!("Title {}", anchor),
            description: "d".into(),
            acceptance: vec![],
            source_format: fmt.into(),
            source_anchor: anchor.into(),
        }
    }

    #[test]
    fn ndjson_one_line_per_package() {
        let pkgs = vec![pkg("1", "gsd"), pkg("2", "bmad")];
        let out = emit_ndjson(&pkgs);
        assert_eq!(out.lines().count(), 2);
    }

    #[test]
    fn markdown_groups_by_format() {
        let pkgs = vec![pkg("1", "gsd"), pkg("2", "gsd"), pkg("1", "bmad")];
        let out = emit_markdown(&pkgs);
        assert!(out.contains("## gsd"));
        assert!(out.contains("## bmad"));
        assert!(out.contains("Total: **3**"));
    }
}
