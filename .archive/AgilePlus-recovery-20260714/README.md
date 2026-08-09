# `AgilePlus-recovery-20260714` — Absorbed Recovery Snapshot

This directory preserves the unique content from
`KooshaPari/zz-archive-AgilePlus-recovery-20260714` (archived 2026-07-15),
which was a recovery snapshot of the `AgilePlus` repo taken on 2026-07-14
when the live working tree had gotten into a dirty state during the cleanup
wave and a clean baseline was needed.

**Date merged:** 2026-08-08
**Source commit:** `KooshaPari/zz-archive-AgilePlus-recovery-20260714@main`
**Merger:** forge-airlock (manual semantic integration)

## What this archive was

A pre-2026-07-14 snapshot of `KooshaPari/AgilePlus`, created via:

```
git clone --depth 1 git@github.com:KooshaPari/AgilePlus.git AgilePlus-recovery-20260714
```

The recovery was performed at baseline `a83a7677ecacac0a3080e41da312d80def74fee5`
while the dirty live working tree at `~/CodeProjects/Phenotype/repos/AgilePlus/`
was quarantined as evidence-only. See
[`docs/sessions/20260714-isolated-recovery/00_SESSION_OVERVIEW.md`](docs/sessions/20260714-isolated-recovery/00_SESSION_OVERVIEW.md)
for the full procedure.

## Diff with live AgilePlus

The recovery was a pre-2026-07-14 snapshot. Live AgilePlus has continued
to evolve. Out of 3,534 source files, only **16 files are truly unique**
to this recovery and are preserved here:

### 1. `agileplus.db` (352 KB, SQLite)

A snapshot of the SQLite database used by the AgilePlus CLI at the
2026-07-14 baseline. Contains a **complete schema** (24 tables: `_migrations`,
`features`, `work_packages`, `governance_contracts`, `audit_log`, `evidence`,
`policy_rules`, `metrics`, `wp_dependencies`, `events`, `snapshots`,
`sync_mappings`, `api_keys`, `device_nodes`, `modules`, `module_feature_tags`,
`cycles`, `cycle_features`, `projects`, `users`, `epics`, `stories`) but
**all tables empty** (zero data rows). Useful as a reference for the
schema structure at this point in time.

### 2. `clippy_out.txt`

A snapshot of `cargo clippy --all-targets --all-features -- -D warnings`
output at the 2026-07-14 baseline. Useful for tracking clippy lint drift
over time.

### 3. `STATUS.md`

A 1-line snapshot of `wtmp` (login history) at the baseline. Minimal
content, preserved for completeness.

### 4. `.commitlintrc.yml`, `.trufflehog.yml`, `gitleaks.toml`

Pre-2026-07-14 security/config configs that were later renamed or
consolidated into `.commitlintrc.json` in live. Preserved as historical
configs.

### 5. `crates/agileplus-api/src/router.rs`, `crates/agileplus-domain/src/ports.rs`

Pre-2026-07-14 versions of these source files. Live AgilePlus has split
these into the `router/{compose,health}.rs` and `ports/*.rs` modular
structures. The older monolithic versions are preserved for reference.

### 6. `PhenoLang-crates-2026-06-20/ORIGIN.md`

Provenance marker for the PhenoLang crate snapshot from 2026-06-20. The
actual `crates/` directory in `archive/PhenoLang-crates-2026-06-20/`
is **identical** to `live AgilePlus/crates/` (no content diffs, only 2
unique top-level paths). Preserved here as provenance for the 2026-06-20
snapshot date.

### 7. `docs/sessions/20260714-isolated-recovery/` (7 files)

The full session documentation explaining the recovery procedure:
- `00_SESSION_OVERVIEW.md` — Goal, boundaries, baseline SHA, evidence archive
- `01_RESEARCH.md` — Investigation of the dirty checkout state
- `02_SPECIFICATIONS.md` — Recovery procedure specification
- `03_DAG_WBS.md` — Work breakdown structure
- `04_IMPLEMENTATION_STRATEGY.md` — Recovery implementation approach
- `05_KNOWN_ISSUES.md` — Issues discovered during recovery
- `06_TESTING_STRATEGY.md` — Test plan for the recovered state

## Status

Absorbed. 16 unique files preserved. The remaining 3,518 files in the
source were duplicates of (or superseded by) live AgilePlus content.
