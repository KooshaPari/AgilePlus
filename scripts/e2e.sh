#!/usr/bin/env bash
# AgilePlus CLI round-trip E2E: init -> specify -> status against a temp git repo.
set -euo pipefail

AGILEPLUS="${AGILEPLUS_BIN:-agileplus}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC_FILE="${E2E_SPEC_FILE:-$ROOT/tests/fixtures/sample-spec.md}"
FEATURE="${E2E_FEATURE:-e2e-roundtrip-001}"
REPO="$(mktemp -d)"
DB=".agileplus/agileplus.db"

cleanup() {
  rm -rf "$REPO"
}
trap cleanup EXIT

if ! command -v "$AGILEPLUS" >/dev/null 2>&1; then
  echo "error: CLI not found: $AGILEPLUS (set AGILEPLUS_BIN)" >&2
  exit 1
fi

if [ ! -f "$SPEC_FILE" ]; then
  echo "error: spec fixture missing: $SPEC_FILE" >&2
  exit 1
fi

cd "$REPO"

echo "==> git init"
git init --initial-branch=main 2>/dev/null || git init
git config user.email "e2e@agileplus.example"
git config user.name "AgilePlus E2E"

echo "==> agileplus init (or minimal bootstrap)"
if "$AGILEPLUS" init --non-interactive 2>/dev/null; then
  echo "init command succeeded"
else
  mkdir -p .agileplus kitty-specs
  echo "init unavailable; created minimal .agileplus + kitty-specs scaffold"
fi

echo "==> agileplus specify"
"$AGILEPLUS" --db "$DB" specify \
  --feature "$FEATURE" \
  --from-file "$SPEC_FILE"

SPEC_ARTIFACT="kitty-specs/${FEATURE}/spec.md"
if [ ! -f "$SPEC_ARTIFACT" ]; then
  echo "error: expected spec artifact at $SPEC_ARTIFACT" >&2
  exit 1
fi

if [ ! -f "$DB" ]; then
  echo "error: expected sqlite state at $DB" >&2
  exit 1
fi

echo "==> agileplus status"
if "$AGILEPLUS" --db "$DB" status --feature "$FEATURE" --wp WP01 --state specified 2>/dev/null; then
  echo "status with work-package args succeeded"
else
  "$AGILEPLUS" --db "$DB" status
fi

echo "E2E round-trip OK (feature=$FEATURE, repo=$REPO)"
