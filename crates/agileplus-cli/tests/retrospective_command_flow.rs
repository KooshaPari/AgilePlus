// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end flow tests for the `retrospective` command
//! (`agileplus_cli::commands::retrospective::run_retrospective`).
//!
//! These run as an integration target, so they link the crate's **rlib** and
//! exercise the production compilation unit that the campaign's production-only
//! coverage basis reports over. The module's own `#[cfg(test)]` suite compiles
//! into the test harness and contributes nothing to that basis.
//!
//! Everything here is real: a real SQLite database from the production
//! adapter, and a `PlanVcs` that writes actual files under a temp root, so
//! `retrospective.md` is asserted where it lands on disk alongside the state
//! transition, the hash-chained audit entry, and the state-transition event
//! the command persists.

use std::path::Path;

use clap::error::ErrorKind;
use serde_json::json;

use agileplus_cli::commands::retrospective::run_retrospective;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::StoragePort;
use agileplus_events::EventStore;

mod support;
use support::implement::block_on;
use support::retrospective::{
    args, args_with_output, args_with_raw_metrics, harness, parse, seed_audit, seed_feature,
    seed_metric, seed_wp,
};

// ── Argument parsing ─────────────────────────────────────────────────────────

#[test]
fn retrospective_cli_requires_feature_and_rejects_unknown_flags() {
    let missing = parse(&[]).expect_err("a retrospective without --feature must not parse");
    assert_eq!(missing.kind(), ErrorKind::MissingRequiredArgument);
    let missing_msg = missing.to_string();
    assert!(missing_msg.contains("--feature"), "got: {missing_msg}");

    let unknown =
        parse(&["--feature", "x", "--bogus"]).expect_err("an unknown flag must not parse silently");
    assert_eq!(unknown.kind(), ErrorKind::UnknownArgument);
    let unknown_msg = unknown.to_string();
    assert!(unknown_msg.contains("--bogus"), "got: {unknown_msg}");
    assert!(
        unknown_msg.contains("unexpected argument"),
        "got: {unknown_msg}"
    );
}

#[test]
fn retrospective_cli_parses_flags_with_clap_defaults() {
    let minimal = parse(&["--feature", "ship-it"]).expect("minimal invocation parses");
    assert_eq!(minimal.feature, "ship-it");
    assert!(
        minimal.output.is_none(),
        "no --output defaults to the VCS artifact"
    );
    assert!(
        !minimal.include_raw_metrics,
        "--include-raw-metrics defaults to off"
    );

    let full = parse(&[
        "--feature",
        "ship-it",
        "--output",
        "reports/retro.md",
        "--include-raw-metrics",
    ])
    .expect("full invocation parses");
    assert_eq!(full.output.as_deref(), Some(Path::new("reports/retro.md")));
    assert!(full.include_raw_metrics, "the raw-metrics flag flips on");
}

// ── Preconditions ────────────────────────────────────────────────────────────

#[test]
fn retrospective_unknown_feature_points_to_specify() {
    let (_tmp, storage, vcs) = harness();

    let err = block_on(run_retrospective(args("no-such-feat"), &storage, &vcs))
        .expect_err("an unknown feature cannot be retrospected");

    let msg = err.to_string();
    assert!(
        msg.contains("Feature 'no-such-feat' not found"),
        "message names the missing feature, got: {msg}"
    );
    assert!(
        msg.contains("agileplus specify --feature no-such-feat"),
        "message tells the operator how to proceed, got: {msg}"
    );
}

#[test]
fn retrospective_refuses_non_shipped_state_without_side_effects() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(&storage, "early-feat", FeatureState::Created));

    let err = block_on(run_retrospective(args("early-feat"), &storage, &vcs))
        .expect_err("a Created feature is not ready for a retrospective");

    let msg = err.to_string();
    assert!(msg.contains("is in state 'created'"), "got: {msg}");
    assert!(msg.contains("Expected 'Shipped'"), "got: {msg}");
    assert!(
        msg.contains("agileplus ship --feature early-feat"),
        "message points at the gating command, got: {msg}"
    );

    // The refusal happens before anything is written.
    let feature = block_on(storage.get_feature_by_slug("early-feat"))
        .unwrap()
        .expect("feature still present");
    assert_eq!(
        feature.state,
        FeatureState::Created,
        "state must not change on refusal"
    );
    assert!(
        block_on(storage.get_audit_trail(fid)).unwrap().is_empty(),
        "no audit entry appended"
    );
    assert!(
        block_on(storage.get_events("feature", fid))
            .unwrap()
            .is_empty(),
        "no event appended"
    );
    assert!(
        vcs.read_file("early-feat", "retrospective.md").is_none(),
        "no artifact written"
    );
}

// ── The Shipped -> Retrospected transition ───────────────────────────────────

#[test]
fn retrospective_shipped_feature_transitions_with_audit_event_and_artifact() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(&storage, "done-feat", FeatureState::Shipped));
    block_on(seed_audit(&storage, fid, "Implementing -> Validated"));
    let head = block_on(seed_audit(&storage, fid, "Validated -> Shipped"));

    block_on(run_retrospective(args("done-feat"), &storage, &vcs))
        .expect("a Shipped feature retrospects cleanly");

    // State transition.
    let feature = block_on(storage.get_feature_by_slug("done-feat"))
        .unwrap()
        .expect("feature still present");
    assert_eq!(
        feature.state,
        FeatureState::Retrospected,
        "Shipped -> Retrospected must happen"
    );

    // Audit entry appended and chained onto the seeded head.
    let trail = block_on(storage.get_audit_trail(fid)).unwrap();
    assert_eq!(
        trail.len(),
        3,
        "two seeded entries plus the transition entry"
    );
    assert_eq!(
        trail[0].transition, "Implementing -> Validated",
        "seed untouched"
    );
    let last = trail.last().expect("the appended entry");
    assert_eq!(last.transition, "Shipped -> Retrospected");
    assert_eq!(
        last.prev_hash, head,
        "the new entry links to the previous head"
    );
    assert_ne!(last.hash, [0u8; 32], "the new entry carries a real hash");

    // State-transition event appended to the feature's event stream.
    let events = block_on(storage.get_events("feature", fid)).unwrap();
    assert_eq!(events.len(), 1, "exactly one event persisted");
    let event = &events[0];
    assert_eq!(event.event_type, "state_transitioned");
    assert_eq!(event.sequence, 1, "first event in the feature's stream");
    assert_eq!(event.prev_hash, [0u8; 32], "chain starts from zeroes");
    assert_eq!(event.payload["from"], json!("Shipped"));
    assert_eq!(event.payload["to"], json!("Retrospected"));

    // Report written through the VCS onto real files.
    let report = vcs
        .read_file("done-feat", "retrospective.md")
        .expect("artifact on disk");
    assert!(
        report.contains("# Retrospective: Retro done-feat"),
        "heading uses the feature name: {report}"
    );
    assert!(report.contains("**Feature**: `done-feat`"), "{report}");
    assert!(report.contains("## Summary"), "{report}");
    assert!(
        report.contains("## Phase Breakdown"),
        "a seeded trail renders the breakdown: {report}"
    );
    assert!(
        report.contains("| Implementing -> Validated |"),
        "seeded phase appears in the breakdown: {report}"
    );

    // The transition itself is what guards a second run.
    let rerun = block_on(run_retrospective(args("done-feat"), &storage, &vcs))
        .expect_err("a retrospected feature must not be retrospected again");
    let rerun_msg = rerun.to_string();
    assert!(rerun_msg.contains("Expected 'Shipped'"), "got: {rerun_msg}");
    assert_eq!(
        block_on(storage.get_audit_trail(fid)).unwrap().len(),
        3,
        "the refused rerun appends nothing"
    );
    assert_eq!(
        block_on(storage.get_feature_by_slug("done-feat"))
            .unwrap()
            .expect("feature still present")
            .state,
        FeatureState::Retrospected
    );
}

#[test]
fn retrospective_empty_history_renders_zero_summary_and_chains_from_zero() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(&storage, "blank-feat", FeatureState::Shipped));

    block_on(run_retrospective(args("blank-feat"), &storage, &vcs))
        .expect("a feature with no history still retrospects");

    let report = vcs
        .read_file("blank-feat", "retrospective.md")
        .expect("artifact on disk");
    assert!(report.contains("**Total duration**: 0s"), "{report}");
    assert!(report.contains("**Work packages**: 0"), "{report}");
    assert!(
        report.contains("**Total agent invocations**: 0"),
        "{report}"
    );
    assert!(
        report.contains("**Total review cycles**: 0 (avg 0.0 per WP)"),
        "{report}"
    );
    assert!(report.contains("**Governance exceptions**: 0"), "{report}");
    assert!(!report.contains("## Phase Breakdown"), "{report}");
    assert!(!report.contains("## WP Performance"), "{report}");
    assert!(
        report.contains("No significant issues detected. Development process is healthy."),
        "empty metrics read as healthy: {report}"
    );
    assert!(
        report.contains("No constitution amendments suggested."),
        "{report}"
    );

    let trail = block_on(storage.get_audit_trail(fid)).unwrap();
    assert_eq!(trail.len(), 1, "only the transition entry");
    assert_eq!(trail[0].transition, "Shipped -> Retrospected");
    assert_eq!(
        trail[0].prev_hash, [0u8; 32],
        "the first-ever entry chains from zeroes"
    );
}

// ── Side effects on disk ─────────────────────────────────────────────────────

#[test]
fn retrospective_output_flag_writes_the_report_file() {
    let (tmp, storage, vcs) = harness();
    block_on(seed_feature(&storage, "out-feat", FeatureState::Shipped));
    let out = tmp.path().join("retro-report.md");
    assert!(!out.exists(), "fixture starts with no report file");

    block_on(run_retrospective(
        args_with_output("out-feat", out.clone()),
        &storage,
        &vcs,
    ))
    .expect("--output write succeeds");

    let written = std::fs::read_to_string(&out).expect("--output file on disk");
    assert!(
        written.contains("# Retrospective: Retro out-feat"),
        "{written}"
    );
    let artifact = vcs
        .read_file("out-feat", "retrospective.md")
        .expect("VCS artifact written too");
    assert_eq!(
        written, artifact,
        "both write paths carry the same report content"
    );
    let feature = block_on(storage.get_feature_by_slug("out-feat"))
        .unwrap()
        .expect("feature still present");
    assert_eq!(feature.state, FeatureState::Retrospected);
}

#[test]
fn retrospective_output_write_failure_leaves_the_feature_untouched() {
    let (tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(&storage, "doomed-feat", FeatureState::Shipped));
    let out = tmp.path().join("missing-dir").join("retro.md");
    assert!(
        !out.parent().expect("parent exists in path").exists(),
        "parent dir deliberately absent"
    );

    let err = block_on(run_retrospective(
        args_with_output("doomed-feat", out),
        &storage,
        &vcs,
    ))
    .expect_err("an unwritable --output path must fail the command");

    let msg = err.to_string();
    assert!(msg.contains("writing retro to"), "got: {msg}");

    // The file write runs before any mutation, so nothing may have moved.
    let feature = block_on(storage.get_feature_by_slug("doomed-feat"))
        .unwrap()
        .expect("feature still present");
    assert_eq!(feature.state, FeatureState::Shipped, "state untouched");
    assert!(
        block_on(storage.get_audit_trail(fid)).unwrap().is_empty(),
        "no audit entry appended"
    );
    assert!(
        block_on(storage.get_events("feature", fid))
            .unwrap()
            .is_empty(),
        "no event appended"
    );
    assert!(
        vcs.read_file("doomed-feat", "retrospective.md").is_none(),
        "no artifact written"
    );
}

// ── Report content ───────────────────────────────────────────────────────────

#[test]
fn retrospective_raw_metrics_flag_gates_the_governance_exceptions_section() {
    let (_tmp, storage, vcs) = harness();
    let exception = "WP01: Doing -> Review skipped per exception";

    let plain_id = block_on(seed_feature(&storage, "plain-feat", FeatureState::Shipped));
    block_on(seed_audit(&storage, plain_id, "Validated -> Shipped"));
    block_on(seed_audit(&storage, plain_id, exception));
    block_on(run_retrospective(args("plain-feat"), &storage, &vcs)).expect("plain run");
    let plain = vcs
        .read_file("plain-feat", "retrospective.md")
        .expect("plain artifact");

    let verbose_id = block_on(seed_feature(
        &storage,
        "verbose-feat",
        FeatureState::Shipped,
    ));
    block_on(seed_audit(&storage, verbose_id, "Validated -> Shipped"));
    block_on(seed_audit(&storage, verbose_id, exception));
    block_on(run_retrospective(
        args_with_raw_metrics("verbose-feat"),
        &storage,
        &vcs,
    ))
    .expect("raw-metrics run");
    let verbose = vcs
        .read_file("verbose-feat", "retrospective.md")
        .expect("verbose artifact");

    // The summary always counts governance exceptions...
    assert!(plain.contains("**Governance exceptions**: 1"), "{plain}");
    assert!(
        verbose.contains("**Governance exceptions**: 1"),
        "{verbose}"
    );
    // ...and both reports still get the fast-track amendment suggestion.
    assert!(plain.contains("fast-track"), "{plain}");
    assert!(verbose.contains("fast-track"), "{verbose}");
    assert!(plain.contains("## Phase Breakdown"), "{plain}");

    // Only --include-raw-metrics renders the exception detail section.
    assert!(!plain.contains("## Governance Exceptions"), "{plain}");
    assert!(verbose.contains("## Governance Exceptions"), "{verbose}");
    assert!(verbose.contains(exception), "{verbose}");
    assert!(!plain.contains(exception), "detail stays hidden: {plain}");
}

#[test]
fn retrospective_renders_wp_table_and_review_insights_from_stored_metrics() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(
        &storage,
        "metrics-feat",
        FeatureState::Shipped,
    ));
    block_on(seed_wp(&storage, fid, 1, "Build the widget"));
    block_on(seed_metric(&storage, fid, "implement WP01", 60_000, 2, 5));

    block_on(run_retrospective(args("metrics-feat"), &storage, &vcs))
        .expect("metrics run succeeds");

    let report = vcs
        .read_file("metrics-feat", "retrospective.md")
        .expect("artifact on disk");

    // The metric whose command names WP01 feeds the per-WP table.
    assert!(report.contains("## WP Performance"), "{report}");
    assert!(
        report.contains("| WP01 | Build the widget | 2 | 5 | 1m 0s |"),
        "{report}"
    );

    // Summary totals. NOTE: `run_retrospective` counts a WP-matched metric
    // once through the per-WP rollup and again through the feature-wide
    // metric sum, so this single row (2 runs / 5 cycles) reports 4 and 10.
    // Recorded as observed behavior in the wave report, not fixed here.
    assert!(
        report.contains("**Total agent invocations**: 4"),
        "{report}"
    );
    assert!(
        report.contains("**Total review cycles**: 10 (avg 10.0 per WP)"),
        "{report}"
    );

    // 10 avg review cycles per WP (> 3) drives both insights and the
    // constitution amendment suggestion.
    assert!(
        report.contains("High average review cycles (10.0 per WP)"),
        "{report}"
    );
    assert!(
        report.contains(
            "WPs with >3 review cycles (potential bottlenecks): \
             WP01 'Build the widget' (5 cycles)"
        ),
        "{report}"
    );
    assert!(report.contains("pre-review-self-check"), "{report}");

    // 4 agent runs over 1 WP stays under the 5/WP threshold and there were no
    // governance exceptions, so those two insights must not fire.
    assert!(!report.contains("agent invocation rate"), "{report}");
    assert!(
        !report.contains("governance exception(s) occurred"),
        "{report}"
    );
}
