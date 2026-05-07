# Plan: Polyrepo Ecosystem Stabilization

## Overview

This plan orchestrates the stabilization of 247 repositories across the KooshaPari GitHub organization through a 4-phase approach targeting full ecosystem governance within one quarter.

## Dependency Graph

```
Phase 1 (Days 1-7)
├── P1.1: Close PRs (infrakit) ─────────────────┐
├── P1.2: Delete test repos ────────────────────┤
├── P1.3: Clean build artifacts ────────────────┤
├── P1.4: Enforce .gitignore ──┐                │
├── P1.5: Org .github repo ────┤                │
├── P1.6: Enrich AgilePlus specs ───────────────┤
├── P1.7: Worktree discipline ──────────────────┤
├── P1.8: cargo fmt/clippy ────┘                │
├── P1.9: Commit dirty files ───────────────────┤
└── P1.10: Return to main ─────┘                │
                                                │
Phase 2 (Weeks 2-3)                             │
├── P2.1: Merge duplicates ─────────────────────┘
├── P2.2: Archive odin-* repos
├── P2.3: Move personal repos
├── P2.4: GitHub Packages ──┐
├── P2.5: PyPI publishing ──┤
├── P2.6: infrakit Phase 3 ─┘
├── P2.7: AgilePlus Phase 3
└── P2.8: Distribute templates
          │
Phase 3 (Weeks 4-6)         │
├── P3.1: SDK monorepo ─────┘
├── P3.2: Docs federation
├── P3.3: Health checks
├── P3.4: Sentry setup
├── P3.5: thegent Phase 3
├── P3.6: heliosCLI Phase 2
├── P3.7: Archive personal
└── P3.8: Split infrakit (optional)
          │
Phase 4 (Weeks 7-12)        │
├── P4.1: thegent Phase 4 ──┘
├── P4.2: infrakit Phase 4 ─┘
├── P4.3: Artifact storage
├── P4.4: Template versioning
├── P4.5: Clone remaining repos
├── P4.6: Full CI/CD coverage
├── P4.7: Governance audit
└── P4.8: Performance benchmarks
```

## Execution Strategy

### Parallel Execution Opportunities

**Phase 1** can be heavily parallelized:
- P1.1 (PR merges), P1.2 (repo deletes), P1.3 (artifact cleanup) are independent
- P1.5 (org .github) can proceed while P1.1-P1.4 execute
- P1.6 (spec enrichment) is independent of all others
- P1.9 (dirty commits) should be done first to prevent data loss

**Phase 2** has dependencies:
- P2.1 (merges) must complete before P2.4/P2.5 (publishing)
- P2.3 (personal move) is independent
- P2.6/P2.7 depend on Phase 1 completion

**Phase 3** has partial parallelism:
- P3.1 (SDK monorepo) depends on P2.4/P2.5
- P3.2 (docs federation) depends on Phase 2
- P3.3/P3.4 (health/Sentry) can run in parallel
- P3.5/P3.6 (thegent/heliosCLI phases) can run in parallel

**Phase 4** is mostly sequential:
- P4.1/P4.2 depend on Phase 3 completion
- P4.5-P4.8 depend on earlier phases

## Resource Allocation

### Agent Assignment
- **Forge**: Primary implementation across all phases
- **Muse**: Code review for all merges, governance audit (P4.7)
- **Sage**: Investigation for unknown repos, research for spec enrichment
- **Helios**: Runtime testing, CI/CD validation

### Time Estimates
- **Phase 1**: ~25 hours (3-4 days of focused work)
- **Phase 2**: ~48 hours (2 weeks part-time)
- **Phase 3**: ~52 hours (3 weeks part-time)
- **Phase 4**: ~58 hours (4 weeks part-time)
- **Total**: ~183 hours over 12 weeks

## Risk Mitigation

### Data Loss Prevention
- Full backup before any repo deletion or archival
- Commit all dirty files before any structural changes
- Document restore procedures for each cleanup action

### Breaking Change Management
- Full test suite before each merge
- Parallel run of old and new CI workflows for 1 week
- Migration guides for all consolidated repos

### Scope Control
- Strict enforcement of "stabilization only" scope
- New features deferred to future specs
- Weekly scope review against success criteria

## Success Metrics Tracking

| Metric | Baseline | P1 Target | P2 Target | P3 Target | P4 Target |
|--------|----------|-----------|-----------|-----------|-----------|
| Active repos | 247 | 247 | ~200 | ~195 | ~190 |
| Local disk | 89 GB | 60 GB | 40 GB | 25 GB | 20 GB |
| Build artifacts | 22 GB | 10 GB | 5 GB | 2 GB | 1 GB |
| Open PRs | 15+ | 0 | 0 | 0 | 0 |
| Incomplete specs | ~15 | 5 | 0 | 0 | 0 |
| CI coverage | ~30% | 50% | 80% | 95% | 100% |
| Published packages | 0 | 0 | 8 | 15 | 20+ |
| Docs federation | None | Planned | In progress | Live | Complete |
| Repos on main | 6/9 | 9/9 | 9/9 | 9/9 | 9/9 |

## Checkpoints

### Checkpoint 1: End of Phase 1 (Day 7)
- [ ] All PRs merged in phenotype-infrakit
- [ ] 8 test repos deleted
- [ ] Disk usage < 60 GB
- [ ] All dirty files committed
- [ ] All canonical repos on main
- [ ] Org .github repo created
- [ ] All 35 specs audited and enriched
- [ ] Worktree discipline documented

### Checkpoint 2: End of Phase 2 (Week 3)
- [ ] 15 duplicate repos merged
- [ ] Personal repos moved
- [ ] Package publishing operational
- [ ] ~200 repos in portfolio
- [ ] All active repos using base templates

### Checkpoint 3: End of Phase 3 (Week 6)
- [ ] SDK monorepo created
- [ ] Docs federation live
- [ ] All services reporting health
- [ ] thegent Phase 3 complete
- [ ] heliosCLI Phase 2 complete

### Checkpoint 4: End of Phase 4 (Week 12)
- [ ] All phases of all core repos complete
- [ ] 100% CI/CD coverage
- [ ] Full governance compliance
- [ ] Documented, stable ecosystem
