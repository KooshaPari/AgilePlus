/// Represents one side of a git conflict block.
#[derive(Debug)]
pub(crate) struct ConflictBlock {
    pub ours: String,
    pub theirs: String,
}

/// Parse a file that may contain standard git conflict markers:
/// ```text
/// <<<<<<< HEAD
/// ... ours ...
/// =======
/// ... theirs ...
/// >>>>>>> branch
/// ```
/// Returns the list of conflict blocks found. If none are found the file is
/// returned as-is in a synthetic block with `theirs` set to the same content.
pub(crate) fn parse_conflict_blocks(content: &str) -> Vec<ConflictBlock> {
    let mut blocks: Vec<ConflictBlock> = Vec::new();
    let mut ours_lines: Vec<&str> = Vec::new();
    let mut theirs_lines: Vec<&str> = Vec::new();
    let mut in_conflict = false;
    let mut in_theirs = false;
    let mut found_any = false;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            in_theirs = false;
            ours_lines.clear();
            theirs_lines.clear();
            found_any = true;
        } else if line.starts_with("=======") && in_conflict {
            in_theirs = true;
        } else if line.starts_with(">>>>>>>") && in_conflict {
            blocks.push(ConflictBlock {
                ours: ours_lines.join("\n"),
                theirs: theirs_lines.join("\n"),
            });
            in_conflict = false;
            in_theirs = false;
        } else if in_conflict {
            if in_theirs {
                theirs_lines.push(line);
            } else {
                ours_lines.push(line);
            }
        }
    }

    if !found_any {
        blocks.push(ConflictBlock {
            ours: content.to_string(),
            theirs: content.to_string(),
        });
    }

    blocks
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn no_markers_returns_synthetic_single_block() {
        let blocks = parse_conflict_blocks("line one\nline two");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].ours, "line one\nline two");
        assert_eq!(blocks[0].theirs, "line one\nline two");
    }

    #[test]
    fn empty_content_returns_synthetic_empty_block() {
        let blocks = parse_conflict_blocks("");
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].ours.is_empty());
        assert!(blocks[0].theirs.is_empty());
    }

    #[test]
    fn single_conflict_block_parsed() {
        let content = "<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n";
        let blocks = parse_conflict_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].ours, "ours");
        assert_eq!(blocks[0].theirs, "theirs");
    }

    #[test]
    fn multi_line_sides_preserved() {
        let content = "<<<<<<< HEAD\na\nb\n=======\nc\nd\n>>>>>>> x\n";
        let blocks = parse_conflict_blocks(content);
        assert_eq!(blocks[0].ours, "a\nb");
        assert_eq!(blocks[0].theirs, "c\nd");
    }

    #[test]
    fn empty_theirs_side() {
        let content = "<<<<<<< HEAD\nours\n=======\n>>>>>>> x\n";
        let blocks = parse_conflict_blocks(content);
        assert_eq!(blocks[0].ours, "ours");
        assert!(blocks[0].theirs.is_empty());
    }

    #[test]
    fn multiple_blocks_parsed() {
        let content = concat!(
            "<<<<<<< HEAD\na1\n=======\nb1\n>>>>>>> x\n",
            "context line\n",
            "<<<<<<< HEAD\na2\n=======\nb2\n>>>>>>> x\n"
        );
        let blocks = parse_conflict_blocks(content);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].ours, "a1");
        assert_eq!(blocks[1].ours, "a2");
        assert_eq!(blocks[1].theirs, "b2");
    }

    #[test]
    fn context_lines_outside_blocks_ignored() {
        let content = "before\n<<<<<<< HEAD\na\n=======\nb\n>>>>>>> x\nafter\n";
        let blocks = parse_conflict_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].ours.contains("before"));
        assert!(!blocks[0].theirs.contains("after"));
    }

    #[test]
    fn marker_with_extra_text_still_matches() {
        let content = "<<<<<<< HEAD (abc123)\na\n=======\nb\n>>>>>>> feature/x\n";
        let blocks = parse_conflict_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].ours, "a");
    }

    #[test]
    fn unterminated_block_is_dropped() {
        let content = "<<<<<<< HEAD\na\n=======\nb\n";
        let blocks = parse_conflict_blocks(content);
        // No closing marker: not emitted, and found_any suppresses synthesis.
        assert!(blocks.is_empty());
    }

    #[test]
    fn separator_without_conflict_is_ignored() {
        let blocks = parse_conflict_blocks("plain ======= text");
        assert_eq!(blocks.len(), 1);
        // Synthetic block keeps the whole content.
        assert_eq!(blocks[0].ours, "plain ======= text");
    }

    #[test]
    fn conflict_block_debug_contains_sides() {
        let blocks = parse_conflict_blocks("<<<<<<< HEAD\nLHS\n=======\nRHS\n>>>>>>> x\n");
        let s = format!("{:?}", blocks[0]);
        assert!(s.contains("LHS"));
        assert!(s.contains("RHS"));
    }
}
