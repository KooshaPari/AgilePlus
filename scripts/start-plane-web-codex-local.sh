#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
PLANE_DIR="$PROJ_DIR/.agileplus/plane"
PLANE_WEB_DIR="$PLANE_DIR/apps/web"

source "$SCRIPT_DIR/resolve-local-ports-codex-local.sh"
source "$PROJ_DIR/.agileplus/runtime/local-ports.env"

cd "$PLANE_WEB_DIR"

export VITE_API_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_API_PORT}"
export VITE_WEB_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export VITE_ADMIN_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export VITE_ADMIN_BASE_PATH="/god-mode"
export VITE_SPACE_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export VITE_SPACE_BASE_PATH="/spaces"
export VITE_LIVE_BASE_URL="http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}"
export VITE_LIVE_BASE_PATH="/live"

exec pnpm exec react-router dev --host 127.0.0.1 --port "${AGILEPLUS_PLANE_WEB_PORT}"
