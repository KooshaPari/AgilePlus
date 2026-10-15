# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-error-core-v0.2.4...agileplus-error-core-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(crates)* absorb agileplus-api-types, agileplus-error-core, agileplus-plugin-core from PhenoDevOps (L5-104.6) ([#761](https://github.com/KooshaPari/AgilePlus/pull/761))
- apply governance standards ([#285](https://github.com/KooshaPari/AgilePlus/pull/285))
- *(008)* Temporal + Hatchet deployment + NATS migration

### Fixed

- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- add ALL missing Cargo.toml files (api-types, error-core, cache, contract-tests, import, integration-tests)

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(sync,error-core)* add comprehensive tests
- *(import,error-core)* add unit tests for import and error-core crates
- remove 30 orphan scaffold crates not in workspace
- vendor agileplus boundary types
