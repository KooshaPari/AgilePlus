//! Governance domain types — contracts, rules, evidence, and policies.
//!
//! Traceability: FR-GOVERN-* / WP04

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The kind of evidence that satisfies a governance requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Stable storage/reporting key for this evidence type.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TestResult => "test_result",
            Self::CiOutput => "ci_output",
            Self::ReviewApproval => "review_approval",
            Self::SecurityScan => "security_scan",
            Self::LintResult => "lint_result",
            Self::ManualAttestation => "manual_attestation",
        }
    }
}

/// A single evidence requirement attached to a governance rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub fr_id: String,
    pub evidence_type: EvidenceType,
    pub threshold: Option<serde_json::Value>,
}

/// A governance rule controlling a specific state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    pub transition: String,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub policy_refs: Vec<String>,
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

/// A piece of evidence collected during work-package execution.
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

/// The domain a policy rule applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDomain {
    Security,
    Quality,
    Compliance,
    Performance,
    Custom,
}

impl PolicyDomain {
    /// Stable storage/reporting key for this policy domain.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Quality => "quality",
            Self::Compliance => "compliance",
            Self::Performance => "performance",
            Self::Custom => "custom",
        }
    }
}

/// The definition body of a policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub description: String,
    pub check: PolicyCheck,
}

/// How a policy is evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCheck {
    EvidencePresent { evidence_type: EvidenceType },
    ThresholdMet { metric: String, min: f64 },
    ManualApproval,
    Custom { script: String },
}

impl PolicyCheck {
    /// Evidence type required by checks that can be satisfied with stored evidence.
    pub fn required_evidence_type(&self) -> Option<EvidenceType> {
        match self {
            Self::EvidencePresent { evidence_type } => Some(*evidence_type),
            Self::ManualApproval => Some(EvidenceType::ManualAttestation),
            Self::ThresholdMet { .. } | Self::Custom { .. } => None,
        }
    }
}

/// Outcome of evaluating a policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyResult {
    Pass,
    Fail { reason: String },
    Skipped { reason: String },
}

/// A reusable policy rule stored in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: i64,
    pub domain: PolicyDomain,
    pub rule: PolicyDefinition,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A built-in policy reference emitted by generated governance contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinPolicy {
    pub label: &'static str,
    pub domain: PolicyDomain,
    pub evidence_type: EvidenceType,
}

impl BuiltinPolicy {
    pub fn from_ref(policy_ref: &str) -> Option<Self> {
        let (label, domain, evidence_type) = match policy_ref {
            "policy:ci-required" => (
                "CI evidence required",
                PolicyDomain::Quality,
                EvidenceType::CiOutput,
            ),
            "policy:review-required" => (
                "Review approval required",
                PolicyDomain::Quality,
                EvidenceType::ReviewApproval,
            ),
            "policy:security-scan-required" => (
                "Security scan evidence required",
                PolicyDomain::Security,
                EvidenceType::SecurityScan,
            ),
            "policy:lint-required" => (
                "Lint evidence required",
                PolicyDomain::Quality,
                EvidenceType::LintResult,
            ),
            "policy:test-result-required" => (
                "Test result evidence required",
                PolicyDomain::Quality,
                EvidenceType::TestResult,
            ),
            "policy:manual-attestation-required" => (
                "Manual attestation required",
                PolicyDomain::Compliance,
                EvidenceType::ManualAttestation,
            ),
            _ => return None,
        };
        Some(Self {
            label,
            domain,
            evidence_type,
        })
    }
}

impl PolicyRule {
    /// Stable references that can bind governance contract policy refs to this rule.
    pub fn reference_keys(&self) -> Vec<String> {
        let mut keys = vec![
            format!("policy:{}", self.id),
            format!("policy:{}", self.domain.as_str()),
        ];

        if let Some(evidence_type) = self.rule.check.required_evidence_type() {
            keys.push(format!("policy:{}-required", evidence_type.as_str()));
        }

        keys
    }

    /// Whether a governance contract policy ref targets this rule.
    pub fn matches_reference(&self, policy_ref: &str) -> bool {
        self.reference_keys().iter().any(|key| key == policy_ref)
    }
}
