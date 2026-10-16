# AgilePlus

AgilePlus is a standalone, local-first tooling codebase for **governed code-project work**: specifications, work packages, execution records, evidence, evaluation, audit, and development acceptance.

> **Repository boundary:** this repository is the AgilePlus product source tree. It is not a project shelf, an organization-planning repository, or the canonical home for sibling products. AgilePlus may register and operate on external Git repositories at runtime without copying their source, worktrees, or product specifications into this repository.

## Current state

AgilePlus has substantial implementation, but it is still in recovery and rollout work. Do not infer release readiness from file presence, historical badges, or completion claims.

Implemented or materially present on the current line includes:

- Rust domain, application, governance, event, SQLite, Git, HTTP API, and gRPC crates;
- a Rust CLI;
- a Python FastMCP bridge to the gRPC core;
- an Askama/Axum web application;
- a Tauri desktop/tray client;
- immutable execution/evidence/acceptance records and an atomic SQLite acceptance boundary;
- packaging and release workflows at different maturity levels.

The complete rollout target—one coherent web, device-daemon, tray/desktop, CLI, and MCP bundle installed and verified on both target machines—is **not yet complete**. See [the stable-release definition](STABLE.md) and [the device-daemon rollout issue](https://github.com/KooshaPari/AgilePlus/issues/1119).

## Product and authority boundaries

| Domain | Canonical authority |
|---|---|
| Governed code-project work packages, execution evidence, evaluation, audit, and development acceptance | **AgilePlus** |
| Persistent product/system records and product-state interpretation | **Tracera** |
| Organization/personal planning, scheduling, and coordination, when adopted | **Plane or Paperclip** |

External systems may reference or project AgilePlus work. They cannot create or replace an AgilePlus acceptance receipt. See [ADR-0018](docs/adr/0018-tracera-agileplus-sibling-runtimes.md), [ADR-0019](docs/adr/0019-acceptance-authority-terminal-gates.md), and [issue #1117](https://github.com/KooshaPari/AgilePlus/issues/1117).

## Repository layout

```text
.
├── crates/                 # Rust domain, application, storage, API, gRPC, web, integrations
├── agileplus-agents/       # Agent adapters and services owned by AgilePlus
├── python/                 # FastMCP bridge and generated gRPC bindings
├── desktop/                # Active Tauri desktop/tray client
├── spec/product/           # Mature product contracts and authority records
├── verification/           # Executed-evidence contracts, acceptance and recovery records
├── docs/                   # Architecture, ADRs, operations, ecosystem, research, and history
├── kitty-specs/            # Historical and active specification lineage; status varies by bundle
└── .github/workflows/      # CI, security, release, coverage, performance, and deployment workflows
```

Some tracked paths are historical shelf-era, recovery, vendored, generated, or sibling-owned candidates. Their presence does not make them active AgilePlus requirements. See [the root-scope contamination checkpoint](docs/forensics/ROOT_SCOPE_CONTAMINATION_CHECKPOINT.md) and [issue #1128](https://github.com/KooshaPari/AgilePlus/issues/1128).

## Build and test

From the repository root:

```bash
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Useful entry points:

```bash
cargo run -p agileplus-cli -- --help
cargo run -p agileplus-dashboard
cargo run -p agileplus-grpc
uv run --project python agileplus-mcp
```

Individual surfaces may require platform libraries, credentials, or a configured project root. A command compiling is not proof that its complete user journey is deployed or accepted.

## Working on AgilePlus

1. Treat this repository root as the AgilePlus source root.
2. Read [AGENTS.md](AGENTS.md), [GOVERNANCE.md](GOVERNANCE.md), and the applicable specification/ADR before changing behavior.
3. Keep sibling-product work in its owning repository.
4. Use a branch or isolated worktree; do not place sibling checkouts inside AgilePlus.
5. Bind claims to commands, revisions, logs, and receipts. Not run means not passed.
6. Open a reviewable pull request. Do not merge, deploy, or publish without the required authority.

## Canonical operational references

- [Agent rules](AGENTS.md)
- [Repository governance](GOVERNANCE.md)
- [Stable-release definition](STABLE.md)
- [Product authority index](spec/product/AUTHORITY_INDEX.md)
- [Canonical semantic decisions](spec/product/CANONICAL_SEMANTIC_DECISIONS.md)
- [Forward work breakdown](verification/FORWARD_WBS_2026-10-04.md)
- [Forward delta and release work](verification/FORWARD_DELTA_2026-10-04.md)
- [Next deployable release gate](verification/NEXT_DEPLOYABLE_RELEASE_2026-10-04.md)
- [Repository identity correction tracker](https://github.com/KooshaPari/AgilePlus/issues/1128)

## Historical material

Historical shelf, portfolio, sibling-project, and alternative-client documents remain useful forensic evidence. They are not current instructions merely because they are newer, larger, or located at repository root. Preserve provenance, mark supersession, and migrate ownership through reviewed work rather than deleting or reviving material by assumption.
