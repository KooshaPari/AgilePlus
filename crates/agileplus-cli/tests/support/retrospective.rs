// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fixtures for the `retrospective` command integration tests.
//!
//! `run_retrospective` looks the feature up in storage, enforces `Shipped`,
//! renders `retrospective.md` from the audit trail / work packages / metrics,
//! writes it through the `VcsPort` (and to `--output` when given), then
//! transitions `Shipped -> Retrospected` with an audit entry and a
//! state-transition event. These fixtures supply a real-file `VcsPort`
//! (reused from `support::plan`), clap argument builders that mirror the
//! binary's parser, and storage seeding helpers, so every flow-test
//! assertion lands on a real row or a real file rather than a double.
//!
//! Registered in `support/mod.rs` up front so parallel wave-15 workers never
//! race on that file.
#![allow(dead_code)]

use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;

use agileplus_cli::commands::retrospective::RetrospectiveArgs;
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

/// Real-file `VcsPort` from the plan fixtures: artifacts land under
/// `<root>/<slug>/<relative-path>`, so `retrospective.md` is asserted on disk.
pub use super::plan::PlanVcs;

/// `(temp root, storage, vcs)` for one test — the plan-flow harness shape.
pub fn harness() -> (tempfile::TempDir, SqliteStorageAdapter, PlanVcs) {
    let tmp = tempfile::TempDir::new().expect("temp root");
    let storage = SqliteStorageAdapter::in_memory().expect("in-memory storage");
    let vcs = PlanVcs::new(&tmp.path().join("specs"));
    (tmp, storage, vcs)
}

/// Default `RetrospectiveArgs`: no `--output`, no `--include-raw-metrics`.
pub fn args(feature_slug: &str) -> RetrospectiveArgs {
    RetrospectiveArgs {
        feature: feature_slug.to_string(),
        output: None,
        include_raw_metrics: false,
    }
}

/// `RetrospectiveArgs` with an explicit `--output` path.
pub fn args_with_output(feature_slug: &str, output: PathBuf) -> RetrospectiveArgs {
    RetrospectiveArgs {
        output: Some(output),
        ..args(feature_slug)
    }
}

/// `RetrospectiveArgs` with `--include-raw-metrics` set.
pub fn args_with_raw_metrics(feature_slug: &str) -> RetrospectiveArgs {
    RetrospectiveArgs {
        include_raw_metrics: true,
        ..args(feature_slug)
    }
}

#[derive(Parser)]
#[command(name = "agileplus")]
struct RetrospectiveCli {
    #[command(flatten)]
    args: RetrospectiveArgs,
}

/// Parse a `retrospective` argument tail (the words after the subcommand)
/// through the real clap layer, so required-flag and unknown-flag errors are
/// the binary's own messages rather than hand-written strings.
pub fn parse(argv: &[&str]) -> Result<RetrospectiveArgs, clap::Error> {
    let mut full = vec!["agileplus"];
    full.extend_from_slice(argv);
    RetrospectiveCli::try_parse_from(full).map(|cli| cli.args)
}

// ── Storage seeding ──────────────────────────────────────────────────────────

/// Create a feature row in `state`; its report heading becomes
/// `# Retrospective: Retro <slug>`. Returns the database id.
pub async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str, state: FeatureState) -> i64 {
    let mut feature = super::implement::feature(slug, state);
    feature.friendly_name = format!("Retro {slug}");
    StoragePort::create_feature(storage, &feature)
        .await
        .expect("seeding feature")
}

/// Create a work package at `sequence`; returns its id.
pub async fn seed_wp(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    sequence: i32,
    title: &str,
) -> i64 {
    let mut wp = WorkPackage::new(feature_id, title, sequence, "works end to end");
    wp.id = 0;
    StoragePort::create_work_package(storage, &wp)
        .await
        .expect("seeding work package")
}

/// Record a metric row. `command` must contain the `WPNN` label for the
/// report's per-WP table to pick the row up. Returns the metric id.
pub async fn seed_metric(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    command: &str,
    duration_ms: i64,
    agent_runs: i32,
    review_cycles: i32,
) -> i64 {
    let metric = Metric {
        id: 0,
        feature_id: Some(feature_id),
        command: command.to_string(),
        duration_ms,
        agent_runs,
        review_cycles,
        metadata: None,
        timestamp: Utc::now(),
    };
    StoragePort::record_metric(storage, &metric)
        .await
        .expect("seeding metric")
}

/// Append a hash-chained audit entry with `transition`, chaining from the
/// current head; returns the new entry's hash so tests can assert the next
/// link. Transitions containing `skipped`/`exception` read as governance
/// exceptions in the report.
pub async fn seed_audit(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    transition: &str,
) -> [u8; 32] {
    let prev_hash = match StoragePort::get_latest_audit_entry(storage, feature_id)
        .await
        .expect("reading audit head")
    {
        Some(head) => head.hash,
        None => [0u8; 32],
    };
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: transition.to_string(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    StoragePort::append_audit_entry(storage, &entry)
        .await
        .expect("seeding audit entry");
    entry.hash
}
