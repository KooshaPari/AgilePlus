#!/bin/bash
# action-hygiene-audit.sh
# Checks all GitHub Actions workflow files for unpinned third-party actions.
# eco-011 FR-5: Action hygiene audit
# A third-party action is considered pinned if it uses a 40-character SHA ref.
# First-party actions (KooshaPari/*, actions/*) are exempt.
#
# Repository-local reusable workflows ("./.github/workflows/x.yml") are also
# exempt, because GitHub only accepts them in that form: they are resolved from
# the checked-out commit at run time and cannot be referenced by SHA. This is a
# syntax exemption, not a blanket allow-list — see classify_action below, which
# still rejects an unpinned external ref and rejects a malformed local path.

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

# classify_action <action_ref>
# Prints one of: pinned | exempt | unpinned
#
#   pinned   - third-party ref carrying an immutable 40-char SHA
#   exempt   - first-party owner, or a well-formed repository-local reusable
#              workflow path ("./.github/workflows/<file>.yml") or a
#              repository-local composite action ("./.github/actions/<dir>")
#   unpinned - everything else, including malformed local paths
#
# A local ref is exempt only if it is exactly "./.github/workflows/" followed by
# a plain workflow filename ending in .yml or .yaml, or exactly "./.github/actions/"
# followed by a plain action directory name. Anything else that starts with "./"
# is malformed and is reported as unpinned rather than silently passed.
classify_action() {
    local ref="$1"
    local org_pattern='^KooshaPari/'

    # First-party actions (actions/* and KooshaPari/*)
    if [[ "$ref" =~ ^actions/ || "$ref" =~ $org_pattern ]]; then
        echo "exempt"
        return
    fi

    if [[ "$ref" == ./* ]]; then
        # Repository-local reusable workflow: GitHub requires the "./" form and
        # resolves it from the checked-out commit, so a SHA ref is not applicable.
        if [[ "$ref" =~ ^\./\.github/workflows/[A-Za-z0-9._-]+\.ya?ml$ ]]; then
            echo "exempt"
        # Repository-local composite action: same reasoning — it is resolved
        # from the checked-out commit and cannot carry a SHA ref.
        elif [[ "$ref" =~ ^\./\.github/actions/[A-Za-z0-9._-]+$ ]]; then
            echo "exempt"
        else
            echo "unpinned"
        fi
        return
    fi

    if [[ "$ref" =~ @[a-f0-9]{40} ]]; then
        echo "pinned"
    else
        echo "unpinned"
    fi
}

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
        org_pattern='^KooshaPari/'
        if [[ "$action_ref" =~ ^actions/ || "$action_ref" =~ $org_pattern ]]; then
            continue
        fi
        case "$(classify_action "$action_ref")" in
            pinned)
                PINS_FOUND=$((PINS_FOUND + 1))
                ;;
            exempt)
                continue
                ;;
            unpinned)
                unpinned+=("$action_ref")
                UNPINS_FOUND=$((UNPINS_FOUND + 1))
                ;;
        esac
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
