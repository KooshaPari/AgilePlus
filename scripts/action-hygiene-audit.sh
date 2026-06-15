#!/bin/bash
# action-hygiene-audit.sh
# Checks all GitHub Actions workflow files for unpinned third-party actions.
# eco-011 FR-5: Action hygiene audit
# A third-party action is considered pinned if it uses a 40-character SHA ref.
# First-party actions (KooshaPari/*, actions/*) are exempt.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKFLOWS_DIR="$REPO_ROOT/.github/workflows"
EXIT_CODE=0
PINS_FOUND=0
UNPINS_FOUND=0

if [ ! -d "$WORKFLOWS_DIR" ]; then
    echo "error: workflows directory not found: $WORKFLOWS_DIR"
    exit 1
fi

echo "=== Action Hygiene Audit ==="
echo "Scanning: $WORKFLOWS_DIR"
echo ""

for file in "$WORKFLOWS_DIR"/*.yml; do
    [ -e "$file" ] || continue
    basename_file=$(basename "$file")
    unpinned=()
    while IFS= read -r line; do
        # Skip comments and blank lines
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ -z "$line" ]] && continue
        # Extract the action reference after 'uses:'
        action_ref=$(echo "$line" | sed -n 's/.*uses:[[:space:]]*//p')
        [ -z "$action_ref" ] && continue
        # Skip first-party actions (actions/* and KooshaPari/*)
        if [[ "$action_ref" =~ ^actions/ || "$action_ref" =~ ^KooshaPari/ ]]; then
            continue
        fi
        # Check if the action is pinned to a 40-char SHA
        # SHA pattern: exactly 40 hex characters after @
        if [[ "$action_ref" =~ @[a-f0-9]{40} ]]; then
            PINS_FOUND=$((PINS_FOUND + 1))
        else
            unpinned+=("$action_ref")
            UNPINS_FOUND=$((UNPINS_FOUND + 1))
        fi
    done < <(grep -n 'uses:' "$file" 2>/dev/null || true)

    if [ ${#unpinned[@]} -gt 0 ]; then
        echo "FAIL: $basename_file"
        for ref in "${unpinned[@]}"; do
            echo "  unpinned: $ref"
        done
        EXIT_CODE=1
    fi
done

echo ""
echo "=== Summary ==="
echo "pinned actions:   $PINS_FOUND"
echo "unpinned actions: $UNPINS_FOUND"

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "error: found $UNPINS_FOUND unpinned third-party action(s)."
    echo "       All third-party actions must be pinned to a 40-character SHA ref."
    echo "       Example: uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5"
    exit 1
fi

echo "ok: all third-party actions are pinned to SHA refs."
exit 0
