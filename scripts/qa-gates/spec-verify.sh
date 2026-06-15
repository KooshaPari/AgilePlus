#!/usr/bin/env bash
set -euo pipefail

SPEC_FILE="${SPEC_FILE:-SPEC.md}"
BODY="${PR_BODY:-}"

refs="$(printf '%s\n' "$BODY" | grep --color=never -Eo '(FR|NFR)-[0-9]+([.][0-9]+)?' | sort -u || true)"

if [ -z "$refs" ]; then
  echo "No FR/NFR references found in PR body; spec verification skipped."
  exit 0
fi

if [ ! -f "$SPEC_FILE" ]; then
  echo "$SPEC_FILE is required when PR body references FR/NFR IDs." >&2
  exit 1
fi

missing=0
while IFS= read -r ref; do
  [ -z "$ref" ] && continue
  if ! grep -Fq "$ref" "$SPEC_FILE"; then
    echo "Missing $ref in $SPEC_FILE" >&2
    missing=1
  fi
done <<< "$refs"

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "SPEC references verified."
