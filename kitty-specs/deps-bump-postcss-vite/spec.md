# Spec: bump postcss and vite for npm alerts

## Goal
Update phenoDesign dependency policy to use `postcss@^8.5.10` and `vite@^6.4.2` so the repository clears the current npm alerts.

## Scope
- Update `package.json` to pin the requested versions.
- Use `overrides` if the packages are transitive rather than direct dependencies.
- Regenerate the lockfile so the resolved dependency graph matches the manifest.

## Acceptance Criteria
- `package.json` reflects the requested postcss/vite version policy.
- Lockfile is refreshed and consistent.
- Repository state is ready for commit with no unrelated changes reverted.
