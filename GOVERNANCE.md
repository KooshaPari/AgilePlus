# GOVERNANCE.md — repos shelf governance

## Shelf Governance

This shelf is a **polyrepo of independent projects** owned by a single
individual (koosha-pari). Governance is therefore lightweight — the owner
makes all decisions, with agents acting as trusted helpers.

## Decision Making

| Decision type | Process |
|---------------|---------|
| New project | Owner creates + names, agent documents |
| Architecture (cross-project) | Owner decides, agent researches |
| Architecture (per-project) | Project owner decides |
| Dependency conflicts | Agent proposes options, owner chooses |
| PR merge | Owner reviews + merges. See [PR Requirements Policy](./GOVERNANCE_PR_REQUIREMENTS.md) |
| Deleting/archiving project | Owner initiates, agent executes |

## Code Review Standards

### Per-Project Rules
Each project may define its own review standards in `CONTRIBUTING.md`.

### Shelf-Level Standards
- No shelf-level changes without understanding downstream impact
- Breaking changes across projects must be coordinated
- Test coverage must not regress

### Agent Recommendations
Agents may flag concerns:
- Security implications
- Dependency bloat
- Cross-project duplication
- Performance regressions

Agents flag; owner decides.

## Release Management

Projects manage their own release cycles. Shelf-level coordination only
when:
- A project is archived or deleted
- Cross-project dependency changes
- Major shelf reorganization

## Standards & Conventions

### Naming
- Projects: kebab-case (`heliosCLI`, `thegent`, `phenotype-config`)
- Branches: `<project>/<type>/<description>`
- Sessions: `<project>:<brief-task>`
- Plans: `<project>-<YYYYMMDD>-<task>.md`

### Quality Gates
- **Rust projects**: `cargo clippy -- -D warnings` + tests
- **JS/TS projects**: lint + typecheck + tests
- **Python projects**: ruff check + pyright + tests
- **Cross-project**: duplication audit before major refactors

## Project Lifecycle

### Active Projects
Listed in `projects/INDEX.md` with status `active`.
Regular development, owned by the shelf owner.

### Maintenance Projects
Listed with status `maintenance`.
Minimal changes, bug fixes only, no new features.

### Archived Projects
Listed with status `archived` in `projects/INDEX.md`.
Actual code lives in `.archive/`.
Can be restored to active if needed.

### Deletion
Rare. Only after confirmed backup + no downstream dependencies.

## Tooling Governance

| Tool | Purpose | Governance |
|------|---------|------------|
| `agileplus` | Project management | Per-project spec system |
| `agileplus` CLI | Work tracking | AgilePlus project only |
| `cargo` | Rust build | Project-level |
| `bun` | JS/TS package management | Project-level |
| `task` | Task runner | Project-level |
| `buf` | Proto management | Project-level |
| `mise` | Runtime version management | Shelf-level dotfile |

## Agent Authority Levels

| Agent | Can edit | Can commit | Can push | Can merge |
|-------|----------|------------|----------|-----------|
| Forge | Any file | Any branch | Own worktrees | No |
| Muse | Comments only | No | No | No |
| Sage | Any file | Any branch | Own worktrees | No |
| Helios | Test/config files | Test branches | No | No |

**All agents ask before acting outside their authority.**

## Agent Artifact Population Requirements

### Default Agent Behavior

All agents MUST populate artifacts by default when creating or modifying code:

| When Agent... | Must Populate... |
|--------------|------------------|
| Creates new repo | All 10 required artifacts |
| Adds FR reference | Update `specs/` and traceability |
| Modifies core logic | Update `ADR.md` if architectural |
| Changes API | Update `ARCHITECTURE.md` interfaces |
| Implements feature | Add User Stories to FR specs |
| Creates tests | Add FR annotations to test files |
| Fixes bug | Update `plan.md` completion status |

### Artifact Completeness Check

Before ANY commit, agents must verify:
- [ ] `CLAUDE.md` exists and reflects current state
- [ ] `AGENTS.md` rules up to date
- [ ] `README.md` accurate for users
- [ ] `plan.md` shows current work items
- [ ] All tests have FR traceability annotations

## Documentation & GitHub Pages Governance

### VitePress Documentation Sites

All repositories MUST derive documentation from:
- **Primary Source**: `AgilePlus/docs/` (shared patterns)
- **Repository**: `docs/` folder within each repo
- **Backend**: PhenoDocs VitePress instance
- **Hosting**: GitHub Pages (auto-deploy via CI/CD)

| Repository | Docs Source | GitHub Pages URL |
|------------|-------------|------------------|
| `AgilePlus` | `/docs` + `/specs` | `phenotype.io/agileplus` |
| `phenoSDK` | `/docs` derived from AgilePlus | `phenotype.io/phenosdk` |
| `thegent` | `/docs` + generated API docs | `phenotype.io/thegent` |
| All others | `/docs` folder (minimal) | `phenotype.io/<repo-name>` |

### GitHub Repository Metadata Standards

All repositories MUST maintain complete GitHub metadata:

**Description** (Required)
- Pattern: "[Component Type]: [Purpose] | [Status] | [Owner]"
- Example: "SDK: Python infrastructure toolkit | Active | Phenotype"

**Website** (Required)
- Primary: `https://phenotype.io/[repo-name]`
- Fallback: GitHub Pages URL if custom domain unavailable

**Topics** (Required, 3-5 tags)
```
# Language/Runtime
rust python typescript go elixir

# Component Type  
sdk cli library framework tool

# Domain
observability security governance tracing
mcp testing infrastructure

# Status
active maintenance archived experimental
```

**Repository Features** (Enable All)
- ✅ Releases (with automated changelog)
- ✅ Deployments (GitHub Actions)
- ✅ Packages (container/npm/crates)
- ✅ Issues (bug reports, feature requests)
- ✅ Discussions (community Q&A)
- ✅ Wiki (deprecated - use docs/ instead)
- ✅ Projects (AgilePlus integration)
- ✅ Security (Snyk scanning)

### Documentation Deployment Flow

```
1. Agent updates code
   ↓
2. Agent updates docs/ folder artifacts
   ↓
3. CI/CD triggers on push to main
   ↓
4. VitePress builds documentation
   ↓
5. GitHub Pages deploys to phenotype.io/[repo]
   ↓
6. Validation script verifies docs are current
```

### PhenoDocs Integration

- **Backend**: VitePress with custom theme
- **Search**: Algolia DocSearch integration
- **Versioning**: Semantic versioning per repo
- **Cross-linking**: All docs link to related repos
- **Traceability**: All docs include FR references

## Change Log

This file tracks governance changes to the shelf itself.

| Date | Change |
|------|--------|
| 2026-04-04 | Added PR Requirements Policy - visual evidence, specs, docs required |
| 2026-03-29 | Initial shelf governance written (previously AgilePlus-specific) |
