//! Integration tests for content_hash module.
//!
//! Covers: SHA-256 computation, conflict detection, edge cases, and label ordering.

use agileplus_plane::content_hash::{ConflictStatus, compute_content_hash, detect_conflict};

// ── compute_content_hash ───────────────────────────────────

#[test]
fn hash_deterministic_for_same_inputs() {
    let inputs = ("Title", "Desc", "state", vec!["a".into(), "b".into()]);
    let h1 = compute_content_hash(inputs.0, inputs.1, inputs.2, &inputs.3);
    let h2 = compute_content_hash(inputs.0, inputs.1, inputs.2, &inputs.3);
    assert_eq!(h1, h2);
}

#[test]
fn hash_differs_when_title_changes() {
    let h1 = compute_content_hash("Alpha", "Desc", "s", &[]);
    let h2 = compute_content_hash("Beta", "Desc", "s", &[]);
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_when_description_changes() {
    let h1 = compute_content_hash("T", "First", "s", &[]);
    let h2 = compute_content_hash("T", "Second", "s", &[]);
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_when_state_changes() {
    let h1 = compute_content_hash("T", "D", "created", &[]);
    let h2 = compute_content_hash("T", "D", "implementing", &[]);
    assert_ne!(h1, h2);
}

#[test]
fn hash_differs_when_labels_change() {
    let h1 = compute_content_hash("T", "D", "s", &["bug".into()]);
    let h2 = compute_content_hash("T", "D", "s", &["feature".into()]);
    assert_ne!(h1, h2);
}

#[test]
fn hash_label_order_independent() {
    let h1 = compute_content_hash("T", "D", "s", &["z".into(), "a".into(), "m".into()]);
    let h2 = compute_content_hash("T", "D", "s", &["a".into(), "m".into(), "z".into()]);
    assert_eq!(h1, h2);
}

#[test]
fn hash_empty_all_fields() {
    let h = compute_content_hash("", "", "", &[]);
    assert_eq!(h.len(), 64); // SHA-256 hex is 64 chars
}

#[test]
fn hash_empty_labels_array() {
    let h = compute_content_hash("T", "D", "s", &[]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_single_label() {
    let h = compute_content_hash("T", "D", "s", &["solo".into()]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_many_labels() {
    let labels: Vec<String> = (0..100).map(|i| format!("label-{i}")).collect();
    let h1 = compute_content_hash("T", "D", "s", &labels);
    let h2 = compute_content_hash("T", "D", "s", &labels);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}

#[test]
fn hash_unicode_title() {
    let h = compute_content_hash("日本語タイトル", "", "", &[]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_unicode_labels() {
    let h = compute_content_hash("T", "D", "s", &["标签".into(), "ラベル".into()]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_special_chars_in_description() {
    let h = compute_content_hash("T", "line1\nline2\ttab\rcarriage", "s", &[]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_long_title() {
    let long_title = "x".repeat(10_000);
    let h = compute_content_hash(&long_title, "", "", &[]);
    assert_eq!(h.len(), 64);
}

#[test]
fn hash_duplicate_labels_different_count() {
    let h1 = compute_content_hash("T", "D", "s", &["bug".into()]);
    let h2 = compute_content_hash("T", "D", "s", &["bug".into(), "bug".into()]);
    assert_ne!(h1, h2);
}

#[test]
fn hash_extra_whitespace_in_title() {
    let h1 = compute_content_hash("hello world", "", "", &[]);
    let h2 = compute_content_hash("hello  world", "", "", &[]);
    assert_ne!(h1, h2); // whitespace is significant
}

// ── detect_conflict ────────────────────────────────────────

#[test]
fn conflict_when_both_sides_changed_from_baseline() {
    assert_eq!(
        detect_conflict("baseline", "local_changed", "remote_changed"),
        ConflictStatus::Conflict
    );
}

#[test]
fn clean_when_only_local_changed() {
    assert_eq!(
        detect_conflict("baseline", "local_changed", "baseline"),
        ConflictStatus::Clean
    );
}

#[test]
fn clean_when_only_remote_changed() {
    assert_eq!(
        detect_conflict("baseline", "baseline", "remote_changed"),
        ConflictStatus::Clean
    );
}

#[test]
fn clean_when_neither_changed() {
    assert_eq!(
        detect_conflict("hash_a", "hash_a", "hash_a"),
        ConflictStatus::Clean
    );
}

#[test]
fn both_changed_even_if_same_value_is_conflict() {
    // detect_conflict considers ANY divergence from baseline as conflict,
    // even if local and remote arrived at the same new value.
    assert_eq!(
        detect_conflict("baseline", "new_val", "new_val"),
        ConflictStatus::Conflict
    );
}

#[test]
fn conflict_status_debug_format() {
    let c = ConflictStatus::Conflict;
    assert_eq!(format!("{:?}", c), "Conflict");
    let clean = ConflictStatus::Clean;
    assert_eq!(format!("{:?}", clean), "Clean");
}

#[test]
fn conflict_status_partial_eq() {
    assert_eq!(ConflictStatus::Clean, ConflictStatus::Clean);
    assert_eq!(ConflictStatus::Conflict, ConflictStatus::Conflict);
    assert_ne!(ConflictStatus::Clean, ConflictStatus::Conflict);
}

#[test]
fn conflict_with_empty_hashes() {
    assert_eq!(
        detect_conflict("", "changed", "also_changed"),
        ConflictStatus::Conflict
    );
}

#[test]
fn clean_with_all_empty() {
    assert_eq!(detect_conflict("", "", ""), ConflictStatus::Clean);
}

#[test]
fn both_changed_from_empty_baseline_is_conflict() {
    // Even when baseline is empty and local == remote, the function
    // treats any deviation from baseline as a conflict.
    assert_eq!(
        detect_conflict("", "changed", "changed"),
        ConflictStatus::Conflict
    );
}
