#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
PORTS_FILE="$PROJ_DIR/.agileplus/runtime/local-ports.env"

if [[ ! -f "$PORTS_FILE" ]]; then
  echo "Missing $PORTS_FILE. Run scripts/dev-up.sh first." >&2
  exit 1
fi

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
EVIDENCE_DIR="$PROJ_DIR/.agileplus/evidence/local-boot-$TIMESTAMP"
mkdir -p "$EVIDENCE_DIR"

cp "$PORTS_FILE" "$EVIDENCE_DIR/local-ports.env"

if process-compose process list >"$EVIDENCE_DIR/process-compose.txt" 2>&1; then
  :
else
  echo "process-compose process list failed" >>"$EVIDENCE_DIR/process-compose.txt"
fi

if bash "$SCRIPT_DIR/local-health-check.sh" >"$EVIDENCE_DIR/health-check.txt" 2>&1; then
  health_status="passed"
else
  health_status="failed"
fi

for log_name in orb-containers nats neo4j minio plane-api plane-web plane-worker plane-beat agileplus-api; do
  log_path="$PROJ_DIR/.agileplus/logs/${log_name}.log"
  if [[ -f "$log_path" ]]; then
    cp "$log_path" "$EVIDENCE_DIR/${log_name}.log"
  fi
done

cat >"$EVIDENCE_DIR/summary.txt" <<EOF
timestamp=$TIMESTAMP
health_status=$health_status
ports_file=local-ports.env
process_snapshot=process-compose.txt
health_output=health-check.txt
EOF

if [[ "$health_status" == "passed" ]]; then
  echo "Captured local boot evidence in $EVIDENCE_DIR"
  exit 0
fi

echo "Captured local boot evidence in $EVIDENCE_DIR, but health checks failed." >&2
exit 1
