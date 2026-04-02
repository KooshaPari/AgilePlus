#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
PORTS_FILE="$PROJ_DIR/.agileplus/runtime/local-ports.env"
EVIDENCE_ROOT="$PROJ_DIR/.agileplus/evidence"
STAMP="$(date +%Y%m%d-%H%M%S)"
DEST="$EVIDENCE_ROOT/$STAMP"

mkdir -p "$DEST"

cp "$PORTS_FILE" "$DEST/local-ports.env"

bash "$SCRIPT_DIR/local-health-check-codex-local.sh" >"$DEST/health-check.txt"

docker ps --format '{{.Names}} {{.Status}} {{.Ports}}' \
  | rg '^agileplus-(postgres|dragonfly)\b' >"$DEST/docker.txt"

pgrep -fl \
  'process-compose|nats-server|minio|manage.py runserver|react-router dev|start-plane-api-codex-local|start-plane-web-codex-local' \
  >"$DEST/processes.txt" || true

for log_name in process-compose.log orb-containers.log plane-api-bootstrap.log plane-api.log plane-web.log; do
  if [[ -f "$PROJ_DIR/.agileplus/logs/$log_name" ]]; then
    cp "$PROJ_DIR/.agileplus/logs/$log_name" "$DEST/$log_name"
  fi
done

printf '%s\n' "$DEST"
