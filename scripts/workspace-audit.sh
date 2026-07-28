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

# Extract path dependencies only from the workspace.dependencies table.
# Cargo metadata tables (for example workspace.metadata.dist's
# `install-path`) also contain the substring `path =`, but are not dependency
# declarations and must never be interpreted as filesystem paths.
path_deps=$(awk '
    /^\[workspace\.dependencies\][[:space:]]*$/ { in_deps=1; next }
    /^\[/ { in_deps=0 }
    in_deps && $0 ~ /path[[:space:]]*=/ { print }
' "$REPO_ROOT/Cargo.toml" 2>/dev/null || true)

if [ -z "$path_deps" ]; then
    echo "No path dependencies found in workspace."
    exit 0
fi

# Check each member exists
while IFS= read -r line; do
    member=$(printf '%s\n' "$line" | sed -n 's/.*path[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p')
    [ -z "$member" ] && continue
    member_path="$REPO_ROOT/$member"
    if [ ! -d "$member_path" ]; then
        echo "MISSING: $member (path: $member_path)"
        EXIT_CODE=1
    else
        echo "OK:    $member"
    fi
done <<< "$path_deps"

# Also check workspace.members list — extract only the members array (stop at closing ']')
members=$(awk '/^members\s*=\s*\[/{found=1;next} found&&/^\]/{found=0} found{gsub(/[",]/,"",$0); gsub(/^[[:space:]]*/,"",$0); if($0!="")print}' "$REPO_ROOT/Cargo.toml" || true)

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
