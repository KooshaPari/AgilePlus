---
work_package_id: WP27
title: 'Fork Hardening + Upstream Sync Tooling (heliosCLI)'
lane: planned
dependencies: [WP03, WP17]
subtasks: [T081, T082]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP27: Fork Hardening + Upstream Sync Tooling (heliosCLI)

**Implementation command**: `spec-kitty implement WP27 --base WP03,WP17`

## Objective

Harden heliosCLI's fork maintenance by adding CI-enforced alias map validation, automated upstream drift tracking, and a formal ADR documenting the Tauri 2.0 evaluation for heliosApp.

## Context

- heliosCLI is a fork with `helios-rs/Cargo.toml` using alias maps like `codex_protocol = { package = "helios-protocol", ... }` — upstream crate renames silently break these
- WP17 created an upstream sync strategy but it is docs-only (`UPSTREAM_DIFF_SPEC.md`); no automated code-level drift detection exists
- Prior session evaluated Tauri 2.0 as an Electron replacement for heliosApp (600KB vs 150MB binary, better perf) but no formal ADR was recorded

## Subtasks

### T081: Alias map CI + upstream diff tooling

**Steps**:
1. Create `scripts/check-alias-map.sh` that:
   - Parses all `Cargo.toml` files for `package = "..."` alias declarations
   - Verifies each aliased crate name exists in the upstream Codex repo's Cargo workspace
   - Fails CI if any alias references a crate name that does not exist upstream
2. Create `scripts/upstream-diff.sh` that:
   - Clones or fetches upstream codex HEAD (configurable remote)
   - Diffs heliosCLI source tree against upstream, filtering out known Phenotype-only paths
   - Outputs drift metrics: files changed, lines added/removed, new upstream files not in fork, fork files not in upstream
   - Generates a summary report suitable for PR comments
3. Add both scripts to CI workflow (run on PRs targeting main)
4. Add upstream remote configuration to repo (`.upstream-sync.yml` or similar)

**Validation**:
- `scripts/check-alias-map.sh` runs successfully on current codebase
- `scripts/upstream-diff.sh` produces a readable drift report
- Both scripts are invoked in CI and produce visible output on PRs
- Intentionally breaking an alias (renaming a crate) causes CI failure

### T082: Tauri 2.0 ADR

**Steps**:
1. Create `ADR-XXX-tauri-2-evaluation.md` in the appropriate ADR location documenting:
   - Context: heliosApp currently uses Electron (150MB binary, high memory usage)
   - Decision: Evaluate Tauri 2.0 as long-term replacement
   - Pros: 600KB binary, lower memory, Rust backend, better security sandbox
   - Cons: Migration complexity, WebView compatibility variations, ecosystem maturity
   - Migration complexity assessment: affected modules, estimated effort, breaking changes
   - Timeline: recommended phases for incremental migration
   - Status: Proposed (pending team confirmation)
2. Reference the prior session's research findings in the ADR

**Validation**:
- ADR exists with complete context, decision, alternatives, and consequences sections
- Migration complexity assessment includes specific heliosApp modules affected
- Timeline includes phased migration approach with concrete milestones

## Definition of Done

- [ ] `scripts/check-alias-map.sh` validates all Cargo.toml alias maps against upstream
- [ ] `scripts/upstream-diff.sh` produces drift metrics report
- [ ] Both scripts integrated into CI pipeline
- [ ] Tauri 2.0 ADR created with full evaluation and migration assessment
- [ ] All existing tests pass

## Risks

- **Upstream remote may not be publicly accessible**: Ensure the diff script handles authentication or uses a cached/mirrored upstream.
- **Alias map check may have false positives**: Some aliases may reference Phenotype-only crates that intentionally do not exist upstream. Add an allowlist mechanism.
- **Tauri 2.0 ADR may become stale**: Include a review date in the ADR to prompt re-evaluation.

## Reviewer Guidance

- Verify alias map check script handles edge cases (workspace inheritance, path dependencies)
- Check upstream diff script filters out Phenotype-only directories correctly
- Confirm ADR follows the project's ADR format and numbering convention
- Verify CI integration does not add excessive build time (upstream fetch should be cached)
