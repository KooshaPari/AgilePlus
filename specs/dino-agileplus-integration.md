# spec: DINOForge — AgilePlus Integration

## Goal
Fully integrate the DINOForge mod platform (KooshaPari/Dino) into the AgilePlus monorepo governance framework, enabling automated quality gates, spec-driven development, and cross-repo visibility.

## Why now
- DINOForge has 12K+ files, 52 CI workflows, and 21 releases but operates outside AgilePlus governance.
- 3 CI workflows are currently broken on main (Nightly Fuzz, Performance Benchmarks, Update Stats Dashboard). PR#391 fixes these.
- 31 Dependabot PRs are stale with no auto-merge enabled.
- AgilePlus submodules for Dino are registered but uninitialized.
- No spec, no CI linkage, no documentation references exist in AgilePlus for Dino.

## Current state
| Component | Status |
|-----------|--------|
| Git submodule | Registered (SHA: 6dcc193c) but uninitialized |
| CI linkage | None — Dino has its own 52 workflows |
| Spec file | This file (new) |
| Forge config | Missing |
| AGENTS.md reference | Missing |
| README reference | Missing |
| Worktree checkout | Missing |
| Quality gate linkage | Missing |

## Proposed changes

### Phase 1: Immediate (this sprint)
1. **Merge PR#391** — Fixes 3 failing CI workflows on main.
2. **Initialize submodules** — `git submodule update --init Dino DINOForge-UnityDoorstop` (requires long-running clone due to 12K+ files).
3. **Enable auto-merge** for Dependabot PRs — Currently blocked by repo settings.
4. **Triage 31 Dependabot PRs** — Merge safe patch bumps, flag major version bumps.

### Phase 2: Short-term (next sprint)
5. **Add Dino to AGENTS.md** — Document the Dino ecosystem (mod platform, pack system, MCP server, asset pipeline).
6. **Add Dino to README.md** — Reference Dino as a governed subproject.
7. **Create .forge/ config** — Link Dino CI to AgilePlus quality gates.
8. **Wire CI** — Add Dino-specific workflow in AgilePlus that triggers on Dino submodule pointer updates.

### Phase 3: Medium-term (this quarter)
9. **Shared quality gates** — Dino's 30-pillar scorecard (currently C/61) should feed into AgilePlus dashboards.
10. **Cross-repo proof policy** — Dino has proof_policy.py; integrate with AgilePlus proof system.
11. **Worktree management** — Automate Dino worktree creation/teardown for feature branches.
12. **Scorecard ratchet** — Current grade C (61/100). Target: B (75+) by end of quarter.

## DINOForge architecture summary
- **Primary:** .NET 11 (C#) — SDK, Bridge, Domains (Warfare/Economy/Scenario/UI), MCP Server, Unity game
- **Secondary:** Rust — Asset Pipeline (import/validate/optimize/LOD/prefab/Addressables, 38 catalog entries)
- **Tertiary:** Python — MCP Server (game automation + analysis tools)
- **Quaternary:** Go, Zig — Optimization modules
- **Content:** YAML-first pack system with typed registries (units, buildings, factions, weapons, etc.)
- **CI:** 52 GitHub Actions workflows (build, test, security, fuzzing, mutation, benchmarks, game automation)
- **Releases:** 21 semver tags, latest v1.1.0, CHANGELOG maintained

## Scorecard (30-pillar)
| Grade | Count | Key Pillars |
|-------|-------|-------------|
| A (90-100) | 3 | Frontend, Concurrency, Vendor Lock-in |
| B (80-89) | 6 | Observability, Logging, Config, Event Driven, Cost Efficiency, Performance |
| C (70-79) | 5 | Type Safety, Error Handling, Data Layer, Memory, Release |
| D (50-69) | 7 | Compliance, Complexity, API Surface, Migration, Infrastructure, Monitoring, Onboarding |
| F (0-49) | 9 | Architecture, Dev Loop, Agent Loop, Security, Extensibility, Dependencies, I18n/A11y, Testing, Fuzzing |

## Critical gaps (from audit)
1. **server.py is 2,298 lines** — needs decomposition (L1 Architecture: 40/100)
2. **Zero dependency pinning** in Python MCP server (L11 Dependencies: 40/100)
3. **Type coverage only 32%** (L10 Type Safety: 75/100 — high score but low coverage)
4. **No fuzzing** despite fuzz.yml workflow existing (L22 Fuzzing: 35/100)
5. **I18n/A11y completely absent** (L17: 45/100)

## Success criteria
- All 3 previously-failing CI workflows are green after PR#391 merge.
- AgilePlus submodules for Dino are initialized and checked out.
- Dino is referenced in AGENTS.md and README.md.
- 30-pillar scorecard grade improves from C (61) to C+ (65+) within 30 days.
- Auto-merge is enabled for Dependabot patch/minor PRs.

## Revert plan
- Revert AGENTS.md and README.md changes if Dino integration adds too much noise.
- Disable auto-merge if Dependabot PRs cause regressions.
- Remove .forge/ config if quality gates conflict with Dino's existing 52 workflows.

## Owner
- @KooshaPari (sole CODEOWNER of KooshaPari/Dino)

## References
- Dino repo: https://github.com/KooshaPari/Dino
- CI fix PR: https://github.com/KooshaPari/Dino/pull/391
- DINOForge-UnityDoorstop: https://github.com/KooshaPari/DINOForge-UnityDoorstop
- Scorecard: 30-pillar audit, grade C (61/100)
