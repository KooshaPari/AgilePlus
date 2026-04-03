# POLICY_RULES.md — Policy Rule Reference

**Version**: 1.0 | **Status**: Complete | **Crate**: `phenotype-policy-engine`

## Overview

Policy rules in AgilePlus are evaluated using the `phenotype-policy-engine` crate. Rules use regex patterns to match against facts in an evaluation context.

---

## Rule Structure

```rust
pub struct Rule {
    pub rule_type: RuleType,     // Allow | Deny | Require
    pub fact: String,            // Key to look up in context
    pub pattern: String,         // Regex pattern to match
    pub description: Option<String>,
    pub priority: u32,           // Lower = evaluated first (default: 100)
    pub severity: Severity,      // Info | Warning | Error (default: Error)
}
```

### Rule Types

| Type | Behavior | Fact Missing | Fact Matches | Fact No Match |
|------|----------|--------------|--------------|---------------|
| **Allow** | Pass if matches | ✅ Pass | ✅ Pass | ❌ Fail |
| **Deny** | Pass if NOT matches | ✅ Pass | ❌ Fail | ✅ Pass |
| **Require** | Must exist and match | ❌ Fail | ✅ Pass | ❌ Fail |

### Severity Levels

| Level | Value | Behavior |
|-------|-------|----------|
| **Info** | 0 | Logged, doesn't affect pass/fail |
| **Warning** | 1 | Logged, doesn't block (configurable) |
| **Error** | 2 | Blocks transition/operation |

---

## Rule Evaluation

### Evaluation Context

Facts are stored in an `EvaluationContext`:

```rust
use phenotype_policy_engine::EvaluationContext;

let mut ctx = EvaluationContext::new();
ctx.set_string("status", "active");
ctx.set_number("coverage", 85.5);
ctx.set_bool("tests_passed", true);
ctx.set("nested", serde_json::json!({"key": "value"}));
```

### Evaluation Logic

```rust
use phenotype_policy_engine::{Policy, Rule, RuleType, Severity};

let policy = Policy::new("security-check")
    .add_rule(
        Rule::new(RuleType::Require, "security_scan", "^passed$")
            .with_severity(Severity::Error)
            .with_priority(1)
            .with_description("Security scan must pass")
    );

let result = policy.evaluate(&ctx)?;
// result.passed: bool
// result.violations: Vec<Violation>
```

---

## Common Rule Patterns

### Security Rules

```toml
# Require security scan passed
[[policies.rules]]
rule_type = "Require"
fact = "security_scan_status"
pattern = "^passed$"
severity = "Error"

# Deny critical CVEs
[[policies.rules]]
rule_type = "Deny"
fact = "critical_cve_count"
pattern = "^[1-9]"
severity = "Error"

# Require no high CVEs (0 only)
[[policies.rules]]
rule_type = "Require"
fact = "high_cve_count"
pattern = "^0$"
severity = "Error"
```

### Quality Rules

```toml
# Require test coverage >= 80%
[[policies.rules]]
rule_type = "Require"
fact = "test_coverage"
pattern = "^(8[0-9]|9[0-9]|100)$"
severity = "Warning"

# Require all tests passed
[[policies.rules]]
rule_type = "Require"
fact = "test_status"
pattern = "^passed$"
severity = "Error"

# Deny lint errors
[[policies.rules]]
rule_type = "Deny"
fact = "lint_errors"
pattern = "^[1-9]"
severity = "Error"
```

### Process Rules

```toml
# Require spec to have FRs
[[policies.rules]]
rule_type = "Require"
fact = "fr_count"
pattern = "^[1-9][0-9]*$"
severity = "Error"

# Require review approval
[[policies.rules]]
rule_type = "Require"
fact = "review_status"
pattern = "^approved$"
severity = "Error"

# Allow emergency bypass (with pattern)
[[policies.rules]]
rule_type = "Allow"
fact = "emergency_bypass"
pattern = "^true$"
severity = "Warning"
```

### Evidence Rules

```toml
# Require test result evidence
[[policies.rules]]
rule_type = "Require"
fact = "evidence_test_result"
pattern = "^present$"
severity = "Error"

# Require security scan evidence
[[policies.rules]]
rule_type = "Require"
fact = "evidence_security_scan"
pattern = "^present$"
severity = "Error"
```

---

## Priority and Ordering

Rules are evaluated in priority order (lowest first):

```rust
let policy = Policy::new("prioritized")
    .add_rule(
        Rule::new(RuleType::Deny, "banned", "^true$")
            .with_priority(1)  // Evaluated first
    )
    .add_rule(
        Rule::new(RuleType::Require, "email", ".*")
            .with_priority(10)  // Evaluated second
    );
```

**Use case**: Early deny rules can short-circuit evaluation for efficiency:
- Priority 1-10: Critical security checks (always run first)
- Priority 11-50: Important requirements
- Priority 51-100: Standard checks (default)
- Priority 101+: Nice-to-have checks

---

## TOML Configuration Format

### Complete Policy File

```toml
# .agileplus/policies.toml

# Policy: Security ship gate
[[policies]]
name = "security-ship-gate"
description = "Security requirements for shipping features"
enabled = true

[[policies.rules]]
rule_type = "Require"
fact = "security_scan"
pattern = "^passed$"
severity = "Error"
priority = 1
description = "Security scan must pass"

[[policies.rules]]
rule_type = "Deny"
fact = "critical_cves"
pattern = "^[1-9]"
severity = "Error"
priority = 2
description = "No critical CVEs allowed"

# Policy: Quality standards
[[policies]]
name = "quality-standards"
description = "Code quality requirements"
enabled = true

[[policies.rules]]
rule_type = "Require"
fact = "test_coverage"
pattern = "^[8-9][0-9]$|^100$"
severity = "Warning"
priority = 10
description = "Test coverage >= 80%"

[[policies.rules]]
rule_type = "Deny"
fact = "clippy_warnings"
pattern = "^[5-9][0-9]$|^[1-9][0-9]{2,}"
severity = "Warning"
priority = 20
description = "Warn on >50 clippy warnings"
```

### Loading Configuration

```rust
use phenotype_policy_engine::loader::PolicyLoader;
use std::path::Path;

// From file
let policies = PolicyLoader::from_file(Path::new(".agileplus/policies.toml"))?;

// From string
let toml = r#"
[[policies]]
name = "test"
[[policies.rules]]
rule_type = "Require"
fact = "status"
pattern = "^active$"
"#;
let policies = PolicyLoader::from_string(toml)?;
```

---

## Programmatic Rule Creation

### Builder Pattern

```rust
use phenotype_policy_engine::{Policy, Rule, RuleType, Severity};

let policy = Policy::new("feature-governance")
    .with_description("Feature lifecycle governance")
    .add_rule(
        Rule::new(RuleType::Require, "spec_complete", "^true$")
            .with_severity(Severity::Error)
            .with_priority(1)
            .with_description("Specification must be complete")
    )
    .add_rule(
        Rule::new(RuleType::Require, "review_approved", "^true$")
            .with_severity(Severity::Error)
            .with_priority(2)
            .with_description("Code review must be approved")
    )
    .add_rule(
        Rule::new(RuleType::Require, "tests_pass", "^true$")
            .with_severity(Severity::Error)
            .with_priority(3)
            .with_description("All tests must pass")
    );
```

### Dynamic Rule Generation

```rust
fn create_fr_rules(fr_ids: &[String]) -> Vec<Rule> {
    fr_ids.iter()
        .map(|fr_id| {
            Rule::new(
                RuleType::Require,
                format!("evidence_{}", fr_id),
                "^present$"
            )
            .with_severity(Severity::Error)
            .with_description(format!("Evidence required for {}", fr_id))
        })
        .collect()
}
```

---

## Integration with Governance Contracts

### Contract with Embedded Policies

```rust
use agileplus_domain::domain::governance::*;
use chrono::Utc;

let contract = GovernanceContract {
    id: 1,
    feature_id: 42,
    version: 1,
    rules: vec![
        GovernanceRule {
            transition: "Implementing -> Validated".to_string(),
            required_evidence: vec![
                EvidenceRequirement {
                    fr_id: "FR-001".to_string(),
                    evidence_type: EvidenceType::TestResult,
                    threshold: None,
                },
                EvidenceRequirement {
                    fr_id: "FR-002".to_string(),
                    evidence_type: EvidenceType::SecurityScan,
                    threshold: None,
                },
            ],
            policy_refs: vec!["security-ship-gate".to_string()],
        },
    ],
    bound_at: Utc::now(),
};
```

### Evaluating Contract Rules

```rust
async fn evaluate_contract(
    contract: &GovernanceContract,
    storage: &impl StoragePort,
) -> Result<ValidationReport> {
    let mut violations = Vec::new();
    
    for rule in &contract.rules {
        for req in &rule.required_evidence {
            let evidence = storage
                .get_evidence_by_fr(&req.fr_id)
                .await?;
            
            if evidence.is_empty() {
                violations.push(Violation {
                    rule: format!("evidence-{}", req.fr_id),
                    severity: ViolationSeverity::Error,
                    message: format!(
                        "Missing evidence: {:?} for {}",
                        req.evidence_type, req.fr_id
                    ),
                    location: Some(rule.transition.clone()),
                });
            }
        }
    }
    
    Ok(ValidationReport {
        passed: violations.is_empty(),
        violations,
        ..Default::default()
    })
}
```

---

## Testing Rules

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use phenotype_policy_engine::*;

    #[test]
    fn test_security_scan_rule() {
        let rule = Rule::new(
            RuleType::Require,
            "security_scan",
            "^passed$"
        );
        
        // Should pass with matching fact
        let mut ctx = EvaluationContext::new();
        ctx.set_string("security_scan", "passed");
        assert!(rule.evaluate(&ctx).unwrap());
        
        // Should fail without fact
        let empty_ctx = EvaluationContext::new();
        assert!(!rule.evaluate(&empty_ctx).unwrap());
        
        // Should fail with non-matching fact
        let mut ctx = EvaluationContext::new();
        ctx.set_string("security_scan", "failed");
        assert!(!rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_coverage_threshold() {
        let rule = Rule::new(
            RuleType::Require,
            "coverage",
            "^(8[0-9]|9[0-9]|100)$"
        );
        
        let mut ctx = EvaluationContext::new();
        ctx.set_string("coverage", "85");
        assert!(rule.evaluate(&ctx).unwrap());
        
        ctx.set_string("coverage", "75");
        assert!(!rule.evaluate(&ctx).unwrap());
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn allow_rule_passes_on_missing_fact(s in "[a-z]*") {
        let rule = Rule::new(RuleType::Allow, "fact", &s);
        let ctx = EvaluationContext::new();
        prop_assert!(rule.evaluate(&ctx).unwrap());
    }
    
    #[test]
    fn require_rule_fails_on_missing_fact(s in "[a-z]+") {
        let rule = Rule::new(RuleType::Require, "fact", &s);
        let ctx = EvaluationContext::new();
        prop_assert!(!rule.evaluate(&ctx).unwrap());
    }
}
```

---

## Performance Considerations

### Regex Compilation

Rules pre-compile regex patterns on creation:

```rust
impl Rule {
    pub fn new(...) -> Self {
        let pattern = pattern.into();
        Self {
            // Pre-compile regex for performance
            _compiled_regex: Regex::new(&pattern).ok(),
            ...
        }
    }
}
```

### DashMap for Concurrent Access

`PolicyEngine` uses `DashMap` for lock-free concurrent reads:

```rust
pub struct PolicyEngine {
    policies: Arc<DashMap<String, Policy>>,
}
```

### Evaluation Short-Circuit

Disabled policies are skipped during evaluation:

```rust
pub fn evaluate_all(&self, context: &EvaluationContext) -> Result<PolicyResult> {
    for policy_ref in self.policies.iter() {
        let policy = policy_ref.value();
        if !policy.enabled {  // Skip disabled
            continue;
        }
        // ... evaluate
    }
}
```

---

## Error Handling

### Rule Validation Errors

```rust
use phenotype_policy_engine::error::PolicyEngineError;

// Invalid regex pattern
let rule = Rule::new(RuleType::Allow, "field", "[invalid");
// Returns Err(PolicyEngineError::RegexCompilationError { ... })

// Invalid rule type in TOML
// Returns Err(PolicyEngineError::RuleValidationError { ... })

// Policy not found
engine.evaluate_single("missing", &ctx)?;
// Returns Err(PolicyEngineError::PolicyNotFound { ... })
```

### Severity-Based Handling

```rust
fn handle_violations(violations: &[Violation]) -> Result<()> {
    for v in violations {
        match v.severity {
            Severity::Info => tracing::info!("{}", v.message),
            Severity::Warning => tracing::warn!("{}", v.message),
            Severity::Error => {
                tracing::error!("{}", v.message);
                return Err(anyhow!("Governance error: {}", v.message));
            }
        }
    }
    Ok(())
}
```

---

## Best Practices

### 1. Use Specific Patterns

```rust
// Good - exact match
Rule::new(RuleType::Require, "status", "^active$")

// Avoid - overly broad
Rule::new(RuleType::Require, "status", ".*")
```

### 2. Set Appropriate Priorities

```rust
// Security rules first
Rule::new(...).with_priority(1)  // Critical

// Quality rules second  
Rule::new(...).with_priority(10) // Important

// Optional rules last
Rule::new(...).with_priority(100) // Default
```

### 3. Provide Clear Descriptions

```rust
Rule::new(RuleType::Require, "coverage", "^8")
    .with_description("Test coverage must be >= 80%")
    .with_severity(Severity::Error)
```

### 4. Use Meaningful Fact Names

```rust
// Good
"security_scan_status"
"test_coverage_percent"
"cve_critical_count"

// Avoid
"status"
"coverage"
"count"
```

### 5. Group Related Rules

```rust
// Security policy
let security = Policy::new("security")
    .add_rule(Rule::new(RuleType::Require, "scan", "^passed$"))
    .add_rule(Rule::new(RuleType::Deny, "cves", "^[1-9]"));

// Quality policy
let quality = Policy::new("quality")
    .add_rule(Rule::new(RuleType::Require, "coverage", "^8"))
    .add_rule(Rule::new(RuleType::Deny, "lint_errors", "^[1-9]"));
```

---

## See Also

- [GOVERNANCE.md](./GOVERNANCE.md) — Governance framework overview
- [docs/concepts/governance.md](./concepts/governance.md) — Conceptual documentation
- [crates/phenotype-policy-engine/src/rule.rs](../crates/phenotype-policy-engine/src/rule.rs) — Rule implementation
- [crates/phenotype-policy-engine/src/policy.rs](../crates/phenotype-policy-engine/src/policy.rs) — Policy implementation
- [crates/phenotype-policy-engine/src/loader.rs](../crates/phenotype-policy-engine/src/loader.rs) — TOML loader
