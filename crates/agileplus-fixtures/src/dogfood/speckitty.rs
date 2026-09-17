// SPDX-License-Identifier: MIT OR Apache-2.0
//! SpecKitty reference-spec catalogue used by the dogfood seed.
//!
//! Extracted from `dogfood.rs` so the seed orchestration stays readable.

/// Return the canonical SpecKitty reference specs: `(feature_id, slug, name, labels)`.
pub(super) fn speckitty_specs() -> Vec<(i64, &'static str, &'static str, Vec<String>)> {
    vec![
        (
            5,
            "sk-001-mission-system-architecture",
            "Mission System Architecture",
            vec!["architecture".to_owned(), "specKitty".to_owned()],
        ),
        (
            6,
            "sk-002-lightweight-pypi-release",
            "Lightweight PyPI Release",
            vec!["release".to_owned(), "specKitty".to_owned()],
        ),
        (
            7,
            "sk-003-auto-protect-agent",
            "Auto-Protect Agent",
            vec!["agents".to_owned(), "specKitty".to_owned()],
        ),
        (
            8,
            "sk-004-modular-code-refactoring",
            "Modular Code Refactoring",
            vec!["refactoring".to_owned(), "specKitty".to_owned()],
        ),
        (
            9,
            "sk-005-refactor-mission-system",
            "Refactor Mission System",
            vec!["architecture".to_owned(), "specKitty".to_owned()],
        ),
        (
            10,
            "sk-007-frontmatter-only-lane",
            "Frontmatter-Only Lane",
            vec!["documentation".to_owned(), "specKitty".to_owned()],
        ),
        (
            11,
            "sk-008-unified-python-cli",
            "Unified Python CLI",
            vec!["cli".to_owned(), "specKitty".to_owned()],
        ),
        (
            12,
            "sk-010-workspace-per-work-package",
            "Workspace Per Work Package for Parallel Development",
            vec!["organization".to_owned(), "specKitty".to_owned()],
        ),
        (
            13,
            "sk-011-constitution-packaging-safety",
            "Constitution Packaging Safety and Redesign",
            vec!["infrastructure".to_owned(), "specKitty".to_owned()],
        ),
        (
            14,
            "sk-012-documentation-mission",
            "Documentation Mission",
            vec!["documentation".to_owned(), "specKitty".to_owned()],
        ),
        (
            15,
            "sk-013-fix-and-test-dashboard",
            "Fix and Test Dashboard",
            vec!["testing".to_owned(), "specKitty".to_owned()],
        ),
        (
            16,
            "sk-014-comprehensive-end-user-documentation",
            "Comprehensive End-User Documentation",
            vec!["documentation".to_owned(), "specKitty".to_owned()],
        ),
        (
            17,
            "sk-015-first-class-jujutsu-vcs-integration",
            "First-Class Jujutsu VCS Integration",
            vec!["vcs".to_owned(), "specKitty".to_owned()],
        ),
        (
            18,
            "sk-016-jujutsu-vcs-documentation",
            "Jujutsu VCS Documentation",
            vec![
                "documentation".to_owned(),
                "vcs".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            19,
            "sk-017-smarter-feature-merge-with-preflight",
            "Smarter Feature Merge with Preflight",
            vec!["vcs".to_owned(), "specKitty".to_owned()],
        ),
        (
            20,
            "sk-018-merge-preflight-documentation",
            "Merge Preflight Documentation",
            vec!["documentation".to_owned(), "specKitty".to_owned()],
        ),
        (
            21,
            "sk-019-autonomous-multi-agent-orchestration-research",
            "Autonomous Multi-Agent Orchestration Research",
            vec![
                "agents".to_owned(),
                "research".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            22,
            "sk-020-autonomous-multi-agent-orchestrator",
            "Autonomous Multi-Agent Orchestrator",
            vec!["agents".to_owned(), "specKitty".to_owned()],
        ),
        (
            23,
            "sk-021-orchestrator-end-to-end-testing-suite",
            "Orchestrator End-to-End Testing Suite",
            vec![
                "testing".to_owned(),
                "agents".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            24,
            "sk-022-orchestrator-user-documentation",
            "Orchestrator User Documentation",
            vec!["documentation".to_owned(), "specKitty".to_owned()],
        ),
        (
            25,
            "sk-023-documentation-sprint-agent-management-cleanup",
            "Documentation Sprint Agent Management Cleanup",
            vec![
                "documentation".to_owned(),
                "agents".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            26,
            "sk-024-adversarial-test-suite-0-13-0",
            "Adversarial Test Suite v0.13.0",
            vec!["testing".to_owned(), "specKitty".to_owned()],
        ),
        (
            27,
            "sk-025-cli-event-log-integration",
            "CLI Event Log Integration",
            vec!["cli".to_owned(), "specKitty".to_owned()],
        ),
        (
            28,
            "sk-026-agent-directory-centralization-architecture-research",
            "Agent Directory Centralization Architecture Research",
            vec![
                "agents".to_owned(),
                "architecture".to_owned(),
                "research".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            29,
            "sk-027-cli-authentication-module-commands",
            "CLI Authentication Module Commands",
            vec!["cli".to_owned(), "specKitty".to_owned()],
        ),
        (
            30,
            "sk-028-cli-event-emission-sync",
            "CLI Event Emission Sync",
            vec!["cli".to_owned(), "sync".to_owned(), "specKitty".to_owned()],
        ),
        (
            31,
            "sk-029-mission-aware-cleanup-docs-wiring",
            "Mission-Aware Cleanup Docs Wiring",
            vec!["documentation".to_owned(), "specKitty".to_owned()],
        ),
        (
            32,
            "sk-030-2x-sync-auth-docs",
            "2x Sync Auth Docs",
            vec![
                "documentation".to_owned(),
                "sync".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            33,
            "sk-032-identity-aware-cli-event-sync",
            "Identity-Aware CLI Event Sync",
            vec!["cli".to_owned(), "sync".to_owned(), "specKitty".to_owned()],
        ),
        (
            34,
            "sk-038-v0-15-0-quality-bugfix-release",
            "v0.15.0 Quality Bugfix Release",
            vec!["release".to_owned(), "specKitty".to_owned()],
        ),
        (
            35,
            "sk-039-cli-2x-readiness",
            "CLI 2x Readiness",
            vec!["cli".to_owned(), "specKitty".to_owned()],
        ),
        (
            36,
            "sk-040-mission-collaboration-cli-soft-coordination",
            "Mission Collaboration CLI Soft Coordination",
            vec![
                "cli".to_owned(),
                "agents".to_owned(),
                "specKitty".to_owned(),
            ],
        ),
        (
            37,
            "sk-041-enable-plan-mission-runtime-support",
            "Enable Plan Mission Runtime Support",
            vec!["agents".to_owned(), "specKitty".to_owned()],
        ),
    ]
}
