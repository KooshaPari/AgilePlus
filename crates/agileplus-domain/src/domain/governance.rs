// SPDX-License-Identifier: MIT OR Apache-2.0
//! Governance types — contracts, rules, and evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Policy domain category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyDomain {
    Security,
    Quality,
    Compliance,
    Performance,
    Custom,
}

impl PolicyDomain {
    pub fn as_str(self) -> &'static str {
        match self {
            PolicyDomain::Security => "security",
            PolicyDomain::Quality => "quality",
            PolicyDomain::Compliance => "compliance",
            PolicyDomain::Performance => "performance",
            PolicyDomain::Custom => "custom",
        }
    }
}

/// The definition of a policy rule (stored as JSON blob).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub description: String,
    pub check: PolicyCheck,
}

/// An active policy rule in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: i64,
    pub domain: PolicyDomain,
    pub rule: PolicyDefinition,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A required-evidence entry inside a governance rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    /// Functional-requirement ID the evidence must satisfy.
    pub fr_id: String,
    /// Type of evidence required.
    pub evidence_type: EvidenceType,
}

/// A governance rule captured inside a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    pub transition: String,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub policy_refs: Vec<i64>,
}

/// A versioned governance contract bound to a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub id: i64,
    pub feature_id: i64,
    pub version: i32,
    pub rules: Vec<GovernanceRule>,
    pub bound_at: DateTime<Utc>,
}

/// Type of evidence artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    TestResult,
    CiOutput,
    ReviewApproval,
    SecurityScan,
    LintResult,
    ManualAttestation,
}

impl EvidenceType {
    pub fn as_str(self) -> &'static str {
        match self {
            EvidenceType::TestResult => "test_result",
            EvidenceType::CiOutput => "ci_output",
            EvidenceType::ReviewApproval => "review_approval",
            EvidenceType::SecurityScan => "security_scan",
            EvidenceType::LintResult => "lint_result",
            EvidenceType::ManualAttestation => "manual_attestation",
        }
    }
}

/// An evidence artifact attached to a work package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: i64,
    pub wp_id: i64,
    pub fr_id: String,
    pub evidence_type: EvidenceType,
    pub artifact_path: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// The result of a policy check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyCheck {
    ManualApproval,
    Automated,
}

/// A well-known built-in policy that maps a short reference key to a
/// `PolicyDomain` + `EvidenceType` pair.  Used by the `validate` command to
/// resolve policy references without requiring a database lookup.
#[derive(Debug, Clone, Copy)]
pub struct BuiltinPolicy {
    /// Short human-readable label (e.g. "Unit tests passing").
    pub label: &'static str,
    /// Governance domain for grouping.
    pub domain: PolicyDomain,
    /// Required evidence kind.
    pub evidence_type: EvidenceType,
}

impl BuiltinPolicy {
    /// Well-known built-in policies keyed by their reference string.
    const KNOWN: &'static [(&'static str, BuiltinPolicy)] = &[
        (
            "tests-pass",
            BuiltinPolicy {
                label: "Unit tests passing",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::TestResult,
            },
        ),
        (
            "ci-green",
            BuiltinPolicy {
                label: "CI pipeline green",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::CiOutput,
            },
        ),
        (
            "review-approved",
            BuiltinPolicy {
                label: "Peer review approved",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::ReviewApproval,
            },
        ),
        (
            "security-scan",
            BuiltinPolicy {
                label: "Security scan clean",
                domain: PolicyDomain::Security,
                evidence_type: EvidenceType::SecurityScan,
            },
        ),
        (
            "lint-pass",
            BuiltinPolicy {
                label: "Lint checks pass",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::LintResult,
            },
        ),
    ];

    /// Look up a built-in policy by its reference key.
    /// Returns `None` for unknown (custom) policy references.
    pub fn from_ref(policy_ref: &str) -> Option<&'static BuiltinPolicy> {
        Self::KNOWN
            .iter()
            .find(|(key, _)| *key == policy_ref)
            .map(|(_, bp)| bp)
    }
}
