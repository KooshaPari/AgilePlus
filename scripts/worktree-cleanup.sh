#!/bin/bash
# worktree-cleanup.sh
# Cleanup pass for stale worktrees and merged branches.
# eco-028 WP-04 T007-T008.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DRY_RUN=1
FORCE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --execute) DRY_RUN=0; shift ;;
        --force) FORCE=1; shift ;;
        *) echo "usage: $0 [--execute] [--force]"; exit 1 ;;
    esac
done

if [[ $DRY_RUN -eq 1 ]]; then
    echo "DRY RUN mode — no changes will be made. Pass --execute to apply."
fi

# Check for uncommitted changes in main repo
main_dirty=$(cd "$REPO_ROOT" && git status --short | grep -c . || echo "0")
if [[ $main_dirty -gt 0 && $FORCE -eq 0 ]]; then
    echo "error: main repo has $main_dirty uncommitted changes. Stash or commit first, or use --force."
    exit 1
fi

# Find stale worktrees (older than 14 days)
removed=0
stale=0

while IFS= read -r -d '' wt_path; do
    if [[ ! -d "$wt_path/.git" && ! -f "$wt_path/.git" ]]; then
        continue
    fi

    last_commit_epoch=$(cd "$wt_path" && git log -1 --format="%ct" 2>/dev/null || echo "0")
    now_epoch=$(date +%s)
    age_days=$(( (now_epoch - last_commit_epoch) / 86400 ))

    if [[ $age_days -gt 14 ]]; then
        stale=$((stale + 1))
        branch_name=$(cd "$wt_path" && git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
        echo "stale: $wt_path ($age_days days, branch=$branch_name)"
        if [[ $DRY_RUN -eq 0 ]]; then
            # Remove worktree
            rm -rf "$wt_path"
            removed=$((removed + 1))
            echo "  removed: $wt_path"
        fi
    fi
done < <(find "$REPO_ROOT" -maxdepth 2 -type d -name "*.wt" -print0 2>/dev/null || true)

# Find merged branches
merged=0
for branch in $(cd "$REPO_ROOT" && git branch --merged main --format="%(refname:short)" 2>/dev/null || true); do
    if [[ "$branch" == "main" || "$branch" == "master" || "$branch" == "*" ]]; then
        continue
    fi
    echo "merged: $branch"
    if [[ $DRY_RUN -eq 0 ]]; then
        git -C "$REPO_ROOT" branch -d "$branch" 2>/dev/null || true
        merged=$((merged + 1))
    fi
done

# Find dirty divergences
for branch in $(cd "$REPO_ROOT" && git branch --format="%(refname:short)" 2>/dev/null || true); do
    if [[ "$branch" == "main" || "$branch" == "master" ]]; then
        continue
    fi
    ahead=$(cd "$REPO_ROOT" && git rev-list --count "origin/main..$branch" 2>/dev/null || echo "0")
    if [[ $ahead -gt 0 ]]; then
        dirty=$(cd "$REPO_ROOT" && git status --short | grep -c . || echo "0")
        if [[ $dirty -gt 0 ]]; then
            echo "dirty-ahead: $branch ($ahead commits ahead, $dirty uncommitted changes)"
        fi
    fi
done

if [[ $DRY_RUN -eq 1 ]]; then
    echo ""
    echo "DRY RUN complete: $stale stale worktrees, $merged merged branches."
    echo "Run with --execute to apply changes."
else
    echo ""
    echo "Cleanup complete: removed $stale stale worktrees, deleted $merged merged branches."
fi
