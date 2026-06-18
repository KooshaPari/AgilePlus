#!/usr/bin/env bash
set -euo pipefail

changed_files() {
  if [ -n "${CHANGED_FILES:-}" ]; then
    printf '%s\n' "$CHANGED_FILES"
  elif [ -n "${CHANGED_FILES_FILE:-}" ] && [ -f "$CHANGED_FILES_FILE" ]; then
    cat "$CHANGED_FILES_FILE"
  elif git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git diff --name-only origin/main...HEAD
  else
    return 1
  fi
}

files="$(changed_files | sed '/^[[:space:]]*$/d' || true)"

if [ -z "$files" ]; then
  echo "No changed files available for governance spec-first gate." >&2
  exit 1
fi

missing=0
require_touch() {
  local label="$1"
  local pattern="$2"
  if ! printf '%s\n' "$files" | grep -Eiq "$pattern"; then
    echo "Missing required governance document touch: $label" >&2
    missing=1
  fi
}

require_touch "CHANGELOG" '(^|/)CHANGELOG([^/]*\.md|\.md)?$'
require_touch "ADR" '(^|/)(adr|adrs|architecture/decisions)/|(^|/)[0-9]{4}.*adr.*\.md$|(^|/)ADR[^/]*\.md$'
require_touch "QA matrix" '(^|/)(qa[-_ ]?matrix|quality[-_ ]?matrix)[^/]*\.md$|(^|/)qa/matrix[^/]*\.md$'

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "Governance spec-first documents verified."
