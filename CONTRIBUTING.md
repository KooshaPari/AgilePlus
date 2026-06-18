# Contributing to AgilePlus

Thank you for your interest in AgilePlus — the multi-agent delivery
orchestrator that turns spec-kitty work-packages into shippable code.
We welcome bug reports, docs, tests, refactors, new workflow plugins,
and feature contributions from everyone.

This document explains how to set up your development environment,
run the test suite, propose changes, and get them merged safely.

---

## 1. Code of Conduct

By participating, you agree to abide by the
[Phenotype Code of Conduct](CODE_OF_CONDUCT.md) (if present) and the
GitHub Community Guidelines. Be respectful, assume good faith, and
prefer written communication that can be quoted later.

## 2. Project Overview

AgilePlus is a Rust-first, plugin-driven CLI + library that:

- Loads **spec-kitty** work-package definitions (worktrees, plans,
  JSONL event streams).
- Drives a fleet of **coding agents** (Codex, Forge, thegent,
  cheap-LLM) through their `Worktree → Branch → PR → Merge` lifecycle.
- Emits structured **status snapshots** to a Phenotype-org dashboard
  and (optionally) to OpenTelemetry.
- Ships with a small **Python** adapter for legacy jira / trello
  pipelines and a **TypeScript** dashboard frontend.

The repository is a Cargo workspace (root `Cargo.toml`) with
optional `python/` and `dashboard/` sub-projects. The `Justfile` at
the root is the canonical entry point for all build / test / lint
tasks.

## 3. Development Environment

### 3.1 Required Toolchains

| Tool          | Version   | Why                                |
|---------------|-----------|------------------------------------|
| Rust          | `stable`  | Core orchestrator                  |
| `cargo`       | ≥ 1.78    | Build, test, fmt, clippy           |
| `rustfmt`     | stable    | Formatting                         |
| `clippy`      | stable    | Lints (CI fails on warnings)       |
| `cargo-deny`  | ≥ 0.14    | License + advisory gating          |
| `cargo-audit` | ≥ 0.20    | Vulnerability scan                 |
| `cargo-nextest`| ≥ 0.9    | Faster test runner (optional)      |
| Python        | 3.11+     | Legacy adapter, scripts            |
| `uv`          | ≥ 0.4     | Python env + dep manager           |
| `ruff`        | ≥ 0.5     | Python linter + formatter          |
| `mypy`        | ≥ 1.10    | Python type-check                  |
| Node.js       | ≥ 20 LTS  | Dashboard build                    |
| `pnpm`        | ≥ 9       | Dashboard package manager          |
| `just`        | ≥ 1.36    | Task runner (preferred over Make)  |
| `lefthook`    | ≥ 1.6     | Git hooks manager                  |

### 3.2 Clone + Bootstrap

```bash
git clone https://github.com/KooshaPari/agileplus.git
cd agileplus
just bootstrap
```

`just bootstrap` will:

1. Install `lefthook` git hooks (`pre-commit`, `pre-push`).
2. Install `cargo` subcommands we use (`deny`, `audit`, `nextest`,
   `outdated`, `bloat`).
3. Run `cargo fetch` and the smoke build of every workspace member.
4. (Optional) set up the `python/` venv via `uv venv` and `uv pip sync`.
5. (Optional) `pnpm install` for the dashboard.

### 3.3 Editor Setup

- **VS Code**: open `agileplus.code-workspace` (if present) or just the
  root; the recommended extensions are:
  `rust-lang.rust-analyzer`, `tamasfe.even-better-toml`,
  `charliermarsh.ruff`, `ms-python.mypy`, `svelte.svelte-vscode` (for
  the dashboard).
- **Neovim / Helix / Zed**: zero-config LSPs; the `rust-analyzer`
  config lives at `.config/rust-analyzer.toml`.
- **JetBrains RustRover**: open the root, and RustRover will pick up
  the workspace members automatically.

## 4. Building

```bash
# Everything (Rust workspace + Python + dashboard)
just build

# Just the Rust workspace
cargo build --workspace --all-targets

# Release-mode binaries
cargo build --release --workspace

# Python adapter smoke build
(cd python && uv build)

# Dashboard
(cd dashboard && pnpm build)
```

Useful binary outputs:

- `target/release/agileplus` — main CLI.
- `target/release/agileplusd` — long-running daemon (used by the
  dashboard).
- `target/release/ap-fleet` — bulk-fleet runner.

## 5. Testing

| Tier          | Command                                       | Owner       | Wall-clock |
|---------------|-----------------------------------------------|-------------|------------|
| Unit (Rust)   | `cargo test --workspace`                      | Core team   | < 2 min    |
| Unit (Python) | `uv run pytest` (in `python/`)                | Adapter     | < 1 min    |
| Unit (Web)    | `pnpm --filter ./dashboard test`              | UI team     | < 1 min    |
| Integration   | `just test-integration`                       | Core team   | < 10 min   |
| Snapshot      | `cargo insta test --workspace --review`       | Core team   | < 2 min    |
| Property      | `cargo test --features proptest`              | Core team   | < 5 min    |
| Fuzz          | `cargo +nightly fuzz run parser -- -max_total_time=600` | Security | 10 min |
| E2E           | `just test-e2e`                               | Core team   | < 30 min   |

CI runs unit + snapshot + integration + property on every PR. Fuzz
and E2E run nightly and on release tags.

## 6. Coding Standards

- **Rust**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.
  Use `tracing` for structured logs; never `eprintln!` in library code.
- **Errors**: `thiserror` for typed error enums, `anyhow` only at the
  binary boundary. Wrap context with `.with_context()`.
- **Public APIs**: every public function has a doc-comment and a
  `#[non_exhaustive]` attribute on enums where we may add variants.
- **Python**: `ruff format`, `ruff check`, `mypy --strict`. Type
  hints are mandatory on all new code.
- **TypeScript / Svelte**: `prettier --check`, `eslint`, `svelte-check`.
- **Tests**: name files `<module>.test.rs` colocated with the module
  they test; integration tests live in `tests/` at the workspace root.
- **Commits**: conventional commits — see §9.

## 7. Branching

- Default branch: `main`.
- Long-lived integration branches: `release/X.Y`.
- Feature / fix / chore branches: `<type>/<scope>-<short-desc>`
  (kebab-case, ≤ 60 chars). The `<type>` matches the conventional
  commit type and the `<scope>` matches the commit scope.
  Examples: `feat/cli-fleet-mode`, `fix/rpc-reconnect`,
  `chore/l2-30-governance-2026-06-11`.

## 8. Pull Request Process

1. **Open an issue first** for non-trivial changes. Bug fixes and
   documentation improvements may go straight to PR.
2. **Fork** the repo (or push to a feature branch if you have write
   access via the Phenotype org).
3. **Keep PRs focused**: < 400 lines diff where possible. Split
   larger refactors into a stack of dependent PRs.
4. **Fill the PR template** — it links to the design doc / spec /
   issue, the test plan, and the rollout / risk notes.
5. **Pass CI**: fmt, clippy, all tier-1 tests, `cargo deny` (license +
   advisory), `cargo audit`, CodeQL, OpenSSF Scorecard check.
6. **Request a review** from the CODEOWNERS — for AgilePlus the
   default reviewer is `@KooshaPari`. Add a domain reviewer (e.g.
   security, dashboard) for cross-cutting changes.
7. **Address review feedback** in additional commits; the maintainer
   will squash-merge once the conversation is resolved.
8. **After merge**, delete the source branch.

## 9. Commit Message Format (Conventional Commits)

AgilePlus uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/).

```
<type>(<scope>): <short summary>

<body — wrap at 72 cols; explain *what* and *why*>

<footer — e.g. "BREAKING CHANGE: ...", "Closes #123", "Refs: SPEC-42">
```

### Allowed types

| Type       | Semantics                                                    |
|------------|--------------------------------------------------------------|
| `feat`     | A new user-facing feature                                    |
| `fix`      | A bug fix                                                    |
| `docs`     | Documentation only                                           |
| `style`    | Whitespace/formatting, no code change                        |
| `refactor` | Code change that neither fixes a bug nor adds a feature      |
| `perf`     | Performance improvement                                      |
| `test`     | Add or correct tests                                         |
| `build`    | Build system, CI, or dependency change                       |
| `chore`    | Tooling, repo hygiene, governance (this PR)                  |
| `revert`   | Reverts a previous commit (include `Reverts: <sha>`)         |
| `security` | Security fix (also notify `security@phenotype.internal`)     |

### Scopes (non-exhaustive)

`cli`, `daemon`, `fleet`, `agent`, `parser`, `rpc`, `dashboard`,
`python`, `ci`, `docs`, `deps`, `governance`.

### Examples

```
feat(fleet): add --max-concurrent-jobs flag to ap-fleet

Previously the fleet runner forked one OS process per work-package
which exhausted the file-descriptor limit on a 256-core CI host.
We now cap the in-flight count with a tokio semaphore sourced
from the new flag (default: nproc * 2).

Adds a soak test under `tests/soak/fleet.rs`.

Closes #213
Refs: SPEC-12 §3
```

```
fix(parser): reject empty worktree spec with a clear error

Empty `<worktree></worktree>` blocks used to produce a
`None`-propagation panic deep inside the parser. We now surface
a `ParseError::EmptyWorktree` with the offending line number.

Fixes #487
```

## 10. Reviewer Expectations

- **First response** within 2 business days.
- Reviews cover: correctness, test coverage, security, performance,
  API stability, observability, and documentation.
- Maintainer privilege: squash-merge with the PR title as the squash
  subject and the PR body as the squash body. Override only when the
  history itself is meaningful (rare; discuss in the PR).

## 11. Release Process

AgilePlus follows semver. Releases are cut from `main` by the
release-please GitHub App configured in
`.github/release-please-config.json`. The maintainer approves the
release PR, which is auto-generated and bumps versions, CHANGELOG,
and tags.

## 12. Getting Help

- **Discord**: `#agileplus` on the Phenotype Discord.
- **Discussions**: GitHub Discussions → *Q&A*.
- **Office hours**: Tuesdays 15:00 UTC, calendar link in the
  pinned issue.

Welcome aboard — we are glad you are here.
