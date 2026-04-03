# pheno-cli Implementation Plan

## Overview

Implementation roadmap for `pheno` CLI - org-wide release governance and developer experience automation.

## Phases

### Phase 1: Core Infrastructure

| Task | Status | Milestone | Dependencies |
|------|--------|-----------|--------------|
| Project structure setup | ✅ | M1 | None |
| CLI framework (Cobra) | ✅ | M1 | None |
| Configuration management | ✅ | M1 | None |
| GitHub API client | ✅ | M1 | Token auth |
| HTTP client setup | ✅ | M1 | None |

**Milestone M1: Core Framework**
- Basic CLI with `pheno --version` working
- Configuration file loading
- GitHub API authentication

### Phase 2: Release Management

| Task | Status | Milestone | Dependencies |
|------|--------|-----------|--------------|
| Version parsing (semver) | ✅ | M2 | M1 |
| Changelog generation | ✅ | M2 | M1 |
| Release creation | ✅ | M2 | GitHub API |
| Tag management | ✅ | M2 | GitHub API |

**Milestone M2: Release Management**
- `pheno release new` creates GitHub release
- Changelog auto-generated from conventional commits
- Version bump with changelog update

### Phase 3: Publishing

| Task | Status | Milestone | Dependencies |
|------|--------|-----------|--------------|
| Registry abstraction | ✅ | M3 | M2 |
| Cargo publish | ✅ | M3 | Registry |
| Npm publish | 🔄 | M3 | Registry |
| PyPI publish | 🔄 | M3 | Registry |

**Milestone M3: Publishing**
- `pheno publish --dry-run` validates release
- Publish to crates.io, npm, PyPI
- Multi-registry support

### Phase 4: Governance

| Task | Status | Milestone | Dependencies |
|------|--------|-----------|--------------|
| Spec validation | 🔄 | M4 | AgilePlus |
| Test coverage check | 🔄 | M4 | M3 |
| License audit | 🔄 | M4 | M3 |
| Policy gate | 🔄 | M4 | M4 |

**Milestone M4: Governance**
- All governance rules enforced
- Prerequisite checks before publish
- Audit reports generated

### Phase 5: Promotion

| Task | Status | Milestone | Dependencies |
|------|--------|-----------|--------------|
| Environment model | 🔄 | M5 | M4 |
| Promotion workflow | 🔄 | M5 | M4 |
| Approval workflow | 🔄 | M5 | GitHub API |
| Notification system | 🔄 | M5 | M5 |

**Milestone M5: Promotion**
- `pheno promote dev→staging`
- Approval gates enforced
- Slack/email notifications

### Phase 6: Scaffolding

| Task | Status | Milestone | Dependencies |
|------|--------|-----------|--------------|
| Template system | 🔄 | M6 | M1 |
| Cargo templates | 🔄 | M6 | Templates |
| Go templates | 🔄 | M6 | Templates |
| Pre-commit hooks | 🔄 | M6 | M6 |

**Milestone M6: Scaffolding**
- `pheno scaffold library` creates Rust crate
- Pre-commit hooks installed
- CI/CD pipeline configured

## Current Progress

```
Phase 1: ████████████ 100%
Phase 2: ████████████ 100%
Phase 3: ████████░░░░  60%
Phase 4: ███░░░░░░░░░  20%
Phase 5: ░░░░░░░░░░░░   0%
Phase 6: ████░░░░░░░░  20%
```

**Overall: 45% complete**

## Upcoming Tasks

### This Sprint (Week 1-2)

1. Complete npm publish support
2. Add PyPI publish support  
3. Implement spec validation against AgilePlus
4. Add test coverage check

### Next Sprint (Week 3-4)

1. Environment promotion workflow
2. Approval system integration
3. Notification system
4. Documentation complete

### Future Enhancements

- Plugin system for custom commands
- Telemetry and analytics
- Dashboard integration
- Multi-org support

## Dependencies

```
┌─────────────────┐
│   User/Agent    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   pheno CLI    │
│  (Cobra/Viper) │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌───────┐ ┌────────┐
│ GitHub │ │AgileP │
│  API   │ │lus API │
└───┬───┘ └────┬──┘
    │          │
    └────┬─────┘
         │
         ▼
┌─────────────────┐
│   Registries    │
│ (crates,npm,PyPI)│
└─────────────────┘
```

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| API rate limits | Medium | Caching, exponential backoff |
| Template drift | Low | Versioned templates |
| Registry down | High | Retry with fallback |
| Breaking changes | Medium | Version bumps, changelog |

## Notes

- All commands should have `--dry-run` for safety
- Config should support environment variables
- Secrets should use external secret manager
- All operations should be logged
