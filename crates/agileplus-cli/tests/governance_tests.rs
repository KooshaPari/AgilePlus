// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus_cli::commands::governance`.
//!
//! Tests `validate_spec_consistency`, `enforce_governance`, and
//! `ViolationSeverity` display. The existing inline tests cover basics;
//! these add edge cases for comprehensive line coverage.

use agileplus_cli::commands::governance::{
    Constitution, Violation, ViolationSeverity, enforce_governance, load_constitution,
    validate_spec_consistency,
};

mod support;
use support::MockVcs;

/// Well-known constitution path probed by `load_constitution`.
const CONSTITUTION_PATH: &str = "../.kittify/memory/constitution.md";

// ── load_constitution ────────────────────────────────────────────────────────

#[tokio::test]
async fn load_constitution_returns_content_when_artifact_present() {
    let vcs = MockVcs::new().with_artifact("", CONSTITUTION_PATH, "gov rules");
    let constitution = load_constitution(&vcs)
        .await
        .expect("constitution should load");
    assert_eq!(constitution.content, "gov rules");
}

#[tokio::test]
async fn load_constitution_returns_none_when_absent() {
    let vcs = MockVcs::new();
    assert!(load_constitution(&vcs).await.is_none());
}

#[tokio::test]
async fn load_constitution_returns_none_when_read_fails() {
    let mut vcs = MockVcs::new().with_artifact("", CONSTITUTION_PATH, "gov rules");
    vcs.read_fails = true;
    assert!(load_constitution(&vcs).await.is_none());
}

fn dummy_constitution() -> Constitution {
    Constitution {
        content: String::new(),
    }
}

fn full_spec() -> &'static str {
    "# My Spec\n\n## Problem Statement\nWe need auth.\n\n## Functional Requirements\n- **FR-001**: Login\n\n## Acceptance Criteria\n- Login works\n"
}

// ── validate_spec_consistency edge cases ──────────────────────────────────────

#[test]
fn full_valid_spec_has_no_violations() {
    let violations = validate_spec_consistency(full_spec(), &dummy_constitution());
    assert!(
        violations.is_empty(),
        "full spec should be clean: {violations:?}"
    );
}

#[test]
fn missing_problem_statement_only() {
    let spec = "## Functional Requirements\n- **FR-001**: X\n## Acceptance Criteria\nY\n";
    let v = validate_spec_consistency(spec, &dummy_constitution());
    assert!(v.iter().any(|x| x.message.contains("Problem Statement")));
    // The other sections are present, so they should not produce violations.
    assert!(!v.iter().any(|x| x.message.contains("Functional Requirements")));
    assert!(!v.iter().any(|x| x.message.contains("Acceptance Criteria")));
}

#[test]
fn missing_functional_requirements_only() {
    let spec = "## Problem Statement\nX\n## Acceptance Criteria\nY\n";
    let v = validate_spec_consistency(spec, &dummy_constitution());
    assert!(v.iter().any(|x| x.message.contains("Functional Requirements")));
    assert!(!v.iter().any(|x| x.message.contains("Problem Statement")));
}

#[test]
fn missing_acceptance_criteria_only() {
    let spec = "## Problem Statement\nX\n## Functional Requirements\n- **FR-001**: Y\n";
    let v = validate_spec_consistency(spec, &dummy_constitution());
    assert!(v.iter().any(|x| x.message.contains("Acceptance Criteria")));
}

#[test]
fn missing_all_three_sections() {
    let spec = "# Just a title\nNothing else here.";
    let v = validate_spec_consistency(spec, &dummy_constitution());
    let error_count = v
        .iter()
        .filter(|x| x.severity == ViolationSeverity::Error)
        .count();
    assert_eq!(error_count, 3);
}

#[test]
fn missing_fr_adds_warning() {
    let spec = "## Problem Statement\nX\n## Functional Requirements\nNo FRs here\n## Acceptance Criteria\nY\n";
    let v = validate_spec_consistency(spec, &dummy_constitution());
    let warning = v
        .iter()
        .find(|x| x.severity == ViolationSeverity::Warning);
    assert!(warning.is_some(), "should have a warning for missing FRs");
    assert_eq!(warning.unwrap().rule, "fr-required");
}

#[test]
fn empty_spec() {
    let v = validate_spec_consistency("", &dummy_constitution());
    assert_eq!(v.len(), 4); // 3 missing sections + 1 missing FR
}

#[test]
fn spec_with_partial_fr_markdown() {
    // Has bold FR marker but inside a section that's present
    let spec = "## Problem Statement\nX\n## Functional Requirements\n- **FR-002**: do thing\n## Acceptance Criteria\nY\n";
    let v = validate_spec_consistency(spec, &dummy_constitution());
    assert!(v.is_empty());
}

// ── enforce_governance edge cases ─────────────────────────────────────────────

#[test]
fn enforce_empty_violations_passes() {
    assert!(enforce_governance(&[]).is_ok());
}

#[test]
fn enforce_only_warnings_passes() {
    let v = vec![
        Violation {
            rule: "r1".into(),
            severity: ViolationSeverity::Warning,
            message: "w1".into(),
            location: None,
        },
        Violation {
            rule: "r2".into(),
            severity: ViolationSeverity::Warning,
            message: "w2".into(),
            location: Some("loc".into()),
        },
    ];
    assert!(enforce_governance(&v).is_ok());
}

#[test]
fn enforce_only_infos_passes() {
    let v = vec![Violation {
        rule: "r".into(),
        severity: ViolationSeverity::Info,
        message: "info".into(),
        location: None,
    }];
    assert!(enforce_governance(&v).is_ok());
}

#[test]
fn enforce_mixed_errors_and_warnings_fails() {
    let v = vec![
        Violation {
            rule: "ok".into(),
            severity: ViolationSeverity::Warning,
            message: "warn".into(),
            location: None,
        },
        Violation {
            rule: "bad".into(),
            severity: ViolationSeverity::Error,
            message: "fail".into(),
            location: None,
        },
    ];
    let err = enforce_governance(&v).unwrap_err();
    assert!(err.to_string().contains("1 error(s)"));
}

#[test]
fn enforce_multiple_errors_reports_count() {
    let v = vec![
        Violation {
            rule: "a".into(),
            severity: ViolationSeverity::Error,
            message: "e1".into(),
            location: None,
        },
        Violation {
            rule: "b".into(),
            severity: ViolationSeverity::Error,
            message: "e2".into(),
            location: None,
        },
    ];
    let err = enforce_governance(&v).unwrap_err();
    assert!(err.to_string().contains("2 error(s)"));
}

// ── Violation display ─────────────────────────────────────────────────────────

#[test]
fn violation_severity_display_variants() {
    assert_eq!(ViolationSeverity::Error.to_string(), "ERROR");
    assert_eq!(ViolationSeverity::Warning.to_string(), "WARN");
    assert_eq!(ViolationSeverity::Info.to_string(), "INFO");
}

#[test]
fn violation_severity_debug_format() {
    assert_eq!(format!("{:?}", ViolationSeverity::Error), "Error");
    assert_eq!(format!("{:?}", ViolationSeverity::Warning), "Warning");
    assert_eq!(format!("{:?}", ViolationSeverity::Info), "Info");
}

#[test]
fn violation_severity_clone_and_eq() {
    let v = ViolationSeverity::Error;
    let v2 = v.clone();
    assert_eq!(v, v2);
}
