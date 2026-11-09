# Plan: Agent Framework Expansion — Complete Across 6 Repositories

> Specification: `spec.md` (this directory) · Spec ID: `AgilePlus-016` · Status: IN_PROGRESS
>
> Planner mandate: This document contains specs, acceptance criteria, references,
> and dependency structure only. **No code.** Implementers consult per-repo
> source trees and existing crates referenced below.

## Scope Reaffirmation

Complete and integrate the agent framework across six Phenotype-org repositories
so that orchestration, MCP protocol, event-driven communication, policy
distribution, agent mobility, and DevOps templates compose into a single
unified agent lifecycle. See `spec.md` "Repositories Affected" and
"Success Criteria" for normative scope.

## Cross-Project Reuse Audit (mandatory before Build)

Per Phenotype Org Cross-Project Reuse Protocol, the Discovery phase MUST inspect
existing assets that overlap with this work and prefer extension over re-author.
Inputs to consult:

- `repos/agentkit/` — active Rust agent framework with skill system; candidate
  shared substrate for `Agentora` orchestration types and `AgentMCP` tool
  surface.
- `repos/agentapi-plusplus/` — Go multi-model AI gateway; routing/transport
  patterns inform `agent-wave` event bus and MCP transport choices.
- `repos/cheap-llm-mcp/` and `repos/thegent-dispatch/` — validated Python
  FastMCP server and Rust dispatch crate (memory: session_2026_04_22). Reuse
  the dispatch contract before authoring a new agent-call surface.
- `repos/phenotype-shared/crates/` — `phenotype-event-sourcing`,
  `phenotype-cache-adapter`, `phenotype-policy-engine`,
  `phenotype-state-machine`. **Mandatory** dependencies for orchestration
  state, policy evaluation, and event audit.
- Device-automation rebuild repos (`KDesktopVirt`, `KVirtualStage`, `kmobile`)
  are out of scope for this spec but record any orchestration-layer touchpoints
  for spec 011 handoff.

Reuse decisions and rejections are logged in
`repos/worklogs/RESEARCH.md` under `[AgilePlus]` `[cross-repo]` tags before
Phase 2 (Design) begins.

## Phased WBS (Discovery → Design → Build → Test/Validate → Deploy/Handoff)

### Phase 1 — Discovery

Survey current state of all six repos and the shared crates listed above.
Output: gap analysis per repo, integration interface inventory, reuse
decisions. No code authored.

- D1.1 Audit Agentora orchestration surface and lifecycle gaps.
- D1.2 Audit AgentMCP protocol coverage versus current MCP spec.
- D1.3 Audit agent-wave event bus, routing, persistence semantics.
- D1.4 Audit agentops-policy-federation distribution + enforcement model.
- D1.5 Audit helMo mobility prototype: state capture, transport, rollback.
- D1.6 Audit agent-devops-setups templates against current GitHub Actions /
  GitLab CI feature set and Phenotype scripting hierarchy (no shell-only
  templates accepted).
- D1.7 Cross-repo reuse audit (see "Cross-Project Reuse Audit" above).

### Phase 2 — Design

Author architecture decision records (ADRs) and integration interface
specifications. References, diagrams, acceptance criteria. No code.

- D2.1 Unified agent-lifecycle state machine spec
  (init → ready → running → suspended → migrating → terminated → cleaned).
- D2.2 Orchestration ↔ MCP integration contract: how orchestrated agents
  surface as MCP tools, capability auto-derivation rules.
- D2.3 Lifecycle ↔ event-bus contract: delivery guarantees per state,
  queueing on starting, drop-on-terminated semantics.
- D2.4 Policy attachment model: per-agent vs per-group, evaluation order,
  conflict resolution (deny-overrides default).
- D2.5 Mobility protocol: snapshot format, transport, rollback checkpoints,
  cross-host messaging.
- D2.6 DevOps template matrix: which orchestration topologies and which CI
  systems are first-class.

### Phase 3 — Build

Implement per-repo work packages. Each work package is owned by exactly one
repository; integration points are exercised through interface contracts
defined in Phase 2.

- B3.1 Agentora orchestration core (WP-001).
- B3.2 AgentMCP protocol server + Agentora bridge (WP-002).
- B3.3 agent-wave bus + lifecycle integration (WP-003).
- B3.4 agentops-policy-federation distribution + enforcement (WP-004).
- B3.5 helMo mobility + orchestration hooks (WP-005).
- B3.6 agent-devops-setups templates (WP-006).

### Phase 4 — Test / Validate

Per-repo unit + integration suites, then cross-repo end-to-end scenarios.

- V4.1 Per-repo quality gates: workspace test, clippy with warnings denied,
  fmt check.
- V4.2 Multi-agent orchestration scenario (leader-follower, peer-to-peer).
- V4.3 MCP-client-driven agent invocation through AgentMCP → Agentora.
- V4.4 Event-routing scenario across three agents with policy enforcement.
- V4.5 Mobility scenario with rollback and cross-host messaging.
- V4.6 Template smoke: dry-run lint of every devops template.

### Phase 5 — Deploy / Handoff

- H5.1 Publish per-repo CHANGELOG entries; tag with CalVer/SemVer per repo
  policy.
- H5.2 Update org-pages portfolio entry referencing the unified framework.
- H5.3 Update `repos/worklogs/ARCHITECTURE.md` with the integration ADR
  pointers.
- H5.4 Hand off open follow-ups (device-automation, plugin system) to
  specs 011 and 015 via tracker updates.

## Dependency DAG

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| 1 | D1.1 | Audit Agentora orchestration | — |
| 1 | D1.2 | Audit AgentMCP protocol | — |
| 1 | D1.3 | Audit agent-wave events | — |
| 1 | D1.4 | Audit agentops-policy-federation | — |
| 1 | D1.5 | Audit helMo mobility | — |
| 1 | D1.6 | Audit agent-devops-setups | — |
| 1 | D1.7 | Cross-repo reuse audit | D1.1–D1.6 |
| 2 | D2.1 | Unified lifecycle state machine spec | D1.1, D1.7 |
| 2 | D2.2 | Orchestration ↔ MCP contract | D1.1, D1.2, D2.1 |
| 2 | D2.3 | Lifecycle ↔ event-bus contract | D1.3, D2.1 |
| 2 | D2.4 | Policy attachment model | D1.4, D2.1 |
| 2 | D2.5 | Mobility protocol | D1.5, D2.1 |
| 2 | D2.6 | DevOps template matrix | D1.6, D2.1–D2.5 |
| 3 | WP-001 | Agentora orchestration build | D2.1 |
| 3 | WP-002 | AgentMCP build | WP-001, D2.2 |
| 3 | WP-003 | agent-wave build | WP-001, D2.3 |
| 3 | WP-004 | agentops-policy-federation build | WP-001, D2.4 |
| 3 | WP-005 | helMo build | WP-001, D2.5 |
| 3 | WP-006 | agent-devops-setups build | WP-001..WP-005, D2.6 |
| 4 | V4.1 | Per-repo quality gates | WP-001..WP-006 |
| 4 | V4.2 | Multi-agent orchestration scenario | WP-001 |
| 4 | V4.3 | MCP-client → Agentora scenario | WP-001, WP-002 |
| 4 | V4.4 | Event + policy scenario | WP-001, WP-003, WP-004 |
| 4 | V4.5 | Mobility scenario | WP-001, WP-005 |
| 4 | V4.6 | Template smoke | WP-006 |
| 5 | H5.1 | CHANGELOG + tag | V4.1–V4.6 |
| 5 | H5.2 | Org-pages portfolio update | H5.1 |
| 5 | H5.3 | ARCHITECTURE worklog update | H5.1 |
| 5 | H5.4 | Handoff to specs 011, 015 | H5.1 |

Critical path: D1.1 → D1.7 → D2.1 → WP-001 → WP-002 → V4.3 → H5.1 → H5.4
(8 nodes). After WP-001, WP-002..WP-005 fan out in parallel; WP-006 is the
final convergence node before validation.

## Effort Profile (agent-led)

Per global "Timescales: Agent-Led, Aggressive Estimates" — quoted in tool
calls / parallel subagents / wall-clock minutes. No human checkpoints.

| Phase | Profile | Wall-clock target |
|-------|---------|-------------------|
| 1 — Discovery | 6 parallel `Explore` subagents (one per repo) + 1 reuse audit | 8–12 min |
| 2 — Design | 3 parallel design subagents on independent contracts; lifecycle anchor sequenced first | 10–15 min |
| 3 — Build (WP-001) | Major refactor: 3–5 parallel `general-purpose` subagents, 15–30 tool calls | 15–20 min |
| 3 — Build (WP-002..WP-005) | 4 cross-stack features in parallel, each 8–15 tool calls | 12–18 min |
| 3 — Build (WP-006) | Small feature, 3–6 tool calls | ~3 min |
| 4 — Test / Validate | Per-repo gates parallel; scenarios sequential on shared harness | 8–12 min |
| 5 — Deploy / Handoff | 4 small docs/tag tasks in parallel | 3–5 min |

Total wall-clock budget: ~60–90 min of orchestrated agent activity, no human
intervention beyond initial prompt and merge approvals.

## Risks & Mitigations

Inherits and refines `spec.md` "Risks" table:

- **Integration drift between repos** — bind each integration contract to a
  versioned interface in `phenotype-shared`; CI lints downstream consumers
  against the published contract.
- **Quality-gate fan-out cost** — run per-repo gates in parallel, never
  serially; respect the 4-concurrent cargo cap from disk-budget policy.
- **Cross-repo PR sequencing** — gate WP-002..WP-005 PRs behind the WP-001
  contract crate version bump rather than coordinating six PRs by hand.
- **Doc + worklog rot** — Phase 5 explicitly updates ARCHITECTURE worklog and
  org-pages so this work does not become invisible after merge.

## Out-of-Scope Confirmations

Mirrors `spec.md` "Non-Goals". No new agent types, no UI/UX, no marketplace,
no training/fine-tuning. Device-automation rebuilds (KDesktopVirt /
KVirtualStage / kmobile) remain owned by spec 011.

## Traces

- `spec.md` (this directory) — normative scope.
- Spec 007 — thegent-completion (consumer of orchestration).
- Spec 015 — plugin-system-completion (peer integration surface).
- Spec 001 — spec-driven-development-engine (workflow substrate).
- Memory: `session_2026_04_22_cheap_llm_build.md` — reused dispatch contract.
