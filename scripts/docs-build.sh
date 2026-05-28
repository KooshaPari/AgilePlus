#!/usr/bin/env bash
# Build the VitePress documentation site.
# Usage: bash scripts/docs-build.sh [--ci]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$REPO_ROOT"

CI_MODE=""
if [[ "${1:-}" == "--ci" ]]; then
  CI_MODE="1"
fi

DOCS_DIR="$REPO_ROOT/docs"
PACKAGE_JSON="$DOCS_DIR/package.json"

if [[ ! -f "$PACKAGE_JSON" ]]; then
  echo "ERROR: $PACKAGE_JSON not found" >&2
  exit 1
fi

# Detect package manager
detect_pm() {
  if command -v bun &>/dev/null; then
    echo "bun"
  elif command -v npm &>/dev/null; then
    echo "npm"
  else
    echo "ERROR: neither bun nor npm is available" >&2
    exit 1
  fi
}

PM="$(detect_pm)"

# Ensure dependencies are installed
if [[ ! -d "$DOCS_DIR/node_modules" ]]; then
  echo "Installing docs dependencies ..."
  (cd "$DOCS_DIR" && "$PM" install)
fi

if [[ -n "$CI_MODE" ]]; then
  echo "::group::docs:build"
fi

echo "=== Building VitePress documentation ==="
cd "$DOCS_DIR"
"$PM" run docs:build

if [[ -n "$CI_MODE" ]]; then
  echo "::endgroup::"
fi

echo "Docs build complete."