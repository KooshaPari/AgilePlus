# justfile for AgilePlus
# 21-crate Cargo workspace plus a top-level `rust/` mirror. Use `just`
# (or `just <recipe>`) to run recipes.
# `just` is the casey/just command runner: https://just.systems

set shell := ["bash", "-uc"]
set dotenv-load

# ---- Detected features (eval once, exported as env vars) ----

export HAS_CARGO := `test -f Cargo.toml && echo 1 || echo 0`
export HAS_RUST_MIRROR := `test -f rust/Cargo.toml && echo 1 || echo 0`
export HAS_DASHBOARD := `test -d crates/agileplus-dashboard && echo 1 || echo 0`
export HAS_DASHBOARD_PACKAGE := `test -f crates/agileplus-dashboard/package.json && echo 1 || echo 0`
export HAS_DENY := `test -f deny.toml && echo 1 || echo 0`
export JS_RUNNER := `command -v bun >/dev/null 2>&1 && echo bun || (command -v pnpm >/dev/null 2>&1 && echo pnpm || echo npm)`

# ---- Default recipe: list available recipes ----

default: list

# Show all available recipes
list:
    @just --list

# ---- Dev: cargo watch on the workspace, recompiling on file changes ----

dev:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v cargo-watch >/dev/null 2>&1; then
      echo "error: cargo-watch is not installed (run: cargo install cargo-watch)" >&2
      exit 1
    fi

    cargo watch --quiet --clear --workspace --shell "cargo check --workspace --all-targets"

# ---- Build: build release artifacts for the workspace ----

build:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_CARGO}" = "1" ]; then
      cargo build --workspace --release
    fi

    if [ "${HAS_RUST_MIRROR}" = "1" ]; then
      cargo build --manifest-path rust/Cargo.toml --release
    fi

# ---- Test: run the full test suite ----

test:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_CARGO}" = "1" ]; then
      cargo test --workspace --all-features
    fi

    if [ "${HAS_RUST_MIRROR}" = "1" ]; then
      cargo test --manifest-path rust/Cargo.toml --all-features
    fi

# ---- Lint: rustfmt + clippy with -D warnings; cargo-deny if configured ----

lint:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_CARGO}" = "1" ]; then
      cargo fmt --all -- --check
      cargo clippy --workspace --all-targets --all-features -- -D warnings
    fi

    if [ "${HAS_RUST_MIRROR}" = "1" ]; then
      cargo fmt --manifest-path rust/Cargo.toml --all -- --check
      cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features -- -D warnings
    fi

    if [ "${HAS_DENY}" = "1" ] && command -v cargo-deny >/dev/null 2>&1; then
      cargo deny check
    fi

# ---- Fmt: apply rustfmt in place ----

fmt:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_CARGO}" = "1" ]; then
      cargo fmt --all
    fi

    if [ "${HAS_RUST_MIRROR}" = "1" ]; then
      cargo fmt --manifest-path rust/Cargo.toml --all
    fi

# ---- Clean: remove generated artifacts ----

clean:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ "${HAS_CARGO}" = "1" ]; then
      cargo clean
    fi

    if [ "${HAS_RUST_MIRROR}" = "1" ]; then
      rm -rf rust/target
    fi

    if [ "${HAS_DASHBOARD}" = "1" ]; then
      rm -rf crates/agileplus-dashboard/dist crates/agileplus-dashboard/node_modules crates/agileplus-dashboard/.turbo
    fi
