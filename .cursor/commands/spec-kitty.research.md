---
description: DEPRECATED — use `ap plan research` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.research` | `ap plan research "<feature description>"` |

```bash
ap plan research "<feature description>"
```

## What it does

Research-driven spec generation: scans the repo, gathers context, scaffolds research artifacts before task planning.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap plan research "<feature description>"`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
