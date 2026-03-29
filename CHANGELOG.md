# Changelog

All notable changes to this project will be documented in this file.

The project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-03-29

### Added
- Kitty-specs for phenosdk-decompose-llm (WP01: initial implementation)
- Kitty-specs for phenosdk-decompose-mcp (structured decomposition framework)
- Kitty-specs for phenosdk-sanitize-atoms (atom type validation and sanitization)
- Module manifest and snapshot tests to phench runtime test suite
- Phench integration testing infrastructure

### Fixed
- Quality: Resolved all 48+ Clippy warnings across `agileplus-api`, `agileplus-sqlite`, `agileplus-domain`, `agileplus-events`, `agileplus-git`, `agileplus-plane`, `agileplus-triage`, and `agileplus-subcmds`.
- Quality: Simplified `manual_async_fn` usage in tests.
- Quality: Fixed `DoubleEndedIterator` usage (`filter().next_back() -> rfind()`).
- Quality: Fixed `await_holding_lock` in `agileplus-sqlite` tests.
- CLI: Resolved `E0061` argument count errors in `agileplus-cli` build.
- Update phench import paths from thegent to phench (module reorganization)
- Gitleaks hooks exit code handling (case-insensitive 'no leaks found' match)

### Changed
- Domain: Implemented `Default` for `KeychainCredentialStore`.
- Phenotype workspace structure updated with new module boundaries

### Security
- Add .env to gitignore
- Add gitleaks configuration for secret scanning
- Add secret-scanning hooks to prevent credential leaks

### Chore
- Ignore phenotype-config-wtrees workspace directory
- Ignore sibling workspace files
- Update evidence ledger and WORKLOG for waves 86-87
- Add workspace members: hexagonal-rs, hexkit, cipher, gauge, logger, metrics, tracing, cli-framework, config-core, xdd-lib-rs

## [0.1.2] - 2026-03-25 

### Fixed 

- CI: Stabilized and resolved clippy warnings across all crates. 
- CI: Resolved workspace-level synchronization and configuration conflicts. 
- Quality: Refactored async implementations and satisfied lock-holding lints. 


## [0.1.1] - 2026-03-25

### Fixed

- CI: Removed duplicated permissions key in `buf` job.
- Dashboard: Resolved Alpine scope loss on kanban board by removing `hx-trigger load`.
- Security: Bumped `time` crate to >=0.3.49 to resolve dependabot alerts.
- Quality: Resolved clippy warnings in dashboard and git snapshot.

## [0.1.0] - 2026-03-23

### Added

- Initial AgilePlus workspace release.
- Core feature, work-package, audit, governance, and workflow tooling.
