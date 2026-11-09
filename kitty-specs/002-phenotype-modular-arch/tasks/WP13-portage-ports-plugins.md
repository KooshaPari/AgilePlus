---
work_package_id: WP13
title: 'portage: Ports, Plugins, Shared Consumption'
lane: planned
dependencies: []
subtasks: [T040, T042, T043, T044]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
- date: '2026-03-03'
  event: revised
  by: external-library-leverage research integration
---

# WP13: portage — Ports, Plugins, Shared Consumption

**Implementation command**: `spec-kitty implement WP13 --base WP10`

## Objective

Formalize portage's extension points as `typing.Protocol` ports. Implement benchmark adapter plugin registry (Tier 1 via entry_points, Tier 2 via Extism). Consume `phenotype-py-infra` for shared utilities.

## Context

- portage has ~40 benchmark adapters with informal patterns
- Shares infrastructure needs with thegent (file ops, subprocess, cache)
- phenotype-py-infra published in WP10
- See plan.md Lane 5

## Subtasks

### T040: Formalize ports via typing.Protocol (Inspect AI as framework)

**Steps**:
1. Define port protocols in `portage/ports/`, aligning with Inspect AI primitives:
   ```python
   class ExecutorPort(Protocol):
       def run_trial(self, config: TrialConfig) -> TrialResult: ...

   class ReporterPort(Protocol):
       def report(self, results: list[TrialResult]) -> Report: ...

   class LoaderPort(Protocol):
       def load_benchmark(self, path: str) -> BenchmarkSpec: ...
   ```
2. **Inspect AI adoption**: `UKGovernmentBEIS/inspect_ai` provides the evaluation framework that replaces the custom BaseAgent, task harness, and ~40 benchmark adapters (~15-25K LOC saved). Port protocols should wrap Inspect AI primitives (Task, Solver, Scorer, Dataset). New benchmarks are authored as Inspect AI Task definitions rather than custom adapter classes.
3. Retain custom environment drivers (Modal, Runloop, Daytona, E2B, GKE) — Inspect AI is Docker-first, so these remain custom.
4. Ensure existing implementations conform to port protocols
5. Add pyright strict mode on port modules

**Implementation guidance (library research)**:
- Inspect AI is Phase C (high complexity, transformative). This is the single highest-leverage adoption for portage.
- The fundamental shift: instead of building 40 custom adapters, port benchmarks as Inspect AI Task definitions that use Inspect's solver/scorer/dataset primitives.
- Inspect AI handles: model interaction, tool use, scoring, logging, and dataset management out of the box.

**Validation**: Type checking passes on all port implementations

### T042: Tier 1 — Inspect AI Task adapter registry

**Steps**:
1. Add entry_points for each adapter in pyproject.toml
2. Create registry discovering adapters via `importlib.metadata`
3. Replace hardcoded adapter list with registry discovery
4. **Key shift**: The "benchmark adapter registry" becomes an "Inspect AI Task adapter registry". Each entry_point registers an Inspect AI Task definition (not a custom adapter class). The registry discovers and loads Task definitions that wrap Inspect primitives.

**Implementation guidance (library research)**:
- With Inspect AI adoption (T040), the adapter surface area shrinks dramatically — each adapter is a thin Task definition rather than a full custom class.
- **Dramatiq or arq** can be added as a lightweight job queue for batch benchmark execution (~0.5-0.7K LOC saved vs custom queue logic). Phase A, low complexity.

**Validation**: All adapters discoverable via entry_points as Inspect AI Task definitions

### T043: Tier 2 — Extism host for user adapters

**Steps**:
1. Add extism Python SDK
2. Create plugin host loading .wasm adapters from `~/.portage/plugins/`
3. Validate against ExecutorPort contract
4. Sandbox constraints: memory, timeout, no fs

**Validation**: Test WASM adapter loads and executes

### T044: Consume phenotype-py-infra (+ shared LiteLLM proxy)

**Steps**:
1. Add `phenotype-py-infra` dependency to portage
2. Replace local utility code with shared imports (subprocess, file ops, cache)
3. Remove duplicated utility code
4. **Shared LiteLLM proxy**: Point portage at the shared `BerriAI/litellm` proxy configured in WP10/T028, eliminating portage's ~700 LOC of duplicate LiteLLM wrappers.

**Validation**: `pytest` passes with shared dependency

## Definition of Done

- [ ] typing.Protocol ports formalized for executor, reporter, loader
- [ ] Entry_points registry replaces hardcoded adapter discovery
- [ ] Extism host prototype for .wasm adapters
- [ ] phenotype-py-infra consumed, local duplicates removed
- [ ] All tests pass

## Reviewer Guidance

- Verify port protocols are minimal (not over-specified)
- Check adapter migration didn't break any benchmark workflows
