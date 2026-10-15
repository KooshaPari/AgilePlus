# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-bdd-v0.2.4...agileplus-bdd-v0.2.5) - 2026-10-15

### Added

- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(workflows)* add GitHub workflows and templates
- *(WP16)* BDD acceptance tests and integration suite

### Fixed

- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- resolve CI release build failures
- comprehensive tests, BDD fixes, and gRPC command implementations
- *(core)* restore reachable gRPC and MCP runtime
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- resolve compilation errors and apply formatting standards ([#154](https://github.com/KooshaPari/AgilePlus/pull/154))
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete
- wire dashboard/platform CLI subcommands, fix BDD test fields and race condition - 622 tests passing

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- chore(deps)(deps): bump the major group across 1 directory with 2 updates ([#1080](https://github.com/KooshaPari/AgilePlus/pull/1080))
- *(fixtures)* align governance and audit schemas ([#1067](https://github.com/KooshaPari/AgilePlus/pull/1067))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- remove deprecated config files and GitHub templates
