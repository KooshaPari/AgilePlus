# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/traceability-core-v0.2.4...traceability-core-v0.2.5) - 2026-10-15

### Added

- *(tests)* coverage tests for traceability, git, CLI, sqlite
- *(AgilePlus)* graph layer hardening + intent-graph matrix projection
- *(cli)* wire full-deps SDD surface and restore platform dogfood path ([#917](https://github.com/KooshaPari/AgilePlus/pull/917))
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e

### Fixed

- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- *(traceability-core)* make the 10k impact gate load-independent
- resolve CI release build failures
- *(tests)* traceability intent_graph test adjustments
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(traceability-core)* cover unexercised vocabulary, impact, matrix and gate paths
- *(traceability-core)* add integration tests to improve line coverage
- *(traceability-core)* add tests for lifecycle, governance, tracelink, contract, matrix
- *(traceability-core)* add tests for lib, artifact, ids
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
