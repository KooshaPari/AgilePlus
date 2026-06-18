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
    fmt_pkgs=$(cargo metadata --no-deps --format-version 1 \
      | python3 -c 'import json,sys; print(" ".join(f"-p {p[\"name\"]}" for p in json.load(sys.stdin)["packages"]))')
    # shellcheck disable=SC2086
    cargo fmt $fmt_pkgs -- --check
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
