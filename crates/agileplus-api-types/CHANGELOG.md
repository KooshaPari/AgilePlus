# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-api-types-v0.2.4...agileplus-api-types-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(crates)* absorb agileplus-api-types, agileplus-error-core, agileplus-plugin-core from PhenoDevOps (L5-104.6) ([#761](https://github.com/KooshaPari/AgilePlus/pull/761))
- apply governance standards ([#285](https://github.com/KooshaPari/AgilePlus/pull/285))
- *(008)* Temporal + Hatchet deployment + NATS migration

### Fixed

- *(security)* replace deprecated serde_yaml with serde_norway 0.9 to clear RUSTSEC-2024-0449
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- fix github test failures (serde default for labels, merged state mapping)
- add ALL missing Cargo.toml files (api-types, error-core, cache, contract-tests, import, integration-tests)

### Other

- remove 30 orphan scaffold crates not in workspace
