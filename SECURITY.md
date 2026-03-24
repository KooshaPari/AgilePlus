# Security Policy

## Scope

`phench` manages local runtime targets, mirrored state files, and git worktree materialization. Treat path handling, state sync, and command execution surfaces as security-sensitive.

## Reporting

Report vulnerabilities privately to the repository owner through GitHub security reporting or a direct private channel.

Do not open public issues for exploitable path traversal, arbitrary command execution, or local state corruption bugs until a fix is available.

## Current controls

- repo identifiers are sanitized before materialized path use
- state files are written through atomic temp-file replacement
- runtime materialization pins explicit commit SHAs before checkout
