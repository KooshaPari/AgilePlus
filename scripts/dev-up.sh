#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

source "$SCRIPT_DIR/resolve-local-ports.sh"

LOG_DIR="$PROJ_DIR/.agileplus/logs"
mkdir -p "$LOG_DIR"

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
echo "  pc ctl:   ${AGILEPLUS_PROCESS_COMPOSE_PORT}"

if [[ "${1:-}" == "--foreground" ]]; then
  exec env PC_PORT_NUM="${AGILEPLUS_PROCESS_COMPOSE_PORT}" process-compose up -f process-compose.yml
fi

exec env PC_PORT_NUM="${AGILEPLUS_PROCESS_COMPOSE_PORT}" process-compose up \
  -f process-compose.yml \
  -t=false \
  -D \
  -L "$LOG_DIR/process-compose.log"
