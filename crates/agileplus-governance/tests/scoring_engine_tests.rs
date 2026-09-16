//! Integration tests for the scoring engine: evaluate, render_markdown, probes.
//! Complements the inline unit tests in src/scoring_engine.rs.

use agileplus_governance::*;
use agileplus_governance::scoring_engine::{ProbeRule, ProbeEvidence, TaggedProbeEvidence, SCORING_PROBES};
use std::collections::BTreeMap;

fn _scoring_spec() -> ScoringSpec {
    let mut glyphs = BTreeMap::new();
    glyphs.insert("0".into(), "\u{2717}".into());
    glyphs.insert("1".into(), "\u{25B3}".into());
    glyphs.insert("2".into(), "~".into());
    glyphs.insert("3".into(), "\u{2713}".into());
    let mut grade = BTreeMap::new();
    grade.insert("A".into(), 90);
    grade.insert("B".into(), 75);
    grade.insert("C".into(), 60);
    grade.insert("D".into(), 40);
    grade.insert("F".into(), 0);
    ScoringSpec {
        scale: "0-3".into(),
        glyphs,
        grade,
    }
}

fn _minimal_catalog() -> RubricCatalog {
    let json = r#"{
        "version": "1.0",
        "schema": "test",
        "clusters": 2,
        "sub_pillars_enumerated": 1,
        "note": "test",
        "pillars": [
            {
                "cluster": "C00", "pillar_range": "L0-L9", "category": "Architecture",
                "source": "x/", "defs_ref": "ref.md",
                "scoring": {"scale": "0-3", "glyphs": {"0": "\u{2717}", "3": "\u{2713}"}, "grade": {"A": 90, "F": 0}},
                "sub_pillars": []
            },
            {
                "cluster": "C03", "pillar_range": "L30", "category": "Agent Readiness",
                "source": "y/",
                "scoring": {"scale": "0-3", "glyphs": {"0": "\u{2717}", "3": "\u{2713}"}, "grade": {"A": 90, "F": 0}},
                "sub_pillars": [
                    {"id": "L30.1", "title": "Spec Clarity", "acceptance": "FR.md exists", "evidence_pattern": "file:line"}
                ]
            }
        ]
    }"#;
    RubricCatalog::from_json(json).unwrap()
}

// ── ProbeRule ────────────────────────────────────────────────────────

#[test]
fn probe_rule_compiles_valid_regex() {
    let rule = ProbeRule {
        cluster: "C01",
        rule_text: "test",
        target_file: "test.toml",
        regex_src: r"(?m)^\[advisories\]",
    };
    assert!(rule.compiled().is_ok());
}

#[test]
fn probe_rule_compilation_error_for_invalid_regex() {
    let rule = ProbeRule {
        cluster: "C01",
        rule_text: "test",
        target_file: "test.toml",
        regex_src: r"[invalid",
    };
    assert!(rule.compiled().is_err());
}

// ── ProbeEvidence ────────────────────────────────────────────────────

#[test]
fn probe_evidence_collect_matches() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("deny.toml"),
        "[advisories]\nignore = true\n",
    )
    .unwrap();
    let probes = &[ProbeRule {
        cluster: "C01",
        rule_text: "Has cargo-deny",
        target_file: "deny.toml",
        regex_src: r"(?m)^\[advisories\]",
    }];
    let evidence = ProbeEvidence::collect(dir.path(), probes);
    assert_eq!(evidence.matches.len(), 1);
    assert_eq!(evidence.matches[0].0, "Has cargo-deny");
}

#[test]
fn probe_evidence_skips_missing_files() {
    let dir = tempfile::tempdir().unwrap();
    let probes = &[ProbeRule {
        cluster: "C01",
        rule_text: "missing",
        target_file: "nonexistent.toml",
        regex_src: r"foo",
    }];
    let evidence = ProbeEvidence::collect(dir.path(), probes);
    assert!(evidence.matches.is_empty());
}

// ── TaggedProbeEvidence ──────────────────────────────────────────────

#[test]
fn tagged_probe_evidence_count_for() {
    let ev = TaggedProbeEvidence {
        matches: vec![
            ("C01", "rule1", "file1", "evidence1".into()),
            ("C01", "rule2", "file2", "evidence2".into()),
            ("C04", "rule3", "file3", "evidence3".into()),
        ],
    };
    assert_eq!(ev.count_for("C01"), 2);
    assert_eq!(ev.count_for("C04"), 1);
    assert_eq!(ev.count_for("C99"), 0);
}

// ── render_markdown ──────────────────────────────────────────────────

#[test]
fn render_markdown_empty_report() {
    let report = ScoreReport {
        repo: "empty-repo".into(),
        date: "2024-01-01".into(),
        clusters: vec![],
    };
    let md = render_markdown(&report);
    assert!(md.is_empty());
}

#[test]
fn render_markdown_contains_cluster_start() {
    let report = ScoreReport {
        repo: "test-repo".into(),
        date: "2024-01-01".into(),
        clusters: vec![ClusterScore {
            cluster: "C03".into(),
            pillars: vec![PillarScore {
                pillar_id: "L30".into(),
                title: "Agent Readiness".into(),
                score: 2,
                glyph: "~",
                evidence: vec!["AGENTS.md:1".into()],
                gaps: vec![],
                soft_goal_delta: "notable".into(),
            }],
            total_points: 2,
            max_points: 3,
        }],
    };
    let md = render_markdown(&report);
    assert!(md.contains("CLUSTER_START cluster=C03 repo=test-repo"));
    assert!(md.contains("### L30"));
    assert!(md.contains("CLUSTER_TOTAL"));
    assert!(md.contains("CLUSTER_DONE"));
}

#[test]
fn render_markdown_shows_no_evidence_when_empty() {
    let report = ScoreReport {
        repo: "r".into(),
        date: "d".into(),
        clusters: vec![ClusterScore {
            cluster: "C00".into(),
            pillars: vec![PillarScore {
                pillar_id: "L0".into(),
                title: "T".into(),
                score: 0,
                glyph: "\u{2717}",
                evidence: vec![],
                gaps: vec![],
                soft_goal_delta: "partial".into(),
            }],
            total_points: 0,
            max_points: 3,
        }],
    };
    let md = render_markdown(&report);
    assert!(md.contains("(no evidence found)"));
    assert!(md.contains("gaps:\n  - none"));
}

#[test]
fn render_markdown_shows_gaps() {
    let report = ScoreReport {
        repo: "r".into(),
        date: "d".into(),
        clusters: vec![ClusterScore {
            cluster: "C00".into(),
            pillars: vec![PillarScore {
                pillar_id: "L0".into(),
                title: "T".into(),
                score: 1,
                glyph: "\u{25B3}",
                evidence: vec!["evidence1".into()],
                gaps: vec!["Missing CLAUDE.md".into()],
                soft_goal_delta: "partial".into(),
            }],
            total_points: 1,
            max_points: 3,
        }],
    };
    let md = render_markdown(&report);
    assert!(md.contains("Missing CLAUDE.md"));
    assert!(!md.contains("gaps:\n  - none"));
}

// ── ScoreReport / ClusterScore / PillarScore structures ──────────────

#[test]
fn score_report_struct_fields() {
    let report = ScoreReport {
        repo: "my-repo".into(),
        date: "2024-06-15".into(),
        clusters: vec![ClusterScore {
            cluster: "C00".into(),
            pillars: vec![PillarScore {
                pillar_id: "L0".into(),
                title: "Test".into(),
                score: 3,
                glyph: "\u{2713}",
                evidence: vec!["file:1".into()],
                gaps: vec![],
                soft_goal_delta: "complete".into(),
            }],
            total_points: 3,
            max_points: 3,
        }],
    };
    assert_eq!(report.repo, "my-repo");
    assert_eq!(report.clusters.len(), 1);
    assert_eq!(report.clusters[0].total_points, 3);
    assert_eq!(report.clusters[0].max_points, 3);
    assert_eq!(report.clusters[0].pillars[0].score, 3);
    assert_eq!(report.clusters[0].pillars[0].glyph, "\u{2713}");
}

#[test]
fn pillar_score_evidence_and_gaps() {
    let ps = PillarScore {
        pillar_id: "L30".into(),
        title: "Agent Readiness".into(),
        score: 1,
        glyph: "\u{25B3}",
        evidence: vec!["AGENTS.md:1".into(), "CLAUDE.md:1".into()],
        gaps: vec!["Missing FR doc".into()],
        soft_goal_delta: "notable — partial".into(),
    };
    assert_eq!(ps.evidence.len(), 2);
    assert_eq!(ps.gaps.len(), 1);
    assert_eq!(ps.soft_goal_delta, "notable — partial");
}

// ── SCORING_PROBES ───────────────────────────────────────────────────

#[test]
fn scoring_probes_has_at_least_five() {
    assert!(SCORING_PROBES.len() >= 5);
}

#[test]
fn scoring_probes_clusters_cover_required() {
    let clusters: BTreeMap<&str, bool> = SCORING_PROBES
        .iter()
        .map(|p| (p.cluster, true))
        .collect();
    for required in ["C01", "C04", "C05", "C08", "C11"] {
        assert!(clusters.contains_key(required), "missing probe for {required}");
    }
}

#[test]
fn scoring_probes_all_compile() {
    for p in SCORING_PROBES {
        p.compiled()
            .unwrap_or_else(|e| panic!("probe {} failed: {e}", p.rule_text));
    }
}

#[test]
fn scoring_probes_target_files_are_nonempty() {
    for p in SCORING_PROBES {
        assert!(
            !p.target_file.is_empty(),
            "probe {} has empty target_file",
            p.rule_text
        );
    }
}

// ── grade_for ────────────────────────────────────────────────────────

#[test]
fn grade_boundaries() {
    // grade_for is private but exercised via render_markdown CLUSTER_TOTAL lines
    // We can test it indirectly through render_markdown output
    let report = ScoreReport {
        repo: "r".into(),
        date: "d".into(),
        clusters: vec![ClusterScore {
            cluster: "C00".into(),
            pillars: vec![PillarScore {
                pillar_id: "L0".into(),
                title: "T".into(),
                score: 3,
                glyph: "\u{2713}",
                evidence: vec![],
                gaps: vec![],
                soft_goal_delta: "complete".into(),
            }],
            total_points: 3,
            max_points: 3, // 100% -> A
        }],
    };
    let md = render_markdown(&report);
    assert!(md.contains("pct=100% grade=A"));
}

#[test]
fn grade_f_for_low_score() {
    let report = ScoreReport {
        repo: "r".into(),
        date: "d".into(),
        clusters: vec![ClusterScore {
            cluster: "C00".into(),
            pillars: vec![PillarScore {
                pillar_id: "L0".into(),
                title: "T".into(),
                score: 0,
                glyph: "\u{2717}",
                evidence: vec![],
                gaps: vec![],
                soft_goal_delta: "partial".into(),
            }],
            total_points: 0,
            max_points: 3, // 0% -> F
        }],
    };
    let md = render_markdown(&report);
    assert!(md.contains("pct=0% grade=F"));
}

// ── Multiple clusters in report ──────────────────────────────────────

#[test]
fn render_markdown_multiple_clusters() {
    let report = ScoreReport {
        repo: "multi".into(),
        date: "2024-01-01".into(),
        clusters: vec![
            ClusterScore {
                cluster: "C00".into(),
                pillars: vec![PillarScore {
                    pillar_id: "L0".into(),
                    title: "Arch".into(),
                    score: 3,
                    glyph: "\u{2713}",
                    evidence: vec![],
                    gaps: vec![],
                    soft_goal_delta: "complete".into(),
                }],
                total_points: 3,
                max_points: 3,
            },
            ClusterScore {
                cluster: "C03".into(),
                pillars: vec![PillarScore {
                    pillar_id: "L30".into(),
                    title: "Agent".into(),
                    score: 1,
                    glyph: "\u{25B3}",
                    evidence: vec!["file:1".into()],
                    gaps: vec!["gap".into()],
                    soft_goal_delta: "partial".into(),
                }],
                total_points: 1,
                max_points: 3,
            },
        ],
    };
    let md = render_markdown(&report);
    assert!(md.contains("cluster=C00"));
    assert!(md.contains("cluster=C03"));
    // Both CLUSTER_START and CLUSTER_DONE should appear twice
    assert_eq!(md.matches("CLUSTER_START").count(), 2);
    assert_eq!(md.matches("CLUSTER_DONE").count(), 2);
}
