//! Governance checks for CLI planning commands.
//!
//! Loads the project constitution (if present) and validates spec consistency
//! against governance rules before state transitions proceed.
//! Traceability: FR-009 / WP11-T064

use agileplus_domain::ports::VcsPort;
use anyhow::Result;

/// A governance violation found during spec validation.
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub location: Option<String>,
}

/// Severity level for governance violations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// A parsed constitution with governance rules.
#[derive(Debug)]
pub struct Constitution {
    pub content: String,
}

/// Attempt to load the project constitution from well-known paths.
pub async fn load_constitution<V: VcsPort>(vcs: &V) -> Option<Constitution> {
    // Try .kittify/memory/constitution.md
    if let Ok(content) = vcs
        .read_artifact("", "../.kittify/memory/constitution.md")
        .await
    {
        return Some(Constitution { content });
    }
    None
}

/// Validate spec content against basic structural governance rules.
pub fn validate_spec_consistency(
    spec_content: &str,
    _constitution: &Constitution,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Check required sections
    let required_sections = [
        (
            "## Problem Statement",
            "Problem Statement section is required",
        ),
        (
            "## Functional Requirements",
            "Functional Requirements section is required",
        ),
        (
            "## Acceptance Criteria",
            "Acceptance Criteria section is required",
        ),
    ];

    for (section, message) in &required_sections {
        if !spec_content.contains(section) {
            violations.push(Violation {
                rule: "required-sections".into(),
                severity: ViolationSeverity::Error,
                message: message.to_string(),
                location: None,
            });
        }
    }

    // Check for at least one FR
    if !spec_content.contains("**FR-") {
        violations.push(Violation {
            rule: "fr-required".into(),
            severity: ViolationSeverity::Warning,
            message: "No functional requirements found (expected **FR-N**: format)".into(),
            location: Some("## Functional Requirements".into()),
        });
    }

    violations
}

/// Enforce governance: return Err if error-severity violations exist.
/// For warning violations, print them but continue.
pub fn enforce_governance(violations: &[Violation]) -> Result<()> {
    let errors: Vec<_> = violations
        .iter()
        .filter(|v| v.severity == ViolationSeverity::Error)
        .collect();
    let warnings: Vec<_> = violations
        .iter()
        .filter(|v| v.severity == ViolationSeverity::Warning)
        .collect();
    let infos: Vec<_> = violations
        .iter()
        .filter(|v| v.severity == ViolationSeverity::Info)
        .collect();

    for v in &infos {
        println!("[INFO] {}: {}", v.rule, v.message);
    }
    for v in &warnings {
        println!("[WARN] {}: {}", v.rule, v.message);
    }

    if !errors.is_empty() {
        for v in &errors {
            eprintln!("[ERROR] {}: {}", v.rule, v.message);
        }
        anyhow::bail!("Governance checks failed with {} error(s)", errors.len());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_constitution() -> Constitution {
        Constitution {
            content: String::new(),
        }
    }

    #[test]
    fn validates_required_sections() {
        let spec = "# Spec\n## Problem Statement\nfoo\n## Functional Requirements\n- **FR-1**: bar\n## Acceptance Criteria\nbaz\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        assert!(
            violations.is_empty(),
            "should have no violations: {violations:?}"
        );
    }

    #[test]
    fn detects_missing_sections() {
        let spec = "# Spec\nno sections here";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        assert!(
            violations
                .iter()
                .any(|v| v.severity == ViolationSeverity::Error)
        );
    }

    #[test]
    fn enforce_passes_with_warnings() {
        let violations = vec![Violation {
            rule: "test".into(),
            severity: ViolationSeverity::Warning,
            message: "a warning".into(),
            location: None,
        }];
        assert!(enforce_governance(&violations).is_ok());
    }

    #[test]
    fn enforce_fails_with_errors() {
        let violations = vec![Violation {
            rule: "test".into(),
            severity: ViolationSeverity::Error,
            message: "an error".into(),
            location: None,
        }];
        assert!(enforce_governance(&violations).is_err());
    }

    // ── additional tests ───────────────────────────────────────────────

    #[test]
    fn violation_severity_display() {
        assert_eq!(ViolationSeverity::Error.to_string(), "ERROR");
        assert_eq!(ViolationSeverity::Warning.to_string(), "WARN");
        assert_eq!(ViolationSeverity::Info.to_string(), "INFO");
    }

    #[test]
    fn validates_all_required_sections() {
        let spec = "# Spec\n## Problem Statement\nps\n## Functional Requirements\n- **FR-1**: x\n## Acceptance Criteria\nac\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        assert!(violations.is_empty(), "should have no violations: {violations:?}");
    }

    #[test]
    fn detects_missing_problem_statement() {
        let spec = "# Spec\n## Functional Requirements\n- **FR-1**: x\n## Acceptance Criteria\nac\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        let msgs: Vec<&str> = violations.iter().map(|v| v.message.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("Problem Statement")));
    }

    #[test]
    fn detects_missing_functional_requirements() {
        let spec = "# Spec\n## Problem Statement\nps\n## Acceptance Criteria\nac\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        let msgs: Vec<&str> = violations.iter().map(|v| v.message.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("Functional Requirements")));
    }

    #[test]
    fn detects_missing_acceptance_criteria() {
        let spec = "# Spec\n## Problem Statement\nps\n## Functional Requirements\n- **FR-1**: x\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        let msgs: Vec<&str> = violations.iter().map(|v| v.message.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("Acceptance Criteria")));
    }

    #[test]
    fn warns_when_no_fr_found() {
        let spec = "# Spec\n## Problem Statement\nps\n## Functional Requirements\nno FR here\n## Acceptance Criteria\nac\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        let has_fr_warning = violations.iter().any(|v| v.rule == "fr-required");
        assert!(has_fr_warning, "should warn about missing FRs");
    }

    #[test]
    fn no_fr_warning_when_present() {
        let spec = "# Spec\n## Problem Statement\nps\n## Functional Requirements\n- **FR-1**: login\n## Acceptance Criteria\nac\n";
        let violations = validate_spec_consistency(spec, &dummy_constitution());
        let has_fr_warning = violations.iter().any(|v| v.rule == "fr-required");
        assert!(!has_fr_warning, "should not warn about FRs");
    }

    #[test]
    fn enforce_empty_violations_passes() {
        assert!(enforce_governance(&[]).is_ok());
    }

    #[test]
    fn enforce_only_infos_passes() {
        let violations = vec![Violation {
            rule: "test".into(),
            severity: ViolationSeverity::Info,
            message: "just info".into(),
            location: None,
        }];
        assert!(enforce_governance(&violations).is_ok());
    }

    #[test]
    fn enforce_mixed_warnings_and_errors_fails() {
        let violations = vec![
            Violation {
                rule: "w".into(),
                severity: ViolationSeverity::Warning,
                message: "a warning".into(),
                location: None,
            },
            Violation {
                rule: "e".into(),
                severity: ViolationSeverity::Error,
                message: "an error".into(),
                location: None,
            },
        ];
        assert!(enforce_governance(&violations).is_err());
    }

    #[test]
    fn violation_with_location() {
        let v = Violation {
            rule: "test".into(),
            severity: ViolationSeverity::Warning,
            message: "located warning".into(),
            location: Some("src/main.rs:10".into()),
        };
        assert_eq!(v.location.as_deref(), Some("src/main.rs:10"));
    }
}
