// SPDX-License-Identifier: MIT OR Apache-2.0
//! Conflict parsing for `git merge-tree` and `git merge` output.
//!
//! Extracted from `lib.rs` to keep the main module under 350 lines.
//!
//! Traceability: recs #10, #11 from `AUDIT_BLOC_VS_2026_SOTA.md`.

use std::collections::BTreeMap;

use agileplus_domain::ports::vcs::ConflictInfo;

use super::GitVcsAdapter;

impl GitVcsAdapter {
    /// Extract conflicts from the human-readable output emitted by Git's
    /// merge machinery.  `merge-tree` differs across Git versions: Apple
    /// Git 2.54 writes unmerged index rows (`mode oid stage<TAB>path`) plus
    /// `CONFLICT (...)` diagnostics, while older versions can emit a patch.
    pub(crate) fn parse_conflicts(raw: &str) -> Vec<ConflictInfo> {
        let mut paths = BTreeMap::<String, String>::new();
        let mut current_diff_path: Option<String> = None;
        let mut legacy_conflict_heading = false;

        for line in raw.lines() {
            if line.trim_end() == "changed in both" {
                legacy_conflict_heading = true;
                continue;
            }

            if legacy_conflict_heading {
                if let Some(path) = Self::legacy_merge_tree_path(line) {
                    paths
                        .entry(path.to_string())
                        .or_insert_with(|| "content".to_string());
                    legacy_conflict_heading = false;
                    continue;
                }
                if !line.starts_with(char::is_whitespace) {
                    legacy_conflict_heading = false;
                }
            }

            if let Some((metadata, path)) = line.split_once('\t') {
                let fields: Vec<_> = metadata.split_whitespace().collect();
                if fields.len() == 3
                    && fields[0].chars().all(|c| c.is_ascii_digit())
                    && fields[2].chars().all(|c| c.is_ascii_digit())
                    && !path.is_empty()
                {
                    paths
                        .entry(path.to_string())
                        .or_insert_with(|| "content".to_string());
                    continue;
                }
            }

            if let Some(rest) = line.strip_prefix("CONFLICT (") {
                if let Some((kind, description)) = rest.split_once("): ")
                    && let Some(path) = Self::conflict_diagnostic_path(description)
                {
                    paths.insert(path.to_string(), kind.to_string());
                }
                continue;
            }

            if let Some(path) = line.strip_prefix("changed in both ") {
                if !path.is_empty() {
                    paths
                        .entry(path.to_string())
                        .or_insert_with(|| "content".to_string());
                }
                continue;
            }

            if line.starts_with("diff --git ") {
                current_diff_path = Self::diff_header_destination_path(line);
            } else if (line.starts_with("<<<<<<<")
                || line.starts_with("=======")
                || line.starts_with(">>>>>>>"))
                && current_diff_path.is_some()
            {
                let path = current_diff_path.take().expect("checked above");
                paths.entry(path).or_insert_with(|| "content".to_string());
            }
        }

        paths
            .into_iter()
            .map(|(path, conflict_type)| ConflictInfo {
                path: path.clone(),
                file_path: path,
                conflict_type,
                ours: None,
                theirs: None,
            })
            .collect()
    }

    /// Extract the path from the `base <mode> <oid> <path>` row following a
    /// legacy `changed in both` merge-tree heading without losing spaces in
    /// the path itself.
    fn legacy_merge_tree_path(line: &str) -> Option<&str> {
        let mut remainder = line.trim_start();
        for _ in 0..3 {
            let delimiter = remainder.find(char::is_whitespace)?;
            remainder = remainder[delimiter..].trim_start();
        }
        (!remainder.is_empty()).then_some(remainder)
    }

    /// Return the destination path from a `diff --git` header. Git quotes
    /// paths containing whitespace, so splitting the header on whitespace
    /// would truncate a valid conflicted path.
    fn diff_header_destination_path(line: &str) -> Option<String> {
        let input = line.strip_prefix("diff --git ")?;
        let (_, remainder) = Self::git_path_token(input)?;
        let (destination, _) = Self::git_path_token(remainder.trim_start())?;
        Some(
            destination
                .strip_prefix("b/")
                .unwrap_or(&destination)
                .to_string(),
        )
    }

    /// Parse one Git diff header path token, including Git's C-style quoting
    /// for whitespace and special characters.
    fn git_path_token(input: &str) -> Option<(String, &str)> {
        if !input.starts_with('"') {
            let end = input.find(char::is_whitespace).unwrap_or(input.len());
            return (!input[..end].is_empty()).then(|| (input[..end].to_string(), &input[end..]));
        }

        let mut decoded = Vec::new();
        let mut chars = input[1..].chars();
        while let Some(character) = chars.next() {
            match character {
                '"' => return Some((String::from_utf8(decoded).ok()?, chars.as_str())),
                '\\' => {
                    let escaped = chars.next()?;
                    match escaped {
                        'n' => decoded.push(b'\n'),
                        'r' => decoded.push(b'\r'),
                        't' => decoded.push(b'\t'),
                        '"' | '\\' => decoded.push(escaped as u8),
                        '0'..='7' => {
                            let second = chars.next()?;
                            let third = chars.next()?;
                            let octal = [escaped, second, third].iter().collect::<String>();
                            let byte = u8::from_str_radix(&octal, 8).ok()?;
                            decoded.push(byte);
                        }
                        other => {
                            let mut buffer = [0; 4];
                            decoded.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
                        }
                    }
                }
                other => {
                    let mut buffer = [0; 4];
                    decoded.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
                }
            }
        }
        None
    }

    /// Extract a path only from merge-tree diagnostic forms whose path
    /// position is defined by Git.  In particular, do not split arbitrary
    /// prose on ` in `: branch names and diagnostic prose can contain that
    /// phrase and are not file paths.
    fn conflict_diagnostic_path(description: &str) -> Option<&str> {
        if let Some(path) = description.strip_prefix("Merge conflict in ") {
            return (!path.is_empty()).then_some(path);
        }

        // e.g. "foo.rs deleted in HEAD and modified in topic".
        if let Some((path, _)) = description.split_once(" deleted in ") {
            return (!path.is_empty()).then_some(path);
        }

        // e.g. "foo.rs renamed to bar.rs in HEAD and renamed to baz.rs in topic".
        // The path follows the fixed ` renamed to ` token and ends at the
        // immediately following fixed ` in ` token.
        let (_, renamed) = description.split_once(" renamed to ")?;
        let (path, _) = renamed.split_once(" in ")?;
        (!path.is_empty()).then_some(path)
    }

    /// Return unresolved paths from a merge worktree.  This is authoritative
    /// after `git merge` fails because it reads Git's unmerged index directly.
    pub(crate) fn unresolved_conflicts_in(dir: &std::path::Path) -> Vec<ConflictInfo> {
        use std::process::{Command, Stdio};

        let output = Command::new("git")
            .args(["diff", "--name-only", "--diff-filter=U", "-z"])
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        let Ok(output) = output else {
            return vec![];
        };
        if !output.status.success() {
            return vec![];
        }
        output
            .stdout
            .split(|byte| *byte == b'\0')
            .filter(|path| !path.is_empty())
            .map(|path| {
                let path = String::from_utf8_lossy(path).into_owned();
                ConflictInfo {
                    path: path.clone(),
                    file_path: path,
                    conflict_type: "content".to_string(),
                    ours: None,
                    theirs: None,
                }
            })
            .collect()
    }
}
