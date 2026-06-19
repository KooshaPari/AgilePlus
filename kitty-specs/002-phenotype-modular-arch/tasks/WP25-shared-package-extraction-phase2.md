---
work_package_id: WP25
title: 'Shared Package Extraction Phase 2 (Polyrepo)'
lane: planned
dependencies: [WP02, WP09, WP11]
subtasks: [T075, T076, T077]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP25: Shared Package Extraction Phase 2 (Polyrepo)

**Implementation command**: `spec-kitty implement WP25 --base WP02,WP09,WP11`

## Objective

Extract 15 additional shared package candidates identified in prior audit but not covered by WP02-WP14. These are reusable modules currently embedded in individual repos that have cross-repo consumers or standalone utility value.

## Context

- WP02 scaffolded initial shared repos; WP07/WP09/WP11 extracted first-wave packages
- This WP covers second-wave extractions identified during the cross-repo quality audit
- Candidates span Go, Python, and TypeScript across 7 repos
- Each candidate has either multiple consumers or standalone utility justifying extraction

## Subtasks

### T075: Go shared package extractions

**Steps**:
1. `agentapi++/lib/screentracker/` (523L `pty_conversation.go`) -> publish as `phenotype/screentracker-go`
2. `agentapi++/lib/msgfmt/` (322L) + `lib/termexec/` -> extract as standalone Go libs `phenotype/msgfmt-go` and `phenotype/termexec-go`
3. `cliproxy/sdk/auth/` (2,800L) -> extract as `phenotype/llm-auth-go` (multi-provider OAuth conductor)
4. `trace/codeindex/` -> publish as `phenotype/codeindex-go` (standalone code indexing library)
5. For each: create repo with go.mod, CI, README, and update source repo to consume as dependency

**Validation**:
- Each extracted package builds independently (`go build ./...`)
- Each extracted package passes its own test suite
- Source repos updated to import from shared packages
- Source repo tests still pass after migration
- No copied code remains in source repos (clean extraction)

### T076: Python shared package extractions

**Steps**:
1. `thegent/protocols/jsonrpc_agent_server.py` (1,378L) -> publish as `phenotype/jsonrpc-agent-server` (pyproject.toml, pip-publishable)
2. `thegent/orchestration/` (13,068L) -> extract as `phenotype/agent-orchestration` (Redis locks, resource-aware scheduling — no thegent-specific logic)
3. `thegent/audit/` + `audit_v2/` -> shared `phenotype/audit-trail` utility
4. `portage/terminus_2/` (~2,500L) -> extract as `phenotype/terminus2-harness` (reusable tmux agent executor)
5. `portage/viewer/server.py` (1,237L) -> extract as `phenotype/eval-viewer` (standalone FastAPI results viewer)
6. `portage/registry/` -> extract as `phenotype/harbor-registry-client`
7. For each: create repo with pyproject.toml, CI, tests, and update source repo to consume

**Validation**:
- Each extracted package installs and passes tests independently
- Source repos updated to import from shared packages
- No duplicated code remains in source repos
- `pytest` passes in all affected repos

### T077: TypeScript shared package extractions

**Steps**:
1. `heliosCLI/shell-tool-mcp/` (~300 LOC TS) -> publish as `@phenotype/shell-tool-mcp` npm package
2. heliosApp `ProviderRegistry` -> extract as `@phenotype/provider-registry`
3. heliosApp `mcp-bridge.ts` + `acp-client.ts` + `a2a-router.ts` -> extract as `@phenotype/agent-protocol-bridge`
4. `trace/embeddings/` -> shared service package (portage, thegent, trace consumers)
5. `trace/graph/` -> shared Neo4j service package (portage+thegent potential consumers)
6. For each: create package with package.json, tsconfig, CI, tests, and update source repos

**Validation**:
- Each extracted package builds and passes tests independently
- Source repos updated to import from shared packages
- `pnpm build` and `pnpm test` pass in all affected repos
- npm packages have proper exports and type definitions

## Definition of Done

- [ ] 15 shared packages extracted and published (5 Go, 6 Python, 4 TS)
- [ ] All source repos migrated to consume shared packages
- [ ] No duplicated code remains in source repos for extracted modules
- [ ] Each shared package has CI, tests, and README
- [ ] All repo test suites pass after migration

## Risks

- **Hidden dependencies on repo internals**: Extracted modules may import private symbols from their source repo. Audit imports before extraction and make necessary symbols public or refactor.
- **Version coordination across consumers**: Multiple repos consuming the same shared package need coordinated version bumps. Use semantic versioning from the start.
- **thegent orchestration extraction is large (13K LOC)**: May require iterative extraction — start with the core scheduling logic and expand.

## Reviewer Guidance

- Verify each extraction is clean — no remaining copies in source repos
- Check that shared packages have no imports from their source repo (true independence)
- Confirm CI is set up for each new shared package repo
- Verify consumers pin specific versions, not floating refs
