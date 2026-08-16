# Tasks: Preserve-First Artifact Manifests

## Work Package Index

| ID | Title | Depends on | Planned validation |
|---|---|---|---|
| WP01 | Canonical custody-manifest schema | None | schema fixture and JSON validation |
| WP02 | Agent and cockpit HTML projection | WP01 | deterministic render and digest parity |
| WP03 | Preservation and recovery evidence | WP01 | restore, ref-parity, and `git fsck` evidence checks |
| WP04 | Governance and contract checks | WP01, WP02, WP03 | contract, review, CI, audit-chain, and secret-safety checks |

## Delivery Rules

- Each work package must declare exact file scope, acceptance criteria, dependencies,
  test-first commands, and non-destructive constraints in `tasks/WP01-*.md` through
  `tasks/WP04-*.md`.
- Only canonical JSON may be used as the evidence authority; HTML is derived output.
- No manifest, fixture, rendered HTML, log, or contract may contain a credential, token,
  password, private key, or unredacted secret.
