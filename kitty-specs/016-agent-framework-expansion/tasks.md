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

## Phase 3 — Build

### WP-001 — Agentora — Complete Agent Orchestration Framework

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** plan task D2.1 (lifecycle state-machine spec)
- **Effort:** Cross-stack (15-30 tool calls, ~15–20 min wall clock)
- **File scope (read-only):**
  - `repos/Agentora/src/`
  - `repos/Agentora/tests/`
  - `repos/Agentora/docs/`
  - `repos/phenotype-shared/crates/phenotype-state-machine/src/`
  - `repos/phenotype-shared/crates/phenotype-event-sourcing/src/`
  - `kitty-specs/016-agent-framework-expansion/plan.md`
- **File scope (write):** `repos/Agentora/src/`
- **Acceptance criteria:**
  1. Multi-agent coordination supporting leader-follower and peer-to-peer patterns, with documented choice point for broadcast.
  2. Task decomposition + assignment engine present and exercised by tests.
  3. Unified agent-lifecycle implementation matching the D2.1 state-machine spec.
  4. Integration interfaces (trait surface) consumed by WP-002, WP-003, WP-004, WP-005; published as a versioned crate contract.
  5. Health monitoring: heartbeat, failure detection, recovery hooks.
  6. ≥80% line coverage on orchestration core.
  7. Multi-agent integration test demonstrates leader-follower and peer-to-peer coordination through public interfaces only.
  8. `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean; no suppressions.
- **Handoff prompt:** "Implement the Agentora orchestration framework per WP-001 acceptance criteria; build on phenotype-state-machine and phenotype-event-sourcing; publish versioned trait interfaces for WP-002..WP-005."

---

### WP-002 — AgentMCP — MCP Protocol Server + Agentora Bridge

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-001
- **Effort:** Cross-stack (8-15 tool calls, ~12 min)
- **File scope (read-only):**
  - `repos/AgentMCP/src/`
  - `repos/AgentMCP/tests/`
  - `repos/AgentMCP/docs/`
  - `repos/Agentora/src/` (orchestrator interfaces)
  - `kitty-specs/016-agent-framework-expansion/plan.md` (task D2.2)
  - Official MCP specification (referenced in `spec.md`)
- **File scope (write):** `repos/AgentMCP/src/`
- **Acceptance criteria:**
  1. MCP protocol server: tools, resources, prompts, sampling — conformant to official MCP spec.
  2. Agentora bridge: orchestrated agents surface as MCP tools; tool definitions auto-derived from agent capability metadata published by WP-001.
  3. MCP resource endpoints serving agent state and run results.
  4. MCP version negotiation with documented supported range.
  5. ≥80% coverage on protocol handlers.
  6. Integration test: external MCP client invokes an Agentora-managed agent end-to-end and receives a result.
  7. `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean; no suppressions.
- **Handoff prompt:** "Implement the AgentMCP protocol server + Agentora bridge per WP-002 acceptance; pin to MCP version in tests; surface negotiation failures loudly."

---

### WP-003 — agent-wave — Event-Driven Agent Communication

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-001
- **Effort:** Cross-stack (8-15 tool calls, ~12 min)
- **File scope (read-only):**
  - `repos/agent-wave/src/`
  - `repos/agent-wave/tests/`
  - `repos/agent-wave/docs/`
  - `repos/phenotype-shared/crates/phenotype-event-sourcing/src/`
  - `kitty-specs/016-agent-framework-expansion/plan.md` (task D2.3)
- **File scope (write):** `repos/agent-wave/src/`
- **Acceptance criteria:**
  1. Event bus with publish / subscribe / unsubscribe and topic-based routing.
  2. Filter expressions: type, source agent, priority, custom predicates.
  3. Lifecycle-aware delivery: queue on `starting`, deliver on `running`, drop with audit on `terminated`.
  4. Event persistence + replay backed by `phenotype-event-sourcing` (do not reimplement hash-chained log).
  5. ≥80% coverage on bus + routing.
  6. Integration test: three agents exchange events under back-pressure with documented ordering guarantees.
  7. `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean; no suppressions.
- **Handoff prompt:** "Implement the agent-wave event bus per WP-003 acceptance; depend on phenotype-event-sourcing for persistence; document ordering guarantees."

---

### WP-004 — agentops-policy-federation — Policy Distribution

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-001
- **Effort:** Cross-stack (8-15 tool calls, ~12 min)
- **File scope (read-only):**
  - `repos/agentops-policy-federation/src/`
  - `repos/agentops-policy-federation/tests/`
  - `repos/agentops-policy-federation/docs/`
  - `repos/phenotype-shared/crates/phenotype-policy-engine/src/`
  - `kitty-specs/016-agent-framework-expansion/plan.md` (task D2.4)
- **File scope (write):** `repos/agentops-policy-federation/src/`
- **Acceptance criteria:**
  1. Policy schema with rules, scopes, priorities, versioning.
  2. Distribution: push to agent groups; version tracking.
  3. Enforcement at agent level; deny-overrides default per D2.4.
  4. Conflict detection + resolution with explicit resolution rules in docs and tests.
  5. Audit log of evaluations, violations, overrides — backed by `phenotype-event-sourcing`.
  6. ≥80% coverage on distribution + enforcement.
  7. Integration test: multi-agent scenario with conflicting policies; expected resolution outcome verified.
  8. `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean; no suppressions.
- **Handoff prompt:** "Implement policy distribution + enforcement per WP-004 acceptance; build on phenotype-policy-engine; make conflict resolution rules explicit in docs and tests."

---

### WP-005 — helMo — Agent Mobility

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-001
- **Effort:** Cross-stack (8-15 tool calls, ~12 min)
- **File scope (read-only):**
  - `repos/helMo/src/`
  - `repos/helMo/tests/`
  - `repos/helMo/docs/`
  - `repos/Agentora/src/` (lifecycle + orchestrator action surface)
  - `kitty-specs/016-agent-framework-expansion/plan.md` (task D2.5)
- **File scope (write):** `repos/helMo/src/`
- **Acceptance criteria:**
  1. Snapshot format for agent state, memory, execution context; uncaptured state treated as a documented defect.
  2. Capture + restore implementation honoring D2.5 protocol.
  3. Migration coordinator: source ↔ target host handshake, transfer, commit.
  4. Rollback on failure with checkpoint restore; checkpoint-based rollback exercised in tests.
  5. Mobile ↔ stationary agent messaging across migration boundaries.
  6. ≥80% coverage on serialization + migration.
  7. Integration test: mock multi-host migration with induced failure + rollback.
  8. `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean; no suppressions.
- **Handoff prompt:** "Implement agent mobility per WP-005 acceptance; enumerate captured fields in docs; implement checkpoint-based rollback exercised in tests."

---

### WP-006 — agent-devops-setups — CI/CD + Deployment Templates

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-001, WP-002, WP-003, WP-004, WP-005
- **Effort:** Small (3-6 tool calls, ~3 min)
- **File scope (read-only):**
  - `repos/agent-devops-setups/templates/`
  - `repos/agent-devops-setups/docs/`
  - `kitty-specs/016-agent-framework-expansion/plan.md` (task D2.6)
- **File scope (write):** `repos/agent-devops-setups/templates/`
- **Acceptance criteria:**
  1. GitHub Actions templates: agent-project CI (build, test, lint) and orchestration-cluster deployment.
  2. GitLab CI templates mirroring functionality.
  3. Deployment template for an orchestration cluster (container + manifest).
  4. Monitoring template (Prometheus + Grafana dashboards) for agent health.
  5. Alerting template covering agent failure and policy violations.
  6. End-to-end example agent project consuming all templates.
  7. Templates conform to Phenotype scripting hierarchy: Rust default; no new shell beyond ≤5-line glue with inline justification.
  8. All templates pass platform-native lint + dry-run; example project boots through CI green.
- **Handoff prompt:** "Produce CI/CD + deployment templates per WP-006 acceptance; conform to Phenotype scripting hierarchy; pin upstream actions/images by digest where possible."

---

## Phase 4 — Test / Validate

### WP-007 — W3C Conformance + Protocol Test Plan

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-002
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `kitty-specs/016-agent-framework-expansion/research/w3c-conformance-test-plan.md`
- **Acceptance criteria:**
  1. Test matrix vs. official MCP specification test suite.
  2. Pass criteria: 100% of MUST tests; documented exclusions for SHOULD/MAY.
  3. Reference to upstream conformance harness.
  4. Plan for cross-repo integration tests (WP-002..WP-005 consuming Agentora interfaces).
- **Handoff prompt:** "Author W3C + MCP conformance test plan per WP-007 acceptance."

---

### WP-008 — Event Bus + Policy Integration Test Plan

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-003, WP-004
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `kitty-specs/016-agent-framework-expansion/research/integration-test-plan.md`
- **Acceptance criteria:**
  1. Scenarios: three agents exchange events under back-pressure (WP-003) with policy enforcement active (WP-004).
  2. Conflicting policy resolution end-to-end.
  3. Missing-context fallback path.
  4. Multi-hop propagation (3 agents).
- **Handoff prompt:** "Author event bus + policy integration test plan per WP-008 acceptance."

---

### WP-009 — Migration + Mobility Test Plan

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-005
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `kitty-specs/016-agent-framework-expansion/research/migration-test-plan.md`
- **Acceptance criteria:**
  1. Multi-host migration scenarios with induced failure + rollback verification.
  2. State completeness check: all enumerated captured fields present post-restore.
  3. Mobile ↔ stationary messaging continuity across migration boundary.
  4. Rollback triggered on host-handshake failure.
- **Handoff prompt:** "Author migration + mobility test plan per WP-009 acceptance."

---

### WP-010 — Quality Gate Sweep

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-001, WP-002, WP-003, WP-004, WP-005, WP-006
- **Effort:** Cross-stack (8-15 tool calls, ~6 min)
- **File scope (read-only):**
  - `repos/Agentora/src/`, `repos/Agentora/tests/`
  - `repos/AgentMCP/src/`, `repos/AgentMCP/tests/`
  - `repos/agent-wave/src/`, `repos/agent-wave/tests/`
  - `repos/agentops-policy-federation/src/`, `repos/agentops-policy-federation/tests/`
  - `repos/helMo/src/`, `repos/helMo/tests/`
  - `repos/agent-devops-setups/templates/`
- **File scope (write):** `kitty-specs/016-agent-framework-expansion/research/quality-gate-report.md`
- **Acceptance criteria:**
  1. `cargo test --workspace` exit 0 on all six repos.
  2. `cargo clippy --workspace -- -D warnings` exit 0 on all six repos.
  3. `cargo fmt --check` exit 0 on all six repos.
  4. `cargo deny check advisories` exit 0 (or documented accept-with-justification).
  5. SBOM regenerated for each repo.
  6. All templates pass platform-native lint + dry-run.
- **Handoff prompt:** "Run all quality gates on all six repositories; record results in quality-gate-report.md."

---

## Phase 5 — Deploy / Handoff

### WP-011 — CHANGELOG + Portfolio Entry

- **State:** planned
- **Phase:** 5 (Deploy/Handoff)
- **Depends on:** WP-010
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (read-only):**
  - All six repo CHANGELOGs
  - `repos/AgilePlus/docs/`
- **File scope (write):**
  - `repos/Agentora/CHANGELOG.md`
  - `repos/AgentMCP/CHANGELOG.md`
  - `repos/agent-wave/CHANGELOG.md`
  - `repos/agentops-policy-federation/CHANGELOG.md`
  - `repos/helMo/CHANGELOG.md`
  - `repos/agent-devops-setups/CHANGELOG.md`
  - org-pages portfolio entry (per `plan.md` H5.1)
- **Acceptance criteria:**
  1. Each of the six repos has a CHANGELOG entry for the new agent framework features.
  2. Org-pages portfolio entry updated to reflect completed work.
  3. `repos/worklogs/ARCHITECTURE.md` updated with agent framework ADR (per plan H5.3).
- **Handoff prompt:** "Update CHANGELOGs and portfolio entry per WP-011; log architecture decision in worklogs/ARCHITECTURE.md."

---

## Dependency & Execution Summary

```
Phase 3 (Build):      WP-001 (gates WP-002..WP-005) --> WP-006
                      WP-002 (AgentMCP) ──┐
                      WP-003 (agent-wave) ├─> WP-006 (devops-setups)
                      WP-004 (policy-fed) │
                      WP-005 (helMo) ──────┘
Phase 4 (Validate):   WP-007 (WP-002) ──┐
                      WP-008 (WP-003+WP-004) ├─> WP-010
                      WP-009 (WP-005) ────┘
Phase 5 (Deploy):     WP-011 (WP-010)
```

WP-001 is the gate; WP-002..WP-005 fan out in parallel; WP-006 converges build. WP-007..WP-009 validate in parallel; WP-010 and WP-011 close the spec.

**Critical path (6 nodes):** WP-001 → WP-002 → WP-007 → WP-010 → WP-011

**Parallelization opportunities:**
- Phase 3 build WPs (002, 003, 004, 005) can dispatch as 4 concurrent implementer agents after WP-001 lands.
- Phase 4 test plans (007, 008, 009) can author in parallel after their respective build WPs land.
- WP-006 (devops) waits for all five build WPs but can be drafted in parallel with WP-001.

**Total WP count:** 11 (WP-001 through WP-011).
