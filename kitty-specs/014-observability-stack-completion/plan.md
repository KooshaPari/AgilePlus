# Plan: Observability Stack Completion

**Spec ID:** AgilePlus-014
**Status:** IN_PROGRESS
**Last Updated:** 2026-04-25
**Scope:** Cross-repo — primary target `repos/PhenoObservability/` (13-crate workspace), with helix-logging, Tracera (PhenoKits), Profila, and Phench surfaces.

> Per global governance: planner output. No code in this document. References file paths and brief pseudocode only. Effort is expressed in agent terms (tool calls / parallel subagent batches / wall-clock minutes), not human days.

---

## Current State Snapshot (2026-04-25)

- `repos/PhenoObservability/` exists as a consolidated 13-crate Rust workspace covering tracing, logging, metrics, sentinel, observably-* macros, and storage adapters (SurrealDB, QuestDB, Dragonfly).
- Recent commits in PhenoObservability: phase-3 error migration complete across 13 sub-crates (`audit(errors)`), monthly SBOM refresh adopted, `phenotype-error-core` references removed, hygiene round-10.
- Open security work: PR #25 closed protobuf MED CVE this session; one residual MED CVE remains per Snyk scan. PhenoObservability pairs with the Snyk Phase 1 deploy stream.
- helix-tracing is already archive-tier; tracely lives in PhenoObservability as `tracely-core` + `tracely-sentinel`.
- Compute mesh (`reference_compute_mesh_state.md`) exposes OTel/observability surfaces for 5-of-6 providers; OCI + Cloudflare DNS pending — not blocking for this spec.

---

## Goals (delta vs. spec.md)

The original spec describes 8 standalone repos. The current reality is consolidation into `PhenoObservability/`. This plan reflects that and adds:

1. Close residual security debt (MED CVE, SBOM verification).
2. Lock the W3C Trace Context contract across the 13 crates.
3. Wire trace-id correlation through logging, metrics, and Sentinel.
4. Complete Phench / Profila / Tracera (PhenoKits) integration hooks.
5. Document archival of helix-tracing and any duplicate crates.
6. Stand up unified dashboard + alerting using existing OSS backends.

Non-goals: new observability backend, custom visualization, observability for archived repos.

---

## Phased WBS

### Phase 1 — Discovery
Verify current state, lock contracts, finalize crate inventory.

- WP-000 Inventory & freshness audit of `PhenoObservability/crates/*`
- WP-001 W3C Trace Context contract finalization (write contract doc; no code)
- WP-002 Residual CVE + SBOM verification (Snyk + cargo-deny advisory pass)

### Phase 2 — Design
Author interface specs and labeling/correlation contracts.

- WP-003 Unified labeling scheme spec (`service`, `env`, `version`, `trace_id`)
- WP-004 Log <-> Trace <-> Metric correlation spec (field names, propagation rules)
- WP-005 Profile-to-span attachment contract (Profila / Tracera output schema)

### Phase 3 — Build
Implementation handoff packages (planner produces specs; implementer agents do code).

- WP-006 `tracely-core` W3C propagation handoff (HTTP / gRPC / async queue)
- WP-007 `phenotype-observably-logging` trace-id injection handoff
- WP-008 metrics correlation handoff (`tracely-sentinel` + observably-sentinel)
- WP-009 Phench reporting + observability emission handoff
- WP-010 Profila/Tracera profiling integration handoff

### Phase 4 — Test / Validate
Acceptance verification batches.

- WP-011 W3C conformance test plan (against W3C trace-context test suite)
- WP-012 Trace<->log<->metric correlation integration test plan
- WP-013 Performance overhead benchmark plan (sampling on/off deltas)
- WP-014 Quality gate sweep (`cargo clippy -- -D warnings`, `cargo test --workspace`, `cargo fmt --check`, cargo-deny)

### Phase 5 — Deploy / Handoff
Dashboard, alerting, archival, runbooks.

- WP-015 Unified dashboard + alert rule spec (Grafana JSON / Prometheus rules — referenced as files, not authored here)
- WP-016 helix-tracing archival rationale + migration map
- WP-017 Runbook set in `repos/PhenoObservability/docs/runbooks/`
- WP-018 Cross-repo reference sweep (consumers updated to PhenoObservability)

---

## DAG (Phase | Task ID | Description | Depends On)

| Phase | Task ID | Description | Depends On |
|-------|---------|-------------|------------|
| 1 | WP-000 | Inventory + freshness audit | — |
| 1 | WP-001 | W3C Trace Context contract doc | WP-000 |
| 1 | WP-002 | Residual CVE + SBOM verification | WP-000 |
| 2 | WP-003 | Unified labeling scheme spec | WP-001 |
| 2 | WP-004 | Log/trace/metric correlation spec | WP-001, WP-003 |
| 2 | WP-005 | Profile-to-span contract | WP-001 |
| 3 | WP-006 | tracely-core propagation handoff | WP-001, WP-004 |
| 3 | WP-007 | observably-logging trace-id injection handoff | WP-004 |
| 3 | WP-008 | metrics correlation handoff | WP-003, WP-004 |
| 3 | WP-009 | Phench reporting + emission handoff | WP-003 |
| 3 | WP-010 | Profila/Tracera profiling handoff | WP-005 |
| 4 | WP-011 | W3C conformance test plan | WP-006 |
| 4 | WP-012 | Correlation integration test plan | WP-006, WP-007, WP-008 |
| 4 | WP-013 | Overhead benchmark plan | WP-006, WP-008, WP-009 |
| 4 | WP-014 | Quality gate sweep | WP-006..WP-010 |
| 5 | WP-015 | Dashboard + alert rule spec | WP-012 |
| 5 | WP-016 | helix-tracing archival rationale | WP-006, WP-007 |
| 5 | WP-017 | Runbooks | WP-015 |
| 5 | WP-018 | Cross-repo reference sweep | WP-016 |

**Critical path:** WP-000 -> WP-001 -> WP-004 -> WP-006 -> WP-012 -> WP-015 -> WP-017 (7 nodes).

---

## Agent-Time Estimates

All numbers are wall-clock minutes assuming an agent-driven environment. No human checkpoints.

| Phase | Pattern | Estimate |
|-------|---------|----------|
| Phase 1 | 3 parallel explore subagents | ~5 min |
| Phase 2 | 1 planner subagent batch (3 specs sequential) | ~6 min |
| Phase 3 | 5 parallel implementer subagents (WP-006..WP-010) | ~15 min |
| Phase 4 | 4 parallel test-author subagents + 1 quality-gate run | ~10 min |
| Phase 5 | 4 parallel doc/archival subagents | ~6 min |
| **Total** | | **~42 min** of agent wall-clock |

Per-WP rough mapping (per global "Timescales" rule):
- Small (3-6 tool calls, 1-3 min): WP-000, WP-002, WP-016, WP-017, WP-018
- Cross-stack (8-15 tool calls, 3-8 min): WP-001, WP-003, WP-004, WP-005, WP-006, WP-007, WP-008, WP-009, WP-010, WP-011, WP-012, WP-013, WP-014, WP-015

---

## Cross-Project Reuse Opportunities

Per Phenotype Org Cross-Project Reuse Protocol:

- W3C Trace Context contract should live in `phenotype-shared/` so consumers (cliproxyapi-plusplus, AgilePlus, heliosApp, thegent) link to a single canonical spec.
- Labeling scheme + correlation field names should be published as a markdown contract under `phenotype-shared/docs/observability/`.
- Snyk Phase 1 deploy + cargo-deny advisory pipeline should be templated and reused via `repos/.github-shared/` for all observability consumers.

---

## Dependencies on Other Specs

- `013-phenotype-infrakit-stabilization` — infrastructure crates (config-core, error-core consolidation already underway).
- `007-thegent-completion` — thegent integration surfaces.
- `004-modules-and-cycles` — module organization conventions.

No spec must complete before this one starts; all are concurrent.

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope drift from "8 repos" wording in spec.md vs. consolidated PhenoObservability reality | Med | This plan codifies the consolidated layout; spec.md to be amended via a follow-up edit |
| W3C compliance regression after refactor | Med | WP-011 mandates W3C trace-context conformance suite |
| SHM / observably-sentinel hot-path overhead | High | WP-013 benchmark plan with sampling on/off deltas, fail-closed budget |
| Residual MED CVE not closed before deploy | Med | WP-002 gates Phase 3 entry |
| Cross-repo consumers still importing legacy helix-tracing | Low | WP-018 grep sweep across active repos |

---

## Handoff

Phase outputs feed `tasks.md` (work-package detail). Implementer agents read tasks.md and produce code in topic worktrees under `repos/PhenoObservability-wtrees/<topic>/`. Quality gates run via `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` + `cargo fmt --check` + `cargo deny check advisories`. No human review checkpoints; review is agent-driven via `/review` and `/security-review`.
