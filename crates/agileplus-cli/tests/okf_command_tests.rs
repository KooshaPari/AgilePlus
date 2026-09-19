// SPDX-License-Identifier: MIT OR Apache-2.0
//! Direct-call tests for `agileplus_cli::commands::okf::run`.
//!
//! These invoke the library entry point in-process (rather than shelling out)
//! so the dispatch (`run`) and the private `validate_cmd` / `summarize_cmd` /
//! `merge_cmd` / `load_document` paths are all exercised. File I/O is isolated
//! to tempdirs; no DB, no network.

use std::path::{Path, PathBuf};

use agileplus_cli::commands::okf::{OkfArgs, OkfSubcommand, run};

fn args_validate(path: &Path) -> OkfArgs {
    OkfArgs {
        sub: OkfSubcommand::Validate {
            path: path.to_path_buf(),
        },
    }
}

fn args_summarize(path: &Path, top: usize) -> OkfArgs {
    OkfArgs {
        sub: OkfSubcommand::Summarize {
            path: path.to_path_buf(),
            top,
        },
    }
}

fn args_merge(paths: Vec<PathBuf>, output: Option<PathBuf>) -> OkfArgs {
    OkfArgs {
        sub: OkfSubcommand::Merge { paths, output },
    }
}

fn write_file(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).expect("write fixture");
    p
}

const GOOD: &str = r#"{
  "okf": "1.0",
  "source_id": "src-good",
  "entities": [
    { "id": "intent-0", "type": "intent", "label": "do the thing" },
    { "id": "acceptance-0", "type": "acceptance", "label": "it works" }
  ],
  "relations": [
    { "source": "intent-0", "target": "acceptance-0", "type": "verified_by",
      "provenance": { "corpus": "forge", "source_id": "src-good" } }
  ],
  "provenance": { "corpus": "forge", "source_id": "src-good" }
}"#;

const SECOND: &str = r#"{
  "okf": "1.0",
  "source_id": "src-second",
  "entities": [
    { "id": "intent-0", "type": "intent", "label": "second intent" }
  ],
  "provenance": { "corpus": "codex", "source_id": "src-second" }
}"#;

// ── validate ─────────────────────────────────────────────────────────────────

#[test]
fn validate_good_document_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "good.okf.json", GOOD);
    assert_eq!(run(&args_validate(&p)).unwrap(), 0);
}

#[test]
fn validate_unsupported_version_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"2.0","source_id":"s","entities":[],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "v.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_unknown_document_corpus_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[],
        "provenance":{"corpus":"not-a-corpus","source_id":"s"}}"#;
    let p = write_file(dir.path(), "c.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_empty_provenance_and_source_id_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"   ","entities":[],
        "provenance":{"corpus":"forge","source_id":"  "}}"#;
    let p = write_file(dir.path(), "empty-ids.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_duplicate_entity_id_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[
        {"id":"e1","type":"intent","label":"a"},
        {"id":"e1","type":"intent","label":"b"}],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "dup.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_empty_entity_id_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[
        {"id":"  ","type":"intent","label":"a"}],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "eid.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_unknown_entity_type_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[
        {"id":"e1","type":"bogus-type","label":"a"}],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "etype.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_dangling_relation_endpoints_exit_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[
        {"id":"e1","type":"intent","label":"a"}],
        "relations":[{"source":"missing","target":"alsomissing",
          "type":"verified_by","provenance":{"corpus":"forge","source_id":"s"}}],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "dangling.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_relation_type_and_corpus_errors_exit_one() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[
        {"id":"e1","type":"intent","label":"a"},
        {"id":"e2","type":"intent","label":"b"}],
        "relations":[{"source":"e1","target":"e2","type":"bogus-rel",
          "provenance":{"corpus":"bogus-corpus","source_id":"s"}}],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "rel.okf.json", body);
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_missing_file_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("nope.okf.json");
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_empty_file_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "empty.okf.json", "   \n");
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_malformed_json_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "bad.okf.json", "{ not json ");
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_jsonl_single_object_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    // The JSONL loader consumes one object per line, so the fixture must be
    // compact (single line) rather than pretty-printed.
    let compact = r#"{"okf":"1.0","source_id":"src-jl","entities":[{"id":"e1","type":"intent","label":"a"}],"provenance":{"corpus":"forge","source_id":"src-jl"}}"#;
    let p = write_file(dir.path(), "one.okf.jsonl", &format!("{compact}\n"));
    assert_eq!(run(&args_validate(&p)).unwrap(), 0);
}

#[test]
fn validate_jsonl_multiple_objects_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let first = r#"{"okf":"1.0","source_id":"a","entities":[],"provenance":{"corpus":"forge","source_id":"a"}}"#;
    let second = r#"{"okf":"1.0","source_id":"b","entities":[],"provenance":{"corpus":"codex","source_id":"b"}}"#;
    let p = write_file(dir.path(), "two.okf.jsonl", &format!("{first}\n{second}\n"));
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_jsonl_comment_only_blank_lines_exit_one() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "blank.okf.jsonl", "\n  \n");
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

#[test]
fn validate_jsonl_malformed_line_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "bad.okf.jsonl", "{ nope\n");
    assert_eq!(run(&args_validate(&p)).unwrap(), 1);
}

// ── summarize ────────────────────────────────────────────────────────────────

#[test]
fn summarize_valid_document_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "s.okf.json", GOOD);
    assert_eq!(run(&args_summarize(&p, 5)).unwrap(), 0);
}

#[test]
fn summarize_top_zero_skips_label_section() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_file(dir.path(), "s0.okf.json", GOOD);
    assert_eq!(run(&args_summarize(&p, 0)).unwrap(), 0);
}

#[test]
fn summarize_empty_entities_and_relations_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "e.okf.json", body);
    assert_eq!(run(&args_summarize(&p, 3)).unwrap(), 0);
}

#[test]
fn summarize_model_distribution_branch_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[
        {"id":"e1","type":"resource","label":"wd","properties":{"model":"sonnet"}},
        {"id":"e2","type":"resource","label":"wd2","properties":{"model":"sonnet"}}],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p = write_file(dir.path(), "m.okf.json", body);
    assert_eq!(run(&args_summarize(&p, 5)).unwrap(), 0);
}

#[test]
fn summarize_missing_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("missing.okf.json");
    assert!(run(&args_summarize(&p, 5)).is_err());
}

// ── merge ────────────────────────────────────────────────────────────────────

#[test]
fn merge_two_documents_namespaces_ids_and_writes_output() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = write_file(dir.path(), "a.okf.json", GOOD);
    let p2 = write_file(dir.path(), "b.okf.json", SECOND);
    let out = dir.path().join("merged.okf.json");

    assert_eq!(
        run(&args_merge(vec![p1, p2], Some(out.clone()))).unwrap(),
        0
    );

    let text = std::fs::read_to_string(&out).expect("merged output written");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let entities = v["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 3, "2 + 1 entities");
    let ids: std::collections::HashSet<&str> =
        entities.iter().filter_map(|e| e["id"].as_str()).collect();
    assert_eq!(ids.len(), 3, "ids must be collision-free");
    // Relations were rewritten to point at namespaced ids.
    let rels = v["relations"].as_array().unwrap();
    for r in rels {
        assert!(ids.contains(r["source"].as_str().unwrap()));
        assert!(ids.contains(r["target"].as_str().unwrap()));
    }
}

#[test]
fn merge_colliding_corpus_source_id_gets_numeric_suffix() {
    let dir = tempfile::tempdir().unwrap();
    // Same (corpus, source_id) twice -> second input's tag gets `_2`.
    let p1 = write_file(dir.path(), "a.okf.json", GOOD);
    let p2 = write_file(dir.path(), "b.okf.json", GOOD);
    let out = dir.path().join("merged.okf.json");

    assert_eq!(
        run(&args_merge(vec![p1, p2], Some(out.clone()))).unwrap(),
        0
    );
    let text = std::fs::read_to_string(&out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let ids: Vec<&str> = v["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["id"].as_str())
        .collect();
    assert_eq!(ids.len(), 4, "both copies retained");
    // One tag carries the `_2` suffix.
    assert!(
        ids.iter().any(|id| id.contains("_2::")),
        "expected a _2-suffixed namespace, saw {ids:?}"
    );
}

#[test]
fn merge_relation_referencing_unknown_entity_uses_prefix_fallback() {
    let dir = tempfile::tempdir().unwrap();
    // A relation whose source is not an entity in the same doc takes the
    // fallback `{tag}::{old_id}` path in merge_cmd.
    let body = r#"{"okf":"1.0","source_id":"orphan-src","entities":[
        {"id":"e1","type":"intent","label":"a"}],
        "relations":[{"source":"ghost","target":"e1","type":"requires",
          "provenance":{"corpus":"forge","source_id":"orphan-src"}}],
        "provenance":{"corpus":"forge","source_id":"orphan-src"}}"#;
    let p1 = write_file(dir.path(), "orphan.okf.json", body);
    let p2 = write_file(dir.path(), "b.okf.json", SECOND);
    let out = dir.path().join("merged.okf.json");

    assert_eq!(
        run(&args_merge(vec![p1, p2], Some(out.clone()))).unwrap(),
        0
    );
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("ghost"), "fallback relation preserved: {text}");
}

#[test]
fn merge_to_stdout_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = write_file(dir.path(), "a.okf.json", GOOD);
    let p2 = write_file(dir.path(), "b.okf.json", SECOND);
    assert_eq!(run(&args_merge(vec![p1, p2], None)).unwrap(), 0);
}

#[test]
fn merge_single_path_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = write_file(dir.path(), "a.okf.json", GOOD);
    assert!(run(&args_merge(vec![p1], None)).is_err());
}

#[test]
fn merge_empty_documents_refused() {
    let dir = tempfile::tempdir().unwrap();
    let body = r#"{"okf":"1.0","source_id":"s","entities":[],
        "provenance":{"corpus":"forge","source_id":"s"}}"#;
    let p1 = write_file(dir.path(), "empty1.okf.json", body);
    let p2 = write_file(dir.path(), "empty2.okf.json", body);
    let out = dir.path().join("merged.okf.json");
    assert!(run(&args_merge(vec![p1, p2], Some(out))).is_err());
}

#[test]
fn merge_missing_input_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = write_file(dir.path(), "a.okf.json", GOOD);
    let missing = dir.path().join("missing.okf.json");
    assert!(run(&args_merge(vec![p1, missing], None)).is_err());
}
