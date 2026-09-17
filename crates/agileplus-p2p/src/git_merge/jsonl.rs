use std::collections::HashSet;
use std::path::Path;

use agileplus_domain::domain::event::Event;
use tracing::{info, warn};

use super::parser::parse_conflict_blocks;
use super::types::MergeError;

/// Resolve a conflicted JSONL event file.
///
/// Parses all lines from both sides of every conflict block, deduplicates by
/// the event `hash` field, and re-writes the file sorted by sequence.
pub(crate) fn resolve_jsonl_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

    if !content.contains("<<<<<<<") {
        return Ok(false);
    }

    let blocks = parse_conflict_blocks(&content);
    let mut seen_hashes: HashSet<String> = HashSet::new();
    let mut events: Vec<Event> = Vec::new();

    for block in &blocks {
        for side in [&block.ours, &block.theirs] {
            for line in side.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Event>(line) {
                    Ok(event) => {
                        let hash_hex = encode_hash(event.hash);
                        if seen_hashes.insert(hash_hex) {
                            events.push(event);
                        }
                    }
                    Err(e) => {
                        warn!("Skipping unparsable event line in {}: {e}", path.display());
                    }
                }
            }
        }
    }

    events.sort_by_key(|e| e.sequence);

    use std::io::Write as _;
    let mut file = std::fs::File::create(path)?;
    for event in &events {
        let line = serde_json::to_string(event).map_err(|e| MergeError::Parse {
            file: path.display().to_string(),
            source: e,
        })?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }

    info!(
        "Resolved JSONL conflict in {} — {} unique events",
        path.display(),
        events.len()
    );
    Ok(true)
}

fn encode_hash(bytes: [u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    fn make_event(seq: i64, hash_byte: u8) -> Event {
        let mut e = Event::new("Feature", 1, "created", serde_json::json!({}), "test");
        e.sequence = seq;
        e.hash[0] = hash_byte;
        e
    }

    fn conflict(ours: &str, theirs: &str) -> String {
        format!("<<<<<<< HEAD\n{ours}\n=======\n{theirs}\n>>>>>>> x\n")
    }

    #[test]
    fn clean_file_returns_false() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.jsonl");
        std::fs::write(&path, "no markers\n").unwrap();
        assert!(!resolve_jsonl_conflict(&path).unwrap());
    }

    #[test]
    fn missing_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let err = resolve_jsonl_conflict(&tmp.path().join("nope.jsonl")).unwrap_err();
        assert!(matches!(err, MergeError::Io(_)));
    }

    #[test]
    fn dedup_keeps_unique_only() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.jsonl");
        let e1 = serde_json::to_string(&make_event(1, 1)).unwrap();
        let e2 = serde_json::to_string(&make_event(2, 2)).unwrap();
        std::fs::write(&path, conflict(&format!("{e1}\n{e2}"), &e1)).unwrap();
        assert!(resolve_jsonl_conflict(&path).unwrap());
        let lines: Vec<&str> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| Box::leak(l.to_string().into_boxed_str()) as &str)
            .collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn output_sorted_by_sequence() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.jsonl");
        let e5 = serde_json::to_string(&make_event(5, 5)).unwrap();
        let e1 = serde_json::to_string(&make_event(1, 1)).unwrap();
        // ours has seq5 first; theirs adds seq1.
        std::fs::write(&path, conflict(&e5, &e1)).unwrap();
        resolve_jsonl_conflict(&path).unwrap();
        let seqs: Vec<i64> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_str::<Event>(l).unwrap().sequence)
            .collect();
        assert_eq!(seqs, vec![1, 5]);
    }

    #[test]
    fn unparsable_lines_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.jsonl");
        let good = serde_json::to_string(&make_event(1, 1)).unwrap();
        std::fs::write(&path, conflict(&format!("{{bad json\n{good}"), "also bad")).unwrap();
        assert!(resolve_jsonl_conflict(&path).unwrap());
        let lines: Vec<String> = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            serde_json::from_str::<Event>(&lines[0]).unwrap().sequence,
            1
        );
    }

    #[test]
    fn both_sides_unparsable_yields_empty_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.jsonl");
        std::fs::write(&path, conflict("{bad", "also bad")).unwrap();
        assert!(resolve_jsonl_conflict(&path).unwrap());
        assert!(std::fs::read_to_string(&path).unwrap().trim().is_empty());
    }

    #[test]
    fn duplicate_hash_different_content_deduped() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.jsonl");
        // Same hash[0] and same seq => same hash string => dedup.
        let a = serde_json::to_string(&make_event(1, 1)).unwrap();
        let b = serde_json::to_string(&make_event(1, 1)).unwrap();
        std::fs::write(&path, conflict(&a, &b)).unwrap();
        resolve_jsonl_conflict(&path).unwrap();
        let count = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn encode_hash_formats_hex() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xab;
        bytes[31] = 0x0f;
        let s = encode_hash(bytes);
        assert_eq!(s.len(), 64);
        assert!(s.starts_with("ab"));
        assert!(s.ends_with("0f"));
    }
}
