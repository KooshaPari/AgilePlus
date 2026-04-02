#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

source "$SCRIPT_DIR/resolve-local-ports-codex-local.sh"
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
  git clone --depth=1 --branch "$PLANE_REF" https://github.com/makeplane/plane.git "$PLANE_DIR"
else
  echo "Plane already present at $PLANE_DIR"
fi

if [[ ! -d "$PLANE_API_DIR" || ! -d "$PLANE_WEB_DIR" ]]; then
  echo "Plane monorepo layout is incomplete." >&2
  exit 1
fi

if ! command -v pnpm >/dev/null 2>&1 && command -v corepack >/dev/null 2>&1; then
  corepack enable pnpm >/dev/null 2>&1 || true
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required to install Plane dependencies." >&2
  exit 1
fi

if [[ ! -d "$PLANE_DIR/node_modules" ]]; then
  (cd "$PLANE_DIR" && pnpm install)
else
  echo "Plane monorepo dependencies already present."
fi

if [[ ! -x "$PLANE_API_DIR/.venv/bin/python" ]]; then
  (cd "$PLANE_API_DIR" && python3 -m venv .venv)
fi

if ! "$PLANE_API_DIR/.venv/bin/python" -c "import django, dj_database_url, corsheaders, requests" >/dev/null 2>&1; then
  (
    cd "$PLANE_API_DIR"
    source .venv/bin/activate
    pip install -r requirements/local.txt requests
  )
fi

plane_workspace_needs_build="$(
  python3 - "$PLANE_DIR" <<'PY'
import json
import pathlib
import sys

plane_dir = pathlib.Path(sys.argv[1])
missing = []
for package_json in sorted((plane_dir / "packages").glob("*/package.json")):
    data = json.loads(package_json.read_text())
    scripts = data.get("scripts", {})
    if "build" not in scripts:
        continue
    for field in ("main", "module", "types"):
        rel = data.get(field)
        if not rel:
            continue
        candidate = package_json.parent / rel
        if not candidate.exists():
            missing.append(f"{data['name']}:{candidate.relative_to(plane_dir)}")
            break

if missing:
    print("\n".join(missing))
PY
)"

if [[ -n "$plane_workspace_needs_build" ]]; then
  echo "Plane workspace build required for:"
  printf '  %s\n' $plane_workspace_needs_build
  (cd "$PLANE_DIR" && pnpm build)
else
  echo "Plane workspace package builds already present."
fi

GENERATED_SECRET_KEY="$(
  awk -F= '/^SECRET_KEY=/{print $2}' "$API_ENV_FILE" 2>/dev/null | tail -n 1 | tr -d '"' || true
)"
if [[ -z "$GENERATED_SECRET_KEY" ]]; then
  GENERATED_SECRET_KEY="$(openssl rand -hex 32)"
fi

cat >"$API_ENV_FILE" <<EOF
DEBUG=1
DATABASE_URL=postgresql://agileplus:${PLANE_POSTGRES_PASSWORD:-agileplus-dev}@127.0.0.1:${AGILEPLUS_POSTGRES_PORT}/plane
REDIS_URL=redis://127.0.0.1:${AGILEPLUS_REDIS_PORT}
SECRET_KEY=${GENERATED_SECRET_KEY}
WEB_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
CORS_ALLOWED_ORIGINS=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT},http://127.0.0.1:${AGILEPLUS_API_PORT}
APP_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
LIVE_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
LIVE_BASE_PATH=/live
AWS_S3_ENDPOINT_URL=http://127.0.0.1:${AGILEPLUS_MINIO_PORT}
AWS_ACCESS_KEY_ID=agileplus
AWS_SECRET_ACCESS_KEY=agileplus-dev
AWS_S3_BUCKET_NAME=uploads
USE_MINIO=1
AMQP_URL=memory://
EOF

cat >"$WEB_ENV_FILE" <<EOF
VITE_API_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_API_PORT}
VITE_WEB_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
VITE_ADMIN_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
VITE_ADMIN_BASE_PATH=/god-mode
VITE_SPACE_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
VITE_SPACE_BASE_PATH=/spaces
VITE_LIVE_BASE_URL=http://127.0.0.1:${AGILEPLUS_PLANE_WEB_PORT}
VITE_LIVE_BASE_PATH=/live
EOF

echo "Plane API dir: $PLANE_API_DIR"
echo "Plane web dir: $PLANE_WEB_DIR"
echo "Plane API port: ${AGILEPLUS_PLANE_API_PORT}"
echo "Plane web port: ${AGILEPLUS_PLANE_WEB_PORT}"
