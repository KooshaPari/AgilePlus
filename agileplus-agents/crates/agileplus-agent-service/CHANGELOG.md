# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-agent-service-v0.2.4...agileplus-agent-service-v0.2.5) - 2026-10-15

### Added

- *(workflows)* add GitHub workflows and templates
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- *(WP08)* agent dispatch service with Claude/Codex harnesses

### Fixed

- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- resolve compilation errors in cache, sync, agent-service
- *(rust)* tighten deny.toml wildcards policy in AgilePlus

### Other

- *(clippy)* clear the -D warnings failures in agileplus-agent-dispatch
- *(fmt)* format the workspace with CI's exact toolchain
- *(plane)* add unit tests for plane crate
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- combined CI hygiene + workspace audit (v2, resolves conflicts) ([#564](https://github.com/KooshaPari/AgilePlus/pull/564))
- shelf hygiene — CI hardening, Dependabot, audit docs, gitignore ([#558](https://github.com/KooshaPari/AgilePlus/pull/558))
- remove deprecated config files and GitHub templates
