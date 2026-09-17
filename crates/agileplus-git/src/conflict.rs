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

#[cfg(test)]
mod deep_tests {
    use super::*;

    // ── git_path_token ──────────────────────────────────────────────────────

    #[test]
    fn path_token_unquoted_whole_input() {
        let (token, rest) = GitVcsAdapter::git_path_token("a/foo.txt").unwrap();
        assert_eq!(token, "a/foo.txt");
        assert_eq!(rest, "");
    }

    #[test]
    fn path_token_unquoted_stops_at_space() {
        let (token, rest) = GitVcsAdapter::git_path_token("a/foo.txt b/foo.txt").unwrap();
        assert_eq!(token, "a/foo.txt");
        assert_eq!(rest, " b/foo.txt");
    }

    #[test]
    fn path_token_unquoted_empty_returns_none() {
        assert!(GitVcsAdapter::git_path_token(" leading").is_none());
    }

    #[test]
    fn path_token_quoted_plain() {
        let (token, rest) = GitVcsAdapter::git_path_token("\"a/x.txt\" tail").unwrap();
        assert_eq!(token, "a/x.txt");
        assert_eq!(rest, " tail");
    }

    #[test]
    fn path_token_quoted_escaped_quote() {
        let (token, _) = GitVcsAdapter::git_path_token("\"a/q\\\"uote.txt\"").unwrap();
        assert_eq!(token, "a/q\"uote.txt");
    }

    #[test]
    fn path_token_quoted_escaped_backslash() {
        let (token, _) = GitVcsAdapter::git_path_token("\"a/back\\\\slash\"").unwrap();
        assert_eq!(token, "a/back\\slash");
    }

    #[test]
    fn path_token_quoted_escapes_newline_tab_return() {
        assert_eq!(
            GitVcsAdapter::git_path_token("\"a\\nb\"").unwrap().0,
            "a\nb"
        );
        assert_eq!(
            GitVcsAdapter::git_path_token("\"a\\tb\"").unwrap().0,
            "a\tb"
        );
        assert_eq!(
            GitVcsAdapter::git_path_token("\"a\\rb\"").unwrap().0,
            "a\rb"
        );
    }

    #[test]
    fn path_token_quoted_octal_utf8() {
        // "café" encoded as caf\303\251
        let (token, _) = GitVcsAdapter::git_path_token("\"caf\\303\\251.txt\"").unwrap();
        assert_eq!(token, "café.txt");
    }

    #[test]
    fn path_token_quoted_unterminated_returns_none() {
        assert!(GitVcsAdapter::git_path_token("\"a/unterminated").is_none());
    }

    #[test]
    fn path_token_quoted_octal_incomplete_returns_none() {
        assert!(GitVcsAdapter::git_path_token("\"a\\30\"").is_none());
    }

    #[test]
    fn path_token_quoted_unknown_escape_keeps_literal_char() {
        let (token, _) = GitVcsAdapter::git_path_token("\"a\\zb\"").unwrap();
        assert_eq!(token, "azb");
    }

    // ── legacy_merge_tree_path ──────────────────────────────────────────────

    #[test]
    fn legacy_path_skips_mode_and_oid() {
        let line = "  base   100644 abcdef12 docs/a b.txt";
        assert_eq!(
            GitVcsAdapter::legacy_merge_tree_path(line),
            Some("docs/a b.txt")
        );
    }

    #[test]
    fn legacy_path_requires_three_tokens() {
        assert_eq!(GitVcsAdapter::legacy_merge_tree_path("base 100644"), None);
    }

    #[test]
    fn legacy_path_empty_remainder_is_none() {
        assert_eq!(
            GitVcsAdapter::legacy_merge_tree_path("base 100644 abcdef   "),
            None
        );
    }

    // ── diff_header_destination_path ────────────────────────────────────────

    #[test]
    fn diff_header_strips_b_prefix() {
        let p = GitVcsAdapter::diff_header_destination_path("diff --git a/x.txt b/x.txt").unwrap();
        assert_eq!(p, "x.txt");
    }

    #[test]
    fn diff_header_quoted_with_spaces() {
        let p = GitVcsAdapter::diff_header_destination_path(
            "diff --git \"a/docs/a b.txt\" \"b/docs/a b.txt\"",
        )
        .unwrap();
        assert_eq!(p, "docs/a b.txt");
    }

    #[test]
    fn diff_header_non_diff_returns_none() {
        assert!(GitVcsAdapter::diff_header_destination_path("not a diff").is_none());
    }

    #[test]
    fn diff_header_missing_destination_returns_none() {
        assert!(GitVcsAdapter::diff_header_destination_path("diff --git a/only.txt").is_none());
    }

    #[test]
    fn diff_header_without_b_prefix_keeps_path() {
        let p = GitVcsAdapter::diff_header_destination_path("diff --git x.txt x.txt").unwrap();
        assert_eq!(p, "x.txt");
    }

    // ── conflict_diagnostic_path ────────────────────────────────────────────

    #[test]
    fn diagnostic_merge_conflict_in() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("Merge conflict in src/lib.rs"),
            Some("src/lib.rs")
        );
    }

    #[test]
    fn diagnostic_merge_conflict_empty_is_none() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("Merge conflict in "),
            None
        );
    }

    #[test]
    fn diagnostic_deleted_in() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path(
                "foo.rs deleted in HEAD and modified in topic"
            ),
            Some("foo.rs")
        );
    }

    #[test]
    fn diagnostic_renamed_to() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path(
                "old.rs renamed to new.rs in HEAD and renamed to other.rs in topic"
            ),
            Some("new.rs")
        );
    }

    #[test]
    fn diagnostic_unmatched_prose_is_none() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("both sides modified a thing"),
            None
        );
    }

    #[test]
    fn diagnostic_does_not_split_arbitrary_in() {
        // "in" appears in prose but there is no "renamed to" token.
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("conflict in feature branch x"),
            None
        );
    }

    // ── parse_conflicts end-to-end ──────────────────────────────────────────

    fn marker_block(path: &str) -> String {
        format!(
            "diff --git a/{path} b/{path}\n{} HEAD\nours\n{}\ntheirs\n{} topic\n",
            "<".repeat(7),
            "=".repeat(7),
            ">".repeat(7)
        )
    }

    #[test]
    fn parse_conflicts_conflict_diagnostic_entry() {
        let raw = "CONFLICT (content): Merge conflict in src/main.rs\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "src/main.rs");
        assert_eq!(out[0].conflict_type, "content");
        assert!(out[0].ours.is_none());
        assert!(out[0].theirs.is_none());
    }

    #[test]
    fn parse_conflicts_con_flict_add_add_type() {
        let raw = "CONFLICT (add/add): Merge conflict in docs/readme.md\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].conflict_type, "add/add");
    }

    #[test]
    fn parse_conflicts_stage_row_format() {
        let raw = "100644 aaaaaaaa 1\tsrc/lib.rs\n100644 bbbbbbbb 2\tsrc/lib.rs\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "src/lib.rs");
        assert_eq!(out[0].conflict_type, "content");
    }

    #[test]
    fn parse_conflicts_stage_row_requires_numeric_mode_and_stage() {
        let raw = "abcdef aaaaaaaa x\tsrc/lib.rs\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert!(out.is_empty());
    }

    #[test]
    fn parse_conflicts_ignores_non_conflict_tab_lines() {
        let raw = "some\ttabbed line\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert!(out.is_empty());
    }

    #[test]
    fn parse_conflicts_changed_in_both_inline_heading() {
        let raw = "changed in both docs/a.txt\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "docs/a.txt");
    }

    #[test]
    fn parse_conflicts_changed_in_both_heading_with_row() {
        let raw = "changed in both\n  base   100644 abc docs/x y.txt\n  our    100644 def docs/x y.txt\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].file_path, "docs/x y.txt");
    }

    #[test]
    fn parse_conflicts_marker_block_only_with_diff_header() {
        let raw = marker_block("src/a.rs");
        let out = GitVcsAdapter::parse_conflicts(&raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, "src/a.rs");
    }

    #[test]
    fn parse_conflicts_markers_without_diff_header_are_ignored() {
        let raw = format!("{} HEAD\nours\n{}\ntheirs\n{} topic\n", "<".repeat(7), "=".repeat(7), ">".repeat(7));
        assert!(GitVcsAdapter::parse_conflicts(&raw).is_empty());
    }

    #[test]
    fn parse_conflicts_multiple_sources_dedup_by_path() {
        let raw = format!(
            "CONFLICT (content): Merge conflict in a.rs\nCONFLICT (add/add): Merge conflict in a.rs\n{}",
            marker_block("b.rs")
        );
        let out = GitVcsAdapter::parse_conflicts(&raw);
        assert_eq!(out.len(), 2);
        // The last CONFLICT diagnostic wins for a given path.
        let a = out.iter().find(|c| c.path == "a.rs").unwrap();
        assert_eq!(a.conflict_type, "add/add");
    }

    #[test]
    fn parse_conflicts_output_sorted_by_path() {
        let raw = "CONFLICT (content): Merge conflict in z.rs\nCONFLICT (content): Merge conflict in a.rs\n";
        let out = GitVcsAdapter::parse_conflicts(raw);
        assert_eq!(out[0].path, "a.rs");
        assert_eq!(out[1].path, "z.rs");
    }

    #[test]
    fn parse_conflicts_empty_path_after_tab_ignored() {
        let raw = "100644 aaaaaaaa 1\t\n";
        assert!(GitVcsAdapter::parse_conflicts(raw).is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn git_path_token_plain() {
        let (tok, rest) = GitVcsAdapter::git_path_token("a/file.rs b/file.rs").unwrap();
        assert_eq!(tok, "a/file.rs");
        assert_eq!(rest, " b/file.rs");
    }

    #[test]
    fn git_path_token_plain_no_trailing() {
        let (tok, rest) = GitVcsAdapter::git_path_token("solo").unwrap();
        assert_eq!(tok, "solo");
        assert_eq!(rest, "");
    }

    #[test]
    fn git_path_token_quoted_decodes_escapes() {
        let (tok, rest) = GitVcsAdapter::git_path_token("\"a b\\tc\" tail").unwrap();
        assert_eq!(tok, "a b\tc");
        assert_eq!(rest, " tail");
    }

    #[test]
    fn git_path_token_quoted_decodes_octal_utf8() {
        // \303\251 is UTF-8 for 'é'.
        let (tok, _) = GitVcsAdapter::git_path_token("\"caf\\303\\251\" x").unwrap();
        assert_eq!(tok, "café");
    }

    #[test]
    fn git_path_token_unterminated_quote_is_none() {
        assert!(GitVcsAdapter::git_path_token("\"no end").is_none());
    }

    #[test]
    fn git_path_token_empty_plain_is_none() {
        assert!(GitVcsAdapter::git_path_token(" rest").is_none());
    }

    #[test]
    fn diff_header_destination_strips_b_prefix() {
        let p = GitVcsAdapter::diff_header_destination_path("diff --git a/src/lib.rs b/src/lib.rs")
            .unwrap();
        assert_eq!(p, "src/lib.rs");
    }

    #[test]
    fn diff_header_destination_handles_quoted_paths() {
        let p = GitVcsAdapter::diff_header_destination_path(
            "diff --git \"a/has space.txt\" \"b/has space.txt\"",
        )
        .unwrap();
        assert_eq!(p, "has space.txt");
    }

    #[test]
    fn diff_header_destination_non_diff_line_is_none() {
        assert!(GitVcsAdapter::diff_header_destination_path("not a diff").is_none());
    }

    #[test]
    fn conflict_diagnostic_path_merge_conflict_form() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("Merge conflict in src/main.rs"),
            Some("src/main.rs")
        );
    }

    #[test]
    fn conflict_diagnostic_path_deleted_form() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("a.rs deleted in HEAD and modified in topic"),
            Some("a.rs")
        );
    }

    #[test]
    fn conflict_diagnostic_path_renamed_form() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path(
                "old.rs renamed to new.rs in HEAD and renamed to newer.rs in topic"
            ),
            Some("new.rs")
        );
    }

    #[test]
    fn conflict_diagnostic_path_unknown_form_is_none() {
        assert_eq!(
            GitVcsAdapter::conflict_diagnostic_path("something else entirely"),
            None
        );
    }

    #[test]
    fn conflict_diagnostic_path_empty_merge_conflict_is_none() {
        assert_eq!(GitVcsAdapter::conflict_diagnostic_path("Merge conflict in "), None);
    }

    #[test]
    fn legacy_merge_tree_path_keeps_spaces() {
        let line = "  base   100644 abcdef0123456789 path with spaces.rs";
        assert_eq!(
            GitVcsAdapter::legacy_merge_tree_path(line),
            Some("path with spaces.rs")
        );
    }

    #[test]
    fn legacy_merge_tree_path_too_few_fields_is_none() {
        assert_eq!(GitVcsAdapter::legacy_merge_tree_path("base 100644"), None);
    }

    #[test]
    fn unresolved_conflicts_in_non_repo_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(GitVcsAdapter::unresolved_conflicts_in(dir.path()).is_empty());
    }

    /// Build a repo whose current index has one unmerged path.
    fn conflicted_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "t@example.com"]);
        git(dir.path(), &["config", "user.name", "tester"]);
        std::fs::write(dir.path().join("file.txt"), "base\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", "base"]);

        git(dir.path(), &["checkout", "-q", "-b", "topic"]);
        std::fs::write(dir.path().join("file.txt"), "topic\n").unwrap();
        git(dir.path(), &["commit", "-q", "-am", "topic change"]);

        git(dir.path(), &["checkout", "-q", "main"]);
        std::fs::write(dir.path().join("file.txt"), "main\n").unwrap();
        git(dir.path(), &["commit", "-q", "-am", "main change"]);

        // Merge is expected to fail, leaving an unmerged index entry.
        let _ = Command::new("git")
            .args(["merge", "topic"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    #[test]
    fn unresolved_conflicts_in_reports_conflicted_path() {
        let dir = conflicted_repo();
        let conflicts = GitVcsAdapter::unresolved_conflicts_in(dir.path());
        assert_eq!(conflicts.len(), 1, "got {conflicts:?}");
        assert_eq!(conflicts[0].path, "file.txt");
        assert_eq!(conflicts[0].conflict_type, "content");
        assert!(conflicts[0].ours.is_none());
    }

    #[test]
    fn unresolved_conflicts_in_clean_repo_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        git(dir.path(), &["config", "user.email", "t@example.com"]);
        git(dir.path(), &["config", "user.name", "tester"]);
        std::fs::write(dir.path().join("f.txt"), "x\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", "init"]);
        assert!(GitVcsAdapter::unresolved_conflicts_in(dir.path()).is_empty());
    }
}
