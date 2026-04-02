#!/usr/bin/env bash
set -euo pipefail

# Bootstrap Plane.so Community Edition for local AgilePlus development.
# This script clones and configures plane.so to run natively via process-compose.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

source "$SCRIPT_DIR/resolve-local-ports.sh"
source "$PROJ_DIR/.agileplus/runtime/local-ports.env"

PLANE_REF="${PLANE_REF:-v1.2.3}"
PLANE_DIR=".agileplus/plane"
PLANE_API_DIR="$PLANE_DIR/apps/api"
PLANE_WEB_DIR="$PLANE_DIR/apps/web"
API_ENV_FILE="$PLANE_API_DIR/.env"
WEB_ENV_FILE="$PLANE_WEB_DIR/.env"

echo "=== AgilePlus: Plane.so Local Setup ==="

mkdir -p .agileplus/logs .agileplus/evidence

if [[ ! -d "$PLANE_DIR/.git" ]]; then
  echo "Cloning Plane.so into $PLANE_DIR..."
  git clone --depth=1 --branch "$PLANE_REF" https://github.com/makeplane/plane.git "$PLANE_DIR"
else
  echo "Plane already present at $PLANE_DIR"
fi

if [[ ! -d "$PLANE_API_DIR" ]]; then
  echo "Missing Plane API directory: $PLANE_API_DIR" >&2
  exit 1
fi

if [[ ! -d "$PLANE_WEB_DIR" ]]; then
  echo "Missing Plane web directory: $PLANE_WEB_DIR" >&2
  exit 1
fi

if ! command -v pnpm >/dev/null 2>&1; then
  if command -v corepack >/dev/null 2>&1; then
    corepack enable pnpm >/dev/null 2>&1
  fi
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required to install Plane dependencies." >&2
  exit 1
fi

if [[ ! -d "$PLANE_DIR/node_modules" ]]; then
  echo "Installing Plane monorepo dependencies..."
  (cd "$PLANE_DIR" && pnpm install)
else
  echo "Plane monorepo dependencies already present."
fi

if [[ ! -x "$PLANE_API_DIR/.venv/bin/python" ]]; then
  echo "Creating Plane API virtualenv..."
  (
    cd "$PLANE_API_DIR"
    python3 -m venv .venv
  )
fi

if ! "$PLANE_API_DIR/.venv/bin/python" -c "import django" >/dev/null 2>&1; then
  echo "Installing Plane API Python dependencies..."
  (
    cd "$PLANE_API_DIR"
    source .venv/bin/activate
    pip install -r requirements/local.txt
  )
fi

GENERATED_SECRET_KEY="$(
  awk -F= '/^SECRET_KEY=/{print $2}' "$API_ENV_FILE" 2>/dev/null | tail -n 1 | tr -d '"' || true
)"
if [[ -z "$GENERATED_SECRET_KEY" ]]; then
  GENERATED_SECRET_KEY="$(openssl rand -hex 32)"
fi

cat >"$API_ENV_FILE" <<EOF
DEBUG=1
DATABASE_URL=postgresql://agileplus:${PLANE_POSTGRES_PASSWORD:-agileplus-dev}@localhost:${AGILEPLUS_POSTGRES_PORT}/plane
REDIS_URL=redis://localhost:${AGILEPLUS_REDIS_PORT}
SECRET_KEY=${GENERATED_SECRET_KEY}
WEB_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
CORS_ALLOWED_ORIGINS=http://localhost:${AGILEPLUS_PLANE_WEB_PORT},http://localhost:${AGILEPLUS_API_PORT}
APP_BASE_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
LIVE_BASE_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
LIVE_BASE_PATH=/live
AWS_S3_ENDPOINT_URL=http://localhost:${AGILEPLUS_MINIO_PORT}
AWS_ACCESS_KEY_ID=agileplus
AWS_SECRET_ACCESS_KEY=agileplus-dev
AWS_S3_BUCKET_NAME=uploads
USE_MINIO=1
EOF

cat >"$WEB_ENV_FILE" <<EOF
VITE_API_BASE_URL=http://localhost:${AGILEPLUS_PLANE_API_PORT}
VITE_WEB_BASE_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
VITE_ADMIN_BASE_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
VITE_ADMIN_BASE_PATH=/god-mode
VITE_SPACE_BASE_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
VITE_SPACE_BASE_PATH=/spaces
VITE_LIVE_BASE_URL=http://localhost:${AGILEPLUS_PLANE_WEB_PORT}
VITE_LIVE_BASE_PATH=/live
EOF

echo ""
echo "=== Plane Setup Complete ==="
echo "Plane API dir: $PLANE_API_DIR"
echo "Plane web dir: $PLANE_WEB_DIR"
echo "Plane API port: ${AGILEPLUS_PLANE_API_PORT}"
echo "Plane web port: ${AGILEPLUS_PLANE_WEB_PORT}"
