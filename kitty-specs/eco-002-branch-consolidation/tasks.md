# Tasks: eco-002 — Branch Consolidation

**Status**: COMPLETED ✅

## Work Packages

| ID | Description | Status |
|----|-------------|--------|
| WP-ECO201 | Audit branch proliferation | ✅ COMPLETE |
| WP-ECO202 | Delete stale/abandoned branches | ✅ COMPLETE |
| WP-ECO203 | Establish branch naming convention | ✅ COMPLETE |

## Evidence

### Branch Audit (WP-ECO201)
- Aggressive branch deletion campaigns executed across rounds
- AgilePlus: 74→6 total branches (59 deleted, `releases/stable` protected, 4 worktree branches)
- Many branches were squash-merged ghosts (orphaned SHAs from previous merge patterns)
- Detection via `gh pr list --state merged` + `git patch-id --stable`

### Stale Branch Cleanup (WP-ECO202)
- Branches merged to main are cleaned up automatically (delete_branch_on_merge enabled)
- Ghost branches (orphaned SHAs from squash merges) manually identified and deleted
- Pattern: squash-merge creates new SHA but identical content; use `git patch-id` to deduplicate

### Naming Convention (WP-ECO203)
- Feature: `feature/<short-description>`
- Bug fix: `fix/<short-description>`
- Chore: `chore/<short-description>`
- Docs: `docs/<short-description>`
- Worktree pattern: `<name>-wtrees/<topic>/`

## Commands

```bash
# List merged branches (local)
git branch --merged main | grep -v main | xargs -r git branch -d

# List stale remote branches (no PR)
for b in $(git branch -r --no-merged main | grep -v main); do
  if ! gh pr list --head "${b#origin/}" --state all --json number --jq 'length' | grep -q .; then
    echo "Stale: $b"
  fi
done

# Detect ghost branches (squash-merged)
git log --oneline main | head -50 | awk '{print $1}' | while read sha; do
  patch_id=$(git patch-id --stable $sha | cut -d' ' -f1)
  echo "$patch_id $sha"
done | sort | uniq -d -f1
```
