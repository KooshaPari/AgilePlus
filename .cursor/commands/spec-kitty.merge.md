---
description: DEPRECATED — use `ap pr-builder merge` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.merge` | `ap pr-builder merge <feature-id>` |

```bash
ap pr-builder merge <feature-id>
```

## What it does

Squash-merges the feature branch into main after acceptance, runs changelog + tag automation.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap pr-builder merge <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
