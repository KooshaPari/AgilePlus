# Justfile - task runner for AgilePlus
# Anti-pattern checks consolidated from retired xtask-anti-patterns crate.

set dotenv-load

default:
    @just --list

ci: fmt lint test audit docs

# Run the full grading gate (alias of `ci`)
[private]
grade: ci

# Quick grading mode — build, lint, fmt, test only
grade-fast:
    cargo build --workspace --all-targets
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo fmt --all --check
    cargo test --workspace

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --all --check

test:
    cargo test --workspace --all-features

audit:
    cargo deny check

# Consolidated anti-pattern checks (replaces xtask-anti-patterns crate)
check:
    cargo build --workspace --all-targets
    cargo test --workspace
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo deny check
    cargo machete

machete:
    cargo machete

# Security audit: aggregates cargo audit, secret scan, and dep checks
# eco-030 FR-6: make security-audit equivalent
security-audit:
    cargo audit
    cargo deny check
    @echo "Security audit complete. If trufflehog is installed, run: just secret-scan"

# Quick secret scan on current repo (requires trufflehog CLI)
secret-scan:
    @which trufflehog > /dev/null || (echo "trufflehog not found. Install: brew install trufflesecurity/trufflehog/trufflehog" && exit 1)
    trufflehog filesystem . --only-verified --no-update

docs:
    cargo doc --workspace --all-features --no-deps

release:
    cargo build --workspace --all-targets --release

crates:
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sort

test-crate crate:
    @cargo metadata --no-deps --format-version 1 | jq -e --arg crate "{{crate}}" 'any(.packages[].name; . == $crate)' >/dev/null
    cargo test -p "{{crate}}" --all-features

test-agileplus-api: (test-crate "agileplus-api")

test-agileplus-cli: (test-crate "agileplus-cli")

# ── Intent Converter ─────────────────────────────────────────────────────────

# Convert a prompt to an intent graph JSON (one-shot)
# Usage: just convert-intent "Add dark mode"
#        just convert-intent --store "Add dark mode"
convert-intent prompt store="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{store}}" = "--store" ]; then
        cargo run -p agileplus-mcp-intent -- convert --store "{{prompt}}"
    else
        cargo run -p agileplus-mcp-intent -- convert "{{prompt}}"
    fi

# Convert a prompt and store the resulting graph in the AgilePlus database
convert-intent-store prompt="Add dark mode to settings":
    cargo run -p agileplus-mcp-intent -- convert --store "{{prompt}}"

# Run the intent converter HTTP server
run-intent-http port="8080":
    cargo run -p agileplus-mcp-intent -- http --port {{port}}

# Run the intent converter MCP server (stdio JSON-RPC)
run-intent-mcp:
    cargo run -p agileplus-mcp-intent -- mcp
