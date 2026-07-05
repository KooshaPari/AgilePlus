---
description: DEPRECATED — use `ap validate` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.analyze` | `ap validate <feature-id>` |

```bash
ap validate <feature-id>
```

## What it does

Detects inconsistencies between spec.md, plan.md, tasks.md; checks FR coverage.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap validate <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
