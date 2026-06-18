# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

- L3-032: `SECURITY.md` updated with Phenotype Org security.txt reference and `private-vuln-reporting@phenotype.local`.
- L3-031: `.github/workflows/workspace-audit.yml` — CI workspace path dependency audit.

### Changed

- L3-026: Retired `xtask-anti-patterns` crate; consolidated checks into justfile recipes.

### Fixed

- Pre-existing type error in `agileplus-cli/src/commands/worklog.rs` (`truncate` call on `Option<String>`).
- Pre-existing merge conflict markers in `Tracera/crates/tracera-core/src/health.rs`.

[Unreleased]: https://github.com/KooshaPari/AgilePlus/compare/HEAD
