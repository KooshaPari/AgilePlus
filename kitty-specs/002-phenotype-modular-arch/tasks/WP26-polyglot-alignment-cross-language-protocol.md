---
work_package_id: WP26
title: 'Polyglot Alignment + Cross-Language Protocol'
lane: planned
dependencies: [WP01, WP07, WP09]
subtasks: [T078, T079, T080]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP26: Polyglot Alignment + Cross-Language Protocol

**Implementation command**: `spec-kitty implement WP26 --base WP01,WP07,WP09`

## Objective

Align cross-language implementations that duplicate the same logic in different languages, publish missing language SDK bindings, and define cross-language consumption interfaces for governance and routing.

## Context

- PTY tracking is implemented twice: `agentapi++/lib/screentracker/` (Go) and `portage/terminus_2/` (Python/tmux) — no shared protocol format
- Python cliproxy SDK is not published; thegent uses `cliproxy_adapter.py` (1,253L) reimplementing Go SDK logic in Python
- thegent `governance/` (13,252L) has no cross-language consumption interface — WP09 adopted OPA but did not define how non-Python consumers invoke policy checks
- AgentBifrost routing logic exists in both `agentapi++/internal/routing/agent_bifrost.go` and `thegent/cliproxy_adapter.py` — should have canonical implementation in one language with thin client in the other

## Subtasks

### T078: PTY protocol alignment

**Steps**:
1. Define a shared PTY session protocol format (protobuf or JSON schema) covering: session start/end, keystroke events, screen state snapshots, command detection
2. Update `agentapi++/lib/screentracker/` (Go) to emit/consume the shared format
3. Update `portage/terminus_2/` (Python) to emit/consume the shared format
4. Add integration tests verifying both implementations produce compatible output for the same PTY session
5. Document the protocol in `phenotype-proto` or a dedicated spec file

**Validation**:
- Shared PTY protocol schema exists and is versioned
- Both Go and Python implementations pass cross-language compatibility tests
- Protocol format is documented with examples

### T079: Python cliproxy SDK + governance PolicyPort

**Steps**:
1. Publish `phenotype-py-cliproxy` Python SDK that consumes the same auth contracts as the Go SDK
2. Migrate thegent `cliproxy_adapter.py` (1,253L) to use the published SDK instead of reimplementing Go SDK logic
3. Define `PolicyPort` interface in thegent `governance/` that is consumable cross-language:
   - Option A: OPA Rego policies served via OPA REST API (language-agnostic)
   - Option B: gRPC service wrapping governance logic (generated clients in Go, Rust, TS)
4. Document the chosen cross-language governance consumption pattern

**Validation**:
- `phenotype-py-cliproxy` published and installable via pip
- thegent `cliproxy_adapter.py` reduced to <200 LOC (thin SDK consumer)
- At least one non-Python consumer can invoke policy checks via the defined interface
- All thegent tests pass after migration

### T080: Routing alignment

**Steps**:
1. Identify canonical routing implementation between `agentapi++/internal/routing/agent_bifrost.go` and `thegent/cliproxy_adapter.py`
2. Designate one as the canonical source of truth (likely Go, given Bifrost is Go-native)
3. Reduce the non-canonical implementation to a thin client calling the canonical one (via gRPC, REST, or SDK)
4. Ensure routing decisions are consistent across both entry points
5. Add cross-language routing integration tests

**Validation**:
- Only one codebase contains routing decision logic
- The other codebase is a thin client (<300 LOC)
- Cross-language routing tests pass
- No routing decision duplication between Go and Python

## Definition of Done

- [ ] Shared PTY protocol format defined and implemented in both languages
- [ ] `phenotype-py-cliproxy` published and consumed by thegent
- [ ] Cross-language PolicyPort interface defined and at least one non-Python consumer exists
- [ ] Canonical routing implementation designated; non-canonical reduced to thin client
- [ ] All test suites pass across affected repos

## Risks

- **Protocol format design is a long-lived decision**: PTY protocol format will be consumed by multiple repos for years. Invest in forward-compatible schema design (protobuf with reserved fields).
- **OPA REST API adds latency to policy checks**: If governance checks are on the hot path, the REST overhead may be unacceptable. Profile and consider embedded OPA (Go) or compiled Rego for latency-sensitive paths.
- **Routing canonical designation may be contentious**: Both implementations may have features the other lacks. Do a thorough feature matrix comparison before designating canonical.

## Reviewer Guidance

- Verify the PTY protocol schema is forward-compatible (reserved fields, versioning)
- Check that the Python cliproxy SDK has feature parity with the Go SDK
- Confirm the PolicyPort interface is truly language-agnostic (no Python-specific constructs in the contract)
- Verify routing thin client faithfully delegates all decisions to canonical implementation
