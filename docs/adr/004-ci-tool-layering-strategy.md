# CI Tool Layering Strategy

## Problem Statement

Current CI has multiple tools competing for quotas:
- **Snyk**: Private test quota exhausted
- **CodeRabbit**: Rate limit exceeded
- **Kilo Code Review**: Timeouts/failures
- **FOSSA**: Analysis timeouts
- **SonarCloud**: Configuration issues

## Layering Strategy

### Tier 1: Always Run (Blocking)

| Tool | Purpose | Quota | Action |
|------|---------|-------|--------|
| cargo fmt | Format check | None | ✅ Run always |
| cargo clippy | Lint check | None | ✅ Run always |
| go vet / gofmt | Go format/lint | None | ✅ Run always |
| ruff / black | Python format/lint | None | ✅ Run always |
| GitGuardian | Secret scanning | Generous | ✅ Run always |
| Gitleaks | Secret scanning | None | ✅ Run always |
| Socket Security | Dependency alerts | Generous | ✅ Run always |
| Semgrep OSS | SAST | None | ✅ Run always |
| License Compliance | License check | None | ✅ Run always (use REUSE.toml not license_finder) |

### Tier 2: Run on PR (Non-blocking)

| Tool | Purpose | Quota | Action |
|------|---------|-------|--------|
| cargo test | Unit tests | None | Run always |
| cargo bench | Benchmarks | None | Run always |
| cargo audit | Vulnerability audit | None | Run always |
| CodeQL | Deep SAST | 500min/month | Only on main/important branches |
| Semgrep Cloud | Full ruleset | Project quota | Weekly only |

### Tier 3: Run Daily/Weekly (Non-blocking)

| Tool | Purpose | Quota | Action |
|------|---------|-------|--------|
| Snyk | Vulnerability scanning | Private tests exhausted | Daily at 2AM UTC only |
| FOSSA | License compliance | Limited | Weekly on main |
| SonarCloud | Code analysis | Limited | Weekly on main |
| Snyk Monitor | Continuous | Unlimited | Daily |
| Trivy | Container scanning | None | Weekly |

### Tier 4: AI Review (Tiered)

| Tool | Purpose | Quota | Action |
|------|---------|-------|--------|
| GitHub Copilot | Inline suggestions | Team quota | Always (PR) |
| Kilo Code Review | AI review | Rate limited | 1 PR at a time |
| Claude (external) | AI review | External | Only on important PRs |

## Implementation

### PR Workflow Order

```yaml
# 1. Fast checks (seconds)
- name: Format checks
- name: Secret scanning
- name: Socket Security

# 2. Build & test (minutes)
- name: Build
- name: Unit tests
- name: Integration tests

# 3. Quality gates (minutes)
- name: Clippy/Ruff checks
- name: Semgrep quick scan
- name: License check

# 4. Deep scans (optional, non-blocking)
- name: CodeQL (on main only)
- name: Semgrep full (on main only)
```

### Configuration Changes

1. **Snyk**: Add `schedule: cron: '0 2 * * *'` (2AM UTC daily)
2. **FOSSA**: Remove from PR, run weekly
3. **SonarCloud**: Remove from PR, run weekly
4. **CodeRabbit**: Remove, use Copilot only
5. **Kilo**: Keep but add `continue-on-error: true`
6. **License Finder**: Replace with fsfe/reuse-action

## Repository-Specific Rules

### phenotype-infrakit (Rust)
```yaml
# Quick checks: < 2 min
- cargo fmt --check
- cargo clippy --all-targets -- -D warnings
- cargo test --all
- cargo bench

# Deep checks: Only on main (weekly)
- cargo audit
- CodeQL
- SonarCloud
```

### cliproxyapi-plusplus (Go)
```yaml
# Quick checks: < 2 min
- go fmt
- go vet
- go build
- go test ./...

# Deep checks: Only on main (weekly)
- staticcheck
- CodeQL
- Snyk (schedule)
```

### thegent (Multi-language)
```yaml
# Quick checks: < 5 min
- Semgrep quick
- Ruff/Python lint
- go vet
- cargo clippy
- Secret scanning
- License check

# Deep checks: Only on main (weekly)
- Snyk (schedule: weekly)
- FOSSA
- CodeQL
- SonarCloud
```

## Migration Commands

### Replace License Finder
```bash
# Old (deprecated)
- uses: licensefinder/license_finder_action@v7

# New
- uses: fsfe/reuse-action@v2
```

### Replace Snyk on PR
```bash
# Old (on PR)
- uses: snyk/actions@v1

# New (daily schedule only)
on:
  schedule:
    - cron: '0 2 * * *'
  push:
    branches: [main]
```

### Replace CodeRabbit
```bash
# Remove from PR
# Use GitHub Copilot instead (no quota for team accounts)
```

## Implementation Status

### ✅ Completed

| File | Change | Status |
|------|---------|--------|
| `.github/workflows/sast-quick.yml` | CodeQL → Semgrep-only, removed v3 | ✅ |
| `phenotype-infrakit/.github/workflows/sast-full.yml` | Added Tier 3 comment, continue-on-error | ✅ |
| `phenotype-infrakit/.github/workflows/snyk-scan.yml` | Schedule only, continue-on-error | ✅ |
| `AgilePlus/.github/workflows/snyk-scan.yml` | Added Tier 3 comment, continue-on-error | ✅ |
| `thegent/.github/workflows/security.yml` | Semgrep with continue-on-error | ✅ |

### 📋 Remaining Changes Needed

| File | Change | Priority |
|------|--------|----------|
| `AgilePlus/.github/workflows/sast-full.yml` | CodeQL upgrade to v4 | P1 |
| `thegent/.github/workflows/sast.yml` | CodeQL upgrade to v4 | P1 |
| `AgilePlus/.github/workflows/license-compliance.yml` | Replace license_finder with reuse | P2 |
| `thegent/.github/workflows/license-compliance.yml` | Replace license_finder with reuse | P2 |

## Verification

Run before merging:
```bash
# Verify no quota-intensive tools on PR
grep -r "snyk/actions\|fossa\|license_finder\|coderabbit" .github/workflows/*.yml
# Should return nothing for PR workflows
```

## Rollback Plan

If issues arise:
1. Re-enable tools one at a time
2. Monitor quota usage
3. Adjust schedule frequency

## Monitoring

Track quota usage weekly:
```bash
gh api /orgs/phenotype/actions/runners
gh api /repos/KooshaPari/thegent/actions/billing
```
