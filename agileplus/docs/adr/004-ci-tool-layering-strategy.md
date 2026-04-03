# CI Tool Layering Strategy

## Context

Our CI pipeline uses multiple security and quality tools, some with usage limits:
- **Snyk**: Private test quota exceeded
- **CodeRabbit**: Rate limit exceeded
- **Kilo Code Review**: Long-running/failing
- **License Compliance**: Using deprecated `license_finder_action`
- **SonarCloud**: Configuration issues

## Tool Tiers

### Tier 1: Fail-Fast (Required for PR)
Must pass before code review.

| Tool | Purpose | Limit | Tier |
|------|---------|-------|------|
| `cargo clippy` | Rust lint | None | 1 |
| `cargo fmt` | Format check | None | 1 |
| `cargo test` | Unit tests | None | 1 |
| `cargo check` | Type check | None | 1 |
| GitGuardian | Secret scan | None | 1 |
| `go vet` / `gofmt` | Go lint | None | 1 |

### Tier 2: Code Quality (PR Blocking)
Run in parallel, block on failure.

| Tool | Purpose | Limit | Alternative |
|------|---------|-------|-------------|
| CodeRabbit AI | PR review | 100 PRs/month | Kilo, Gemini, Copilot |
| Kilo Code Review | AI review | Unknown | Kilo (if working) |
| Semgrep Cloud | SAST scan | Unlimited | CodeQL |

### Tier 3: Security Scanning (Daily/Weekly)
Run on schedule, not blocking PRs.

| Tool | Purpose | Limit | Alternative |
|------|---------|-------|-------------|
| Snyk | Vulnerability scan | Private tests quota | OSV, SNYK (public) |
| FOSSA | License compliance | None | REUSE.toml |
| SonarCloud | Code analysis | 1M lines free | None |

## Actions Checklist

- [ ] Replace `license_finder/license_finder_action` with `fsfe/reuse-action`
- [ ] Make Snyk checks non-blocking (run on schedule only)
- [ ] Configure Semgrep as primary SAST tool
- [ ] Use GitGuardian as primary secret scanning
- [ ] Add Kilo Code Review as CodeRabbit backup

## Workflow Updates

### Before (PR blocking)
```yaml
- Snyk Dependency Check    # BLOCKS PR - remove
- License Compliance        # BLOCKS PR - move to schedule
- CodeRabbit              # BLOCKS PR - use conditional
- Kilo Code Review        # BLOCKS PR - use conditional
```

### After (PR blocking)
```yaml
jobs:
  # Fast fail-fast checks (required)
  quality:
    runs-on: ubuntu-latest
    steps:
      - cargo clippy -- -D warnings
      - cargo fmt --check
      - cargo test

  # Fast security (required)
  security-fast:
    steps:
      - uses: github/super-linter@v5
      - uses: trufflehog/trufflehog@main

  # Slow security (non-blocking, schedule)
  security-weekly:
    if: github.event_name == 'schedule'
    steps:
      - snyk security scan
      - fossa license check
```

## Implementation Plan

1. **Immediate**: Remove Snyk/CodeRabbit from required checks
2. **This week**: Add FOSSA as license checker
3. **Next week**: Configure Kilo as CodeRabbit backup
4. **Monthly**: Review tool usage and quotas

## File Changes

- `AgilePlus/.github/workflows/license-compliance.yml` → Replace with FOSSA
- `phenotype-infrakit/.github/workflows/license-compliance.yml` → Remove (FOSSA runs)
- `AgilePlus/.github/workflows/snyk-scan.yml` → Add `if: always()` or schedule-only
