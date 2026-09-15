use chrono::Utc;

/// Result of checking a single evidence requirement.
#[derive(Debug, Clone)]
pub struct EvidenceCheck {
    pub fr_id: String,
    pub evidence_type: String,
    pub found: bool,
    pub threshold_met: bool,
    pub message: String,
}

impl EvidenceCheck {
    fn to_markdown_row(&self) -> String {
        format!(
            "| {} | {} | {} | {} | {} |",
            self.fr_id,
            self.evidence_type,
            if self.found { "Yes" } else { "No" },
            if self.threshold_met { "Yes" } else { "N/A" },
            self.message
        )
    }
}

/// Result of evaluating a policy rule.
#[derive(Debug, Clone)]
pub struct PolicyEvalResult {
    pub policy_id: i64,
    pub domain: String,
    pub passed: bool,
    pub message: String,
}

impl PolicyEvalResult {
    fn to_markdown_row(&self) -> String {
        format!(
            "| {} | {} | {} | {} |",
            self.policy_id,
            self.domain,
            if self.passed { "Yes" } else { "No" },
            self.message
        )
    }
}

/// Aggregated validation report.
#[derive(Debug)]
pub struct ValidationReport {
    pub feature_slug: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub overall_pass: bool,
    pub evidence_results: Vec<EvidenceCheck>,
    pub policy_results: Vec<PolicyEvalResult>,
    pub missing_evidence: Vec<(String, String)>,
    pub governance_exceptions: Vec<String>,
}

impl ValidationReport {
    pub(crate) fn to_markdown(&self) -> String {
        let status = if self.overall_pass { "PASS" } else { "FAIL" };
        let mut lines = vec![
            format!("# Validation Report: {}", self.feature_slug),
            format!(
                "**Timestamp**: {} | **Result**: {}",
                self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                status
            ),
            String::new(),
            "## Evidence Checks".to_string(),
            String::new(),
        ];

        if self.evidence_results.is_empty() {
            lines.push("_(no evidence requirements defined in governance contract)_".to_string());
        } else {
            lines.push("| FR ID | Type | Found | Threshold Met | Notes |".to_string());
            lines.push("|-------|------|-------|---------------|-------|".to_string());
            for check in &self.evidence_results {
                lines.push(check.to_markdown_row());
            }
        }

        if !self.policy_results.is_empty() {
            lines.push(String::new());
            lines.push("## Policy Checks".to_string());
            lines.push(String::new());
            lines.push("| Policy ID | Domain | Passed | Notes |".to_string());
            lines.push("|-----------|--------|--------|-------|".to_string());
            for p in &self.policy_results {
                lines.push(p.to_markdown_row());
            }
        }

        if !self.missing_evidence.is_empty() {
            lines.push(String::new());
            lines.push("## Missing Evidence".to_string());
            lines.push(String::new());
            for (fr_id, ev_type) in &self.missing_evidence {
                lines.push(format!("- FR `{}`: missing `{}` evidence", fr_id, ev_type));
            }
        }

        if !self.governance_exceptions.is_empty() {
            lines.push(String::new());
            lines.push("## Governance Exceptions".to_string());
            lines.push(String::new());
            for exc in &self.governance_exceptions {
                lines.push(format!("- {exc}"));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }

    pub(crate) fn to_json(&self) -> String {
        let missing: Vec<serde_json::Value> = self
            .missing_evidence
            .iter()
            .map(|(f, t)| serde_json::json!({"fr_id": f, "type": t}))
            .collect();
        let evidence: Vec<serde_json::Value> = self
            .evidence_results
            .iter()
            .map(|e| {
                serde_json::json!({
                    "fr_id": e.fr_id,
                    "type": e.evidence_type,
                    "found": e.found,
                    "threshold_met": e.threshold_met,
                    "message": e.message,
                })
            })
            .collect();
        let policies: Vec<serde_json::Value> = self
            .policy_results
            .iter()
            .map(|p| {
                serde_json::json!({
                    "policy_id": p.policy_id,
                    "domain": p.domain,
                    "passed": p.passed,
                    "message": p.message,
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "feature_slug": self.feature_slug,
            "timestamp": self.timestamp.to_rfc3339(),
            "overall_pass": self.overall_pass,
            "evidence_results": evidence,
            "policy_results": policies,
            "missing_evidence": missing,
            "governance_exceptions": self.governance_exceptions,
        }))
        .unwrap_or_default()
    }

    pub(crate) fn summary(&self) -> String {
        let total_evidence = self.evidence_results.len();
        let passed_evidence = self
            .evidence_results
            .iter()
            .filter(|e| e.found && e.threshold_met)
            .count();
        let total_policies = self.policy_results.len();
        let passed_policies = self.policy_results.iter().filter(|p| p.passed).count();
        let missing = self.missing_evidence.len();
        let exceptions = self.governance_exceptions.len();
        let status = if self.overall_pass { "PASS" } else { "FAIL" };
        format!(
            "{}: Evidence {}/{} passed, policies {}/{} passed, missing evidence {}, exceptions {}",
            status,
            passed_evidence,
            total_evidence,
            passed_policies,
            total_policies,
            missing,
            exceptions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_report(overall_pass: bool) -> ValidationReport {
        ValidationReport {
            feature_slug: "test-feat".to_string(),
            timestamp: Utc::now(),
            overall_pass,
            evidence_results: vec![],
            policy_results: vec![],
            missing_evidence: vec![],
            governance_exceptions: vec![],
        }
    }

    #[test]
    fn to_markdown_pass_shows_pass() {
        let report = make_report(true);
        let md = report.to_markdown();
        assert!(md.contains("PASS"));
        assert!(md.contains("test-feat"));
    }

    #[test]
    fn to_markdown_fail_shows_fail() {
        let report = make_report(false);
        let md = report.to_markdown();
        assert!(md.contains("FAIL"));
    }

    #[test]
    fn to_markdown_empty_evidence_shows_placeholder() {
        let report = make_report(true);
        let md = report.to_markdown();
        assert!(md.contains("no evidence requirements defined"));
    }

    #[test]
    fn to_markdown_with_evidence_results() {
        let mut report = make_report(true);
        report.evidence_results = vec![EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "TestResult".to_string(),
            found: true,
            threshold_met: true,
            message: "OK".to_string(),
        }];
        let md = report.to_markdown();
        assert!(md.contains("Evidence Checks"));
        assert!(md.contains("FR-001"));
        assert!(md.contains("TestResult"));
    }

    #[test]
    fn to_markdown_with_policy_results() {
        let mut report = make_report(true);
        report.policy_results = vec![PolicyEvalResult {
            policy_id: 42,
            domain: "security".to_string(),
            passed: true,
            message: "All good".to_string(),
        }];
        let md = report.to_markdown();
        assert!(md.contains("Policy Checks"));
        assert!(md.contains("42"));
        assert!(md.contains("security"));
    }

    #[test]
    fn to_markdown_with_governance_exceptions() {
        let mut report = make_report(false);
        report.governance_exceptions = vec![
            "Missing signature".to_string(),
            "Stale approval".to_string(),
        ];
        let md = report.to_markdown();
        assert!(md.contains("Governance Exceptions"));
        assert!(md.contains("Missing signature"));
        assert!(md.contains("Stale approval"));
    }

    #[test]
    fn to_json_has_all_fields() {
        let mut report = make_report(true);
        report.evidence_results = vec![EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "TestResult".to_string(),
            found: true,
            threshold_met: true,
            message: "OK".to_string(),
        }];
        report.policy_results = vec![PolicyEvalResult {
            policy_id: 1,
            domain: "ci".to_string(),
            passed: true,
            message: "passed".to_string(),
        }];
        report.missing_evidence = vec![("FR-002".to_string(), "CiOutput".to_string())];
        report.governance_exceptions = vec!["exception".to_string()];
        let json = report.to_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["evidence_results"].as_array().unwrap().len(), 1);
        assert_eq!(v["policy_results"].as_array().unwrap().len(), 1);
        assert_eq!(v["missing_evidence"].as_array().unwrap().len(), 1);
        assert_eq!(v["governance_exceptions"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn summary_pass_format() {
        let mut report = make_report(true);
        report.evidence_results = vec![EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "TestResult".to_string(),
            found: true,
            threshold_met: true,
            message: "OK".to_string(),
        }];
        let s = report.summary();
        assert!(s.starts_with("PASS"));
        assert!(s.contains("Evidence 1/1"));
    }

    #[test]
    fn summary_fail_format() {
        let mut report = make_report(false);
        report.missing_evidence = vec![("FR-001".to_string(), "any".to_string())];
        let s = report.summary();
        assert!(s.starts_with("FAIL"));
        assert!(s.contains("missing evidence 1"));
    }
}
