# Work Packages: Observability Stack Completion

**Inputs:** `spec.md`, `plan.md` in this directory.
**Prerequisites:** Rust toolchain, OpenTelemetry knowledge, Snyk + cargo-deny installed.
**Primary scope:** `repos/PhenoObservability/` (13-crate Rust workspace).
**Secondary scope:** consumer repos (AgilePlus, heliosApp, thegent, cliproxyapi-plusplus) for reference sweep.

> Per global governance: planner WPs equip implementers. No code in this document. Acceptance criteria + file scope + dependencies + handoff prompts only.

---

## Phase 1 — Discovery

### WP-000 — Inventory & Freshness Audit

- **State:** planned
- **Phase:** 1 (Discovery)
- **Depends on:** —
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (read-only):**
  - `repos/PhenoObservability/crates/*`
  - `repos/PhenoObservability/CHANGELOG.md`
  - `repos/PhenoObservability/Cargo.toml`
  - `repos/AgilePlus/kitty-specs/014-observability-stack-completion/spec.md`
- **File scope (write):** `kitty-specs/014-observability-stack-completion/research/inventory.md`
- **Acceptance criteria:**
  - Table of all 13 crates: name, version, last-touched commit, role (tracing/logging/metrics/sentinel/macros/storage), public surface summary.
  - Delta vs. spec.md "8 repos" list documented (which crates absorbed which legacy repo).
  - List of duplicate or orphaned crates flagged for archival.
- **Handoff prompt:** "Audit `PhenoObservability/crates/`; produce inventory.md per WP-000 acceptance."

---

### WP-001 — W3C Trace Context Contract Doc

- **State:** planned
- **Phase:** 1 (Discovery)
- **Depends on:** WP-000
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `kitty-specs/014-observability-stack-completion/research/w3c-contract.md`
- **Acceptance criteria:**
  - References W3C Trace Context spec (level 1 + level 2) by section number.
  - Defines `traceparent` and `tracestate` parsing/emission requirements.
  - Lists protocol adapters in scope: HTTP (axum, reqwest, hyper), gRPC (tonic metadata), async queues (NATS, Kafka).
  - Specifies sampling decision propagation rules.
  - Brief pseudocode (3-5 lines) showing extract/inject signature shape — no implementation.
- **Handoff prompt:** "Author w3c-contract.md per WP-001 acceptance using PhenoObservability inventory."

---

### WP-002 — Residual CVE + SBOM Verification

- **State:** planned
- **Phase:** 1 (Discovery)
- **Depends on:** WP-000
- **Effort:** Small (3-6 tool calls, ~3 min)
- **File scope (read):** `repos/PhenoObservability/Cargo.lock`, `repos/PhenoObservability/deny.toml`, recent SBOM under `repos/PhenoObservability/.sbom/` if present.
- **File scope (write):** `kitty-specs/014-observability-stack-completion/research/cve-status.md`
- **Acceptance criteria:**
  - List of all open CVEs (HIGH/CRIT/MED) with crate + version + advisory ID.
  - Confirmation PR #25 protobuf MED CVE is closed.
  - Action plan for residual MED CVE (upgrade path or accept-with-justification).
  - `cargo deny check advisories` exit code recorded.
- **Handoff prompt:** "Run `cargo deny check advisories` and Snyk scan on PhenoObservability; produce cve-status.md."

---

## Phase 2 — Design

### WP-003 — Unified Labeling Scheme Spec

- **State:** planned
- **Phase:** 2 (Design)
- **Depends on:** WP-001
- **Effort:** Cross-stack (8-15 tool calls, ~4 min)
- **File scope (write):** `phenotype-shared/docs/observability/labeling-scheme.md` (cross-repo reuse target).
- **Acceptance criteria:**
  - Required labels: `service`, `env`, `version`, `trace_id`, `span_id`.
  - Optional labels: `tenant`, `region`, `instance_id`.
  - Cardinality budget per label documented.
  - Forbidden label patterns (high-cardinality user IDs, free-text).
  - Conformance check: deterministic label set per metric family.
- **Handoff prompt:** "Author labeling-scheme.md per WP-003 acceptance; place in phenotype-shared for cross-repo reuse."

---

### WP-004 — Log/Trace/Metric Correlation Spec

- **State:** planned
- **Phase:** 2 (Design)
- **Depends on:** WP-001, WP-003
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `phenotype-shared/docs/observability/correlation-spec.md`
- **Acceptance criteria:**
  - JSON log field names: `timestamp`, `level`, `service`, `trace_id`, `span_id`, `message`, `fields`.
  - Metric exemplar format (OTLP exemplar with `trace_id` + `span_id`).
  - Span attribute conventions (semconv aligned).
  - Propagation rules across async boundaries documented.
  - Failure mode: missing trace_id -> log without it + warn-once metric.
- **Handoff prompt:** "Author correlation-spec.md per WP-004 acceptance referencing labeling-scheme.md."

---

### WP-005 — Profile-to-Span Attachment Contract

- **State:** planned
- **Phase:** 2 (Design)
- **Depends on:** WP-001
- **Effort:** Cross-stack (8-15 tool calls, ~4 min)
- **File scope (write):** `kitty-specs/014-observability-stack-completion/research/profile-span-contract.md`
- **Acceptance criteria:**
  - Defines pprof + flamegraph output schema.
  - Span attribute name for attached profile artifact (e.g. `phenotype.profile.uri`).
  - Rules for sampling (do not profile every span; budget-bound).
  - Tracera vs. Profila role split (final state).
- **Handoff prompt:** "Author profile-span-contract.md per WP-005 acceptance."

---

## Phase 3 — Build (Handoff Packages)

### WP-006 — `tracely-core` W3C Propagation Handoff

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-001, WP-004
- **Effort:** Cross-stack (8-15 tool calls, ~6 min planning; implementation in worktree)
- **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-006-tracely-w3c/`
- **File scope (target):** `repos/PhenoObservability/crates/tracely-core/src/`
- **Acceptance criteria for implementer agent:**
  - W3C `traceparent` + `tracestate` parse/emit functions in tracely-core.
  - HTTP middleware (axum + reqwest) for inject/extract.
  - gRPC tonic metadata adapter.
  - Async queue helper (generic, NATS-first).
  - Sampling strategies: always-on, probability, parent-based.
  - >= 80% line coverage on tracing core.
  - `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean.
- **Handoff prompt:** "Implement W3C propagation in tracely-core per WP-006 acceptance; reference research/w3c-contract.md."

---

### WP-007 — `phenotype-observably-logging` Trace-ID Injection Handoff

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-004
- **Effort:** Cross-stack (8-15 tool calls, ~5 min planning)
- **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-007-logging-traceid/`
- **File scope (target):** `repos/PhenoObservability/crates/phenotype-observably-logging/src/`, `repos/PhenoObservability/crates/helix-logging/src/` (if duplicate, merge).
- **Acceptance criteria:**
  - JSON log formatter emitting all fields per correlation-spec.md.
  - tracing-subscriber layer that pulls current span's trace_id/span_id.
  - Configurable per-module log levels via env or config.
  - Output targets: stdout, file, OTLP log exporter.
  - Graceful degradation when no span is active.
  - >= 80% coverage; clippy + fmt clean.
- **Handoff prompt:** "Implement trace-id injection in phenotype-observably-logging per WP-007."

---

### WP-008 — Metrics Correlation Handoff

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-003, WP-004
- **Effort:** Cross-stack (8-15 tool calls, ~5 min planning)
- **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-008-metrics-corr/`
- **File scope (target):** `repos/PhenoObservability/crates/tracely-sentinel/src/`, `repos/PhenoObservability/crates/phenotype-observably-sentinel/src/`.
- **Acceptance criteria:**
  - OTLP metric exporter emits exemplars with trace_id + span_id.
  - Label set conforms to labeling-scheme.md.
  - Cardinality enforcement at registration (reject metrics exceeding budget).
  - Integration test: trace -> metric exemplar present.
  - SHM hot-path benchmark recorded (regression budget: <= +5% vs. baseline).
  - clippy + fmt clean.
- **Handoff prompt:** "Implement metrics correlation per WP-008; honor labeling + correlation specs."

---

### WP-009 — Phench Reporting + Observability Emission Handoff

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-003
- **Effort:** Cross-stack (8-15 tool calls, ~5 min planning)
- **Implementation worktree:** `repos/Phench-wtrees/wp-009-obs-emission/` (or PhenoObservability if Phench already absorbed).
- **File scope (target):** Phench source tree (`src/`, `tests/`).
- **Acceptance criteria:**
  - Statistical analysis: mean, stddev, p50/p95/p99, confidence intervals.
  - Exporters: JSON, CSV, Markdown.
  - Warmup + iteration + cooldown phases.
  - Observability emission hook: benchmark results emitted as OTLP metrics with labeling-scheme labels.
  - >= 80% coverage; clippy + fmt clean.
- **Handoff prompt:** "Complete Phench reporting + emission per WP-009."

---

### WP-010 — Profila / Tracera Profiling Integration Handoff

- **State:** planned
- **Phase:** 3 (Build)
- **Depends on:** WP-005
- **Effort:** Cross-stack (8-15 tool calls, ~5 min planning)
- **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-010-profile-integration/`
- **File scope (target):** Profila source tree, Tracera source tree under PhenoKits.
- **Acceptance criteria:**
  - CPU profiling with pprof-compatible output.
  - Memory/allocation profiling.
  - Span attachment per profile-span-contract.md.
  - Flamegraph generation utility.
  - macOS + Linux platform tests.
  - Tracera <-> Profila role split documented; redundant code merged.
- **Handoff prompt:** "Integrate Profila/Tracera per WP-010; attach profiles to spans."

---

## Phase 4 — Test / Validate

### WP-011 — W3C Conformance Test Plan

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-006
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `repos/PhenoObservability/tests/w3c_conformance/README.md` (test plan, not code).
- **Acceptance criteria:**
  - Test matrix vs. W3C trace-context test suite.
  - Pass criteria: 100% of MUST tests; documented exclusions for SHOULD/MAY.
  - Reference to upstream conformance harness.
- **Handoff prompt:** "Author W3C conformance test plan per WP-011."

---

### WP-012 — Correlation Integration Test Plan

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-006, WP-007, WP-008
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `repos/PhenoObservability/tests/correlation/README.md`
- **Acceptance criteria:**
  - Scenarios: HTTP request -> log + metric + span all carry same trace_id.
  - Async boundary preservation.
  - Missing-context fallback path.
  - Multi-hop propagation (3 services).
- **Handoff prompt:** "Author correlation integration test plan per WP-012."

---

### WP-013 — Performance Overhead Benchmark Plan

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-006, WP-008, WP-009
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `repos/PhenoObservability/benches/README.md`
- **Acceptance criteria:**
  - Baseline (no observability) vs. sampled vs. always-on deltas.
  - Hot-path budget: <= 5% throughput regression at p99.
  - SHM read/write latency targets documented.
  - Reproducibility: bench command + environment fingerprint.
- **Handoff prompt:** "Author overhead benchmark plan per WP-013."

---

### WP-014 — Quality Gate Sweep

- **State:** planned
- **Phase:** 4 (Test/Validate)
- **Depends on:** WP-006, WP-007, WP-008, WP-009, WP-010
- **Effort:** Cross-stack (8-15 tool calls, ~6 min)
- **File scope (write):** `kitty-specs/014-observability-stack-completion/research/quality-gate-report.md`
- **Acceptance criteria:**
  - `cargo test --workspace` exit 0.
  - `cargo clippy --workspace -- -D warnings` exit 0.
  - `cargo fmt --check` exit 0.
  - `cargo deny check advisories` exit 0 (or documented accept-with-justification).
  - SBOM regenerated.
- **Handoff prompt:** "Run all quality gates on PhenoObservability; record results in quality-gate-report.md."

---

## Phase 5 — Deploy / Handoff

### WP-015 — Unified Dashboard + Alert Rule Spec

- **State:** planned
- **Phase:** 5 (Deploy/Handoff)
- **Depends on:** WP-012
- **Effort:** Cross-stack (8-15 tool calls, ~5 min)
- **File scope (write):** `repos/PhenoObservability/dashboards/README.md`, `repos/PhenoObservability/alerting/README.md`.
- **Acceptance criteria:**
  - Dashboard inventory: which panels exist, what they query.
  - Alert rule inventory: SLO-based, error-rate, latency p99, saturation.
  - References to existing OSS backends (Grafana, Prometheus, Loki, Tempo).
  - No new visualization code authored in this WP — references file paths to dashboard JSON / Prometheus rule YAML to be produced by implementer.
- **Handoff prompt:** "Author dashboard + alert spec per WP-015."

---

### WP-016 — helix-tracing Archival Rationale

- **State:** planned
- **Phase:** 5 (Deploy/Handoff)
- **Depends on:** WP-006, WP-007
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (write):** `repos/PhenoObservability/docs/archival/helix-tracing.md`.
- **Acceptance criteria:**
  - Why archived (functionality absorbed by tracely-core).
  - Migration map: helix-tracing symbol -> tracely-core symbol.
  - GitHub archive action recorded.
- **Handoff prompt:** "Author helix-tracing archival doc per WP-016 and archive the repo."

---

### WP-017 — Runbooks

- **State:** planned
- **Phase:** 5 (Deploy/Handoff)
- **Depends on:** WP-015
- **Effort:** Small (3-6 tool calls, ~3 min)
- **File scope (write):** `repos/PhenoObservability/docs/runbooks/`
  - `trace-correlation-missing.md`
  - `metric-cardinality-explosion.md`
  - `sampler-config.md`
  - `profile-attach-failure.md`
- **Acceptance criteria:**
  - Each runbook: symptom, diagnosis steps, remediation, escalation path.
  - Linked from `repos/PhenoObservability/docs/README.md`.
- **Handoff prompt:** "Author 4 runbooks per WP-017."

---

### WP-018 — Cross-Repo Reference Sweep

- **State:** planned
- **Phase:** 5 (Deploy/Handoff)
- **Depends on:** WP-016
- **Effort:** Small (3-6 tool calls, ~3 min)
- **File scope (read):** all active repos under `repos/`.
- **File scope (write):** `kitty-specs/014-observability-stack-completion/research/reference-sweep.md`
- **Acceptance criteria:**
  - Grep all active repos for `helix-tracing`, legacy tracely paths, and removed crates.
  - Open one PR per consumer repo (or one stacked PR) updating imports.
  - Verify no broken builds post-update.
- **Handoff prompt:** "Sweep all active repos for legacy observability references per WP-018."

---

## Dependency & Execution Summary

```
Phase 1 (Discovery):   WP-000 -> { WP-001, WP-002 }
Phase 2 (Design):      WP-001 -> { WP-003, WP-005 }; WP-003 -> WP-004
Phase 3 (Build):       WP-006 (needs WP-001+WP-004); WP-007 (WP-004); WP-008 (WP-003+WP-004); WP-009 (WP-003); WP-010 (WP-005)
Phase 4 (Validate):    WP-011 (WP-006); WP-012 (WP-006+007+008); WP-013 (WP-006+008+009); WP-014 (all build WPs)
Phase 5 (Deploy):      WP-015 (WP-012); WP-016 (WP-006+007); WP-017 (WP-015); WP-018 (WP-016)
```

**Critical path (7 nodes):** WP-000 -> WP-001 -> WP-004 -> WP-006 -> WP-012 -> WP-015 -> WP-017

**Parallelization opportunities:**
- Phase 3 build WPs (006, 007, 008, 009, 010) can dispatch as 5 concurrent implementer agents after Phase 2 closes.
- Phase 4 test plans (011, 012, 013) can author in parallel after their respective build WPs land.

**Total WP count:** 19 (WP-000 through WP-018).
