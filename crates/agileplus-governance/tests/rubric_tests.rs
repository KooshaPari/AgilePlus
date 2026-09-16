//! Integration tests for RubricCatalog, Pillar, SubPillar, ScoringSpec.
//! Complements the inline unit tests in src/rubric.rs.

use agileplus_governance::*;
use std::collections::BTreeMap;

fn make_scoring_spec() -> ScoringSpec {
    let mut glyphs = BTreeMap::new();
    glyphs.insert("0".into(), "x".into());
    glyphs.insert("3".into(), "v".into());
    let mut grade = BTreeMap::new();
    grade.insert("A".into(), 90);
    grade.insert("F".into(), 0);
    ScoringSpec {
        scale: "0-3".into(),
        glyphs,
        grade,
    }
}

fn make_catalog_json(cluster_count: usize, sub_pillars: &[&str]) -> String {
    let sub_pillars_json: String = sub_pillars
        .iter()
        .map(|id| {
            format!(
                r#"{{"id": "{}", "title": "Title {}", "acceptance": "accept", "evidence_pattern": "file:line"}}"#,
                id, id
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let sub_pillar_count = sub_pillars.len();

    // Build pillars array: first cluster has defs_ref, rest have sub_pillars
    let mut pillars = Vec::new();
    if cluster_count > 0 {
        pillars.push(format!(
            r#"{{"cluster": "C00", "pillar_range": "L0-L9", "category": "Arch", "source": "x/", "defs_ref": "ref.md", "scoring": {{"scale": "0-3", "glyphs": {{"0": "x"}}, "grade": {{"A": 90}}}}, "sub_pillars": []}}"#
        ));
    }
    for i in 1..cluster_count {
        pillars.push(format!(
            r#"{{"cluster": "C{:02}", "pillar_range": "L{}", "category": "Cat{}", "source": "y/", "scoring": {{"scale": "0-3", "glyphs": {{"0": "x", "3": "v"}}, "grade": {{"A": 90, "F": 0}}}}, "sub_pillars": [{}]}}"#,
            i, i * 10, i, sub_pillars_json
        ));
    }

    format!(
        r#"{{"version": "1.0", "schema": "test", "clusters": {}, "sub_pillars_enumerated": {}, "note": "test", "pillars": [{}]}}"#,
        cluster_count,
        sub_pillar_count,
        pillars.join(",")
    )
}

// ── Parsing and validation ───────────────────────────────────────────

#[test]
fn catalog_parses_valid_json() {
    let json = make_catalog_json(2, &["L10.1"]);
    let catalog = RubricCatalog::from_json(&json).unwrap();
    assert_eq!(catalog.clusters, 2);
    assert_eq!(catalog.version, "1.0");
    assert_eq!(catalog.schema, "test");
}

#[test]
fn catalog_rejects_invalid_json() {
    assert!(RubricCatalog::from_json("{bad").is_err());
}

#[test]
fn catalog_rejects_cluster_count_mismatch() {
    let mut json = make_catalog_json(2, &[]);
    json = json.replace("\"clusters\": 2", "\"clusters\": 5");
    assert!(RubricCatalog::from_json(&json).is_err());
}

#[test]
fn catalog_rejects_sub_pillar_count_mismatch() {
    let mut json = make_catalog_json(2, &[]);
    json = json.replace("\"sub_pillars_enumerated\": 0", "\"sub_pillars_enumerated\": 3");
    assert!(RubricCatalog::from_json(&json).is_err());
}

#[test]
fn catalog_validate_unique_cluster_ids() {
    let mut json = make_catalog_json(2, &[]);
    // Make both clusters have same id
    json = json.replace("\"cluster\": \"C01\"", "\"cluster\": \"C00\"");
    assert!(RubricCatalog::from_json(&json).is_err());
}

#[test]
fn catalog_validate_rejects_empty_glyphs() {
    let json = r#"{
        "version": "1.0",
        "schema": "test",
        "clusters": 1,
        "sub_pillars_enumerated": 0,
        "note": "",
        "pillars": [{
            "cluster": "C00", "pillar_range": "L0", "category": "X",
            "source": "x/", "defs_ref": "ref.md",
            "scoring": {"scale": "0-3", "glyphs": {}, "grade": {"A": 90}},
            "sub_pillars": []
        }]
    }"#;
    assert!(RubricCatalog::from_json(json).is_err());
}

#[test]
fn catalog_validate_rejects_empty_grade() {
    let json = r#"{
        "version": "1.0",
        "schema": "test",
        "clusters": 1,
        "sub_pillars_enumerated": 0,
        "note": "",
        "pillars": [{
            "cluster": "C00", "pillar_range": "L0", "category": "X",
            "source": "x/", "defs_ref": "ref.md",
            "scoring": {"scale": "0-3", "glyphs": {"0": "x"}, "grade": {}},
            "sub_pillars": []
        }]
    }"#;
    assert!(RubricCatalog::from_json(json).is_err());
}

#[test]
fn catalog_validate_rejects_no_sub_pillars_no_defs_ref() {
    let json = r#"{
        "version": "1.0",
        "schema": "test",
        "clusters": 1,
        "sub_pillars_enumerated": 0,
        "note": "",
        "pillars": [{
            "cluster": "C00", "pillar_range": "L0", "category": "X",
            "source": "x/",
            "scoring": {"scale": "0-3", "glyphs": {"0": "x"}, "grade": {"A": 90}},
            "sub_pillars": []
        }]
    }"#;
    assert!(RubricCatalog::from_json(json).is_err());
}

#[test]
fn catalog_validate_rejects_duplicate_sub_pillar_ids() {
    let json = r#"{
        "version": "1.0",
        "schema": "test",
        "clusters": 1,
        "sub_pillars_enumerated": 2,
        "note": "",
        "pillars": [{
            "cluster": "C00", "pillar_range": "L0", "category": "X",
            "source": "x/",
            "scoring": {"scale": "0-3", "glyphs": {"0": "x", "3": "v"}, "grade": {"A": 90, "F": 0}},
            "sub_pillars": [
                {"id": "L0.1", "title": "A", "evidence_pattern": ""},
                {"id": "L0.1", "title": "B", "evidence_pattern": ""}
            ]
        }]
    }"#;
    assert!(RubricCatalog::from_json(json).is_err());
}

// ── Lookup ───────────────────────────────────────────────────────────

#[test]
fn catalog_cluster_lookup() {
    // Use serde directly to bypass validation for a minimal lookup test
    let json = r#"{
        "version": "1.0",
        "schema": "test",
        "clusters": 3,
        "sub_pillars_enumerated": 0,
        "note": "lookup test",
        "pillars": [
            {"cluster": "C00", "pillar_range": "L0-L9", "category": "Arch", "source": "x/", "defs_ref": "ref.md", "scoring": {"scale": "0-3", "glyphs": {"0": "x"}, "grade": {"A": 90}}},
            {"cluster": "C01", "pillar_range": "L10", "category": "Cat1", "source": "y/", "defs_ref": "ref2.md", "scoring": {"scale": "0-3", "glyphs": {"0": "x", "3": "v"}, "grade": {"A": 90, "F": 0}}},
            {"cluster": "C02", "pillar_range": "L20", "category": "Cat2", "source": "y/", "defs_ref": "ref3.md", "scoring": {"scale": "0-3", "glyphs": {"0": "x", "3": "v"}, "grade": {"A": 90, "F": 0}}}
        ]
    }"#;
    let catalog: RubricCatalog = serde_json::from_str(json).unwrap();
    assert!(catalog.cluster("C00").is_some());
    assert!(catalog.cluster("C01").is_some());
    assert!(catalog.cluster("C02").is_some());
    assert!(catalog.cluster("C99").is_none());
}

#[test]
fn catalog_enumerated_count() {
    let json = make_catalog_json(2, &["L10.1", "L10.2"]);
    let catalog = RubricCatalog::from_json(&json).unwrap();
    assert_eq!(catalog.enumerated_count(), 2);
}

#[test]
fn catalog_enumerated_count_zero_when_no_sub_pillars() {
    let json = make_catalog_json(1, &[]);
    let catalog = RubricCatalog::from_json(&json).unwrap();
    assert_eq!(catalog.enumerated_count(), 0);
}

// ── Serde roundtrip ──────────────────────────────────────────────────

#[test]
fn catalog_serde_roundtrip() {
    let json = make_catalog_json(2, &["L10.1"]);
    let catalog = RubricCatalog::from_json(&json).unwrap();
    let serialized = serde_json::to_string(&catalog).unwrap();
    let back: RubricCatalog = serde_json::from_str(&serialized).unwrap();
    assert_eq!(back.clusters, catalog.clusters);
    assert_eq!(back.pillars.len(), catalog.pillars.len());
    assert_eq!(back.version, catalog.version);
}

// ── Pillar and SubPillar serde ───────────────────────────────────────

#[test]
fn pillar_serde_roundtrip() {
    let pillar = Pillar {
        cluster: "C05".into(),
        pillar_range: "L50".into(),
        category: "Observability".into(),
        source: "audit/".into(),
        defs_ref: Some("ref.md".into()),
        scoring: make_scoring_spec(),
        sub_pillars: vec![],
    };
    let json = serde_json::to_string(&pillar).unwrap();
    let back: Pillar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cluster, "C05");
    assert_eq!(back.category, "Observability");
    assert!(back.defs_ref.is_some());
}

#[test]
fn sub_pillar_serde_roundtrip() {
    let sp = SubPillar {
        id: "L30.1".into(),
        title: "Agent Readiness".into(),
        name: Some("FR Clarity".into()),
        acceptance: Some("docs/FR.md exists".into()),
        soft_goal: Some("85% traceability".into()),
        evidence_pattern: "file:line".into(),
    };
    let json = serde_json::to_string(&sp).unwrap();
    let back: SubPillar = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "L30.1");
    assert_eq!(back.title, "Agent Readiness");
    assert_eq!(back.name.as_deref(), Some("FR Clarity"));
    assert_eq!(back.acceptance.as_deref(), Some("docs/FR.md exists"));
}

#[test]
fn scoring_spec_serde_roundtrip() {
    let spec = make_scoring_spec();
    let json = serde_json::to_string(&spec).unwrap();
    let back: ScoringSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scale, "0-3");
    assert_eq!(back.glyphs.len(), 2);
    assert_eq!(back.grade.len(), 2);
}

// ── Load from file ───────────────────────────────────────────────────

#[test]
fn catalog_load_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.json");
    let json = make_catalog_json(1, &[]);
    std::fs::write(&path, &json).unwrap();
    let catalog = RubricCatalog::load(&path).unwrap();
    assert_eq!(catalog.clusters, 1);
}

#[test]
fn catalog_load_from_nonexistent_file() {
    let result = RubricCatalog::load("/nonexistent/path/catalog.json");
    assert!(result.is_err());
}
