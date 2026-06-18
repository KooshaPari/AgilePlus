---
work_package_id: WP19
title: 'thegent: Migration Cleanup + God Function Decomposition'
lane: planned
dependencies: [WP09]
subtasks: [T056, T057, T058]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP19: thegent — Migration Cleanup + God Function Decomposition

**Implementation command**: `spec-kitty implement WP19 --base WP09`

## Objective

Complete the stalled thegent package migration by deleting 469 duplicate files, decompose the largest function in the entire ecosystem (run_impl_core at 1,022 lines), and reduce the 221-file circular dependency surface. This addresses the top three quality findings for thegent from the codebase quality analysis.

## Context

- thegent has ~160K Python LOC (unique, excluding duplicates)
- Package migration is 32% complete: 469 of 1,467 src/ files have been migrated to packages/ but the src/ copies were never deleted
- `run_impl_core` at 1,022 lines is the single largest function in the entire Phenotype ecosystem
- 221 files (15% of the codebase) use `TYPE_CHECKING` blocks to work around circular imports — a massive dependency health indicator
- `run_execution_core_helpers.py` exists in 3 locations with potential divergence
- WP09 (thegent package migration) must be in place before this cleanup begins

## Subtasks

### T056: Delete 469 duplicate files from src/thegent/

**Steps**:
1. Generate the definitive list of byte-identical files between `src/thegent/` and `packages/thegent-*/`:
   - Use `diff -rq src/thegent packages/` or equivalent checksum comparison
   - Catalog each duplicate pair: src path, packages path, byte count
2. Delete the `src/thegent/` copy for all 469 confirmed duplicates
3. Update all import statements across the codebase to point to `packages/` locations:
   - Search for `from thegent.` and `import thegent.` patterns
   - Update to the corresponding `packages/thegent-*` import path
4. Fix the tri-copy of `run_execution_core_helpers.py`:
   - Identify which of the 3 copies is canonical (most recent, most complete)
   - Delete the other 2 copies
   - Update all imports to point to the canonical location
5. Remove any now-empty directories under `src/thegent/`

**Validation**:
- `diff -rq src/thegent packages/` shows no remaining identical files
- Estimated LOC reduction: ~100K (duplicate elimination)
- All tests pass (`pytest` or equivalent test runner)
- No import errors at runtime — `python -c "import thegent"` succeeds
- `src/thegent/` directory either removed entirely or contains only non-migrated files with a tracking manifest

### T057: Decompose run_impl_core (1,022 lines)

**Steps**:
1. Analyze `run_impl_core` to identify logical segments:
   - Step execution logic
   - Context and state management
   - Result collection and aggregation
   - Error handling and recovery
2. Extract into focused modules:
   - `run_impl_core()` — orchestration only, <100 lines, calls into extracted modules
   - `step_executor.py` — individual step execution logic
   - `context_manager.py` — context accumulation, state tracking, window management
   - `result_aggregator.py` — result collection, formatting, output handling
3. Also decompose `bg_impl_core` (513 lines) in the same file:
   - Apply the same extraction pattern
   - Share common utilities with `run_impl_core` decomposition where applicable
4. Ensure all extracted functions have type annotations and docstrings

**Validation**:
- No function exceeds 200 lines in the decomposed modules
- `run_impl_core()` orchestration body is <100 lines
- All tests pass
- Type checking passes (`mypy` or `pyright`)
- No behavioral changes — identical output for identical inputs

### T058: Break circular dependency surface (221 TYPE_CHECKING files)

**Steps**:
1. Map the circular dependency graph:
   - Identify which modules import from each other via TYPE_CHECKING
   - Cluster by dependency group: cli<->agents, execution<->orchestration, governance<->core
   - Rank files by import count (most-imported files first)
2. Extract shared types into `packages/thegent-core/types/`:
   - Create type-only modules for shared data classes, protocols, and type aliases
   - Move type definitions that cause circular imports into the shared types package
   - Update TYPE_CHECKING imports to regular imports from the new shared location
3. Phase approach — fix top 50 files by import count first:
   - Prioritize files that are imported by the most other TYPE_CHECKING blocks
   - Each fix should eliminate TYPE_CHECKING usage in multiple downstream files
4. Document remaining legitimate TYPE_CHECKING uses:
   - Forward references within the same package
   - Genuinely optional type hints for rarely-used integrations

**Validation**:
- `grep -r 'TYPE_CHECKING' --include='*.py' | wc -l` shows <50 files (down from 221)
- All remaining TYPE_CHECKING uses have an inline comment explaining why they are necessary
- No new circular imports introduced — `import-linter` or equivalent passes
- All tests pass
- Type checking passes

## Definition of Done

- [ ] 469 duplicate files deleted from src/thegent/, ~100K LOC eliminated
- [ ] Tri-copy of run_execution_core_helpers.py resolved to single canonical location
- [ ] run_impl_core decomposed from 1,022 lines to <100 line orchestrator + extracted modules
- [ ] bg_impl_core (513 lines) similarly decomposed
- [ ] No function >200 lines in decomposed modules
- [ ] TYPE_CHECKING usage reduced from 221 files to <50
- [ ] Shared types extracted to packages/thegent-core/types/
- [ ] All tests pass, type checking clean, no import errors

## Risks

- **Diverged duplicates**: Some of the 469 "duplicates" may have diverged since migration. Byte-identical check mitigates this, but verify with checksums, not just file names.
- **Import chain breakage**: Deleting src/thegent/ copies may break transitive imports that other packages depend on. Run full import resolution before deleting.
- **run_impl_core hidden state**: A 1,022-line function likely relies on local variable state accumulated across its body. Extraction requires careful identification of shared state and explicit parameter passing.
- **TYPE_CHECKING removal may slow imports**: If circular deps exist because of heavy import-time computation, removing TYPE_CHECKING guards may increase startup time. Monitor import time before and after.

## Reviewer Guidance

- Verify the 469 duplicate deletion used byte-level comparison, not just filename matching
- Check that run_impl_core decomposition preserves the exact execution order and error handling semantics
- Confirm TYPE_CHECKING removals do not introduce import-time circular failures — run `python -c "import thegent"` as a smoke test
- Verify shared types in thegent-core/types/ do not themselves create new dependency edges
- Check that the remaining <50 TYPE_CHECKING files each have justification comments
