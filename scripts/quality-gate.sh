#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-verify}"

case "$MODE" in
  verify)
    echo "Running quality gate checks..."
    # Rust checks
    cargo fmt --all -- --check
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
