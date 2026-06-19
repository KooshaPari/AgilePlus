---
work_package_id: WP18
title: 'heliosCLI: God Module Decomposition + Safety'
lane: planned
dependencies: [WP03]
subtasks: [T053, T054, T055]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP18: heliosCLI — God Module Decomposition + Safety

**Implementation command**: `spec-kitty implement WP18 --base WP03`

## Objective

Decompose the 6 largest and most complex Rust files in heliosCLI (all >5,000 LOC with the highest branch keyword density), eliminate production `unwrap()` crash surface, and clean up dead code and suppressions. This directly addresses the top quality and safety findings from the codebase quality analysis.

## Context

- heliosCLI has 422K Rust LOC across 65 crates and 956 .rs files
- 6 files exceed 5,000 LOC and are simultaneously the highest-complexity modules by branch keyword density
- 934 `unwrap()` calls in production (non-test) Rust code represent a significant crash risk surface
- 54 `#[allow(dead_code)]` suppressions and 100 `// TODO` comments indicate accumulated technical debt
- Connector crates (ollama, lmstudio, chatgpt, backend-client) duplicate near-identical interfaces
- WP03 (workspace structure) must be in place before decomposition begins

## Subtasks

### T053: Split top 4 god modules

**Steps**:
1. Decompose `codex.rs` (9,571 LOC, 369 branches):
   - `codex.rs` — entry point and orchestration only
   - `session.rs` — session lifecycle and state management
   - `model_config.rs` — model configuration and selection
   - `tool_registry.rs` — tool registration and dispatch
2. Decompose `chat_composer.rs` (9,499 LOC, 469 branches — highest complexity in codebase):
   - `chat_composer.rs` — top-level composition orchestration
   - `message_builder.rs` — message construction and formatting
   - `context_window.rs` — context window management and truncation
3. Decompose `codex_message_processor.rs` (8,460 LOC, 423 branches):
   - Split by message type handlers — one module per message category
   - Extract shared message processing utilities
4. Decompose `chatwidget.rs` (8,146 LOC, 457 branches):
   - Split by rendering concern — input handling, display rendering, layout, widget state
5. Update all `mod.rs` and `use` declarations to re-export public API unchanged
6. Update Bazel BUILD files for new source files

**Validation**:
- No file exceeds 3,000 LOC after split
- `cargo build --workspace` succeeds
- `cargo test --workspace` — all tests pass
- `cargo clippy --workspace -- -D warnings` clean
- Public API unchanged (no downstream breakage)

### T054: Production unwrap() elimination

**Steps**:
1. Audit all 934 `unwrap()` calls in production (non-test) Rust code
2. Prioritize by crash risk — files with highest concentration:
   - `apply-patch/lib.rs` — 74 unwrap() calls
   - `network-proxy/runtime.rs` — 38 unwrap() calls
   - `turn_diff_tracker.rs` — 37 unwrap() calls
   - `network-proxy/config.rs` — 33 unwrap() calls
   - `network-proxy/policy.rs` — 29 unwrap() calls
3. Replace each `unwrap()` with appropriate error handling:
   - `.context("descriptive message")?` using anyhow for fallible operations
   - `.expect("invariant: reason")` only where the unwrap is provably safe with a documented invariant
   - Proper `match` or `if let` for cases requiring specific error paths
4. Add `#![deny(clippy::unwrap_used)]` to crates that are fully cleaned

**Validation**:
- `grep -r 'unwrap()' --include='*.rs'` in non-test files shows <100 remaining
- All remaining `unwrap()` calls have an adjacent `// SAFETY:` or `// INVARIANT:` comment justifying them
- `cargo test --workspace` — all tests pass
- No new panics introduced (run with `RUST_BACKTRACE=1` in CI)

### T055: Dead code audit + cleanup

**Steps**:
1. Audit all 54 `#[allow(dead_code)]` suppressions:
   - If the code is actually dead, delete it
   - If the code is used but the compiler cannot see the usage (e.g., cfg-gated, test-only), document with an inline comment explaining why the suppression is needed
   - Target: <10 remaining, all with justification comments
2. Triage all 100 `// TODO` comments:
   - Convert actionable items into tracked issues
   - Remove stale or completed TODOs
   - For TODOs that represent intentional future work, add an issue reference: `// TODO(#123): description`
3. Connector crate deduplication:
   - ollama, lmstudio, chatgpt, and backend-client crates implement near-identical model list/connect interfaces
   - Extract a shared `ModelConnector` trait into a common crate (e.g., `codex-connector-core`)
   - Define the trait with standard methods: `list_models()`, `connect()`, `health_check()`, `name()`
   - Refactor each connector to implement the shared trait
   - Remove duplicated interface definitions

**Validation**:
- `grep -r '#\[allow(dead_code)\]' --include='*.rs'` shows <10 remaining, each with inline justification
- `grep -r '// TODO' --include='*.rs'` shows 0 untracked TODOs (all either removed or have issue references)
- `ModelConnector` trait exists in a shared crate; `grep -r "class BaseAdapter\|impl.*Connector" --include='*.rs'` confirms all connectors use it
- `cargo build --workspace` and `cargo test --workspace` pass

## Definition of Done

- [ ] Top 4 god modules decomposed — no file >3,000 LOC
- [ ] Production unwrap() count reduced from 934 to <100, all remaining justified
- [ ] `#[allow(dead_code)]` count reduced from 54 to <10, all remaining justified
- [ ] 100 `// TODO` comments triaged — zero untracked
- [ ] Shared `ModelConnector` trait extracted, 4 connector crates refactored
- [ ] All tests pass, clippy clean, no public API changes

## Risks

- **Splitting god modules may break internal state assumptions**: Some modules rely on private fields being accessible within the same file. Splitting requires careful `pub(crate)` scoping.
- **unwrap() elimination may change error behavior**: Some unwraps mask intentional panics (e.g., initialization assertions). Each must be individually assessed.
- **Connector trait extraction may reveal subtle behavioral differences**: The "near-identical" interfaces may have provider-specific quirks that a uniform trait cannot capture. Allow trait default methods or associated types for provider-specific extensions.

## Reviewer Guidance

- Verify that decomposed modules maintain the same public API via re-exports
- Check that unwrap replacements use descriptive context strings, not generic messages
- Confirm dead_code suppressions that remain have meaningful justification comments
- Verify the ModelConnector trait captures the full interface, not just a subset
- Run the full test suite with `RUST_BACKTRACE=1` to catch any new panic paths
