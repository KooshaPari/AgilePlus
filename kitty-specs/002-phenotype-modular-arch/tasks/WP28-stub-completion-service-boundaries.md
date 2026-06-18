---
work_package_id: WP28
title: 'Architectural Stub Completion + Service Boundaries'
lane: planned
dependencies: [WP09, WP11, WP14, WP16]
subtasks: [T083, T084, T085]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP28: Architectural Stub Completion + Service Boundaries

**Implementation command**: `spec-kitty implement WP28 --base WP09,WP11,WP14,WP16`

## Objective

Complete architectural stubs that represent intended product direction, formalize unclear service boundaries, and migrate remaining consumers to shared infrastructure (LiteLLM proxy).

## Context

- `agentapi++/internal/phenotype/` exists as an empty `init.go` stub — intended as the formal isolation boundary for Phenotype-specific extensions but never completed
- trace `backend/internal/plugin/` anticipated a plugin registry but was never implemented
- heliosApp `sessions/` and `workspace/` services are embedded in the same runtime as provider routing — should be separate services
- thegent `governance/` (13K LOC) sits directly in the call path with no port abstraction — needs `PolicyEngine` port for hot-swapping backends (OPA, Guardrails AI, custom)
- portage `environments/` has 8 environment plugins with clean `BaseEnvironment` ABC — ready to formalize as entry_points plugin system and extract
- WP10 set up LiteLLM shared proxy for thegent, but portage `lite_llm.py` (727L) was not migrated (not addressed in WP13)

## Subtasks

### T083: agentapi++ phenotype boundary + trace plugin system

**Steps**:
1. Complete `agentapi++/internal/phenotype/` as the formal isolation boundary:
   - Define `PhenotypeExtension` interface listing all Phenotype-specific hooks (routing overrides, audit emission, auth integration)
   - Implement the interface with concrete Phenotype extensions
   - Ensure all Phenotype-specific code in agentapi++ routes through this boundary
2. Implement trace `backend/internal/plugin/` plugin registry:
   - Define `PluginSource` interface for runtime data source addition
   - Support plugin types: Neo4j, Postgres, Redis, NATS
   - Implement plugin discovery and lifecycle management (init, health check, shutdown)
   - Migrate existing hardcoded data sources to use the plugin interface

**Validation**:
- `agentapi++/internal/phenotype/` has non-empty implementation with clear interface
- All Phenotype-specific code routes through the phenotype boundary (no Phenotype logic in generic agentapi code)
- trace plugin registry supports adding data sources at runtime
- `go build ./...` and `go test ./...` pass for both repos

### T084: heliosApp service separation

**Steps**:
1. Separate `sessions/` service from provider routing runtime:
   - Define session service interface (create, resume, list, delete)
   - Extract session state management into standalone service module
   - Communicate with provider routing via protocol bus (IPC/events)
2. Separate `workspace/` service similarly:
   - Define workspace service interface
   - Extract workspace state and file management
   - Communicate via protocol bus
3. Document the new service boundaries and allowed communication patterns
4. Update Electron main process to bootstrap services independently

**Validation**:
- Sessions and workspace modules have no direct imports from provider routing
- Services communicate only via protocol bus events
- Service boundary documentation exists
- heliosApp builds and all tests pass

### T085: thegent PolicyPort + portage LiteLLM migration

**Steps**:
1. Refactor thegent `governance/` (13K LOC) to sit behind a `PolicyEngine` port interface:
   - Define `PolicyEngine` protocol with methods: `evaluate(policy_context) -> PolicyDecision`
   - Current governance logic becomes the default `OPABackedPolicyEngine` implementation
   - Interface enables hot-swapping to Guardrails AI, custom backends, or mock implementations
2. Migrate portage `lite_llm.py` (727L) to use the shared LiteLLM proxy established in WP10:
   - Replace direct LiteLLM library usage with shared proxy client calls
   - Remove portage-local LiteLLM configuration
   - Ensure all model routing goes through the shared proxy
3. Formalize portage `environments/` as `entry_points` plugin package:
   - Add `pyproject.toml` entry_points configuration for 8 environment plugins
   - Extract as `phenotype/harbor-environments` package
   - Source repo consumes via pip dependency

**Validation**:
- thegent governance accessible via `PolicyEngine` interface
- At least one alternative PolicyEngine implementation exists (mock or Guardrails AI stub)
- portage no longer has local LiteLLM configuration; all calls route through shared proxy
- portage environments discoverable via entry_points
- All test suites pass in thegent and portage

## Definition of Done

- [ ] agentapi++ phenotype boundary completed with concrete implementation
- [ ] trace plugin registry implemented with PluginSource interface
- [ ] heliosApp sessions/ and workspace/ separated from provider routing
- [ ] thegent governance behind PolicyEngine port interface
- [ ] portage LiteLLM migrated to shared proxy
- [ ] portage environments formalized as entry_points plugin and extracted
- [ ] All test suites pass across all affected repos

## Risks

- **heliosApp service separation may introduce race conditions**: Services communicating via protocol bus instead of direct calls need careful event ordering. Add integration tests for concurrent operations.
- **PolicyEngine abstraction may lose governance nuance**: 13K LOC of governance logic may have subtle behaviors that a generic interface cannot capture. Allow the interface to be rich enough (policy context objects, decision metadata).
- **portage LiteLLM migration depends on shared proxy availability**: Ensure WP10 shared proxy is deployed and accessible before migration.

## Reviewer Guidance

- Verify agentapi++ phenotype boundary is a true isolation layer (grep for Phenotype-specific imports outside the boundary)
- Check trace plugin interface supports all current data source types
- Confirm heliosApp service separation does not break existing IPC contracts
- Verify PolicyEngine interface is rich enough to express current governance decisions without loss
- Check portage LiteLLM migration does not change model routing behavior
