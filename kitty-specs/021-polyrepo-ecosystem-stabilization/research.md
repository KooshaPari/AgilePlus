# Research: Polyrepo Ecosystem Stabilization

## Audit Methodology

This research was conducted on 2026-04-02 using parallel worker agents to audit:
1. **GitHub Organization**: All 247 repos under KooshaPari
2. **Local Shelf State**: 9 cloned repos at /Users/kooshapari/CodeProjects/Phenotype/repos
3. **AgilePlus State**: 35 specs, worklogs, governance docs
4. **In-Progress Tasks**: All dirty files, open PRs, active branches across repos

## Key Findings

### 1. Repo Explosion Pattern

The 247 repos grew through distinct phases:
- **Pre-2024**: Personal projects, course exercises (odin-*, Frostify, etc.)
- **2024**: Early experiments (agslag-*, BytePort-*, localbase-*)
- **Jan-Mar 2025**: Academic projects (340-p2, 340P1, hoohacks, canvasApp)
- **Mar 2026**: Massive burst (~100 repos created in 1 week: Mar 20-27)
- **Apr 2026**: Active core development (45 repos updated in last 24h)

The March 2026 burst represents the "big bang" moment where the agent-driven development approach led to rapid repo creation without consolidation.

### 2. Language Distribution Analysis

| Language | Count | % | Core? | Notes |
|----------|-------|---|-------|-------|
| Rust | 65 | 28.6% | Yes | Primary infrastructure language |
| Python | 45 | 19.8% | Yes | Agent orchestration, SDK |
| TypeScript | 30 | 13.2% | Yes | Frontend, SDK |
| Go | 25 | 11.0% | Yes | Infrastructure, networking |
| JavaScript | 15 | 6.6% | Mixed | Docs, legacy |
| HTML | 12 | 5.3% | No | Learning exercises |
| CSS | 8 | 3.5% | No | Styling, learning |
| Shell | 8 | 3.5% | Mixed | Scripts, templates |
| Svelte | 5 | 2.2% | Mixed | UI components |
| C# | 3 | 1.3% | No | Legacy (Dino) |
| Swift | 2 | 0.9% | No | Personal projects |
| Other | 8 | 3.5% | No | Zig, Raku, C++ |

**Insight**: The core 4 languages (Rust, Python, TypeScript, Go) represent 72.6% of repos and 100% of active development.

### 3. Local State Crisis

Only 9 of 247 repos are cloned locally, but they consume 89 GB:

| Repo | Size | Primary Consumer | Action |
|------|------|-----------------|--------|
| heliosCLI | 39 GB | bazel-* artifacts | Clean bazel cache |
| AgilePlus | 20 GB | target/, .venv, node_modules | Clean build artifacts |
| thegent | 8.1 GB | node_modules, .venv | Clean dependencies |
| platforms/ | 5.1 GB | Non-git directory | Evaluate for git conversion |
| cloud | 2.7 GB | Next.js build | Clean .next/ |
| target/ | 2.0 GB | Root-level Rust | Move into workspace |
| phenotype-infrakit | 1.8 GB | Source + target | Clean target/ |
| phenotype-hub | 1.2 GB | Non-git directory | Evaluate for git conversion |

**Insight**: 77% of disk usage is build artifacts, not source code. Cleanup will reduce from 89 GB to ~20 GB.

### 4. AgilePlus Spec Completeness

Of 35 specs in kitty-specs/:
- **Complete** (spec + plan + tasks + research): 3 specs (001, 002, 003)
- **Partial** (spec + some artifacts): 8 specs (004, 008, eco-005, eco-006, eco-012, phenosdk-*)
- **Spec only**: 15 specs (005, 006, 007, 012, 013, 014-020, kooshapari-stale-repo-triage, etc.)
- **Ecosystem cleanup**: 6 specs (eco-001 through eco-004 complete, eco-005/006 partial)

**Insight**: 43% of specs have only spec.md files — they need plans, tasks, and research to be actionable.

### 5. In-Progress Work Inventory

Across 9 cloned repos:
- **Dirty files**: 7 of 9 repos have uncommitted changes
- **Open PRs**: 15+ open PRs (10 in infrakit, 5 in thegent)
- **Active worktrees**: 8 worktrees (some empty, some detached HEAD)
- **Stale branches**: 50+ branches without PRs

**Quick wins** (< 1 hour each):
- Commit dirty files across all repos
- Merge ready PRs (all 90-95% complete)
- Delete empty worktree directories
- Clean stale local branches

**Major efforts** (> 1 day each):
- cloud: Gastown refactor (822-line plan, 20% done)
- agentapi-plusplus: 20 upstream PRs pending
- heliosCLI: 4 active worktrees need finish/close decisions
- thegent: BytePort feature implementation (40% done)

### 6. Merge Opportunities

15 repos can be merged into 8 targets:
- **Duplicate naming**: phenotype-contract/contracts, hexagon-rs/rust
- **Unnecessary splits**: error-core/errors/error-macros, FixitGo/FixitRs
- **Subsumed functionality**: thegent-plugin-host → thegent, agileplus-agents/mcp → AgilePlus
- **Doc sidecars**: router-docs → phenotype-hub/docs/

**Net reduction**: 15 → 8 = 7 fewer repos to manage

### 7. Archive Candidates

28 repos identified for archival:
- **8 immediate deletes**: Test artifacts, typos, placeholders
- **4 course exercises**: odin-* repos
- **11 low-signal personal**: Frostify, AppGen, TripleM, etc.
- **5 language variants**: FixitGo/FixitRs, agentapi/agentapi-plusplus

### 8. Missing Infrastructure

| Infrastructure | Current State | Needed |
|---------------|---------------|--------|
| CI/CD | 32 workflows at shelf root, not distributed | Org-level .github with reusable workflows |
| Package registry | None published | npm, PyPI, crates.io, Go modules |
| Docs federation | None | VitePress hub at docs.phenotype.dev |
| Health monitoring | Per-service | Unified /health endpoint pattern |
| Error tracking | Sentry (partial) | All production services |
| Artifact storage | Local only | GitHub Actions cache, Releases, GHCR |
| Template distribution | Scattered | Template registry + scaffolding CLI |

## Recommendations

### Immediate (This Week)
1. **Commit all dirty files** — prevents data loss
2. **Merge all ready PRs** — clears backlog
3. **Clean build artifacts** — 77% disk reduction
4. **Set up org .github** — enables distributed CI/CD
5. **Enrich incomplete specs** — makes AgilePlus actionable

### Structural (This Month)
1. **Merge duplicate repos** — reduces management overhead
2. **Archive stale repos** — improves signal-to-noise
3. **Set up package publishing** — enables consumption
4. **Create SDK monorepo** — consolidates SDK development
5. **Set up docs federation** — unified documentation

### Long-term (This Quarter)
1. **Full CI/CD coverage** — all active repos
2. **Governance compliance** — all repos meet baseline
3. **Performance optimization** — benchmarks and tuning
4. **Template ecosystem** — versioned, tested, distributed

## Conclusions

The 247-repo explosion is a symptom of rapid agent-driven development without consolidation discipline. The stabilization plan addresses this through:

1. **Grouping**: 6 logical clusters for manageable oversight
2. **Reduction**: 247 → ~190 repos through archival and merging
3. **Infrastructure**: Shared CI/CD, publishing, docs, monitoring
4. **Governance**: AgilePlus spec completion, worktree discipline, branch hygiene
5. **Disk optimization**: 89 GB → 20 GB through artifact cleanup

The key insight is that **agents struggle with 247 repos but can effectively manage 6 clusters of ~30 repos each**. This grouping strategy is essential for sustainable agent-assisted development.
