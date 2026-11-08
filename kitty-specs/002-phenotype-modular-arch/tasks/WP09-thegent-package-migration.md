---
work_package_id: "WP09"
title: "thegent: Package Migration Completion"
lane: "planned"
dependencies: []
subtasks: ["T024", "T025", "T026", "T027"]
history:
  - date: "2026-03-03"
    event: "created"
    by: "spec-kitty.tasks"
  - date: "2026-03-03"
    event: "revised"
    by: "external-library-leverage research integration"
---

# WP09: thegent — Package Migration Completion

**Implementation command**: `spec-kitty implement WP09`

## Objective

Complete thegent's partial migration from monolithic `src/thegent/` to `packages/thegent-*`. Extract config, infra, governance packages. Merge protocols+mcp.

## Context

- thegent is ~60% migrated; `packages/` already contains thegent-core, thegent-cli, thegent-agents, thegent-execution, thegent-sync, thegent-audit, thegent-planning, thegent-observability, thegent-skills
- Remaining in `src/thegent/`: config*.py (~500 LOC), infra/ (~11K LOC), governance/ (~13K LOC)
- thegent-protocols and thegent-mcp exist separately but overlap — merge into single thegent-mcp
- See `kitty-specs/002-phenotype-modular-arch/research.md` Decision 4

## Subtasks

### T024: Extract thegent-config (pydantic-settings adoption)

**Steps**:
1. Identify config files: `src/thegent/config.py`, `src/thegent/config_schema.py`, related
2. Create `packages/thegent-config/`:
   - `pyproject.toml` with package metadata; add `pydantic-settings` as dependency
   - `src/thegent_config/__init__.py` — re-exports
   - Move config schemas, validation, loading logic
3. **Replace custom config loaders with `pydantic-settings`**: Convert config classes to `BaseSettings` subclasses with env/file/secrets sources. This eliminates custom YAML/env loading logic (~2-4K LOC saved).
4. Update `src/thegent/` imports to use `thegent-config`
5. Add to workspace `pyproject.toml`

**Implementation guidance (library research)**:
- `pydantic/pydantic-settings` is a Phase A drop-in (very low complexity, no risk).
- Use `SettingsConfigDict` for env prefix, nested model support, and `.env` file loading.
- Preserve existing config key names via `Field(alias=...)` for backward compatibility.

**Validation**: `uv sync && pytest packages/thegent-config/`

### T025: Extract thegent-infra

**Steps**:
1. Identify infra code: `src/thegent/infra/` (~11K LOC)
   - Fast YAML, subprocess runner, file ops, IPC bridge, cache
2. Create `packages/thegent-infra/`:
   - Preserve module structure from `infra/`
   - Ensure zero thegent-specific imports (this is the reusable layer)
3. Update all `src/thegent/` and `packages/*/` imports
4. This package will later be published as `phenotype-py-infra` (WP10)

**Validation**: `pytest packages/thegent-infra/` passes; no thegent-specific deps

### T026: Extract thegent-governance (OPA + Guardrails AI adoption)

**Steps**:
1. Identify: `src/thegent/governance/` (~13K LOC) — policy engine, HITL, escalation
2. Create `packages/thegent-governance/`
3. Move governance code; update imports
4. May depend on thegent-core for domain types
5. **OPA adoption**: Replace the custom policy engine with `open-policy-agent/opa` (sidecar or embedded). Define policies in Rego. This replaces custom RBAC, compliance, and policy evaluation logic (~4-7K LOC saved).
6. **Guardrails AI adoption**: Integrate `guardrails-ai/guardrails` for semantic validation (output validation, firewall). This replaces custom semantic firewall and output validation code (~4-6K LOC saved).

**Implementation guidance (library research)**:
- OPA is Phase C (medium complexity): requires Rego DSL for policy definitions and a sidecar process or embedded Go library. Map existing policy rules to Rego before migration.
- Guardrails AI is Phase C (medium complexity): Flask-based daemon for validation. Evaluate whether to run as sidecar or in-process.
- Combined, these two libraries save ~8-13K LOC out of the 13K governance module — the extraction and library adoption should be coordinated.
- Retain HITL and escalation logic as custom code; OPA and Guardrails handle policy evaluation and semantic validation respectively.

**Validation**: `pytest packages/thegent-governance/`

### T027: Merge thegent-protocols + thegent-mcp (official MCP SDK adoption)

**Steps**:
1. Audit overlap between `packages/thegent-protocols/` and `packages/thegent-mcp/`
2. Merge into single `packages/thegent-mcp/`:
   - Combine protocol definitions with MCP implementation
   - Remove thegent-protocols package
3. **Replace custom MCP handler with `modelcontextprotocol/python-sdk`**: Swap hand-rolled MCP transport and protocol handling for the official SDK (~2-4K LOC saved). Use the SDK's transport, session, and tool-call primitives directly.
4. Update all imports from thegent-protocols → thegent-mcp
5. Update workspace pyproject.toml

**Implementation guidance (library research)**:
- `modelcontextprotocol/python-sdk` is Phase A (low-medium complexity, no major risk).
- Note: v2 API changes expected Q1 2026 — pin to a stable release and track upstream.
- The merge and SDK adoption can be done in one pass since both touch the same code.

**Validation**: `pytest packages/thegent-mcp/` passes; no references to thegent-protocols remain

## Definition of Done

- [ ] thegent-config extracted and passing tests
- [ ] thegent-infra extracted with zero thegent-specific deps
- [ ] thegent-governance extracted and passing tests
- [ ] thegent-protocols merged into thegent-mcp
- [ ] All workspace tests pass (`pytest`)
- [ ] `src/thegent/` is significantly smaller

## Risks

- **Import cycles**: Governance may depend on config which depends on core. Map dependency graph before extracting.
- **Test fixtures**: Some tests may use fixtures from `src/thegent/` that need to move with their packages.

## Reviewer Guidance

- Verify thegent-infra has truly zero thegent-specific imports (grep for `thegent`)
- Check dependency direction: infra should be at the bottom (no upward deps)
- Ensure merged thegent-mcp covers all protocol+MCP functionality
