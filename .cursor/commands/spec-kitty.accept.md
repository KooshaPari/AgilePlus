---
description: DEPRECATED — use `ap ship` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.accept` | `ap ship <feature-id>` |

```bash
ap ship <feature-id>
```

## What it does

Validates all work packages are complete + runs the final acceptance checks (CI green, FR coverage, audit scorecard).

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap ship <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
