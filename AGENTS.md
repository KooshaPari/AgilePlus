# AgilePlus — AGENTS.md

> **AI-agent constitution for `AgilePlus`.** Generated from the V3 §120
> SD4 SOTA pattern (V18 build/test/style/do-not-touch constitution) on
> 2026-06-12. Read this fully before making changes.

---

## 1. Quick start (build, test, lint)

```bash
# Build the full workspace (21 crates, ~3 min cold / <2s incremental)
cargo build --workspace

# Test the full workspace
cargo test --workspace

# Lint (zero warnings enforced; matches L1 quality gate)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Supply-chain (configured via deny.toml)
cargo deny check
cargo audit
```

Python MCP server (`agileplus-mcp/`) uses `uv` / `ruff`:

```bash
cd agileplus-mcp && uv sync && uv run pytest
cd python/phenotype_traceability && uv sync && ruff check .
```

Web dashboard (`crates/agileplus-dashboard/web`) uses `bun`:

```bash
cd crates/agileplus-dashboard/web && bun install && bun run dev
```

---

## 2. Project layout (top-level dirs + purpose)

| Path | Purpose |
|------|---------|
| `crates/agileplus-domain/` | Core entities, invariants — zero framework deps |
| `crates/agileplus-application/` | Use-case layer — zero framework deps |
| `crates/agileplus-cli/`, `crates/agileplus-subcmds/` | `agileplus` command-line client |
| `crates/agileplus-api/` | HTTP API surface (axum) |
| `crates/agileplus-grpc/`, `crates/agileplus-proto/` | gRPC layer + tonic types |
| `crates/agileplus-dashboard/` | Web dashboard (Askama + React/TS) + Electrobun desktop |
| `crates/agileplus-sqlite/` | SQLite persistence adapter |
| `crates/agileplus-events/`, `crates/agileplus-nats/` | Event model + NATS transport |
| `crates/agileplus-sync/`, `crates/agileplus-p2p/` | Sync orchestrator + P2P merge |
| `crates/agileplus-github/`, `crates/agileplus-plane/`, `crates/agileplus-import/`, `crates/agileplus-git/` | External integrations + Git |
| `crates/agileplus-governance/` | Release channels, audit logging, policy enforcement |
| `crates/agileplus-config/`, `crates/agileplus-cache/`, `crates/agileplus-telemetry/`, `crates/agileplus-triage/`, `crates/agileplus-graph/`, `crates/agileplus-artifacts/`, `crates/agileplus-fixtures/` | Cross-cutting adapters + fixtures |
| `crates/agileplus-benchmarks/`, `crates/agileplus-contract-tests/`, `crates/agileplus-integration-tests/` | Bench (criterion) + cross-crate test suites |
| `agileplus-mcp/` | Separate FastMCP server (Python) |
| `python/phenotype_traceability/` | Python traceability package |
| `kitty-specs/` | Legacy spec archive (read-only) |
| `docs/`, `findings/`, `assets/`, `brand/`, `proto/` | Docs, audits, brand assets, protobuf sources |
| `.github/workflows/` | 20 CI workflows (ci, quality-gate, cargo-audit, security, …) |

Hexagonal architecture: `agileplus-domain` and `agileplus-application` are the
stable center; every other crate is a swappable adapter.

---

## 3. Key files (entry points, config files)

| File | Role |
|------|------|
| `Cargo.toml` | Root workspace manifest (21 members) |
| `Cargo.lock` | Pinned dependency graph (committed) |
| `Justfile` | Canonical task runner (L2 spec; replaces Taskfile/Makefile) |
| `rust-toolchain.toml` | Nightly toolchain pin |
| `clippy.toml`, `rustfmt.toml` | Lint + format config |
| `deny.toml` | cargo-deny advisories/bans/licenses/sources |
| `codecov.yml` | Coverage upload (Codecov) |
| `lefthook.yml` | Git hooks (mirror of `.pre-commit-config.yaml`) |
| `.pre-commit-config.yaml` | pre-commit hooks (rustfmt, clippy, trufflehog, gitleaks) |
| `ARCHITECTURE.md` | Hexagonal-architecture deep dive |
| `CARGO-WORKSPACE.md` | Crate-by-crate responsibilities |
| `CLAUDE.md` | Claude-specific operating notes |
| `CHANGELOG.md` | Release history (cliff.toml-driven) |
| `CODEOWNERS` | Per-path review routing |
| `cliff.toml` | git-cliff release-notes template |
| `crates/agileplus-cli/src/main.rs` | CLI entrypoint |
| `crates/agileplus-api/src/main.rs` | API server entrypoint |
| `kitty-specs/<feature-id>/spec.md` | Spec-per-feature (governance input) |
| `.work-audit/worklog.md` | Per-crate worklog mirror (if present) |

---

## 4. Conventions

- **Commit message format** — Conventional Commits, scoped to the crate:
  `feat(agileplus-cli): …`, `fix(agileplus-domain): …`, `chore(agileplus-telemetry): …`,
  `docs(agileplus): …`. The scope is always a single crate name.
- **Branch naming** — `<prefix>/<TID>-<topic>-<date>` where prefix ∈
  `{feat, fix, chore, ci, docs, refactor, test, perf, build}` and
  TID is a V3 DAG task ID (e.g. `L1-001`, `CC2-001`, `SD4`). Examples:
  `chore/L1-006-sota-pgo-2026-06-11`, `feat/L2-011-async-trait-tokio-2026-06-11`,
  `chore/SD4-2026-06-12` (this worktree).
- **Worklog schema** — V2 10-column JSON schema. Canonical reference:
  [`pheno-worklog-schema`](https://github.com/KooshaPari/pheno-worklog-schema)
  (or local `pheno-worklog-schema/` if vendored). Each SD/L/CC/QC task
  produces one worklog JSON at the repo root: `worklog-<TID>-<topic>.json`.
- **PR policy** — `main` is protected (1 reviewer required, no force-push).
  All changes flow through PRs; do **not** commit directly to `main`.
- **Encoding** — UTF-8, no BOM. Never commit agent dirs (`.claude/`, `.codex/`,
  `.cursor/`). Already enforced by `.pre-commit-config.yaml`.

---

## 5. Common tasks

### Add a Rust dependency to a crate

```bash
# From the crate root (e.g. crates/agileplus-cli/)
cargo add <crate-name> --features <feature>

# Or edit crates/<crate>/Cargo.toml [dependencies] directly, then:
cargo build -p agileplus-<crate>
cargo deny check     # advisories + license + bans + sources
```

After adding, regenerate the lockfile and verify CI:

```bash
cargo update -p <crate-name>     # only if upgrading
cargo build --workspace
cargo test --workspace
```

### Add a Rust test

- **Unit test** — add a `#[cfg(test)] mod tests` block at the bottom of the
  same file. Idiomatic, no extra setup.
- **Integration test** — add a new file under
  `crates/<crate>/tests/integration_<topic>.rs`. Use
  `agileplus-fixtures` (or `tempfile` for filesystem cases) for shared setup.
- **Property test** — use `proptest!` from the existing dev-dep in
  `agileplus-domain` and `agileplus-application`.
- Always run `cargo test -p <crate>` and `cargo clippy -p <crate> -- -D warnings`
  locally before pushing.

### Run benchmarks (criterion)

```bash
cargo bench -p agileplus-benchmarks
# HTML report: target/criterion/report/index.html
```

---

## 6. Tooling

- **Task runner: `Justfile` (casey/just).** Chosen for the L2 #15 SOTA
  because: (1) casey/just is the cross-platform standard across the org
  (mirrors PlayCua, nanovms, PhenoCompose, BytePort), (2) just recipes
  compose cleanly (`just ci` = lint + fmt + test + deny), (3) just 1.36+
  is a single static binary with no runtime deps.
- **Linter: `cargo clippy --workspace -- -D warnings`** (CI-enforced).
- **Formatter: `cargo fmt --all`** (CI-enforced via `fmt-check` workflow).
- **Pre-commit: `lefthook` + `.pre-commit-config.yaml`** running rustfmt,
  clippy, trufflehog, gitleaks. Install with `brew install lefthook && lefthook install`.
- **Supply-chain: `cargo-deny` + `cargo-audit`** (deny.toml + weekly
  `rustsec/audit-check@v2` workflow).
- **Coverage: `cargo llvm-cov` + Codecov** (codecov.yml).
- **Releases: `git-cliff`** (cliff.toml) → tags + GitHub release notes.
- **VCS: git worktrees** — work in `AgilePlus-wtrees/<topic>/`, never
  directly in the bare `AgilePlus/` checkout on `main`.

---

## 7. Do not touch (without an explicit task)

- `Cargo.toml [workspace]` — adding/removing members is an L2 SOTA task.
- `rust-toolchain.toml` — toolchain pin is contractually nightly (L1 stabilization).
- `deny.toml` and `clippy.toml` — version pins are intentional.
- `kitty-specs/` (root) — legacy archive, read-only.
- `crates/agileplus-domain/` public API — every variant of every error
  enum and every entity field is part of the spec contract; PRs that
  change them need a `## API diff` section.
- `crates/agileplus-application/` port traits — adding a method is a
  breaking change for every adapter.
- The `.pre-commit-config.yaml` `trufflehog` hook id — replaced by the
  `phenotype-secret-scan` workflow in a future L2 pass.
- `CODEOWNERS` — review routing is governance-mandated.

---

## 8. Reference

- **V3 §120 (SD4 SOTA pattern)** — this file's section layout.
- **V18 §110 pheno-otel AI-DD crutches** — the 5-convention-file pattern
  (AGENTS.md, llms.txt, WORKLOG.md, CHANGELOG.md, LICENSE-MIT) that
  AgilePlus inherits at the per-crate level.
- **V11 §70.3 (AX/L16 acceptance)** — `cargo clippy --workspace -- -D warnings`
  is the canonical zero-warnings gate.
- **FLEET_100TASK_DAG_V3.md** — task IDs (`L1-001`, `CC2-001`, `SD4`).
- **CLAUDE.md** — Claude-specific operating notes (parallel work policy,
  workspace audit findings, encoding rules).
- **phenotype-org-governance/SUPERSEDED.md** — governance authority
  (when present, supersedes local conventions).

---

## 9. License

MIT OR Apache-2.0 (dual). See `LICENSE`, `LICENSE-MIT`, and `LICENSE-APACHE`.
Copyright 2026 Koosha Pari.
