# Governance Framework

**Last Updated**: 2026-04-03
**Applies to**: AgilePlus and all phenotype projects

## Overview

This document defines the governance framework for the phenotype ecosystem, including policy enforcement, evidence evaluation, and work package management.

## Policy Engine

### Architecture

The governance system is built on the `phenotype-policy-engine` crate:

```
phenotype-policy-engine/
├── src/
│   ├── rule.rs      - Rule definitions with regex patterns
│   ├── policy.rs    - Policy evaluation logic
│   ├── engine.rs    - Multi-policy engine with priority
│   ├── loader.rs    - TOML configuration loader
│   ├── context.rs   - Evaluation context
│   └── result.rs    - Violation/Severity types
```

### Rule Types

| Type | Description | Example |
|------|-------------|---------|
| `Allow` | Permits specific actions | Allow PR merge if tests pass |
| `Deny` | Blocks specific actions | Deny PR merge if security scan fails |
| `Require` | Mandates specific conditions | Require code review approval |

### Severity Levels

| Level | Action | Color |
|-------|--------|-------|
| `Info` | Log only | 🟦 Blue |
| `Warning` | Notify but allow | 🟨 Yellow |
| `Error` | Block and require resolution | 🟥 Red |

## Evidence Types

| Type | Description | Source |
|------|-------------|--------|
| `TestResult` | Unit/integration test results | CI pipeline |
| `CiOutput` | Build and deployment logs | GitHub Actions |
| `ReviewApproval` | Code review approvals | GitHub PR |
| `SecurityScan` | Snyk, Semgrep, CodeQL results | Security tab |
| `LintResult` | Clippy, ESLint, Ruff results | CI pipeline |
| `ManualAttestation` | Human verification | Manual process |

## Work Package Governance

### States

```
Backlog → Specified → Planned → In Progress → Implemented → Validated → Done
```

### Gates

Each state transition requires evidence:

| Transition | Required Evidence |
|------------|-------------------|
| Specified → Planned | Problem statement, acceptance criteria |
| Planned → In Progress | Resource allocation, task breakdown |
| In Progress → Implemented | Implementation complete, tests passing |
| Implemented → Validated | QA sign-off, benchmarks acceptable |
| Validated → Done | Deployment confirmed, monitoring active |

## CLI Commands

### Validation

```bash
# Validate a repository against governance rules
pheno validate --repos .. --check-only

# Check specific feature
agileplus validate --feature 021-polyrepo-ecosystem-stabilization
```

### Audit

```bash
# Run full governance audit
pheno audit --repos-dir .

# Check evidence coverage
agileplus evidence --check --feature <id>
```

## Policy Configuration

Policies are defined in `.agileplus/policies.toml`:

```toml
[[policy]]
name = "Security Ship Gate"
description = "Security requirements for production deployment"

[[policy.rules]]
id = "security-scan"
type = "Require"
pattern = "security-scan == 'pass'"
severity = "Error"
message = "Security scan must pass before deployment"

[[policy.rules]]
id = "no-critical-cves"
type = "Deny"
pattern = "cve.severity == 'critical'"
severity = "Error"
message = "Critical CVEs must be resolved"
```

## Evidence Linking

Evidence is linked to Work Packages via FR IDs:

```rust
// In test code
#[test]
fn test_feature_x() {
    // Traces to: FR-FEATURE-X-001
    // Evidence: test passes → linked to WP-001
}
```

## Integration

### GitHub Integration

- PR status checks enforce policy gates
- Webhook events trigger evidence evaluation
- Security tab feeds into governance dashboard

### CI Integration

```yaml
# .github/workflows/governance.yml
- name: Validate
  run: pheno validate --repos ..
- name: Check Evidence
  run: agileplus evidence --check --all
```

## Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Policy compliance | >95% | — |
| Evidence coverage | 100% | — |
| Gate transition time | <24h | — |
| Audit frequency | Weekly | — |

## References

- `docs/POLICY_RULES.md` - Policy rule reference
- `.agileplus/policies.toml` - Sample configuration
- `phenotype-policy-engine/` - Implementation
