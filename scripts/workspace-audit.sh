#!/bin/bash
# workspace-audit.sh
# Checks for missing path dependency targets in the workspace.
# eco-027: Cargo Workspace Cleanup

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EXIT_CODE=0

echo "=== Workspace Path Dependency Audit ==="
echo "Scanning: $REPO_ROOT/Cargo.toml"

# Extract path dependencies from workspace Cargo.toml
path_deps=$(grep -E 'path\s*=' "$REPO_ROOT/Cargo.toml" 2>/dev/null || true)

if [ -z "$path_deps" ]; then
    echo "No path dependencies found in workspace."
    exit 0
fi

# Check each member exists
while IFS= read -r line; do
    member=$(echo "$line" | tr -d '"' | tr -d ',' | sed 's/.*= *//')
    member_path="$REPO_ROOT/$member"
    if [ ! -d "$member_path" ]; then
        echo "MISSING: $member (path: $member_path)"
        EXIT_CODE=1
    else
        echo "OK:    $member"
    fi
done <<< "$path_deps"

# Also check workspace.members list
members=$(grep -A 100 'members\s*=' "$REPO_ROOT/Cargo.toml" | grep -E '^\s*"' | tr -d '"' | tr -d ',' | sed 's/^[[:space:]]*//' || true)

if [ -n "$members" ]; then
    echo ""
    echo "=== Workspace Members Check ==="
    while IFS= read -r member; do
        [ -z "$member" ] && continue
        member_path="$REPO_ROOT/$member"
        if [ ! -d "$member_path" ]; then
            echo "MISSING: $member (path: $member_path)"
            EXIT_CODE=1
        fi
    done <<< "$members"
fi

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "error: workspace audit found missing path dependencies. Fix before merging."
    exit 1
fi

echo ""
echo "ok: all workspace members and path dependencies are present."
exit 0
