# External Library Leverage Research

**Date**: 2026-03-03
**Purpose**: Identify OSS libraries/frameworks that can replace custom code across the Phenotype ecosystem, reducing LOC while maintaining or growing feature coverage.
**Constraint**: Every removed line must result in feature growth/maintenance (>=0) and maintainability/quality growth (>=1).

---

## Executive Summary

Across ~410K LOC in 7 repos, we identified **94-153K LOC** of potential reduction through aggressive library adoption. The top 5 highest-leverage swaps:

1. **Bifrost** (Go LLM gateway) → replaces cliproxy++ 15 executors: **25-40K LOC**
2. **Inspect AI** (Python eval framework) → replaces portage benchmark harness: **15-25K LOC**
3. **Temporal** (durable workflows) → replaces thegent dispatcher+Redis locks: **6-10K LOC**
4. **OPA + Guardrails AI** → replaces thegent governance: **6-10K LOC**
5. **x/oauth2 + Goth + go-sse** → replaces cliproxy++ auth+streaming: **9-22K LOC**

---

## Per-Repo Adoption Plan

### cliproxyapi++ (~80K LOC → ~25-40K LOC)

| Library | What It Replaces | LOC Saved | Complexity | Risk |
|---------|-----------------|-----------|------------|------|
| **maximhq/bifrost** | 15 provider executors + translation matrix + retry + streaming | 25-40K | Medium | Kiro/Copilot/Cursor not covered; per-user OAuth needs wrapper |
| **golang.org/x/oauth2** | 3 parallel auth systems → 1 TokenSource registry | 3-8K | Low-Medium | Machine-to-machine flows need custom TokenSource impls |
| **markbates/goth** | Web OAuth callback handlers (75+ providers) | 3-6K | Low-Medium | Browser flows only |
| **tmaxmax/go-sse** | Custom SSE parsers + reconnect logic | 2-5K | Low | Provider JSON envelopes still per-provider |
| **coder/websocket** | Custom WebSocket streaming parsers | 1-3K | Low | None |
| **avast/retry-go** | Per-executor retry loops (15x duplicated) | 1-3K | Very Low | None |
| **hashicorp/go-retryablehttp** | HTTP retry wrappers in executors | 1.5-4K | Very Low | MPL-2.0 license review |
| **sony/gobreaker** | Circuit breaking / provider health | 0.5-2K | Very Low | None |

**Strategy**: Embed Bifrost as Go package for standard providers (OpenAI, Anthropic, Gemini, Bedrock, Azure, Cohere, Mistral, Groq, Ollama). Keep Kiro/Copilot/Cursor as thin custom executors using go-retryablehttp. Collapse 3 auth systems to x/oauth2 + Goth.

### thegent (~100K LOC → ~60-75K LOC)

| Library | What It Replaces | LOC Saved | Complexity | Risk |
|---------|-----------------|-----------|------------|------|
| **temporalio/sdk-python** | Hierarchical dispatcher + Redis locks + scheduler | 6-10K | High | Temporal server dependency |
| **pydantic/pydantic-ai** | Agent classes, tool registration, OTel wiring | 5-8K | Medium | v0.x API |
| **guardrails-ai/guardrails** | Semantic firewall, output validation | 4-6K | Medium | Flask-based daemon |
| **open-policy-agent/opa** | Policy engine, compliance, RBAC | 4-7K | Medium | Sidecar process, Rego DSL |
| **modelcontextprotocol/python-sdk** | Custom MCP handler | 2-4K | Low-Medium | v2 API changes Q1 2026 |
| **pydantic/pydantic-settings** | Custom config loaders | 2-4K | Very Low | None |
| **BerriAI/litellm** (proxy mode) | Custom LiteLLM wrapper | 1K | Very Low | Network hop |
| **Textualize/textual** | Custom TUI | 1-8K | Medium | v0.x, asyncio-only |

**Strategy**: Temporal for durable orchestration (eliminates Redis lock complexity). PydanticAI for agent definitions. OPA for policy-as-code + Guardrails AI for semantic validation. LiteLLM proxy shared with portage.

### portage (~40K LOC → ~13-23K LOC)

| Library | What It Replaces | LOC Saved | Complexity | Risk |
|---------|-----------------|-----------|------------|------|
| **UKGovernmentBEIS/inspect_ai** | 40 benchmark adapters + BaseAgent + task harness | 15-25K | High | UK AISI continuity; Docker-first |
| **BerriAI/litellm** (proxy) | Duplicate LiteLLM wrapper | 0.7K | Very Low | Shared with thegent |
| **pydantic/pydantic-settings** | Config loaders | 1K | Very Low | None |
| **Bogdanp/dramatiq** or **samuelcolvin/arq** | Job queue orchestrator | 0.5-0.7K | Low | None |

**Strategy**: Port new benchmarks as Inspect AI Task definitions. Retain custom environment drivers (Modal, Runloop, Daytona, E2B, GKE). Share LiteLLM proxy with thegent.

### heliosCLI (~110K LOC → ~102-106K LOC)

| Library | What It Replaces | LOC Saved | Complexity | Risk |
|---------|-----------------|-----------|------------|------|
| **0xPlaygrounds/rig** | thegent-router, thegent-runtime, provider crates | 2-4.5K | Medium | Pre-1.0 |
| **rmcp 0.16** (#[tool] macros) | MCP schema boilerplate | 200-500 | Very Low | None |
| **@modelcontextprotocol/sdk** | Hand-rolled MCP transport in codex-cli | 500-1.5K | Low | None |
| **hakoniwa** | Linux bwrap orchestration | 500-1K | Medium | Linux-only |

**Already adopted correctly**: landlock crate, gix (gitoxide), rmcp 0.15, Extism (planned).
**Do NOT adopt**: Cap'n Proto/FlatBuffers (negative ROI), oclif (startup penalty).

### heliosApp (~77K LOC → ~66-73K LOC)

| Library | What It Replaces | LOC Saved | Complexity | Risk |
|---------|-----------------|-----------|------------|------|
| **event-driven-io/emmett** | Custom audit ledger/replay/snapshot | 2-5K | Medium | Pre-1.0 |
| **jsonnull/electron-trpc** | Custom Envelope/Command/Response IPC | 1-3K | Medium | Pre-1.0, tRPC v11 fork |
| Tauri 2.0 (long-term) | Electron runtime | 3-8K | HIGH | 6-12 month migration |

**Strategy**: Emmett + electron-trpc now. Tauri evaluation on greenfield companion app later.

### trace (Go+Python+React)

| Library | What It Replaces | LOC Saved | Complexity | Risk |
|---------|-----------------|-----------|------------|------|
| **tree-sitter/go-tree-sitter** | Custom code indexer/parsers | 2-6K | Medium | CGo build requirement |
| **sourcegraph/scip-go** | Deep Go cross-reference indexing | 3-8K | High | Sourcegraph commercial interests |
| **philippgille/chromem-go** | Custom embedding service | 1-3K | Low | Brute-force search only |
| **neo4j/neo4j-go-driver v5** | Custom Neo4j client wrappers | 0.5-1.5K | Low | None |
| **nats-io/nats.go** (JetStream) | Custom message routing | 0.5-2K | Low | Already on NATS |

**No OSS replacement exists for trace itself** — its code-semantic + requirement traceability with graph storage is differentiated.

---

## Cross-Repo Deduplication via Libraries

| Duplication | Current State | Solution |
|------------|--------------|---------|
| LiteLLM wrappers | thegent 1K + portage 700 LOC | Shared LiteLLM proxy instance |
| cliproxy clients | thegent Python + agentapi++ Go + trace Go | Publish cliproxy++ Go SDK; create Python SDK |
| Auth patterns | cliproxy++ 3 systems + agentapi++ local | phenotype-go-authkit backed by x/oauth2 |
| Provider abstractions | cliproxy++ executors + heliosApp ProviderAdapter + thegent ad-hoc | Bifrost (Go) + Rig (Rust) + PydanticAI (Python) |
| Event sourcing | heliosApp custom + proto AuditEvent | Emmett (TS) + proto schema |

---

## Do NOT Adopt (Negative ROI)

| Library | Why Not |
|---------|---------|
| Cap'n Proto / FlatBuffers | Negative ROI for moderate-frequency CLI messages |
| oclif | 70-100ms startup penalty in agent loops |
| NeMo Guardrails | Colang DSL lock-in, overkill |
| Tauri (now) | 6-12 month migration for uncertain gains |
| Wails | Smaller ecosystem than Tauri, Go-first mismatch |
| AutoGen | v0.4 instability, Microsoft dependency |
| wolkenkit | AGPL license incompatibility |

---

## Migration Priority (Phased)

### Phase A: Drop-in / Very Low Complexity
1. LiteLLM Proxy (shared gateway) — 1.7K LOC, config change
2. Pydantic Settings — 3K LOC, standard pattern
3. rmcp 0.16 upgrade — 200-500 LOC, version bump
4. avast/retry-go — 1-3K LOC, loop replacement
5. sony/gobreaker — 0.5-2K LOC, add circuit breaking
6. Official MCP Python SDK — 2-4K LOC, protocol swap

### Phase B: Medium Complexity, High Reward
7. x/oauth2 + Goth — 6-14K LOC, auth consolidation
8. Bifrost — 25-40K LOC, executor replacement
9. Emmett + electron-trpc — 3-8K LOC, heliosApp modernization
10. PydanticAI — 5-8K LOC, agent class replacement
11. Rig — 2-4.5K LOC, Rust provider consolidation

### Phase C: High Complexity, Transformative
12. Temporal — 6-10K LOC, orchestration replacement
13. OPA + Guardrails AI — 6-10K LOC, governance as code
14. Inspect AI — 15-25K LOC, eval framework adoption
15. tree-sitter + chromem-go — 3-9K LOC, trace modernization
