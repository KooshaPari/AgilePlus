#!/bin/bash
MERGED_FF=""
MERGED_NOFF=""
SKIPPED=""
CONFLICTS=""

branches=$(git branch -r --no-merged integration/consolidate | grep -v "HEAD\|main\|integration/consolidate" | sed 's/^ *//')

for branch in $branches; do
  branch_name=${branch#origin/}
  COUNT=$(git diff --name-status integration/consolidate...origin/$branch_name 2>/dev/null | grep -c "^D" || true)
  
  if [ "$COUNT" -eq 0 ]; then
    if git merge --ff-only origin/$branch_name 2>/dev/null; then
      echo "MERGED_FF $branch_name"
      MERGED_FF="$MERGED_FF $branch_name"
    elif git merge --no-ff origin/$branch_name -m "land: $branch_name" 2>/dev/null; then
      echo "MERGED_NOFF $branch_name"
      MERGED_NOFF="$MERGED_NOFF $branch_name"
    else
      git merge --abort 2>/dev/null
      echo "CONFLICT $branch_name"
      CONFLICTS="$CONFLICTS $branch_name"
    fi
  else
    echo "SKIP $branch_name deletions=$COUNT"
    SKIPPED="$SKIPPED $branch_name"
  fi
done

echo ""
echo "=== SUMMARY ==="
echo "MERGED (ff-only):$MERGED_FF"
echo "MERGED (no-ff):$MERGED_NOFF"
echo "SKIPPED:$SKIPPED"
echo "CONFLICTS:$CONFLICTS"
