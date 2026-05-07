#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

source "$SCRIPT_DIR/resolve-local-ports.sh"

LOG_DIR="$PROJ_DIR/.agileplus/logs"
COMPOSE_FILE="$PROJ_DIR/process-compose.local.yml"
mkdir -p \
  "$PROJ_DIR/.agileplus/minio-data" \
  "$PROJ_DIR/.agileplus/neo4j/data" \
  "$PROJ_DIR/.agileplus/neo4j/import" \
  "$PROJ_DIR/.agileplus/neo4j/logs" \
  "$PROJ_DIR/.agileplus/neo4j/plugins"
mkdir -p "$LOG_DIR"

require_port_free() {
  local name="$1"
  local port="$2"
  if lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Port collision: ${name} wants ${port}, but it is already in use." >&2
    echo "Adjust the AGILEPLUS_*_PORT env vars or rerun scripts/resolve-local-ports.sh after freeing the port." >&2
    return 1
  fi
}

if [[ ! -f "$PROJ_DIR/.agileplus/runtime/local-ports.env" ]]; then
  echo "Missing .agileplus/runtime/local-ports.env after port resolution." >&2
  exit 1
fi

source "$PROJ_DIR/.agileplus/runtime/local-ports.env"

require_port_free "postgres" "$AGILEPLUS_POSTGRES_PORT"
require_port_free "redis" "$AGILEPLUS_REDIS_PORT"
require_port_free "nats" "$AGILEPLUS_NATS_PORT"
require_port_free "nats-http" "$AGILEPLUS_NATS_HTTP_PORT"
require_port_free "neo4j-bolt" "$AGILEPLUS_NEO4J_PORT"
require_port_free "neo4j-http" "$AGILEPLUS_NEO4J_HTTP_PORT"
require_port_free "minio" "$AGILEPLUS_MINIO_PORT"
require_port_free "minio-console" "$AGILEPLUS_MINIO_CONSOLE_PORT"
require_port_free "plane-api" "$AGILEPLUS_PLANE_API_PORT"
require_port_free "plane-web" "$AGILEPLUS_PLANE_WEB_PORT"
require_port_free "agileplus-api" "$AGILEPLUS_API_PORT"

bash "$SCRIPT_DIR/setup-plane.sh"

echo "=== AgilePlus local stack ==="
echo "Using ports from .agileplus/runtime/local-ports.env"
echo "  postgres: ${AGILEPLUS_POSTGRES_PORT}"
echo "  redis:    ${AGILEPLUS_REDIS_PORT}"
echo "  nats:     ${AGILEPLUS_NATS_PORT}"
echo "  nats ui:  ${AGILEPLUS_NATS_HTTP_PORT}"
echo "  neo4j:    ${AGILEPLUS_NEO4J_PORT}"
echo "  minio:    ${AGILEPLUS_MINIO_PORT}"
echo "  plane:    ${AGILEPLUS_PLANE_API_PORT}"
echo "  web:      ${AGILEPLUS_PLANE_WEB_PORT}"
echo "  api:      ${AGILEPLUS_API_PORT}"

if [[ "${1:-}" == "--foreground" ]]; then
  exec process-compose up -e .agileplus/runtime/local-ports.env -f "$COMPOSE_FILE"
fi

exec process-compose up \
  -e .agileplus/runtime/local-ports.env \
  -f "$COMPOSE_FILE" \
  -t=false \
  -D \
  -L "$LOG_DIR/process-compose.log"
