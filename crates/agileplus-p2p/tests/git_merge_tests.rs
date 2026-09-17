//! Integration tests for the `agileplus_p2p::git_merge` module.
//!
//! Exercises `resolve_git_conflicts` end-to-end on synthetic conflict-marker
//! files: event JSONL (dedup), snapshots (latest-wins) and sync_state.json
//! (field-level merge).

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_p2p::git_merge::{MergeError, resolve_git_conflicts};

fn make_event_line(seq: i64) -> String {
    let mut e = Event::new(
        "Feature",
        1,
        "created",
        serde_json::json!({"seq": seq}),
        "t",
    );
    e.sequence = seq;
    e.hash[0] = seq as u8;
    serde_json::to_string(&e).unwrap()
}

fn conflict_block(ours: &str, theirs: &str) -> String {
    format!("<<<<<<< HEAD\n{ours}\n=======\n{theirs}\n>>>>>>> branch\n")
}

fn write_event_conflict(root: &std::path::Path, name: &str, ours: &str, theirs: &str) {
    let d = root.join(".agileplus/sync/events/Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(name), conflict_block(ours, theirs)).unwrap();
}

fn sync_dir(root: &std::path::Path) -> std::path::PathBuf {
    let d = root.join(".agileplus/sync");
    std::fs::create_dir_all(&d).unwrap();
    d
}

// ── Empty / no-op cases ───────────────────────────────────────────────────────

#[test]
fn no_sync_dir_is_a_noop() {
    let tmp = tempfile::tempdir().unwrap();
    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 0);
    assert_eq!(r.snapshot_files_resolved, 0);
    assert!(!r.sync_state_merged);
}

#[test]
fn sync_dir_without_conflicts_resolves_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path());
    std::fs::create_dir_all(d.join("events/Feature")).unwrap();
    std::fs::write(
        d.join("events/Feature/1.jsonl"),
        format!("{}\n", make_event_line(1)),
    )
    .unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 0);
    assert_eq!(r.snapshot_files_resolved, 0);
    assert!(!r.sync_state_merged);
}

#[test]
fn ignored_file_extensions_are_not_touched() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path());
    std::fs::create_dir_all(d.join("events/Feature")).unwrap();
    std::fs::write(d.join("events/Feature/notes.md"), conflict_block("a", "b")).unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 0);
}

// ── JSONL resolution ──────────────────────────────────────────────────────────

#[test]
fn jsonl_conflict_deduplicates_by_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let ours = format!("{}\n{}", make_event_line(1), make_event_line(2));
    let theirs = make_event_line(1);
    write_event_conflict(tmp.path(), "1.jsonl", &ours, &theirs);

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 1);

    let text =
        std::fs::read_to_string(tmp.path().join(".agileplus/sync/events/Feature/1.jsonl")).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2, "duplicate event should be removed");
}

#[test]
fn jsonl_resolution_sorts_by_sequence() {
    let tmp = tempfile::tempdir().unwrap();
    let ours = format!("{}\n{}", make_event_line(3), make_event_line(1));
    let theirs = make_event_line(2);
    write_event_conflict(tmp.path(), "1.jsonl", &ours, &theirs);
    resolve_git_conflicts(tmp.path()).unwrap();

    let text =
        std::fs::read_to_string(tmp.path().join(".agileplus/sync/events/Feature/1.jsonl")).unwrap();
    let seqs: Vec<i64> = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str::<Event>(l).unwrap().sequence)
        .collect();
    assert_eq!(seqs, vec![1, 2, 3]);
}

#[test]
fn jsonl_resolution_skips_unparsable_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let ours = format!("not json\n{}", make_event_line(1));
    let theirs = make_event_line(2);
    write_event_conflict(tmp.path(), "1.jsonl", &ours, &theirs);

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 1);

    let text =
        std::fs::read_to_string(tmp.path().join(".agileplus/sync/events/Feature/1.jsonl")).unwrap();
    assert_eq!(text.lines().filter(|l| !l.is_empty()).count(), 2);
}

#[test]
fn jsonl_resolution_handles_multiple_conflict_blocks() {
    let tmp = tempfile::tempdir().unwrap();
    let ours = format!("{}\n{}", make_event_line(1), make_event_line(2));
    let theirs = make_event_line(3);
    let content = format!(
        "{}{}",
        conflict_block(&ours, &make_event_line(9)),
        conflict_block(&theirs, &make_event_line(4)),
    );
    let d = tmp.path().join(".agileplus/sync/events/Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.jsonl"), content).unwrap();

    resolve_git_conflicts(tmp.path()).unwrap();
    let text = std::fs::read_to_string(d.join("1.jsonl")).unwrap();
    let seqs: Vec<i64> = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str::<Event>(l).unwrap().sequence)
        .collect();
    assert_eq!(seqs, vec![1, 2, 3, 4, 9]);
}

// ── Snapshot resolution ───────────────────────────────────────────────────────

#[test]
fn snapshot_conflict_keeps_higher_sequence() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path()).join("snapshots/Feature");
    std::fs::create_dir_all(&d).unwrap();
    let old = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
    let new = Snapshot::new("Feature", 1, serde_json::json!({"v": 9}), 9);
    std::fs::write(
        d.join("1.json"),
        conflict_block(
            &serde_json::to_string_pretty(&old).unwrap(),
            &serde_json::to_string_pretty(&new).unwrap(),
        ),
    )
    .unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.snapshot_files_resolved, 1);
    let resolved: Snapshot =
        serde_json::from_str(&std::fs::read_to_string(d.join("1.json")).unwrap()).unwrap();
    assert_eq!(resolved.event_sequence, 9);
}

#[test]
fn snapshot_conflict_keeps_ours_when_ours_is_newer() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path()).join("snapshots/Feature");
    std::fs::create_dir_all(&d).unwrap();
    let ours = Snapshot::new("Feature", 1, serde_json::json!({}), 7);
    let theirs = Snapshot::new("Feature", 1, serde_json::json!({}), 2);
    std::fs::write(
        d.join("1.json"),
        conflict_block(
            &serde_json::to_string_pretty(&ours).unwrap(),
            &serde_json::to_string_pretty(&theirs).unwrap(),
        ),
    )
    .unwrap();

    resolve_git_conflicts(tmp.path()).unwrap();
    let resolved: Snapshot =
        serde_json::from_str(&std::fs::read_to_string(d.join("1.json")).unwrap()).unwrap();
    assert_eq!(resolved.event_sequence, 7);
}

#[test]
fn snapshot_conflict_with_no_parseable_side_is_not_counted() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path()).join("snapshots/Feature");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("1.json"), conflict_block("junk", "also junk")).unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.snapshot_files_resolved, 0);
}

// ── sync_state.json resolution ────────────────────────────────────────────────

#[test]
fn sync_state_conflict_takes_per_key_maximum() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path());
    let ours = serde_json::json!({
        "sync_mappings": [],
        "sync_vector": {"device_id": "d1", "entries": {"Feature/1": 3}}
    });
    let theirs = serde_json::json!({
        "sync_mappings": [],
        "sync_vector": {"device_id": "d1", "entries": {"Feature/1": 8, "Epic/2": 4}}
    });
    std::fs::write(
        d.join("sync_state.json"),
        conflict_block(
            &serde_json::to_string_pretty(&ours).unwrap(),
            &serde_json::to_string_pretty(&theirs).unwrap(),
        ),
    )
    .unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert!(r.sync_state_merged);

    let merged: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(d.join("sync_state.json")).unwrap()).unwrap();
    assert_eq!(merged["sync_vector"]["entries"]["Feature/1"], 8);
    assert_eq!(merged["sync_vector"]["entries"]["Epic/2"], 4);
}

#[test]
fn sync_state_without_markers_is_not_merged() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path());
    std::fs::write(d.join("sync_state.json"), "{\"sync_mappings\": []}").unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert!(!r.sync_state_merged);
}

#[test]
fn malformed_sync_state_conflict_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path());
    std::fs::write(d.join("sync_state.json"), conflict_block("nope", "nada")).unwrap();

    let err = resolve_git_conflicts(tmp.path()).unwrap_err();
    assert!(matches!(err, MergeError::MalformedConflict(_)));
}

// ── Nested traversal + aggregate ──────────────────────────────────────────────

#[test]
fn walks_nested_entity_directories() {
    let tmp = tempfile::tempdir().unwrap();
    let d = sync_dir(tmp.path()).join("events/Feature/sub");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(
        d.join("1.jsonl"),
        conflict_block(
            &format!("{}\n{}", make_event_line(1), make_event_line(2)),
            &make_event_line(1),
        ),
    )
    .unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 1);
}

#[test]
fn end_to_end_resolves_all_three_categories() {
    let tmp = tempfile::tempdir().unwrap();
    write_event_conflict(
        tmp.path(),
        "1.jsonl",
        &format!("{}\n{}", make_event_line(1), make_event_line(2)),
        &make_event_line(1),
    );

    let snap_dir = sync_dir(tmp.path()).join("snapshots/Feature");
    std::fs::create_dir_all(&snap_dir).unwrap();
    std::fs::write(
        snap_dir.join("1.json"),
        conflict_block(
            &serde_json::to_string_pretty(&Snapshot::new("Feature", 1, serde_json::json!({}), 1))
                .unwrap(),
            &serde_json::to_string_pretty(&Snapshot::new("Feature", 1, serde_json::json!({}), 6))
                .unwrap(),
        ),
    )
    .unwrap();

    let ss = sync_dir(tmp.path());
    std::fs::write(
        ss.join("sync_state.json"),
        conflict_block(
            &serde_json::to_string_pretty(
                &serde_json::json!({"sync_mappings": [], "sync_vector": {"device_id": "d", "entries": {"Feature/1": 2}}}),
            )
            .unwrap(),
            &serde_json::to_string_pretty(
                &serde_json::json!({"sync_mappings": [], "sync_vector": {"device_id": "d", "entries": {"Feature/1": 5}}}),
            )
            .unwrap(),
        ),
    )
    .unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 1);
    assert_eq!(r.snapshot_files_resolved, 1);
    assert!(r.sync_state_merged);
}

#[test]
fn resolves_multiple_jsonl_files_independently() {
    let tmp = tempfile::tempdir().unwrap();
    write_event_conflict(
        tmp.path(),
        "1.jsonl",
        &format!("{}\n{}", make_event_line(1), make_event_line(2)),
        &make_event_line(1),
    );
    write_event_conflict(
        tmp.path(),
        "2.jsonl",
        &format!("{}\n{}", make_event_line(3), make_event_line(4)),
        &make_event_line(3),
    );

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 2);
}
