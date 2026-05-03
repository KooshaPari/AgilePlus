# Work Packages: Agent Framework Expansion — Complete Across 6 Repositories

> Specification: `spec.md` · Plan: `plan.md` · Spec ID: `AgilePlus-016`
>
> Planner mandate: deliverables, acceptance criteria, references, dependencies.
> No code, no shell snippets. Implementers own all source-level decisions.

**Inputs:** `spec.md`, `plan.md`, repo CLAUDE.md files for each affected repo,
`phenotype-shared` crate APIs, MCP specification (referenced in `spec.md`).

**Prerequisites:** Phase 1 (Discovery) and Phase 2 (Design) tasks from
`plan.md` complete; reuse audit logged in `repos/worklogs/RESEARCH.md`.

**Scope:** Six repositories — Agentora, AgentMCP, agent-wave,
agentops-policy-federation, helMo, agent-devops-setups.

---

## WP-001: Agentora — Complete Agent Orchestration Framework

- **Phase:** 3 — Build
- **Sequence:** 1 (gates WP-002..WP-005)
- **Owner Repo:** Agentora
- **File Scope:** Agentora `src/`, `tests/`, `docs/`
- **Depends On:** Plan task D2.1 (lifecycle state-machine spec)
- **Estimated Effort:** Major refactor — 3–5 parallel build subagents,
  15–30 tool calls, ~15–20 min wall clock.

### Deliverables

- Multi-agent coordination supporting leader-follower and peer-to-peer
  patterns, with a documented choice point for broadcast.
- Task decomposition + assignment engine.
- Unified agent-lifecycle implementation matching the D2.1 state-machine spec.
- Integration interfaces (trait surface) consumed by WP-002, WP-003, WP-004,
  WP-005. Interfaces published as a versioned crate contract for downstream
  pinning.
- Health monitoring: heartbeat, failure detection, recovery hooks.
- Reuse: build on `phenotype-state-machine` and `phenotype-event-sourcing`
  rather than re-implementing FSM or audit log primitives.

### Acceptance Criteria

- All deliverables present and exercised by tests.
- ≥80% line coverage on orchestration core.
- Multi-agent integration test demonstrates leader-follower and peer-to-peer
  coordination through public interfaces only.
- Per-repo Phase 4 gates green (V4.1).
- Public APIs documented; coordination examples included in repo docs.
- No suppressions added to bypass clippy or test failures.

### Risks

- Orchestration complexity → start from leader-follower, layer peer-to-peer.
- Failure recovery semantics → use checkpoint-based retry; document drop
  vs replay semantics per lifecycle state.

---

## WP-002: AgentMCP — MCP Protocol Server + Agentora Bridge

- **Phase:** 3 — Build
- **Sequence:** 2
- **Owner Repo:** AgentMCP
- **File Scope:** AgentMCP `src/`, `tests/`, `docs/`
- **Depends On:** WP-001 (orchestrator interfaces), plan task D2.2 (MCP
  contract).
- **Estimated Effort:** Cross-stack feature — 8–15 tool calls, ~12 min.

### Deliverables

- MCP protocol server: tools, resources, prompts, sampling. Conformance
  references the official MCP specification cited in `spec.md`.
- Agentora bridge: orchestrated agents surface as MCP tools; tool
  definitions auto-derived from agent capability metadata published by
  WP-001.
- MCP resource endpoints serving agent state and run results.
- MCP version negotiation with documented supported range.

### Acceptance Criteria

- ≥80% coverage on protocol handlers.
- Integration test: external MCP client invokes an Agentora-managed agent
  end-to-end and receives a result.
- Per-repo Phase 4 gates green.
- No suppressions; protocol non-conformance treated as a defect, not a
  documented exception.

### Risks

- Protocol drift → pin to MCP version in tests; surface negotiation failures
  loudly per "Optionality and Failure Behavior."

---

## WP-003: agent-wave — Event-Driven Agent Communication

- **Phase:** 3 — Build
- **Sequence:** 3 (parallel with WP-002, WP-004, WP-005 after WP-001)
- **Owner Repo:** agent-wave
- **File Scope:** agent-wave `src/`, `tests/`, `docs/`
- **Depends On:** WP-001 (lifecycle hooks), plan task D2.3 (bus contract).
- **Estimated Effort:** Cross-stack feature — 8–15 tool calls, ~12 min.

### Deliverables

- Event bus with publish / subscribe / unsubscribe and topic-based routing.
- Filter expressions: type, source agent, priority, custom predicates.
- Lifecycle-aware delivery: queue on `starting`, deliver on `running`, drop
  with audit on `terminated`.
- Event persistence + replay backed by `phenotype-event-sourcing`.
- Reuse: do not re-implement the hash-chained log; depend on
  `phenotype-event-sourcing`.

### Acceptance Criteria

- ≥80% coverage on bus + routing.
- Integration test: three agents exchange events under back-pressure with
  documented ordering guarantees.
- Per-repo Phase 4 gates green.

### Risks

- Event volume → rate limiting and dedup at publish.
- Ordering → causal ordering for related events; document guarantees.

---

## WP-004: agentops-policy-federation — Policy Distribution

- **Phase:** 3 — Build
- **Sequence:** 4 (parallel with WP-002, WP-003, WP-005)
- **Owner Repo:** agentops-policy-federation
- **File Scope:** agentops-policy-federation `src/`, `tests/`, `docs/`
- **Depends On:** WP-001 (group/agent surfaces), plan task D2.4 (policy
  attachment model).
- **Estimated Effort:** Cross-stack feature — 8–15 tool calls, ~12 min.

### Deliverables

- Policy schema with rules, scopes, priorities, versioning.
- Distribution: push to agent groups; version tracking.
- Enforcement at agent level; deny-overrides default per D2.4.
- Conflict detection + resolution.
- Audit log of evaluations, violations, overrides — backed by
  `phenotype-event-sourcing`.
- Reuse: build on `phenotype-policy-engine` rather than authoring a parallel
  rule evaluator.

### Acceptance Criteria

- ≥80% coverage on distribution + enforcement.
- Integration test: multi-agent scenario with conflicting policies; expected
  resolution outcome verified.
- Per-repo Phase 4 gates green.

### Risks

- Enforcement performance → cache policy evaluations; benchmark hot paths.
- Conflict semantics → resolution rule made explicit in docs and tests.

---

## WP-005: helMo — Agent Mobility

- **Phase:** 3 — Build
- **Sequence:** 5 (parallel with WP-002, WP-003, WP-004)
- **Owner Repo:** helMo
- **File Scope:** helMo `src/`, `tests/`, `docs/`
- **Depends On:** WP-001 (lifecycle + orchestrator action surface), plan
  task D2.5 (mobility protocol).
- **Estimated Effort:** Cross-stack feature — 8–15 tool calls, ~12 min.

### Deliverables

- Snapshot format for agent state, memory, execution context.
- Capture + restore implementation honoring D2.5 protocol.
- Migration coordinator: source ↔ target host handshake, transfer, commit.
- Rollback on failure with checkpoint restore.
- Mobile ↔ stationary agent messaging across migration boundaries.

### Acceptance Criteria

- ≥80% coverage on serialization + migration.
- Integration test: mock multi-host migration with induced failure +
  rollback.
- Per-repo Phase 4 gates green.

### Risks

- State capture completeness → enumerate captured fields in docs; treat
  uncaptured state as a documented defect, not silent loss.
- Migration failure → checkpoint-based rollback exercised in tests.

---

## WP-006: agent-devops-setups — CI/CD + Deployment Templates

- **Phase:** 3 — Build (final convergence)
- **Sequence:** 6
- **Owner Repo:** agent-devops-setups
- **File Scope:** agent-devops-setups `templates/`, `docs/`
- **Depends On:** WP-001 through WP-005 + plan task D2.6 (template matrix).
- **Estimated Effort:** Small feature — 3–6 tool calls, ~3 min.

### Deliverables

- GitHub Actions templates: agent-project CI (build, test, lint) and
  orchestration-cluster deployment.
- GitLab CI templates mirroring functionality.
- Deployment template for an orchestration cluster (container + manifest).
- Monitoring template (Prometheus + Grafana dashboards) for agent health.
- Alerting template covering agent failure and policy violations.
- End-to-end example agent project consuming all templates.
- Compliance: templates conform to Phenotype scripting hierarchy — no new
  shell beyond ≤5-line glue with inline justification.

### Acceptance Criteria

- All templates pass platform-native lint + dry-run.
- Documentation includes per-template setup and customization steps.
- Example project boots through CI green.
- Per-repo Phase 4 gates green (V4.6).

### Risks

- Template drift → pin upstream actions/images by digest where possible;
  document update cadence.
- Cross-platform parity → tested on both GitHub Actions and GitLab CI.

---

## Dependency Summary

```
                    ┌── WP-002 (AgentMCP) ──┐
WP-001 (Agentora) ──┼── WP-003 (agent-wave) ┼── WP-006 (devops-setups)
                    ├── WP-004 (policy-fed) ─┤
                    └── WP-005 (helMo) ──────┘
```

WP-001 is the gate; WP-002..WP-005 fan out in parallel; WP-006 converges.

## Phase 4 Validation Hook

Each WP must satisfy `plan.md` V4.x acceptance before its WP closes. Cross-repo
scenarios (V4.2..V4.5) are owned by Phase 4 and may flag regressions back into
the originating WP. WP-006 is gated on V4.6.

## Phase 5 Handoff

On Phase 4 sign-off, plan tasks H5.1..H5.4 update CHANGELOGs, the org-pages
portfolio entry, and `repos/worklogs/ARCHITECTURE.md`. Open follow-ups
(device-automation, plugin system) are routed to specs 011 and 015.

## MVP Note

Spec MVP slice is WP-001 alone (orchestration-only). WP-001 + WP-002 yields
external MCP-client integration. Full success criteria (`spec.md`) require
all six WPs.
