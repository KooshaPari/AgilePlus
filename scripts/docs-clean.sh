#!/usr/bin/env bash
# Clean VitePress build artefacts and cache.
# Usage: bash scripts/docs-clean.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
DOCS_DIR="$REPO_ROOT/docs"

if [[ ! -d "$DOCS_DIR" ]]; then
  echo "ERROR: docs directory not found at $DOCS_DIR" >&2
  exit 1
fi

echo "=== Cleaning docs build artefacts ==="

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

cd "$DOCS_DIR"
echo "Running docs:clean ..."
"$PM" run docs:clean

# Also remove .vitepress/dist even if rimraf path differs
rm -rf "$DOCS_DIR/.vitepress/dist"
rm -rf "$DOCS_DIR/.vitepress/cache"
rm -rf "$DOCS_DIR/node_modules/.vite"

echo "Docs clean complete."