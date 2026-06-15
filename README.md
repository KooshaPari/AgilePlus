<!-- AI-DD-META:START -->
<!-- This repository is planned, maintained, and managed by AI Agents only. -->
<!-- Slop issues are expected and intentionally present as part of an HITL-less -->
<!-- /minimized AI-DD metaproject of learning, refining, and building brute-force -->
<!-- training for both agents and the human operator. -->
![Downloads](https://img.shields.io/github/downloads/KooshaPari/AgilePlus/total?style=flat-square&label=downloads&color=blue)
![GitHub release](https://img.shields.io/github/v/release/KooshaPari/AgilePlus?style=flat-square&label=release)
![CI](https://img.shields.io/github/actions/workflow/status/KooshaPari/AgilePlus/ci.yml?branch=main&style=flat-square&label=CI)
![License](https://img.shields.io/github/license/KooshaPari/AgilePlus?style=flat-square)
![AI-Slop](https://img.shields.io/badge/AI--DD-Slop%20Expected-orange?style=flat-square)
![AI-Only-Maintained](https://img.shields.io/badge/Planned%20%26%20Maintained%20by-AI%20Agents%20Only-red?style=flat-square)
![HITL-less](https://img.shields.io/badge/HITL--less%20AI--DD-metaproject-yellow?style=flat-square)

> ⚠️ **AI-Agent-Only Repository**
>
> This repo is **planned, maintained, and managed exclusively by AI Agents**.
> Slop issues, rough edges, and AI artifacts are **expected and intentionally
> present** as part of an **HITL-less / minimized AI-DD** metaproject focused
> on learning, refining, and brute-force training both the agents and the
> human operator. Bug reports and contributions are still welcome, but please
> expect AI-generated code, comments, and documentation throughout.
<!-- AI-DD-META:END -->
> **Work state:** ACTIVE · **Progress:** `███████░░░ 70%`
> AI-native spec-driven PM platform (Rust workspace + React/TS dashboard + Electrobun desktop); frontend candidate #1. Core domain/api/dashboard implemented; CI partially red. · updated 2026-06-02

# AgilePlus

> **Pinned references (Phenotype-org)**
> - MSRV: [![Rust](https://img.shields.io/badge/MSRV-1.88.0-orange?logo=rust)](rust-toolchain.toml)
> - cargo-deny config: see `deny.toml`
> - cargo-audit: `rustsec/audit-check@v2` weekly
> - Branch protection: 1 reviewer required, no force-push
> - Branching baseline: canonical checkout stays on `main` unless doing merge/pull
> - Governance authority: `phenotype-org-governance/SUPERSEDED.md` when present

**Local-first, AI-native, spec-driven project management for agent + human teams.**

AgilePlus manages feature specs, work packages, and acceptance criteria with a hexagonal
Rust core, optional GitHub/Plane sync, P2P merge, a web dashboard, and a desktop app. It is
one of the Phenotype org's three project-management frontend candidates (alongside Tracera and
Planify).

## Architecture

AgilePlus is a Cargo workspace following hexagonal (ports-and-adapters) architecture: the
`domain` and `application` crates have no framework dependencies; everything else is an adapter.

| Crate | Role |
|-------|------|
| `agileplus-domain` | Core entities, invariants (no framework deps) |
| `agileplus-application` | Use-case layer (no framework deps) |
| `agileplus-api` | HTTP API surface |
| `agileplus-grpc` / `agileplus-proto` | gRPC layer + compiled tonic types |
| `agileplus-cli` / `agileplus-subcmds` | `agileplus` command-line client |
| `agileplus-dashboard` | Web dashboard (Askama + React/TS under `web/`) + Electrobun desktop |
| `agileplus-sqlite` | SQLite persistence adapter |
| `agileplus-events` / `agileplus-nats` | Event model + NATS transport |
| `agileplus-sync` | Sync orchestrator — conflict detection/resolution + NATS |
| `agileplus-p2p` | Peer-to-peer merge |
| `agileplus-github` / `agileplus-plane` / `agileplus-import` | External integrations + import |
| `agileplus-git` | Git integration |
| `agileplus-governance` | Release channels, audit logging, policy enforcement |
| `agileplus-config` | Shared config-builder macro |
| `agileplus-cache` / `agileplus-telemetry` / `agileplus-triage` | Cache, telemetry, triage |
| `agileplus-graph` / `agileplus-artifacts` / `agileplus-fixtures` | Graph, artifacts, fixtures |
| `agileplus-benchmarks` | Criterion performance benchmarks |
| `agileplus-contract-tests` / `agileplus-integration-tests` | Cross-crate test suites |

The Python `agileplus-mcp/` directory is a separate FastMCP server. `python/phenotype_traceability/`
holds the traceability package.

## Getting Started

```bash
# Build the workspace
cargo build --workspace

# Install and run the CLI (from source)
cargo install --path crates/agileplus-cli
agileplus --help

# Or install via cargo-binstall (prebuilt binary, faster)
cargo binstall agileplus-cli
agileplus --help

# Create a spec / feature
agileplus specify --title "<feature>" --description "<desc>"

# Web dashboard frontend
cd crates/agileplus-dashboard/web
bun install && bun run dev
```

## Development

- `main` is protected — all changes via PR. Branch prefixes: `feat/ fix/ chore/ ci/ docs/`.
- Keep PRs small and focused; fix all CI failures on a PR, including pre-existing ones.
- All files UTF-8, no BOM. Never commit agent dirs (`.claude/`, `.codex/`, `.cursor/`).
- Spec work is tracked in AgilePlus itself (`agileplus specify` / `agileplus status`).

## Quality Standards

- `cargo clippy --workspace -- -D warnings` (zero warnings)
- `cargo fmt` before commit
- Tests for new features; reproduce a bug with a failing test before fixing
- cargo-deny advisories (`deny.toml`) + weekly cargo-audit

## License

See [LICENSE](LICENSE).
