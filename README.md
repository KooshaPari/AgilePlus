> **Work state:** ACTIVE · **Progress:** `███████░░░ 70%`
> AI-native spec-driven PM platform (Rust workspace + React/TS dashboard + Electrobun desktop); frontend candidate #1. Core domain/api/dashboard implemented; CI partially red. · updated 2026-06-02

# AgilePlus

![Scorecard](https://api.securityscorecards.dev/projects/github.com/KooshaPari/AgilePlus/badge)

> **Pinned references (Phenotype-org)**
> - MSRV: see `rust-toolchain.toml`
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

# Install and run the CLI
cargo install --path crates/agileplus-cli
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
