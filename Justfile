# Justfile - task runner for AgilePlus

set dotenv-load

default:
    @just --list

ci: fmt lint test audit docs

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --all --check

test:
    cargo test --workspace --all-features

audit:
    cargo deny check

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
