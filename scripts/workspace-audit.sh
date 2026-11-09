#!/bin/bash
# workspace-audit.sh
# Validates that the root Cargo workspace resolves and that every member
# pattern points at a real package manifest.
# eco-027: Cargo Workspace Cleanup
#
# Usage: scripts/workspace-audit.sh
# Exit codes: 0 = healthy, 1 = broken workspace

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EXIT_CODE=0

cd "$REPO_ROOT"

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
' Cargo.toml 2>/dev/null || true)

if [ -n "$path_deps" ]; then
    while IFS= read -r line; do
        dep=$(printf '%s\n' "$line" | sed -n 's/.*path[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p')
        [ -z "$dep" ] && continue
        if [ -d "$REPO_ROOT/$dep" ]; then
            echo "OK:    $dep"
        else
            echo "MISSING: $dep (path: $REPO_ROOT/$dep)"
            EXIT_CODE=1
        fi
    done <<< "$path_deps"
else
    echo "No path dependencies found in workspace."
fi

# Members may be globs (crates/*, libs/*). Expand each pattern and require
# every matched directory to contain a Cargo.toml, mirroring cargo's behaviour.
echo ""
echo "=== Workspace Members Check ==="
members=$(awk '/^members\s*=\s*\[/{found=1;next} found&&/^\]/{found=0} found{gsub(/#.*/,""); gsub(/[",]/,"",$0); gsub(/^[[:space:]]+/,"",$0); gsub(/[[:space:]]+$/,"",$0); if($0!="")print}' Cargo.toml || true)

if [ -n "$members" ]; then
    while IFS= read -r pattern; do
        [ -z "$pattern" ] && continue
        matched=0
        for dir in $REPO_ROOT/$pattern; do
            [ -d "$dir" ] || continue
            matched=1
            rel=${dir#"$REPO_ROOT"/}
            if [ -f "$dir/Cargo.toml" ]; then
                echo "OK:    $rel"
            else
                echo "MISSING: $rel/Cargo.toml (matched by $pattern)"
                EXIT_CODE=1
            fi
        done
        if [ "$matched" -eq 0 ]; then
            echo "MISSING: $pattern (no directories matched)"
            EXIT_CODE=1
        fi
    done <<< "$members"
fi

echo ""
echo "=== Cargo Metadata Check ==="
if command -v cargo >/dev/null 2>&1; then
    if cargo metadata --no-deps --format-version 1 >/dev/null; then
        echo "ok: cargo metadata --no-deps resolves the workspace."
    else
        echo "error: cargo metadata --no-deps failed; the workspace graph is broken."
        EXIT_CODE=1
    fi
else
    echo "skip: cargo not installed; metadata check requires a Rust toolchain."
fi

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "error: workspace audit found missing members or a broken workspace graph. Fix before merging."
    exit 1
fi

echo ""
echo "ok: workspace members, manifests, and metadata all resolve."
exit 0
