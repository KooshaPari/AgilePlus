// SPDX-License-Identifier: MIT OR Apache-2.0
//! Dogfood seed data for AgilePlus dashboard initialization.
//!
//! Contains all AgilePlus kitty-specs and creates placeholder work packages
//! for dashboard seeding. This module is extracted from agileplus-dashboard
//! to enable reuse across tests and initialization.

use std::collections::HashMap;

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};

mod speckitty;

use speckitty::speckitty_specs;

#[cfg(test)]
mod tests;

/// Drive a feature through a sequence of state transitions, panicking on failure.
fn drive_to_state(feature: &mut Feature, target: FeatureState) {
    use FeatureState::*;
    let path: &[FeatureState] = match target {
        Created => &[],
        Specified => &[Specified],
        Researched => &[Specified, Researched],
        Planned => &[Specified, Researched, Planned],
        Implementing => &[Specified, Researched, Planned, Implementing],
        Validated => &[Specified, Researched, Planned, Implementing, Validated],
        Shipped => &[
            Specified,
            Researched,
            Planned,
            Implementing,
            Validated,
            Shipped,
        ],
        Retrospected => &[
            Specified,
            Researched,
            Planned,
            Implementing,
            Validated,
            Shipped,
            Retrospected,
        ],
    };
    for &state in path {
        feature
            .transition(state)
            .expect("seed: state transition failed");
    }
}

/// Helper function to create a shipped feature with all state transitions.
fn make_shipped_feature(
    id: i64,
    slug: &str,
    name: &str,
    labels: Vec<String>,
    project_id: Option<i64>,
) -> Feature {
    let mut f = Feature::new(slug, name, [0u8; 32], Some("main"));
    f.id = id;
    f.labels = labels;
    f.project_id = project_id;
    drive_to_state(&mut f, FeatureState::Shipped);
    f
}

/// Helper function to create work packages for a feature.
fn make_shipped_wps(feature_id: i64, base_wp_id: i64, titles: &[&str]) -> Vec<WorkPackage> {
    titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let mut wp = WorkPackage::new(feature_id, title, (i + 1) as i32, "Completed");
            wp.id = base_wp_id + i as i64;
            wp.state = WpState::Done;
            wp
        })
        .collect()
}

/// Seed the dashboard with all AgilePlus dogfood features and work packages.
///
/// Returns:
/// - Vec<Feature>: all AgilePlus features with sequential IDs 1-4, plus SpecKitty reference specs 5-37
/// - HashMap<i64, Vec<WorkPackage>>: work packages keyed by feature_id, 2-4 per feature
pub fn seed_dogfood_features() -> (Vec<Feature>, HashMap<i64, Vec<WorkPackage>>) {
    let mut features = Vec::new();
    let mut work_packages: HashMap<i64, Vec<WorkPackage>> = HashMap::new();

    // --- AgilePlus Features (IDs 1-4) ---

    // Feature 1: 001-spec-driven-development-engine
    features.push(make_shipped_feature(
        1,
        "001-spec-driven-development-engine",
        "Spec-Driven Development Engine",
        vec!["platform".to_owned(), "infrastructure".to_owned()],
        Some(1),
    ));

    // Work packages for feature 1
    let mut wp1_1 = WorkPackage::new(
        1,
        "Define spec schema and validation rules",
        1,
        "Schema YAML defined and validated against kitty-specs",
    );
    wp1_1.id = 1;
    wp1_1.state = WpState::Done;
    wp1_1.file_scope = vec!["crates/agileplus-domain/src/domain/feature.rs".to_owned()];

    let mut wp1_2 = WorkPackage::new(
        1,
        "Implement state machine for feature lifecycle",
        2,
        "All 8 states defined with forward-only transitions",
    );
    wp1_2.id = 2;
    wp1_2.state = WpState::Done;
    wp1_2.file_scope = vec!["crates/agileplus-domain/src/domain/state_machine.rs".to_owned()];

    let mut wp1_3 = WorkPackage::new(
        1,
        "Build persistence layer for specs",
        3,
        "SQLite persistence for features and work packages",
    );
    wp1_3.id = 3;
    wp1_3.state = WpState::Done;
    wp1_3.file_scope = vec!["crates/agileplus-storage/src/".to_owned()];

    let mut wp1_4 = WorkPackage::new(
        1,
        "Add retrospective tracking and audit logs",
        4,
        "Retrospected state records completion audit trail",
    );
    wp1_4.id = 4;
    wp1_4.state = WpState::Done;
    wp1_4.file_scope = vec![
        "crates/agileplus-domain/src/domain/feature.rs".to_owned(),
        "crates/agileplus-storage/src/".to_owned(),
    ];

    work_packages.insert(1, vec![wp1_1, wp1_2, wp1_3, wp1_4]);

    // Feature 2: 002-org-wide-release-governance-dx-automation
    features.push(make_shipped_feature(
        2,
        "002-org-wide-release-governance-dx-automation",
        "Org-Wide Release Governance & DX Automation",
        vec!["governance".to_owned(), "automation".to_owned()],
        Some(1),
    ));

    // Work packages for feature 2
    let mut wp2_1 = WorkPackage::new(
        2,
        "Design release governance matrix",
        1,
        "Release decision matrix defined with lanes and roles",
    );
    wp2_1.id = 5;
    wp2_1.state = WpState::Done;
    wp2_1.file_scope = vec!["crates/agileplus-governance/src/".to_owned()];

    let mut wp2_2 = WorkPackage::new(
        2,
        "Implement Plane.so sync for governance decisions",
        2,
        "Governance state syncs with Plane.so issue states",
    );
    wp2_2.id = 6;
    wp2_2.state = WpState::Done;
    wp2_2.file_scope = vec![
        "crates/agileplus-plane-sync/src/".to_owned(),
        "crates/agileplus-domain/src/".to_owned(),
    ];

    let mut wp2_3 = WorkPackage::new(
        2,
        "Add DX automation: CI integration and auto-merge gates",
        3,
        "CI gates enforce linting, testing, and spec verification",
    );
    wp2_3.id = 7;
    wp2_3.state = WpState::Done;
    wp2_3.file_scope = vec![".github/workflows/".to_owned()];

    work_packages.insert(2, vec![wp2_1, wp2_2, wp2_3]);

    // Feature 3: 003-agileplus-platform-completion
    features.push(make_shipped_feature(
        3,
        "003-agileplus-platform-completion",
        "AgilePlus Platform Completion",
        vec!["platform".to_owned(), "integration".to_owned()],
        Some(1),
    ));

    // Work packages for feature 3
    let mut wp3_1 = WorkPackage::new(
        3,
        "Build unified dashboard with Askama templates",
        1,
        "Dashboard renders all features with state-based styling",
    );
    wp3_1.id = 8;
    wp3_1.state = WpState::Done;
    wp3_1.file_scope = vec![
        "crates/agileplus-dashboard/src/templates.rs".to_owned(),
        "crates/agileplus-dashboard/src/routes.rs".to_owned(),
    ];

    let mut wp3_2 = WorkPackage::new(
        3,
        "Implement htmx handlers for real-time feature updates",
        2,
        "htmx handlers enable live state transitions without full page reload",
    );
    wp3_2.id = 9;
    wp3_2.state = WpState::Done;
    wp3_2.file_scope = vec!["crates/agileplus-dashboard/src/routes.rs".to_owned()];

    let mut wp3_3 = WorkPackage::new(
        3,
        "Add health monitoring and service status page",
        3,
        "Dashboard displays health of NATS, Neo4j, MinIO, SQLite, etc.",
    );
    wp3_3.id = 10;
    wp3_3.state = WpState::Done;
    wp3_3.file_scope = vec!["crates/agileplus-dashboard/src/app_state.rs".to_owned()];

    let mut wp3_4 = WorkPackage::new(
        3,
        "Seed dashboard with dogfood features",
        4,
        "Dashboard populated with all AgilePlus specs for self-testing",
    );
    wp3_4.id = 11;
    wp3_4.state = WpState::Done;
    wp3_4.file_scope = vec!["crates/agileplus-fixtures/src/dogfood.rs".to_owned()];

    work_packages.insert(3, vec![wp3_1, wp3_2, wp3_3, wp3_4]);

    // Feature 4: 004-modules-and-cycles (Implementing)
    let mut f4 = Feature::new(
        "004-modules-and-cycles",
        "Modules and Cycles",
        [0u8; 32],
        Some("main"),
    );
    f4.id = 4;
    f4.labels = vec!["organization".to_owned(), "planning".to_owned()];
    f4.project_id = Some(1);
    // Transition to Implementing (Created -> Specified -> Researched -> Planned -> Implementing)
    drive_to_state(&mut f4, FeatureState::Implementing);
    features.push(f4.clone());

    // Work packages for feature 4
    let mut wp4_1 = WorkPackage::new(
        4,
        "Design module ownership model",
        1,
        "Module structure defined with clear ownership boundaries",
    );
    wp4_1.id = 12;
    wp4_1.state = WpState::Done;

    let mut wp4_2 = WorkPackage::new(
        4,
        "Implement module and cycle storage schema",
        2,
        "SQLite tables for modules, cycles, and ownership assignments",
    );
    wp4_2.id = 13;
    wp4_2.state = WpState::Doing;
    wp4_2.file_scope = vec!["crates/agileplus-storage/src/".to_owned()];

    let mut wp4_3 = WorkPackage::new(
        4,
        "Add module views to dashboard",
        3,
        "Dashboard displays modules with assigned features and cycles",
    );
    wp4_3.id = 14;
    wp4_3.state = WpState::Planned;
    wp4_3.file_scope = vec!["crates/agileplus-dashboard/src/".to_owned()];

    work_packages.insert(4, vec![wp4_1, wp4_2, wp4_3]);

    // --- SpecKitty Reference Features (IDs 5-37) ---

    let mut wp_id = work_packages
        .values()
        .flatten()
        .map(|wp| wp.id)
        .max()
        .unwrap_or(0)
        + 1;
    for (id, slug, name, labels) in speckitty_specs() {
        features.push(make_shipped_feature(id, slug, name, labels, Some(1)));
        let wps = make_shipped_wps(id, wp_id, &["Research and design", "Core implementation"]);
        wp_id += wps.len() as i64;
        work_packages.insert(id, wps);
    }

    (features, work_packages)
}
