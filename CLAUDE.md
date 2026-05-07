# AgilePlus — AI-Native Project Management Platform

## Overview

AgilePlus is an AI-native project management platform. The Rust workspace is currently scaffolding (no `.rs` files yet). The primary implementation lives in TypeScript/Go layers.

## Architecture

- **Rust workspace**: Root `Cargo.toml` with `[workspace]` + `[package]` (placeholder). Members added as Rust code is created.
- **TypeScript/Go**: Primary application layers (see root directory structure).

## Branch Discipline

- `main` is protected. All changes via PR.
- Branch naming: `feat/`, `fix/`, `chore/`, `ci/`, `docs/` prefixes.
- Keep PRs small and focused.

## Encoding

All files must be UTF-8. No BOM.

## Bootstrap Status

- ✅ `.github/workflows/trufflehog.yml` — secrets scanning
- ✅ `FUNDING.yml` — GitHub Sponsors
- ✅ `SECURITY.md` — vulnerability reporting
- ✅ `.github/dependabot.yml` — automated dependency updates
- ✅ `deny.toml` — cargo-deny advisories config
