# HANDOFF 1 — Release Pipeline & CI
**Date:** 2026-09-16
**Repo:** agileplus (~/CodeProjects/Phenotype/repos/AgilePlus)
**Branch:** main
**Version:** v0.2.4 (released and passing CI)

## Current State
- cargo-dist v0.33.0 configured, `curl | sh` / `irm | iex` installers working
- CI builds pass on ALL 4 platforms: macOS arm64, macOS x86, Linux x86, Windows x86
- Only `agileplus-cli` is distributed (all other crates have `dist = false`)
- Protobuf installed via `[workspace.metadata.dist.dependencies]` on CI runners
- `precise-builds = true` set to avoid building entire workspace in CI
- Brewfile moved to `.dev/Brewfile` to prevent cargo-dist from running `brew bundle exec`
- Tag: v0.2.4 — passing CI

## What Was Fixed
1. **9 missing Cargo.toml files** — api-types, error-core, cache, contract-tests, import, integration-tests, nats, p2p, sync
2. **Brewfile conflict** — moved from root to `.dev/`
3. **Missing protobuf on CI** — added as dist dependency (homebrew/apt/chocolatey)
4. **agileplus-agent-service building in CI** — `dist = false` added to ALL non-CLI crates
5. **Version mismatches** — internal crate deps must all match workspace version

## Remaining Work
- [ ] Consider `aarch64-unknown-linux-gnu` target (ARM Linux)
- [ ] Homebrew installer tap setup
- [ ] GitHub release attestation (optional)
- [ ] Release Crates workflow — needs crates.io auth token
- [ ] Release Desktop workflow — needs Apple Developer Certificate for macOS signing

## Key Files
- `Cargo.toml` lines 63-90: dist config + dependencies
- `.github/workflows/release.yml` — cargo-dist generated CI
- `.dev/Brewfile` — dev dependencies (moved from root)

## How to Verify
```bash
cargo dist plan  # should show only agileplus-cli
cargo dist build  # local build
gh run view --limit 1  # check latest CI
```
