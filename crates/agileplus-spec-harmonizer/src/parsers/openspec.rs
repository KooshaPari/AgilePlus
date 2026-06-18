//! OpenSpec parser.
//!
//! Format:
//! ```text
//! ## Spec <id> — <title>
//! <description>
//!
//! ## Acceptance
//! - criterion 1
//! - criterion 2
//! ```
//!
//! The acceptance block is delimited by a `## Acceptance` heading and ends
//! at the next `## ` heading or EOF.

use crate::parsers::Parser;
use crate::{AcceptanceCriterion, WorkPackage};
use regex::Regex;

pub struct OpenSpecParser;

impl Parser for OpenSpecParser {
    fn parse(&self, text: &str) -> Result<Vec<WorkPackage>, String> {
        let spec = Regex::new(r"(?m)^##\s+Spec\s+([A-Za-z0-9_\-]+)\s*[—\-:]\s*(.+?)\s*$")
            .map_err(|e| format!("regex: {}", e))?;
        let acc = Regex::new(r"(?m)^##\s+Acceptance\s*$")
            .map_err(|e| format!("regex: {}", e))?;
        let bullet = Regex::new(r"^\s*-\s+(.+?)\s*$")
            .map_err(|e| format!("regex: {}", e))?;

        let mut pkgs: Vec<WorkPackage> = Vec::new();
        let mut current: Option<WorkPackage> = None;
        let mut desc = String::new();
        let mut accs: Vec<AcceptanceCriterion> = Vec::new();
        let mut in_acc = false;

        let flush = |pkgs: &mut Vec<WorkPackage>, cur: &mut Option<WorkPackage>, desc: &mut String, accs: &mut Vec<AcceptanceCriterion>| {
            if let Some(mut p) = cur.take() {
                let d = desc.trim().to_string();
                p.description = if d.is_empty() { "(no description)".into() } else { d };
                p.acceptance = std::mem::take(accs);
                pkgs.push(p);
            }
            desc.clear();
        };

        for line in text.lines() {
            if let Some(c) = spec.captures(line) {
                flush(&mut pkgs, &mut current, &mut desc, &mut accs);
                let id = c.get(1).unwrap().as_str().to_string();
                let title = c.get(2).unwrap().as_str().to_string();
                current = Some(WorkPackage {
                    id: format!("openspec-{}", id),
                    title,
                    description: String::new(),
                    acceptance: Vec::new(),
                    source_format: "openspec".into(),
                    source_anchor: id,
                });
                in_acc = false;
                continue;
            }
            if current.is_none() { continue; }
            if acc.is_match(line) {
                in_acc = true;
                continue;
            }
            if line.trim_start().starts_with("## ") {
                // another major section without going through spec — flush and let next
                // ## Spec handle the rest
                in_acc = false;
                continue;
            }
            if in_acc {
                if let Some(c) = bullet.captures(line) {
                    accs.push(AcceptanceCriterion { text: c[1].to_string(), done: false });
                }
            } else {
                desc.push_str(line);
                desc.push('\n');
            }
        }
        flush(&mut pkgs, &mut current, &mut desc, &mut accs);
        if pkgs.is_empty() {
            return Err("no OpenSpec `## Spec <id>` headings found".into());
        }
        Ok(pkgs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::Parser;

    #[test]
    fn parses_openspec_with_acceptance() {
        let text = "## Spec ABC-1 — Login Flow\nUsers can log in.\n\n## Acceptance\n- email + password work\n- MFA optional\n\n## Spec ABC-2 — Logout\nClick logout.\n";
        let out = OpenSpecParser.parse(text).expect("parse");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "openspec-ABC-1");
        assert_eq!(out[0].title, "Login Flow");
        assert_eq!(out[0].acceptance.len(), 2);
        assert_eq!(out[1].title, "Logout");
        assert_eq!(out[1].acceptance.len(), 0);
    }
}
