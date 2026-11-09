---
description: DEPRECATED — use `ap plan --tasks` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.tasks` | `ap plan --tasks <feature-id>` |

```bash
ap plan --tasks <feature-id>
```

## What it does

Decomposes the plan into claimable atomic tasks with FR-IDs, acceptance criteria, and effort (S/M/L).

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap plan --tasks <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
