# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-agent-dispatch-v0.2.4...agileplus-agent-dispatch-v0.2.5) - 2026-10-15

### Added

- *(workflows)* add GitHub workflows and templates
- *(WP08)* agent dispatch service with Claude/Codex harnesses

### Fixed

- *(tests)* repair 3 CI test failures -- stdin EPIPE, missing init.defaultBranch
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- resolve CI release build failures
- *(rust)* tighten deny.toml wildcards policy in AgilePlus

### Other

- *(fmt)* reformat adapter.rs after the clippy signature change
- *(clippy)* clear the -D warnings failures in agileplus-agent-dispatch
- *(fmt)* format the workspace with CI's exact toolchain
- *(domain,import,dispatch)* cover intent graph, builders, file credentials, WP dependencies, agent harnesses and PR loop
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- remove deprecated config files and GitHub templates
