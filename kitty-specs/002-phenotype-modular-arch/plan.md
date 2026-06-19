# Implementation Plan: Phenotype Ecosystem Modularization & Plugin Architecture

**Branch**: `feat/phenotype-modular-arch` | **Date**: 2026-03-03 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `kitty-specs/002-phenotype-modular-arch/spec.md`

---

## Technical Context

| Decision | Choice | Source |
|----------|--------|--------|
| Shared code strategy | Polyrepo micro-packages, flat dep graph | `docs/governance/project_decomposition_governance.md` |
| Plugin architecture | Two-tier microkernel (compile-time native + runtime Extism/WASM) | `docs/governance/plugin_architecture_governance.md` |
| Cross-language types | Protobuf + buf codegen | `docs/engineering/language_governance_framework.md` |
| CQRS scope | Full audit across all 6 repos | User decision |
| codex/helios merge | Feature flags in single workspace | Research finding E001 |
| Package managers | Go modules, cargo, uv/pip, pnpm | Language governance |

## Constitution Check

No constitution file exists. Skipped.

---

## Phase 0: Research (COMPLETE)

Research was completed prior to planning. All findings documented in:
- `kitty-specs/002-phenotype-modular-arch/research.md` — 6 decisions with rationale
- `kitty-specs/002-phenotype-modular-arch/data-model.md` — Target architecture, shared package registry
- `kitty-specs/002-phenotype-modular-arch/research/evidence-log.csv` — 12 findings
- `kitty-specs/002-phenotype-modular-arch/research/source-register.csv` — 7 sources

All NEEDS CLARIFICATION items resolved. No outstanding unknowns.

---

## Phase 1: Design

### Architecture Overview

```
                    ┌─────────────────────────┐
                    │    phenotype-proto/      │  (Protobuf definitions)
                    │  domain/v1/ plugin/v1/   │
                    └──────────┬──────────────┘
                               │ buf generate
          ┌────────┬───────────┼───────────┬──────────┐
          ▼        ▼           ▼           ▼          ▼
    ┌─────────┐ ┌──────┐ ┌────────┐ ┌─────────┐ ┌────────┐
    │  Go     │ │ Rust │ │ Python │ │   TS    │ │  Go    │
    │ authkit │ │proto-│ │ infra  │ │audit-  │ │httpkit │
    │executor │ │ col  │ │agentpr │ │core    │ │        │
    └────┬────┘ └──┬───┘ └───┬────┘ └───┬────┘ └───┬───┘
         │         │         │          │           │
    ┌────┴────┐┌───┴────┐┌───┴───┐┌────┴────┐┌────┴────┐┌────────┐
    │cliproxy ││helios- ││thegent││helios-  ││agentapi ││portage │
    │   ++    ││  CLI   ││       ││  App    ││   ++    ││        │
    └─────────┘└────────┘└───────┘└─────────┘└─────────┘└────────┘
```

### Work Lanes (All Parallel)

Each lane is independent. No lane blocks another except:
- `phenotype-proto` must exist before repos can consume generated types
- Shared packages must be published before app repos import them

#### Lane 0: Foundation (phenotype-proto + shared package repos)

| Task | Description | Depends On |
|------|-------------|------------|
| L0.1 | Create `phenotype-proto` repo with buf config, initial .proto files | — |
| L0.2 | Define `domain/v1/session.proto`, `agent.proto`, `routing.proto`, `audit.proto` | L0.1 |
| L0.3 | Define `plugin/v1/executor.proto`, `tool.proto`, `adapter.proto` | L0.1 |
| L0.4 | Configure `buf generate` for Rust, Go, Python, TypeScript | L0.2, L0.3 |
| L0.5 | Create empty repos: `phenotype-go-authkit`, `phenotype-go-executor-core`, `phenotype-go-httpkit` | — |
| L0.6 | Create empty repos: `phenotype-rs-protocol`, `phenotype-py-infra`, `@helios/audit-core`, `@helios/protocol-types` | — |
| L0.7 | CI: buf lint + buf breaking on phenotype-proto | L0.4 |

#### Lane 1: heliosCLI (Rust + TypeScript)

| Task | Description | Depends On |
|------|-------------|------------|
| L1.1 | Merge codex-rs + helios-rs into single Cargo workspace with feature flags | — |
| L1.2 | Gate variant-specific code behind `#[cfg(feature = "codex-runtime")]` / `helios-runtime` | L1.1 |
| L1.3 | Remove helios-rs directory, update CI to build both variants | L1.2 |
| L1.4 | Extract `protocol` crate types to `phenotype-rs-protocol` shared repo | L0.6 |
| L1.5 | Implement Tier 1 plugin: `ToolPlugin` trait + `inventory` registration | L1.1 |
| L1.6 | Prototype Tier 2 plugin: Extism host SDK integration for user-defined tools | L1.5 |
| L1.7 | Emit audit events from codex-core session lifecycle | L0.4 |

#### Lane 2: cliproxyapi++ (Go)

| Task | Description | Depends On |
|------|-------------|------------|
| L2.1 | Extract `sdk/executor-core`: `ExecutorInterface`, `BaseExecutor`, retry/HTTP helpers | — |
| L2.2 | Refactor all 15 executors to use `executor-core` base | L2.1 |
| L2.3 | Extract auth logic to `phenotype-go-authkit` shared repo | L0.5 |
| L2.4 | Extract HTTP helpers to `phenotype-go-httpkit` shared repo | L0.5 |
| L2.5 | Implement Tier 1 plugin: `Executor` interface + `init()` registration | L2.1 |
| L2.6 | Prototype Tier 2 plugin: Extism host for third-party executor .wasm | L2.5 |
| L2.7 | Translator matrix builder: codegen or shared builder to reduce N×M duplication | L2.1 |
| L2.8 | Formalize executor + translator as hexagonal ports | L2.2 |
| L2.9 | Emit audit events from proxy routing decisions | L0.4 |

#### Lane 3: thegent (Python)

| Task | Description | Depends On |
|------|-------------|------------|
| L3.1 | Extract `thegent-config` package from `src/thegent/config*.py` | — |
| L3.2 | Extract `thegent-infra` package from `src/thegent/infra/` (11K LOC) | — |
| L3.3 | Extract `thegent-governance` from `src/thegent/governance/` | — |
| L3.4 | Merge `thegent-protocols` + `thegent-mcp` into `thegent-mcp` | — |
| L3.5 | Publish `thegent-infra` internals to `phenotype-py-infra` shared repo | L0.6, L3.2 |
| L3.6 | Formalize `AdapterPort` pattern across all adapters | L3.3 |
| L3.7 | Implement Tier 1 plugin: `entry_points` registration for agents and adapters | L3.6 |
| L3.8 | Prototype Tier 2 plugin: Extism host for user-defined skills | L3.7 |
| L3.9 | Emit audit events from agent execution lifecycle | L0.4 |
| L3.10 | Make `src/thegent/` a thin re-export layer | L3.1-L3.4 |

#### Lane 4: heliosApp (TypeScript)

| Task | Description | Depends On |
|------|-------------|------------|
| L4.1 | Extract `@helios/audit-core` from `apps/runtime/src/audit/` | — |
| L4.2 | Extract `@helios/protocol-types` from `apps/runtime/src/protocol/types.ts` | — |
| L4.3 | Extract service interfaces to `@helios/service-contracts` | — |
| L4.4 | Publish packages to npm/GitHub Packages | L4.1, L4.2, L4.3 |
| L4.5 | Refactor runtime to consume extracted packages | L4.4 |
| L4.6 | Implement aggregated audit trail view (consuming events from all repos) | L4.1, L0.4 |

#### Lane 5: portage (Python)

| Task | Description | Depends On |
|------|-------------|------------|
| L5.1 | Formalize ports: `typing.Protocol` for executor, reporter, loader | — |
| L5.2 | Emit trial events: `TrialStarted`, `TrialCheckpoint`, `TrialCompleted` | L0.4 |
| L5.3 | Implement benchmark adapter plugin registry (Tier 1) | L5.1 |
| L5.4 | Prototype Tier 2 plugin: Extism host for user-defined adapters | L5.3 |
| L5.5 | Consume `phenotype-py-infra` for shared utilities | L3.5 |

#### Lane 6: agentapi++ (Go)

| Task | Description | Depends On |
|------|-------------|------------|
| L6.1 | Extract `internal/domain/` package: Agent, Session, RoutingRule, BenchmarkData | — |
| L6.2 | Consume `phenotype-go-authkit` for auth logic | L2.3 |
| L6.3 | Consume `phenotype-go-httpkit` for HTTP helpers | L2.4 |
| L6.4 | Implement `AgentDriver` interface (Tier 1 plugin) | L6.1 |
| L6.5 | Emit routing audit events: `AgentRouted`, `SessionCreated` | L0.4 |

### Dependency DAG (Critical Path)

```
L0.1 ──▶ L0.2 ──▶ L0.4 ──▶ L1.7, L2.9, L3.9, L5.2, L6.5  (audit events)
L0.1 ──▶ L0.3 ──▶ L0.4
L0.5 ──────────────────▶ L2.3, L2.4 ──▶ L6.2, L6.3
L0.6 ──────────────────▶ L1.4, L3.5, L4.1, L4.2

L1.1 ──▶ L1.2 ──▶ L1.3  (helios merge — independent, no external deps)
L2.1 ──▶ L2.2 ──▶ L2.5 ──▶ L2.6  (executor plugin — independent)
L3.1-L3.4 ──▶ L3.10  (thegent migration — independent)
L4.1-L4.3 ──▶ L4.4 ──▶ L4.5  (heliosApp extraction — independent)
```

**Critical path:** L0.1 → L0.2/L0.3 → L0.4 → audit event integration across all repos.

All other work (helios merge, executor extraction, thegent migration, heliosApp extraction) runs fully parallel with no cross-lane dependencies.

### Execution Strategy

- **Batch 1** (parallel, no deps): L0.1-L0.7, L1.1-L1.3, L2.1-L2.2, L3.1-L3.4, L4.1-L4.3, L5.1, L6.1
- **Batch 2** (after shared repos exist): L1.4, L2.3-L2.4, L3.5, L4.4-L4.5, L5.5, L6.2-L6.3
- **Batch 3** (after proto codegen): L1.7, L2.9, L3.9, L5.2, L6.5 (audit events)
- **Batch 4** (plugin systems): L1.5-L1.6, L2.5-L2.8, L3.6-L3.8, L5.3-L5.4, L6.4
- **Batch 5** (integration): L4.6, L3.10

### Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Extism WASM perf insufficient | Prototype in L2.6 first (cliproxyapi++ executor); fall back to HashiCorp go-plugin if needed |
| buf codegen quality issues | Validate generated types compile in all 4 languages during L0.4 |
| codex/helios merge breaks tests | Feature-flag approach preserves both build paths; run full test suites per variant |
| thegent migration breaks imports | Re-export layer (L3.10) maintains backward compatibility |
| Shared package versioning overhead | Start at v0.1.0; accept breaking changes freely pre-v1.0 |

---

## Artifacts Generated

| Artifact | Path | Status |
|----------|------|--------|
| Research | `kitty-specs/002-phenotype-modular-arch/research.md` | Complete |
| Data Model | `kitty-specs/002-phenotype-modular-arch/data-model.md` | Complete |
| Evidence Log | `kitty-specs/002-phenotype-modular-arch/research/evidence-log.csv` | Complete |
| Source Register | `kitty-specs/002-phenotype-modular-arch/research/source-register.csv` | Complete |
| Spec | `kitty-specs/002-phenotype-modular-arch/spec.md` | Complete |
| Plan | `kitty-specs/002-phenotype-modular-arch/plan.md` | This document |
| Governance: Decomposition | `docs/governance/project_decomposition_governance.md` | Complete |
| Governance: Language/Framework | `docs/engineering/language_governance_framework.md` | Complete |
| Governance: Plugin Architecture | `docs/governance/plugin_architecture_governance.md` | Complete |

---

**Next step**: `/spec-kitty.tasks` to generate work packages from this plan.
