#!/usr/bin/env bash
# Preview the built VitePress documentation site.
# Usage: bash scripts/docs-preview.sh [--port <port>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$REPO_ROOT"

PORT="${1:-5173}"
# Support --port <port> flag
if [[ "${1:-}" == "--port" ]] && [[ -n "${2:-}" ]]; then
  PORT="$2"
fi

DOCS_DIR="$REPO_ROOT/docs"
DIST_DIR="$DOCS_DIR/.vitepress/dist"

if [[ ! -d "$DIST_DIR" ]]; then
  echo "Build artefacts not found at $DIST_DIR" >&2
  echo "Run 'bash scripts/docs-build.sh' first." >&2
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

echo "=== Previewing docs at http://localhost:$PORT ==="
cd "$DOCS_DIR"
"$PM" run docs:preview -- --port "$PORT"
