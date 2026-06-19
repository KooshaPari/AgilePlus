# Tasks: 002 — Org-Wide Release Governance & DX Automation

**Status**: OPERATIONAL (ongoing)

## Work Packages

| ID | Description | Status |
|----|-------------|--------|
| WP-002-001 | Standardize release process across repos | 🔄 IN PROGRESS |
| WP-002-002 | Automate changelog generation (clift.toml) | 🔄 IN PROGRESS |
| WP-002-003 | DX tooling: pre-commit hooks, quality gates | 🔄 IN PROGRESS |
| WP-002-004 | Release cadence policy | 🔄 IN PROGRESS |

## Evidence

### Release Standardization (WP-002-001)
- Phenotype org uses CalVer for releases (e.g., `thegent v0.x.x`, `AgilePlus v0.x.x`)
- Semantic versioning for libraries (crates.io packages)
- Release process documented in `CONTRIBUTING.md`
- Release automation via `release-cut` tooling

### Changelog Generation (WP-002-002)
- `clift.toml` configured in key repos
- Conventional commits enforced (`.commitlintrc.yml`)
- Semantic commit types: feat, fix, chore, docs, refactor, test, etc.

### DX Tooling (WP-002-003)
- Pre-commit hooks configured in key repos
- `quality-gate.sh`: clippy + fmt + cargo test
- `trufflehog.yml`: secrets scanning
- `deny.toml`: Rust dependency auditing
- `commit-msg-check`: DCO enforcement

### Release Cadence (WP-002-004)
- Aggressive release cadence: multiple releases per day across org
- Agent-driven releases via CI
- No formal release freeze policy

## Governance Files Coverage

| File | Coverage | Status |
|------|----------|--------|
| CLAUDE.md | All active repos | ✅ 100% |
| AGENTS.md | All agent-facing repos | ✅ 100% |
| FUNDING.yml | All active repos | ✅ 100% |
| trufflehog.yml | All active repos | ✅ 100% |
| SECURITY.md | All active repos | ✅ 100% |
| deny.toml | All Rust repos | ✅ 100% |
