# AgilePlus MCP integration (sd-agileplus side-DAG)

Phenodag pool tasks `sd-agileplus-01` … `sd-agileplus-05` for MCP fleet wiring.

| Task | Deliverable | Status |
|------|-------------|--------|
| sd-agileplus-01 | [eco-NNN map](#eco-nnn-map) | done |
| sd-agileplus-02 | [kitty-spec link](#kitty-spec-link) | done |
| sd-agileplus-03 | [agileplus-mcp audit](#agileplus-mcp-audit) | done |
| sd-agileplus-04 | [traceability](#traceability) | done |
| sd-agileplus-05 | [spec status](#spec-status) | done |

**Repos:** [KooshaPari/AgilePlus](https://github.com/KooshaPari/AgilePlus) (local: `C:\Users\koosh\dev\AgilePlus`)  
**Catalog:** [KooshaPari/PhenoMCPServers](https://github.com/KooshaPari/PhenoMCPServers) — entries `agileplus-mcp-intent`, `agileplus-mcp`

## eco-NNN map

Canonical eco specs live under `docs/specs/eco/`. MCP-relevant rows:

| eco | title (slug) | state | MCP relevance |
|-----|--------------|-------|---------------|
| eco-014 | ux-dx-ax-richness | PENDING | FR: programmatic MCP/CLI surfaces |
| eco-024 | traceability | PENDING | trace graph for MCP tools → specs → tests |
| eco-027 | cargo-workspace-cleanup | PENDING | unblocks MCPForge / workspace consumers |
| eco-033 | ecosystem-compatibility | PENDING | toolchain pins incl. `agileplus-mcp` Python 3.12 |
| eco-034 | functional-requirements-canonical | PENDING | FR index for MCP-exposed workflows |

Retired (no forward MCP work): eco-001 … eco-006. Active governance baseline: eco-010 … eco-032.

Full index: 34 specs under `docs/specs/eco/` (eco-001 … eco-034, plus `align-version-drift-2026-06-08`, `021-polyrepo-ecosystem-stabilization`).

## kitty-spec link

Legacy Spec-Kitty tree `kitty-specs/` is a **redirect stub**. Canonical specs:

| kitty-specs (archived) | canonical path |
|------------------------|----------------|
| `kitty-specs/eco-003-circular-dep-resolution/` | `docs/specs/eco/eco-003-circular-dep-resolution/` |
| `kitty-specs/eco-004-hexagonal-migration/` | `docs/specs/eco/eco-004-hexagonal-migration/` |
| `kitty-specs/eco-005-xdd-quality/` | `docs/specs/eco/eco-005-xdd-quality/` |
| `kitty-specs/eco-006-governance-sync/` | `docs/specs/eco/eco-006-governance-sync/` |

Archived `meta.json` → `docs/_archive/meta-json/`. New eco specs MUST land in `docs/specs/eco/<slug>/` only.

## agileplus-mcp audit

| Surface | Path | Audit |
|---------|------|-------|
| **agileplus-mcp-intent** (Rust) | `crates/agileplus-mcp-intent/` | **Active** — MCP stdio + HTTP; tool `convert_prompt_to_intent_graph`; workspace member |
| **agileplus-mcp** (Python) | `agileplus-mcp/` | **Stub** — only `.astro/` scaffold, `CLAUDE.md`, dependabot; no `pyproject.toml`, no server |
| Separate GitHub repo | `KooshaPari/agileplus-mcp` | **Does not exist** — monorepo subdir only |

**Catalog linkage:** PhenoMCPServers `registry.yaml` adds `agileplus-mcp-intent` (pointer, active impl) and `agileplus-mcp` (pointer, stub). Tier: Rust intent = tier-0; Python bridge = tier-2 when implemented.

**Next impl steps:** (1) scaffold FastMCP server in `agileplus-mcp/` per PhenoFastMCP template; (2) wire gRPC to `agileplus-api`; (3) promote catalog entry to `active` after `validate_catalog` path exists.

## traceability

MCP integration traces through eco-024:

- Spec: `docs/specs/eco/eco-024-traceability/spec.md`
- Schema: `docs/requirements/traceability/SCHEMA.md`
- Matrix: `docs/requirements/traceability/MATRIX.md`
- Intent design: `docs/superpowers/specs/2026-06-14-intent-artifact-design.md` → `crates/agileplus-mcp-intent/`

Suggested trace anchors for MCP fleet:

```
spec: eco-024 / eco-014
code: crates/agileplus-mcp-intent/, agileplus-mcp/
catalog: PhenoMCPServers/catalog/registry.yaml (agileplus-*)
```

## spec status

| Bucket | count | eco IDs |
|--------|-------|---------|
| RETIRED | 6 | eco-001 … eco-006 |
| ACTIVE (in flight) | 1 | eco-029-consolidate-integration-branch |
| PENDING / active | 24 | eco-007 … eco-034 (excl. retired) |
| MCP-blocking | 5 | eco-014, eco-024, eco-027, eco-033, eco-034 |

Plan-required (open): eco-024, eco-028, eco-030, eco-034.
