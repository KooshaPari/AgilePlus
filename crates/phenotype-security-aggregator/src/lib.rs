//! Phenotype Security Aggregator
//!
//! Aggregates security findings from multiple sources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity level for security findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// Critical severity
    Critical,
    /// High severity
    High,
    /// Medium severity
    Medium,
    /// Low severity
    Low,
    /// Info-level severity
    Info,
}

impl Severity {
    /// Get numeric value for sorting
    pub fn numeric_value(&self) -> u8 {
        match self {
            Severity::Critical => 5,
            Severity::High => 4,
            Severity::Medium => 3,
            Severity::Low => 2,
            Severity::Info => 1,
        }
    }
}

/// Alert source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AlertSource {
    /// Snyk security scanner
    Snyk,
    /// GitHub CodeQL
    CodeQL,
    /// Cargo audit
    CargoAudit,
    /// GitHub Dependabot
    Dependabot,
    /// Trivy scanner
    Trivy,
    /// Custom source
    Custom(String),
}

impl AlertSource {
    /// Get short name for display
    pub fn short_name(&self) -> &'static str {
        match self {
            AlertSource::Snyk => "SNYK",
            AlertSource::CodeQL => "CODEQL",
            AlertSource::CargoAudit => "CARGO",
            AlertSource::Dependabot => "DEPND",
            AlertSource::Trivy => "TRIVY",
            AlertSource::Custom(_) => "CUST",
        }
    }
}

/// A security finding from a scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Unique identifier
    pub id: String,
    /// Finding title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Source of the finding
    pub source: AlertSource,
    /// Package or file affected
    pub target: String,
    /// When it was detected
    pub detected_at: DateTime<Utc>,
}

impl SecurityFinding {
    /// Create a new security finding
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        severity: Severity,
        source: AlertSource,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            severity,
            source,
            target: target.into(),
            detected_at: Utc::now(),
        }
    }

    /// Get CVSS score if available (placeholder)
    pub fn cvss_score(&self) -> Option<f32> {
        match self.severity {
            Severity::Critical => Some(9.5),
            Severity::High => Some(7.5),
            Severity::Medium => Some(5.0),
            Severity::Low => Some(2.5),
            Severity::Info => Some(0.0),
        }
    }
}

/// Aggregates security findings from multiple sources
#[derive(Debug, Clone, Default)]
pub struct SecurityAggregator {
    findings: Vec<SecurityFinding>,
}

impl SecurityAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a finding
    pub fn add_finding(&mut self, finding: SecurityFinding) {
        self.findings.push(finding);
    }

    /// Add multiple findings
    pub fn add_findings(&mut self, findings: impl IntoIterator<Item = SecurityFinding>) {
        self.findings.extend(findings);
    }

    /// Get all findings
    pub fn findings(&self) -> &[SecurityFinding] {
        &self.findings
    }

    /// Get findings by severity
    pub fn by_severity(&self, severity: Severity) -> Vec<&SecurityFinding> {
        self.findings.iter().filter(|f| f.severity == severity).collect()
    }

    /// Get findings by source
    pub fn by_source(&self, source: &AlertSource) -> Vec<&SecurityFinding> {
        self.findings.iter().filter(|f| f.source == *source).collect()
    }

    /// Get unique targets (packages/files) with findings
    pub fn unique_targets(&self) -> Vec<&str> {
        let mut targets: Vec<&str> = self.findings.iter().map(|f| f.target.as_str()).collect();
        targets.sort();
        targets.dedup();
        targets
    }

    /// Count findings by severity
    pub fn count_by_severity(&self) -> std::collections::HashMap<Severity, usize> {
        let mut counts = std::collections::HashMap::new();
        for finding in &self.findings {
            *counts.entry(finding.severity).or_insert(0) += 1;
        }
        counts
    }

    /// Sort findings by severity (most severe first)
    pub fn sorted_by_severity(&self) -> Vec<&SecurityFinding> {
        let mut findings: Vec<&SecurityFinding> = self.findings.iter().collect();
        findings.sort_by(|a, b| {
            b.severity.numeric_value().cmp(&a.severity.numeric_value())
        });
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical.numeric_value() > Severity::High.numeric_value());
        assert!(Severity::High.numeric_value() > Severity::Medium.numeric_value());
    }

    #[test]
    fn test_aggregator() {
        let mut aggregator = SecurityAggregator::new();
        
        aggregator.add_finding(SecurityFinding::new(
            "CVE-2024-1234",
            "Remote Code Execution",
            "A vulnerability allows remote code execution",
            Severity::Critical,
            AlertSource::Snyk,
            "package-a@1.0.0",
        ));

        assert_eq!(aggregator.findings().len(), 1);
        assert_eq!(aggregator.unique_targets(), vec!["package-a@1.0.0"]);
    }
}
