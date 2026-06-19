# Tasks: eco-003 — Circular Dependency Resolution

**Status**: COMPLETED ✅

## Work Packages

| ID | Description | Status |
|----|-------------|--------|
| WP-ECO301 | Full org-wide DAG audit for circular dependencies | ✅ COMPLETE |
| WP-ECO306 | Create `docs/guides/dependency-governance.md` | ✅ COMPLETE |

## Findings

### Org-wide DAG Audit (WP-ECO301)
- Audited 43-member dependency DAG across the Phenotype org
- **Zero cycles found** — no actual circular dependencies exist
- Both spec-hypothesized cycles were phantom:
  - `api↔dashboard` cycle: does not exist — `agileplus-dashboard` NOT a dependency of `agileplus-api`
  - `agent-review/service/dispatch` crates: never committed to workspace

### Governance Document (WP-ECO306)
- Created `docs/guides/dependency-governance.md`
- Wired into AGENTS.md and CLAUDE.md
- Documents:
  - The Dependency Rule (no circular imports)
  - 3 patterns to break cycles (dependency injection, trait objects, interface segregation)
  - Anti-patterns to avoid
  - Audit commands (`cargo tree`, `machete`, depcruise)

## Verification

```bash
cd /path/to/repo
cargo machete  # detects unused/dead dependency chains
depcruise src --exclude 'node_modules|target|tests'  # generates DAG
```

## Notes

- No cycles to break — no WP-ECO302/303/304/305/307 work needed
- eco-004 cross-ref already complete
- PR #537 squash-merged to main
