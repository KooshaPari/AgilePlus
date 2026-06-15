# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- L3-039: `toolchain-versions.json` — canonical toolchain version matrix for the Phenotype ecosystem (Rust 1.88.0, Node 22, Python 3.12, Go 1.23).
- L3-040: `worklogs/README.md` — index and conventions for the worklog directory.
- L3-038: CI status badge in README.md (shields.io GitHub Actions).
- L3-036: `scripts/action-hygiene-audit.sh` — checks all workflow files for unpinned third-party actions.
- L3-037: commit-msg hook now blocks commits if workflow files contain unpinned actions.
- L3-033-L3-035: All GitHub Actions pinned to 40-character SHA refs across all workflows.
- L3-030: cargo-audit gate added to autograder report.
- L3-027: `.githooks/pre-commit` trufflehog secret-scan hook.
- L3-028: `security-audit` and `secret-scan` recipes in justfile.
- L3-029: `scripts/workspace-audit.sh` — validates workspace path dependencies.
- L3-024: `.github/workflows/autograder.yml` — CI autograder with build/test/clippy/deny/machete gates.
- L3-025: `scripts/worktree-cleanup.sh` — stale worktree and merged branch cleanup.
- L3-023: `.github/workflows/branch-hygiene.yml` — CI branch naming and uncommitted-changes gate.
- L2-036: `.githooks/pre-push` — branch naming enforcement hook.
- L2-037: `scripts/sweep-stale-worktrees.sh` — weekly worktree hygiene script.
- L2-034: `.commitlintrc.json` — conventional commit configuration.
- L1-008: `.editorconfig` with comprehensive language sections, restructured `CODEOWNERS`.
- L1-009: MSRV badge and cargo-binstall instructions in README.md.
- L1-007: `libs/xdd-lib-rs` — cross-dialect library (JSON/TOML/YAML) with `DialectConverter` and `DialectRegistry`.

### Security

- L3-032: `SECURITY.md` updated with Phenotype Org security.txt reference and `private-vuln-reporting@phenotype.local`.
- L3-031: `.github/workflows/workspace-audit.yml` — CI workspace path dependency audit.

### Changed

- L3-026: Retired `xtask-anti-patterns` crate; consolidated checks into justfile recipes.

### Fixed

- Pre-existing type error in `agileplus-cli/src/commands/worklog.rs` (`truncate` call on `Option<String>`).
- Pre-existing merge conflict markers in `Tracera/crates/tracera-core/src/health.rs`.

[Unreleased]: https://github.com/KooshaPari/AgilePlus/compare/HEAD
