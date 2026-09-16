# HANDOFF 3 — Architecture, Quality & Code Cleanup
**Date:** 2026-09-16
**Repo:** agileplus (~/CodeProjects/Phenotype/repos/AgilePlus)
**Branch:** main
**Version:** v0.2.4

## Current State
- 33 crates in workspace (30 in crates/, 3 in agileplus-agents/, desktop/)
- Rust nightly-2026-07-31 toolchain
- Edition 2024, rust-version 1.88
- 4 crates with zero tests: api-types, contract-tests, error-core, import
- 12+ dependency repos exist but are external (plugin-core, plugin-git, plugin-sqlite)

## Bugs Fixed This Session
1. **Block comment detection in repl.rs** — `statement_complete()` was detecting `*/` as block comment START instead of `/*`. Fixed line 130: changed `'/' if prev == '*'` to `'*' if prev == '/'`
2. **Missing Cargo.toml** in 9 workspace members — all created with correct workspace deps
3. **bb8/bb8-redis version mismatch** in agileplus-cache
4. **Compilation errors** in sync and agent-service crates

## Architecture Notes
- **Main binary**: `agileplus-cli` (crates/agileplus-cli) — the primary distributed artifact
- **Other binaries** (not distributed): agileplus-governance, agileplus-agent-service, agileplus-api, agileplus-dashboard
- **External plugin crates** (disabled in Cargo.toml): plugin-core, plugin-git, plugin-sqlite — git repos on <REDACTED> account
- **Desktop app**: Tauri-based (desktop/src-tauri)
- **Test directories**: tests/e2e, tests/e2e-desktop, tests/transport, tests/bdd

## Key Structural Issues
1. **File size** — some files approach or exceed 350-line target:
   - `agileplus-sqlite/src/lib/tests_persistence.rs` (90 test functions — huge)
   - `agileplus-plane/src/extended_tests.rs` (76 test functions)
   - `agileplus-cli/src/commands/repl.rs` — recently fixed block comment bug
2. **Naming** — test files follow canonical naming (no `_v2`, `_fast` suffixes)
3. **4 crates with zero tests** need test scaffolding: api-types, contract-tests, error-core, import

## CI/CD Status
- Release CI: PASSING (v0.2.4, all 4 platforms)
- Desktop CI: needs Apple cert for macOS signing
- Release Crates: needs crates.io auth token
- `infra` workflow: failing (unrelated to our changes)

## Developer Notes
- `cargo check -p <crate>` for fast compilation check
- `cargo test -p <crate>` for single-crate tests
- `cargo llvm-cov -p <crate> --summary-only` for single-crate coverage
- `cargo clippy --workspace` for lint warnings (not enforced in CI currently)
- The workspace has many feature flags — check Cargo.toml `[features]` sections

## Cleanup TODOs
- [ ] Enforce clippy in CI
- [ ] Set up rustfmt check in CI
- [ ] Consider splitting oversized test files (>350 lines)
- [ ] Add tests for 4 zero-coverage crates (api-types, contract-tests, error-core, import)
- [ ] Address 7 GitHub Dependabot security alerts (3 high, 2 moderate, 2 low)
