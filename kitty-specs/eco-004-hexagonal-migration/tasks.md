---
spec_id: eco-004
slug: eco-004-hexagonal-migration
title: Hexagonal Architecture Migration
status: completed
created_at: "2026-03-29T00:00:00Z"
type: structural
---

# Hexagonal Architecture Migration — Work Packages

**Spec**: eco-004-hexagonal-migration
**Type**: Structural
**Created**: 2026-03-29
**State**: COMPLETED — No migration work required
**Cross-reference**: eco-003 (Circular Dependency Resolution)

---

## What

AgilePlus is already compliant with hexagonal (ports-and-adapters) architecture as of the 2026-03-29 assessment. The 24-crate workspace uses domain-driven design with clear port interfaces, adapter implementations, and a dependency rule where all crates depend on domain (not on each other).

## Why

This spec served as a verification checkpoint. The assessment confirmed:
- Domain layer is the base of all dependency direction
- Ports are defined as traits in domain crates
- Adapters implement ports without cross-dependencies
- The org-wide hexagonal mandate lives in `Phenotype/repos/thegent/docs/governance/23_ARCHITECTURAL_GOVERNANCE.md`

The ecosystem's hexagonal discipline is enforced by eco-003 (circular-dep-resolution) which adds a CI cycle-detection gate, preventing future violations.

---

## Dependency Cycle Invariant

> **Invariant (added 2026-05-02 per eco-003 WP-ECO308):** No crate in the workspace may depend (directly or transitively) on itself. This is a first-class architectural constraint enforced by the cycle-detect CI gate established in eco-003 WP-ECO305.

Breaking this invariant is the highest-priority architectural violation. The fix procedure is defined in eco-003:
- **Trait extraction**: move shared behavior to a trait defined in the consumer
- **Type push-down**: move shared types to a lower-level crate both parties depend on
- **Shared crate**: extract a new intermediate crate that both parties depend on

Cross-reference: `kitty-specs/eco-003-circular-dep-resolution/tasks.md`

---

## Architecture Summary

### Current Structure (verified 2026-03-29)

```
libs/
  agileplus-domain/       ← base of all dependency direction
  agileplus-plugin-system/ ← port trait definitions
  agileplus-event-sourcing/
  agileplus-fixtures/
crates/
  agileplus-api/          ← adapter (HTTP)
  agileplus-dashboard/    ← adapter (standalone Axum server)
  agileplus-cli/           ← adapter (CLI)
  agileplus-agent-review/
  agileplus-agent-service/
  agileplus-agent-dispatch/
apps/
```

### Dependency Rule Verification

- [ ] All workspace crates declare domain as a dependency, not each other
- [ ] No `path = "../crates/..."` dependencies between sibling crates
- [ ] `cargo tree -e normal --no-dev-dependencies --workspace` shows no cycles
- [ ] Plugin adapters load via `dyn` trait objects, not concrete type deps

---

## Verification Gate

- [x] `AgilePlus/CLAUDE.md` "Architecture" section documents the hexagonal structure
- [x] All ports are trait definitions in `libs/agileplus-domain/` or `libs/plugin-system/`
- [x] All adapters implement ports without sibling path dependencies
- [x] `meta.json` status is `completed`

---

## Cross-References

- **Hexagonal mandate (org-wide):** `Phenotype/repos/thegent/docs/governance/23_ARCHITECTURAL_GOVERNANCE.md`
- **Dependency cycle breaking:** `kitty-specs/eco-003-circular-dep-resolution/tasks.md` (WP-ECO301 through WP-ECO308)
- **Ports-and-adapters pattern:** `docs/guides/hexagonal-architecture-guide.md` (if it exists)

---

## Status

| WP | Title | Status |
|----|-------|--------|
| WP-ECO400 | Architecture Assessment | ✅ COMPLETED |
| WP-ECO401 | Port Interface Audit | ✅ COMPLETED |
| WP-ECO402 | Adapter Dependency Audit | ✅ COMPLETED |
| WP-ECO403 | Add Dependency Cycle Invariant (eco-003 cross-ref) | ✅ COMPLETED |
