// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end flow tests for the `plan` command
//! (`agileplus_cli::commands::plan::run_plan`).
//!
//! These run as an integration target, so they link the crate's **rlib** and
//! exercise the production compilation units that the campaign's
//! production-only coverage basis reports over (see
//! `scripts/coverage-complete.sh`: only rlib objects carry production counts).
//! The module's own `#[cfg(test)]` suite compiles into the test harness and
//! contributes nothing to that basis.
//!
//! Everything here is real: a real SQLite database from the production
//! adapter, and a `PlanVcs` that reads and writes actual files under a temp
//! root, so `plan.md`, the per-WP prompts and the governance contract are
//! asserted as they land on disk.

use tempfile::TempDir;

use agileplus_cli::commands::plan::run_plan;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

mod support;
use support::implement::seed;
use support::plan::{
    PlanVcs, args, args_with_max_wps, block_on, seed_feature, spec_with_frs, spec_without_frs,
    write_spec,
};

/// (temp root, storage, vcs) for one test.
fn harness() -> (TempDir, SqliteStorageAdapter, PlanVcs) {
    let tmp = TempDir::new().unwrap();
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let vcs = PlanVcs::new(&tmp.path().join("specs"));
    (tmp, storage, vcs)
}

// ── Preconditions ────────────────────────────────────────────────────────────

#[test]
fn plan_rejects_unknown_feature_with_specify_guidance() {
    let (_tmp, storage, vcs) = harness();

    let err = block_on(run_plan(args("no-such-feat"), &storage, &vcs))
        .expect_err("an unknown feature cannot be planned");

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
fn plan_bails_when_feature_is_already_planned() {
    let (_tmp, storage, vcs) = harness();
    block_on(seed_feature(
        &storage,
        "already-done",
        FeatureState::Planned,
    ));

    let err = block_on(run_plan(args("already-done"), &storage, &vcs))
        .expect_err("a Planned feature is a no-op that must not rewrite the plan");

    assert!(
        err.to_string().contains("already in 'Planned' state"),
        "got: {err}"
    );
}

// ── Work-package generation ──────────────────────────────────────────────────

#[test]
fn empty_spec_yields_a_single_placeholder_work_package() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(
        &storage,
        "empty-spec",
        FeatureState::Researched,
    ));
    write_spec(&vcs, "empty-spec", &spec_without_frs());

    block_on(run_plan(args("empty-spec"), &storage, &vcs)).expect("planning an empty spec");

    let wps = block_on(storage.list_wps_by_feature(fid)).unwrap();
    assert_eq!(wps.len(), 1, "an empty spec still yields one WP");
    assert_eq!(wps[0].title, "Initial Implementation");
    assert_eq!(wps[0].sequence, 1);
}

#[test]
fn functional_requirements_are_grouped_into_sequenced_work_packages() {
    let (_tmp, storage, vcs) = harness();
    let spec = spec_with_frs(&[
        "Login page",
        "Logout flow",
        "Session expiry",
        "Password reset",
        "OAuth callback",
        "Audit trail",
        "Rate limiting",
        "Metrics export",
    ]);

    // max_wps=20 exceeds the FR count, so the per-WP target floors at 3:
    // 8 FRs chunk 3,3,2 into three work packages.
    let fid_wide = block_on(seed_feature(
        &storage,
        "grouped-wide",
        FeatureState::Researched,
    ));
    write_spec(&vcs, "grouped-wide", &spec);
    block_on(run_plan(args("grouped-wide"), &storage, &vcs)).expect("planning with max_wps=20");
    let wide = block_on(storage.list_wps_by_feature(fid_wide)).unwrap();
    assert_eq!(
        wide.len(),
        3,
        "8 FRs chunk 3/3/2; got {:?}",
        wide.iter().map(|w| &w.title).collect::<Vec<_>>()
    );
    let sequences: Vec<i32> = wide.iter().map(|w| w.sequence).collect();
    assert_eq!(sequences, vec![1, 2, 3], "WPs are numbered in order");

    // max_wps=2 forces 4 FRs per WP, so the same 8 FRs chunk 4,4 into two.
    let fid_tight = block_on(seed_feature(
        &storage,
        "grouped-tight",
        FeatureState::Researched,
    ));
    write_spec(&vcs, "grouped-tight", &spec);
    block_on(run_plan(
        args_with_max_wps("grouped-tight", 2),
        &storage,
        &vcs,
    ))
    .expect("planning with max_wps=2");
    let tight = block_on(storage.list_wps_by_feature(fid_tight)).unwrap();
    assert_eq!(
        tight.len(),
        2,
        "max_wps caps grouping; got {:?}",
        tight.iter().map(|w| &w.title).collect::<Vec<_>>()
    );
}

#[test]
fn planning_transitions_researched_feature_to_planned_and_appends_audit() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(
        &storage,
        "ready-feat",
        FeatureState::Researched,
    ));
    write_spec(&vcs, "ready-feat", &spec_with_frs(&["Ship the feature"]));

    block_on(run_plan(args("ready-feat"), &storage, &vcs)).expect("planning succeeds");

    let feature = block_on(storage.get_feature_by_slug("ready-feat"))
        .unwrap()
        .expect("feature still present");
    assert_eq!(
        feature.state,
        FeatureState::Planned,
        "Researched -> Planned transition must happen"
    );

    let audit = block_on(storage.get_latest_audit_entry(fid))
        .unwrap()
        .expect("an audit entry is appended for the transition");
    assert_eq!(audit.transition, "Researched -> Planned");
    assert_eq!(audit.prev_hash, [0u8; 32], "first entry chains from zeroes");

    // The transition itself is what guards a second run: once Researched ->
    // Planned has happened, replanning is refused up front. That is why
    // `rerun_reuses_work_packages_and_the_governance_contract` below has to
    // start from a state that does not transition -- reconcile can only ever
    // see work packages left by a *partial* earlier run.
    let err = block_on(run_plan(args("ready-feat"), &storage, &vcs))
        .expect_err("a completed plan must not be regenerated silently");
    assert!(
        err.to_string().contains("already in 'Planned' state"),
        "got: {err}"
    );
}

#[test]
fn non_researched_feature_plans_without_transition_or_audit() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(&storage, "too-early", FeatureState::Created));
    write_spec(&vcs, "too-early", &spec_with_frs(&["Ship the feature"]));

    // Not Researched and not Planned: a warning, then planning proceeds.
    block_on(run_plan(args("too-early"), &storage, &vcs))
        .expect("planning proceeds despite the unexpected state");

    let feature = block_on(storage.get_feature_by_slug("too-early"))
        .unwrap()
        .expect("feature still present");
    assert_eq!(
        feature.state,
        FeatureState::Created,
        "only Researched features transition"
    );

    let audit = block_on(storage.get_latest_audit_entry(fid)).unwrap();
    assert!(
        audit.is_none(),
        "the audit entry is gated on Researched exactly like the transition"
    );
}

// ── Artifacts on disk ────────────────────────────────────────────────────────

#[test]
fn plan_writes_plan_prompts_and_governance_contract_to_disk() {
    let (_tmp, storage, vcs) = harness();
    let fid = block_on(seed_feature(
        &storage,
        "artifacts",
        FeatureState::Researched,
    ));
    write_spec(&vcs, "artifacts", &spec_with_frs(&["Build the widget"]));

    block_on(run_plan(args("artifacts"), &storage, &vcs)).expect("planning succeeds");

    let files = vcs.written_files("artifacts");
    assert!(files.contains(&"plan.md".to_string()), "files: {files:?}");
    assert!(
        files.contains(&"contracts/governance-v1.json".to_string()),
        "files: {files:?}"
    );
    let prompts: Vec<&String> = files.iter().filter(|f| f.starts_with("tasks/")).collect();
    assert_eq!(prompts.len(), 1, "one prompt per WP; files: {files:?}");
    assert!(prompts[0].ends_with(".md"), "files: {files:?}");

    let plan_md = vcs.read_file("artifacts", "plan.md").expect("plan.md");
    assert!(plan_md.contains("# Plan: artifacts"), "{plan_md}");
    assert!(plan_md.contains("## Work Packages"), "{plan_md}");
    assert!(plan_md.contains("## Execution Waves"), "{plan_md}");

    let prompt = vcs
        .read_file("artifacts", prompts[0])
        .expect("prompt readable");
    assert!(prompt.contains("# Work Package:"), "{prompt}");
    assert!(prompt.contains("## Acceptance Criteria"), "{prompt}");

    let contract_raw = vcs
        .read_file("artifacts", "contracts/governance-v1.json")
        .expect("contract written");
    let contract: serde_json::Value = serde_json::from_str(&contract_raw).expect("valid JSON");
    assert_eq!(contract["version"], 1);
    assert_eq!(contract["feature_id"], fid);
    assert_eq!(
        contract["rules"].as_array().map(Vec::len),
        Some(2),
        "one CI rule and one review rule per WP"
    );
    assert!(
        contract["rules"][0]["transition"]
            .as_str()
            .unwrap()
            .starts_with("WP01: Doing -> Review"),
        "rules are keyed to the WP sequence: {contract}"
    );
}

// ── Retry semantics ──────────────────────────────────────────────────────────

#[test]
fn rerun_reuses_work_packages_and_the_governance_contract() {
    let (_tmp, storage, vcs) = harness();
    // Deliberately NOT `Researched`: a Researched run transitions to Planned,
    // and the guard then rejects any second run before reconciliation is
    // reached. A state that neither transitions nor blocks is what makes the
    // reconcile-and-reuse path observable at all.
    let fid = block_on(seed_feature(&storage, "retry-me", FeatureState::Created));
    write_spec(&vcs, "retry-me", &spec_with_frs(&["Build the widget"]));

    block_on(run_plan(args("retry-me"), &storage, &vcs)).expect("first run");
    let first = block_on(storage.list_wps_by_feature(fid)).unwrap();
    assert_eq!(first.len(), 1);
    let contract_before = vcs
        .read_file("retry-me", "contracts/governance-v1.json")
        .expect("contract after first run");

    block_on(run_plan(args("retry-me"), &storage, &vcs)).expect("second run");

    let second = block_on(storage.list_wps_by_feature(fid)).unwrap();
    assert_eq!(
        second.len(),
        1,
        "a retry must not create a second row for the same sequence"
    );
    assert_eq!(
        second[0].id, first[0].id,
        "the persisted WP is reused rather than re-created"
    );

    let contract_after = vcs
        .read_file("retry-me", "contracts/governance-v1.json")
        .expect("contract after second run");

    let before: serde_json::Value = serde_json::from_str(&contract_before).expect("valid JSON");
    let after: serde_json::Value = serde_json::from_str(&contract_after).expect("valid JSON");

    // Reuse rather than rebuild: a freshly constructed contract would carry a
    // new `bound_at`, and the rules are regenerated from the WP list.
    assert_eq!(
        before["bound_at"], after["bound_at"],
        "the second run must reuse the persisted contract, not build a new one"
    );
    assert_eq!(
        before["rules"], after["rules"],
        "rules are regenerated identically"
    );
    assert_eq!(before["version"], after["version"]);

    // After the second run the artifact carries the database-assigned id and
    // so matches the row exactly.
    //
    // It did NOT after the first run: `run_plan` discards the id returned by
    // `create_governance_contract` and serialises the in-memory contract with
    // `id: 0`, so the artifact only becomes aligned with the database on a
    // later write. That is a production finding, out of scope for this wave.
    let stored = block_on(storage.get_latest_governance_contract(fid))
        .unwrap()
        .expect("contract persisted");
    assert_eq!(
        after["id"].as_i64(),
        Some(stored.id),
        "the rewritten artifact must match the persisted row"
    );
}

// ── Reconciliation ───────────────────────────────────────────────────────────

#[test]
fn a_persisted_wp_that_the_new_plan_does_not_match_requires_explicit_replan() {
    let (_tmp, storage, vcs) = harness();
    // Seeds a WP at sequence 1 titled "Build the thing".
    let (fid, _wp_id) = block_on(seed(&storage, "conflict-feat", FeatureState::Researched));
    // A spec whose derived title cannot be reconciled with the persisted row.
    write_spec(
        &vcs,
        "conflict-feat",
        &spec_with_frs(&["Something else entirely"]),
    );

    let err = block_on(run_plan(args("conflict-feat"), &storage, &vcs))
        .expect_err("mismatched persisted WP must refuse to plan");

    assert!(
        err.to_string()
            .contains("does not match generated plan; explicit replan required"),
        "got: {err}"
    );

    let wps = block_on(storage.list_wps_by_feature(fid)).unwrap();
    assert_eq!(wps.len(), 1, "the refused plan must not add rows");
    assert_eq!(wps[0].title, "Build the thing", "persisted WP untouched");
}
