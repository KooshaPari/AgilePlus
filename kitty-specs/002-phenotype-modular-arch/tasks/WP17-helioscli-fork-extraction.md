---
work_package_id: WP17
title: 'heliosCLI: Fork-Specific Crate Extraction'
lane: planned
dependencies: [WP03]
subtasks: [T051, T052]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP17: heliosCLI — Fork-Specific Crate Extraction

**Implementation command**: `spec-kitty implement WP17 --base WP03`

## Objective

Extract ~20 `thegent-*` Rust crates from the heliosCLI codex fork into a standalone `phenotype-rs-agents` workspace. This separates Phenotype-specific agent orchestration code from the upstream codex fork, making future upstream rebases clean and predictable.

## Context

- heliosCLI is a fork of OpenAI Codex CLI with Phenotype-specific additions
- ~20 `thegent-*` crates provide agent orchestration, routing, runtime, provider abstractions, and governance
- These crates are NOT upstream codex code — they are Phenotype additions that should not live in the fork
- Without extraction, every upstream rebase requires manually resolving conflicts in Phenotype-added files
- WP03 (workspace structure) must be in place before extraction begins
- Related: WP05 notes that `0xPlaygrounds/rig` can eventually replace `thegent-router`, `thegent-runtime`, and provider crates (2-4.5K LOC)

## Subtasks

### T051: Extract thegent-* crates to standalone workspace

**Steps**:
1. Identify all Phenotype-specific crates in heliosCLI:
   - Scan `crates/thegent-*`, `helios-rs/crates/thegent-*`, and any other paths containing `thegent-` prefixed crates
   - Catalog each crate: name, purpose, dependencies on codex crates, dependencies on other thegent crates
2. Create `phenotype-rs-agents` repository/workspace:
   - Initialize Cargo workspace
   - Set up Bazel BUILD files mirroring heliosCLI patterns
3. Move crates:
   - Copy crate source to new workspace
   - Update `Cargo.toml` for each crate (adjust paths, add git/path dependencies back to codex types if needed)
   - Ensure internal thegent-* cross-dependencies resolve within the new workspace
4. Update heliosCLI to consume extracted crates:
   - Add `phenotype-rs-agents` as a Cargo workspace dependency (git or path)
   - Replace local crate references with external dependency references
   - Verify `cargo build --workspace` and `cargo test --workspace` pass in heliosCLI
5. Verify standalone build:
   - `cargo build --workspace` in `phenotype-rs-agents`
   - `cargo test --workspace` in `phenotype-rs-agents`
   - `cargo clippy --workspace -- -D warnings` in `phenotype-rs-agents`

**Validation**:
- heliosCLI builds and all tests pass with crates consumed externally
- `phenotype-rs-agents` builds and tests independently
- No `thegent-*` source directories remain in heliosCLI (only dependency references)

### T052: Upstream sync strategy + automation

**Steps**:
1. Create upstream sync documentation:
   - Document which files/directories are upstream codex (to be rebased)
   - Document which files/directories are Phenotype additions (now extracted or minimal glue)
   - Document remaining Phenotype modifications to upstream files (patches, config changes)
   - Create a manifest file (e.g., `upstream-manifest.json`) listing every file with its provenance: `upstream`, `phenotype`, or `modified`
2. Create sync automation script (`scripts/upstream-sync.sh`):
   - Fetch latest upstream codex tags/branches
   - Diff upstream changes against local state
   - Flag conflicts between upstream changes and Phenotype modifications
   - Generate a report: new upstream files, modified upstream files, conflict files
3. Add CI job (`.github/workflows/upstream-sync-check.yml`):
   - Run weekly (or on-demand) to check for new upstream releases
   - Run the sync script and post results as a GitHub issue or PR comment
   - Flag if any Phenotype-modified upstream files have changed upstream

**Validation**:
- Upstream manifest correctly classifies all files
- Sync script detects upstream changes and flags conflicts
- CI job runs and produces actionable reports
- A simulated upstream rebase completes cleanly (no thegent-* conflicts)

## Definition of Done

- [ ] All thegent-* crates extracted to `phenotype-rs-agents` workspace
- [ ] heliosCLI consumes extracted crates as external dependencies
- [ ] Both workspaces build, test, and lint cleanly
- [ ] Upstream manifest documents file provenance
- [ ] Sync script detects upstream changes and flags conflicts
- [ ] CI job automates weekly upstream sync checks
- [ ] No thegent-* source code remains in heliosCLI repo

## Risks

- **Dependency cycles**: thegent-* crates may depend on codex core types. These become cross-workspace dependencies — ensure they are pinned to codex versions, not local paths.
- **Bazel build graph**: Extracting crates changes the Bazel dependency graph. Ensure BUILD.bazel files are updated in both workspaces.
- **Runtime linkage**: If any thegent-* crates use `inventory` or similar link-time registration, extraction may break registration. Test thoroughly.
- **Git history**: Moving code loses per-file git history in the new repo. Use `git log --follow` or document the provenance commit in the new repo's initial commit message.

## Reviewer Guidance

- Verify no thegent-* source code remains in heliosCLI after extraction
- Check that cross-workspace dependencies are version-pinned, not path-based (for CI reproducibility)
- Verify the upstream manifest is complete — every file in heliosCLI should be classified
- Test the sync script against a known upstream diff to confirm conflict detection works
- Ensure Bazel targets are updated in both workspaces
