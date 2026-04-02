#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

LOG_DIR="$PROJ_DIR/.agileplus/logs"
COMPOSE_FILE="$PROJ_DIR/process-compose.yml"
mkdir -p "$LOG_DIR"

require_port_free() {
  local name="$1"
  local port="$2"
  if lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Port collision: ${name} wants ${port}, but it is already in use." >&2
    return 1
  fi
}

echo "=== AgilePlus local stack ==="
echo "Using the fixed root compose ports"
echo "  postgres: 5432"
echo "  redis:    6379"
echo "  nats:     4222"
echo "  nats ui:  8222"
echo "  neo4j:    7687"
echo "  minio:    9000"
echo "  web:      3100"
echo "  plane:    8000"
echo "  api:      3000"

require_port_free "postgres" 5432
require_port_free "redis" 6379
require_port_free "nats" 4222
require_port_free "nats ui" 8222
require_port_free "neo4j" 7687
require_port_free "minio" 9000
require_port_free "minio-console" 9001
require_port_free "plane-api" 8000
require_port_free "plane-web" 3100
require_port_free "agileplus-api" 3000

bash "$SCRIPT_DIR/setup-plane.sh"

if [[ "${1:-}" == "--foreground" ]]; then
  exec process-compose up -f "$COMPOSE_FILE"
fi

exec process-compose up \
  -f "$COMPOSE_FILE" \
  -t=false \
  -D \
  -L "$LOG_DIR/process-compose.log"
