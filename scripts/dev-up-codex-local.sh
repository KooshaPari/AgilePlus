#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

source "$SCRIPT_DIR/resolve-local-ports-codex-local.sh"

LOG_DIR="$PROJ_DIR/.agileplus/logs"
COMPOSE_FILE="$PROJ_DIR/process-compose.codex-local.yml"
mkdir -p "$PROJ_DIR/.agileplus/minio-data" "$LOG_DIR"

require_port_free() {
  local name="$1"
  local port="$2"
  if lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Port collision: ${name} wants ${port}, but it is already in use." >&2
    return 1
  fi
}

source "$PROJ_DIR/.agileplus/runtime/local-ports.env"

require_port_free "nats" "$AGILEPLUS_NATS_PORT"
require_port_free "nats-http" "$AGILEPLUS_NATS_HTTP_PORT"
require_port_free "minio" "$AGILEPLUS_MINIO_PORT"
require_port_free "minio-console" "$AGILEPLUS_MINIO_CONSOLE_PORT"
require_port_free "plane-api" "$AGILEPLUS_PLANE_API_PORT"
require_port_free "plane-web" "$AGILEPLUS_PLANE_WEB_PORT"

bash "$SCRIPT_DIR/setup-plane-codex-local.sh"
bash "$SCRIPT_DIR/orb-up-codex-local.sh" >"$LOG_DIR/orb-containers.log" 2>&1
bash "$SCRIPT_DIR/start-plane-api-codex-local.sh" --prepare-only >"$LOG_DIR/plane-api-bootstrap.log" 2>&1

echo "=== AgilePlus codex-local stack ==="
echo "  postgres: ${AGILEPLUS_POSTGRES_PORT}"
echo "  redis:    ${AGILEPLUS_REDIS_PORT}"
echo "  nats:     ${AGILEPLUS_NATS_PORT}"
echo "  nats ui:  ${AGILEPLUS_NATS_HTTP_PORT}"
echo "  minio:    ${AGILEPLUS_MINIO_PORT}"
echo "  plane:    ${AGILEPLUS_PLANE_API_PORT}"
echo "  web:      ${AGILEPLUS_PLANE_WEB_PORT}"

if [[ "${1:-}" == "--foreground" ]]; then
  exec process-compose up --no-server -e .agileplus/runtime/local-ports.env -f "$COMPOSE_FILE"
fi

nohup process-compose up \
  --no-server \
  -e .agileplus/runtime/local-ports.env \
  -f "$COMPOSE_FILE" \
  -t=false \
  >"$LOG_DIR/process-compose.log" 2>&1 &
