---
description: DEPRECATED — use `ap specify` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.specify` | `ap specify "<feature description>"` |

```bash
ap specify "<feature description>"
```

## What it does

Generates a machine-readable spec with FR/NFR IDs from a free-text feature description.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap specify "<feature description>"`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
