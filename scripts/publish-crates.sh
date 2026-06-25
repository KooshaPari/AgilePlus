#!/usr/bin/env bash
# Publish AgilePlus workspace crates to crates.io in dependency order.
set -euo pipefail

if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
  echo "error: CARGO_REGISTRY_TOKEN is required" >&2
  exit 1
fi

# Leaf-to-root order for agileplus-cli path dependencies.
PACKAGES=(
  agileplus-domain
  agileplus-error-core
  agileplus-telemetry
  agileplus-git
  agileplus-github
  agileplus-graph
  agileplus-triage
  agileplus-plane
  agileplus-sqlite
  agileplus-application
  agileplus-cli
)

for pkg in "${PACKAGES[@]}"; do
  if ! cargo metadata --format-version=1 --no-deps 2>/dev/null | grep -q "\"name\":\"${pkg}\""; then
    echo "skip $pkg (not in workspace metadata)"
    continue
  fi
  echo "==> publishing $pkg"
  if cargo publish -p "$pkg" --locked --allow-dirty; then
    echo "published $pkg"
  else
    echo "note: $pkg publish skipped (already published or blocked)"
  fi
done
