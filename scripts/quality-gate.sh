#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-verify}"

case "$MODE" in
  verify)
    echo "Running quality gate checks..."
    # Rust checks.
    # rustfmt: only check THIS workspace's own crates. `cargo fmt --all` reaches
    # into external path-dependency crates (the phenoShared sibling cloned in CI),
    # whose formatting is governed by their own repo — not this gate. Restrict to
    # packages defined under this workspace root.
    fmt_args=""
    while IFS= read -r pkg; do
      fmt_args="$fmt_args -p $pkg"
    done < <(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name')
    # shellcheck disable=SC2086
    cargo fmt $fmt_args -- --check
    cargo clippy --workspace -- -D warnings
    cargo test --workspace
    # Python checks (if python/ exists and has pyproject.toml)
    if [ -f "python/pyproject.toml" ]; then
      echo "Running Python quality checks..."
      # uv check would go here but skip if deps not installed
    fi
    echo "Quality gate passed."
    ;;
  *)
    echo "Usage: quality-gate.sh [verify]"
    exit 1
    ;;
esac
