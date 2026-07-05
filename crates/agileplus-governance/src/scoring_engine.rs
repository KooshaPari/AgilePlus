//! Scoring engine: consume a [`RubricCatalog`] + [`RepoScan`] + optional
//! heuristic rules, produce a v38-formatted scorecard markdown.
//!
//! This is the orchestrator that closes the gap between the rubric
//! parser (P2.1 in PR #893) and the code scanner (P2.2 in PR #893).
//! The output is exactly the v38 scorecard format that
//! `phenotype-org-audits/audit-v38/catalog/` defines — so an
//! operator can run a score locally and diff it against the
//! canonical `phenotype-org-audits/audit-v38/output/<repo>/<cluster>.md`.
//!
//! SpecKitty was a standalone enforcement engine; AgilePlus is the
//! owned successor. SpecKitty's `score --repo ...` workflow maps to
//! `agileplus-cli`'s `ap rubric score --repo ...`. See
//! `docs/design/SPECKITTY-MIGRATION.md` for the full mapping.

use std::fmt::Write as _;
use std::path::Path;

use crate::code_scanner::{scan_repo, RepoScan};
use crate::error::Result;
use crate::rubric::{Pillar, RubricCatalog, ScoringSpec, SubPillar};

/// Default scoring heuristic for a single evidence fact.
///
/// The heuristic is intentionally tiny (we ship heuristics into the
/// orchestrator, not a plug-in engine) — the rule registry lives in
/// [`SCORING_RULES`] and is keyed by the cluster/cluster-pillar id.
#[derive(Debug, Clone)]
pub struct ClusterScore {
    /// Cluster id, e.g. "C03".
    pub cluster: String,
    /// Pillar(s) scored, in cluster order.
    pub pillars: Vec<PillarScore>,
    /// Sum of pillar scores.
    pub total_points: u32,
    /// Maximum possible (pillars × 3).
    pub max_points: u32,
}

/// Per-pillar scoring result.
#[derive(Debug, Clone)]
pub struct PillarScore {
    /// Pillar id from the catalog, e.g. "L30" or "L31-L40".
    pub pillar_id: String,
    /// Display title from the catalog (e.g. "L30 — Agent Readiness").
    pub title: String,
    /// Score 0-3.
    pub score: u32,
    /// 0=✗, 1=△, 2=~, 3=✓.
    pub glyph: &'static str,
    /// File:line evidence applied to this pillar.
    pub evidence: Vec<String>,
    /// Open gaps with explicit effort (S/M/L).
    pub gaps: Vec<String>,
    /// `soft_goal_delta` summary.
    pub soft_goal_delta: String,
}

/// Top-level orchestrator output.
#[derive(Debug, Clone)]
pub struct ScoreReport {
    /// Repo display name (taken from the path basename if not explicit).
    pub repo: String,
    /// ISO 8601 date the score was produced.
    pub date: String,
    /// Per-cluster scores, in catalog order.
    pub clusters: Vec<ClusterScore>,
}

/// Run the orchestrator: load rubric + scan target, emit a ScoreReport.
pub fn evaluate<R: AsRef<Path>, C: AsRef<Path>>(
    repo: R,
    catalog: C,
    cluster_filter: &[String],
) -> Result<ScoreReport> {
    let catalog = RubricCatalog::load(catalog)?;
    catalog.validate()?;

    let repo_path = repo.as_ref();
    let repo_name = repo_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());

    let scan = scan_repo(repo_path)?;
    let date = current_iso_date();

    let clusters: Vec<ClusterScore> = catalog
        .pillars
        .iter()
        .filter(|p| cluster_filter.is_empty() || cluster_filter.iter().any(|c| c == &p.cluster))
        .map(|p| score_cluster(p, &scan, repo_path))
        .collect();

    Ok(ScoreReport {
        repo: repo_name,
        date,
        clusters,
    })
}

/// All rules whose cluster_id matches `cluster`.
fn rules_for(cluster: &str) -> Vec<(&'static str, &'static str, &'static [&'static str])> {
    SCORING_RULES
        .iter()
        .copied()
        .filter(|(c, _, _)| *c == cluster)
        .collect()
}

/// Score one cluster against the scan evidence.
fn score_cluster(pillar: &Pillar, scan: &RepoScan, _repo: &Path) -> ClusterScore {
    let rules = rules_for(&pillar.cluster);
    let rules_slice: &[(&str, &str, &[&str])] = &rules;

    let pillar_scores: Vec<PillarScore> = if !pillar.sub_pillars.is_empty() {
        // Enumerated sub-pillars (L30.x, L81.x, L96.x, L108.x, etc.).
        pillar
            .sub_pillars
            .iter()
            .map(|sp| score_sub_pillar(pillar, sp, scan, rules_slice))
            .collect()
    } else {
        // No enumerated sub-pillars — score the cluster as one aggregate pillar.
        vec![score_pillar_aggregate(pillar, scan, rules_slice)]
    };

    let total: u32 = pillar_scores.iter().map(|p| p.score).sum();
    let max = pillar_scores.len() as u32 * 3;

    ClusterScore {
        cluster: pillar.cluster.clone(),
        pillars: pillar_scores,
        total_points: total,
        max_points: max,
    }
}

/// Score one enumerated sub-pillar.
fn score_sub_pillar(
    cluster: &Pillar,
    sp: &SubPillar,
    scan: &RepoScan,
    rules: &[(&str, &str, &[&str])],
) -> PillarScore {
    // Run every matching rule for this pillar-id.
    let mut score: u32 = 0;
    let mut evidence: Vec<String> = Vec::new();
    let mut gaps: Vec<String> = Vec::new();

    for (rule_pillar, rule_text, rule_presence) in rules {
        if !rule_pillar.contains(&sp.id) {
            continue;
        }
        // Coarse heuristic: count present evidence signals; map 0..=rule_presence.len() to 0..=3.
        let present = rule_presence
            .iter()
            .filter(|rel| scan.has(rel))
            .count();
        let total = rule_presence.len();
        let rule_score = match (present, total) {
            (0, _) => 0,
            (p, t) if p == t => 3,
            (p, t) if p * 2 >= t => 2,
            _ => 1,
        };
        if rule_score >= 1 {
            score = score.max(rule_score);
            // Cite the matched artifacts (file:line refs approximated to "<path>:1").
            for rel in rule_presence.iter() {
                if scan.has(rel) {
                    evidence.push(format!("{rel}:1"));
                }
            }
        } else {
            gaps.push(format!(
                "{} — effort: S",
                rule_text
            ));
        }
    }

    let glyph = glyph_for(score, &cluster.scoring);
    let soft_goal_delta = if score >= 2 {
        "notable — extensible platform".to_string()
    } else {
        "notable — partial".to_string()
    };

    PillarScore {
        pillar_id: sp.id.clone(),
        title: sp.title.clone(),
        score,
        glyph,
        evidence,
        gaps,
        soft_goal_delta,
    }
}

/// Cluster with no enumerated sub-pillars → score as one aggregate.
fn score_pillar_aggregate(
    pillar: &Pillar,
    scan: &RepoScan,
    rules: &[(&str, &str, &[&str])],
) -> PillarScore {
    // Pick the first rule that matches the pillar range label.
    let (_rule_id, rule_text, rule_presence) = rules
        .first()
        .copied()
        .unwrap_or(("", "no rule", &[][..]));

    let present = rule_presence
        .iter()
        .filter(|rel| scan.has(rel))
        .count();
    let total = rule_presence.len();
    let score = match (present, total) {
        (0, _) => 0,
        (p, t) if p == t => 3,
        (p, t) if p * 2 >= t => 2,
        _ => 1,
    };

    let evidence: Vec<String> = rule_presence
        .iter()
        .filter(|rel| scan.has(rel))
        .map(|rel| format!("{rel}:1"))
        .collect();

    let gaps: Vec<String> = if score < 3 {
        vec![format!("{rule_text} — effort: M")]
    } else {
        Vec::new()
    };

    PillarScore {
        pillar_id: pillar.pillar_range.clone(),
        title: format!("{} — {}", pillar.pillar_range, cluster_title(pillar)),
        score,
        glyph: glyph_for(score, &pillar.scoring),
        evidence,
        gaps,
        soft_goal_delta: if score >= 2 { "complete".into() } else { "partial".into() },
    }
}

fn cluster_title(pillar: &Pillar) -> String {
    match pillar.cluster.as_str() {
        "C00" => "Architecture + Module",
        "C01" => "CI, DX, Observability",
        "C02" => "Error handling, API, Governance",
        "C03" => "Agent Readiness",
        "C04" => "Security",
        "C05" => "Observability (deep)",
        "C06" => "Supply Chain",
        "C07" => "DX, QEng, Portability",
        "C08" => "Eval Coverage",
        "C09" => "Accessibility + UX",
        "C10" => "Visual Identity",
        "C11" => "Packaging + Distribution",
        _ => "Unknown",
    }
    .into()
}

fn glyph_for(score: u32, spec: &ScoringSpec) -> &'static str {
    match score {
        0 => glyph_static(spec, "0"),
        1 => glyph_static(spec, "1"),
        2 => glyph_static(spec, "2"),
        3 => glyph_static(spec, "3"),
        _ => "?",
    }
}

/// Resolve a glyph from `scoring.glyphs` by string key.
/// Returns a canonical `&'static str` so `PillarScore.glyph: &'static str`
/// can hold it without allocation. Unknown catalog text collapses to `"?"`.
///
/// `ScoringSpec.glyphs` is a required `BTreeMap<String, String>`; the rubric
/// validator already rejects empty glyph tables.
fn glyph_static(spec: &ScoringSpec, key: &str) -> &'static str {
    // Resolve to a borrowed view of the catalog string, falling back to a
    // printable default that mirrors the score key.
    let raw: &str = spec.glyphs.get(key).map(String::as_str).unwrap_or(key);
    // All canonical outputs are `'static` literals — we never return `raw`
    // directly, even when `raw` already looks like an ASCII digit.
    match raw {
        "✗" | "X" | "BAD" => "\u{2717}",
        "△" | "TRI" => "\u{25B3}",
        "~" | "WAVE" => "\u{223C}",
        "✓" | "✔" | "OK" => "\u{2713}",
        // Digits: return an explicit `'static` literal (not the borrowed slice).
        "0" => "0",
        "1" => "1",
        "2" => "2",
        "3" => "3",
        // Empty glyphs entry (validator should have caught this) or novel
        // catalog text — collapse to "?".
        _ => "?",
    }
}

/// Render a ScoreReport as v38-cluster-format markdown.
///
/// Format is byte-compatible with `phenotype-org-audits/audit-v38/output/<repo>/<cluster>.md`.
pub fn render_markdown(report: &ScoreReport) -> String {
    let mut s = String::new();
    for cluster in &report.clusters {
        let _ = writeln!(
            s,
            "CLUSTER_START cluster={} repo={} pillars={} date={}",
            cluster.cluster,
            report.repo,
            cluster
                .pillars
                .first()
                .map(|p| p.pillar_id.clone())
                .unwrap_or_default(),
            report.date
        );
        s.push('\n');
        for pillar in &cluster.pillars {
            let _ = writeln!(s, "### {} — {}", pillar.pillar_id, pillar.title);
            let _ = writeln!(s, "score: {}  glyph: {}", pillar.score, pillar.glyph);
            if pillar.evidence.is_empty() {
                s.push_str("evidence:\n  - (no evidence found)\n");
            } else {
                s.push_str("evidence:\n");
                for ev in &pillar.evidence {
                    let _ = writeln!(s, "  - {}", ev);
                }
            }
            if pillar.gaps.is_empty() {
                s.push_str("gaps:\n  - none\n");
            } else {
                s.push_str("gaps:\n");
                for g in &pillar.gaps {
                    let _ = writeln!(s, "  - {}", g);
                }
            }
            let _ = writeln!(s, "soft_goal_delta: {}\n", pillar.soft_goal_delta);
        }
        let pct = if cluster.max_points == 0 {
            0
        } else {
            (cluster.total_points * 100) / cluster.max_points
        };
        let grade = grade_for(pct);
        let _ = writeln!(
            s,
            "CLUSTER_TOTAL score={}/{} pct={}% grade={}",
            cluster.total_points, cluster.max_points, pct, grade
        );
        let _ = writeln!(
            s,
            "CLUSTER_DONE cluster={} repo={}\n",
            cluster.cluster, report.repo
        );
    }
    s
}

fn grade_for(pct: u32) -> &'static str {
    match pct {
        90..=100 => "A",
        75..=89 => "B",
        60..=74 => "C",
        40..=59 => "D",
        _ => "F",
    }
}

fn current_iso_date() -> String {
    // We avoid a chrono dep here — the orchestrator is path-agnostic, and the
    // orchestrator lives next to the CLI which already gates on Rust 1.86 stdlib.
    // If chrono is already in scope (governance crate uses it transitively) the
    // operator can swap this to `chrono::Utc::now().date_naive().to_string()`.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format!("epoch:{secs}")
}

// === Heuristic rule registry ===
//
// Each rule is keyed by (cluster_id, free-text description, presence-set).
// A pillar score is the max rule score across all matching rules for that
// pillar-id. The presence set is a slice of repo-relative paths scanned by
// `code_scanner`.
//
// The heuristic is intentionally narrow — for parent clusters (L0-L29, L31-L80)
// we run one rule per cluster. For clusters with enumerated sub-pillars (L30,
// L81-L107, L108-L122) we run a coarser rule per cluster because the sub-pillar
// catalog already carries the descriptive title.
//
// Operators can replace this with a richer rule engine via `evaluate_with_rules`
// (TODO) without breaking the ScoreReport shape.

/// `(cluster_id, rule_text, &[presence paths])`.
const SCORING_RULES: &[(&str, &str, &[&str])] = &[
    (
        "C00",
        "Architecture + Module decomposition",
        &["Cargo.toml", "README.md", "src/lib.rs", "tests/"],
    ),
    (
        "C01",
        "CI/DX/Observability baseline",
        &[".github/workflows/", "README.md", "deny.toml"],
    ),
    (
        "C02",
        "Error handling / API surface",
        &["Cargo.toml", "src/lib.rs"],
    ),
    (
        "C03",
        "Agent-readiness: AGENTS/CLAUDE + FR + llms + PR template",
        &[
            "AGENTS.md",
            "CLAUDE.md",
            "docs/functional_requirements.md",
            "llms.txt",
            ".github/PULL_REQUEST_TEMPLATE.md",
        ],
    ),
    (
        "C04",
        "Security: gitleaks/trufflehog + SBOM + cargo-audit + signed commits",
        &[
            "gitleaks.toml",
            ".github/workflows/trufflehog.yml",
            "deny.toml",
            ".github/workflows/security-guard-hook-audit.yml",
            "SECURITY.md",
        ],
    ),
    (
        "C05",
        "Observability deep: OTel + tracing",
        &[".env.example", "docs/observability"],
    ),
    (
        "C06",
        "Supply chain: SBOM + Sigstore + signed releases",
        &[".github/dependabot.yml", "deny.toml"],
    ),
    (
        "C07",
        "DX / QEng / Portability: devcontainer + task runner + editorconfig",
        &[
            ".devcontainer",
            "Taskfile.yml",
            ".editorconfig",
        ],
    ),
    (
        "C08",
        "Eval coverage: tests + coverage gate + nightly",
        &[
            "tests/",
            ".github/workflows/nightly.yml",
            ".github/workflows/coverage.yml",
        ],
    ),
    (
        "C09",
        "Accessibility + UX: WCAG + ARIA",
        &["docs/AX_STANDARD.md"],
    ),
    (
        "C10",
        "Visual identity: tokens + PROVENANCE",
        &[
            "docs/visual/PROVENANCE.md",
            "styles/tokens.css",
            "src/theme/tokens.ts",
        ],
    ),
    (
        "C11",
        "Packaging + Distribution: installers + signing + OCI",
        &[
            "release.yml",
            "Containerfile",
            ".devcontainer",
        ],
    ),
];

/// Loads a PILLARS-CATALOG.json + scans + reports — used by both the
/// `ap rubric score` CLI subcommand and the standalone `agileplus-score`
/// governance binary.
pub fn run(repo: &Path, catalog: &Path, filter: &[String], _date_override: Option<&str>) -> Result<ScoreReport> {
    evaluate(repo, catalog, filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn bare_spec() -> ScoringSpec {
        // Reproduces a minimally valid `scoring` block: validator requires
        // non-empty `glyphs` + `grade`.
        ScoringSpec {
            scale: "0-3".into(),
            glyphs: BTreeMap::new(),
            grade: BTreeMap::new(),
        }
    }

    #[test]
    fn glyph_static_returns_canonical_for_known_keys() {
        // With empty `glyphs`, glyph_for falls back to the ASCII digit of the key.
        let spec = bare_spec();
        assert_eq!(glyph_for(0, &spec), "0");
        assert_eq!(glyph_for(1, &spec), "1");
        assert_eq!(glyph_for(2, &spec), "2");
        assert_eq!(glyph_for(3, &spec), "3");
    }

    #[test]
    fn glyph_static_returns_checkmark_when_catalog_supplies_one() {
        // Regression: ScoringSpec.glyphs is a BTreeMap, not Option<BTreeMap>.
        // A rubric that supplies "✓" for key "3" must surface it as the canonical
        // Unicode checkmark — and an empty glyphs map must still resolve "0" → "0".
        let mut glyphs = BTreeMap::new();
        glyphs.insert("3".into(), "\u{2713}".into());
        let spec = ScoringSpec {
            scale: "0-3".into(),
            glyphs,
            grade: BTreeMap::new(),
        };
        assert_eq!(glyph_for(3, &spec), "\u{2713}");
        assert_eq!(glyph_for(0, &spec), "0");
    }

    #[test]
    fn glyph_static_normalizes_ascii_spellings_to_canonical_unicode() {
        // ASCII shorthand ("OK", "BAD", "TRI", "WAVE") must normalize to the
        // canonical Unicode glyphs so renderers see a single representation.
        let mut glyphs = BTreeMap::new();
        glyphs.insert("3".into(), "OK".into());
        glyphs.insert("2".into(), "TRI".into());
        glyphs.insert("1".into(), "WAVE".into());
        glyphs.insert("0".into(), "BAD".into());
        let spec = ScoringSpec {
            scale: "0-3".into(),
            glyphs,
            grade: BTreeMap::new(),
        };
        assert_eq!(glyph_for(3, &spec), "\u{2713}"); // OK → ✓
        assert_eq!(glyph_for(2, &spec), "\u{25B3}"); // TRI → △
        assert_eq!(glyph_for(1, &spec), "\u{223C}"); // WAVE → ~
        assert_eq!(glyph_for(0, &spec), "\u{2717}"); // BAD → ✗
    }

    #[test]
    fn glyph_static_collapses_novel_catalog_strings_to_question_mark() {
        // Pin the public contract: anything outside the recognized canonical
        // set returns "?" so a misconfigured catalog can't leak raw bytes into
        // the scorecard surface.
        let mut glyphs = BTreeMap::new();
        glyphs.insert("3".into(), "FIRE".into());
        let spec = ScoringSpec {
            scale: "0-3".into(),
            glyphs,
            grade: BTreeMap::new(),
        };
        assert_eq!(glyph_for(3, &spec), "?");
    }

    #[test]
    fn grade_for_handles_boundaries() {
        assert_eq!(grade_for(90), "A");
        assert_eq!(grade_for(75), "B");
        assert_eq!(grade_for(60), "C");
        assert_eq!(grade_for(40), "D");
        assert_eq!(grade_for(0), "F");
    }

    #[test]
    fn evaluate_on_real_repo_does_not_panic() {
        // Smoke test: scan + score runs against this very crate.
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let catalog = crate_root.join("data/PILLARS-CATALOG.json");
        if catalog.exists() {
            let res = run(&crate_root, &catalog, &[], None);
            assert!(res.is_ok(), "scoring-orchestrator should evaluate cleanly");
        }
    }
}
