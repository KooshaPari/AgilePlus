---
description: DEPRECATED — use `ap plan --checklist` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.checklist` | `ap plan --checklist <feature-id>` |

```bash
ap plan --checklist <feature-id>
```

## What it does

Generates 'unit tests for English' — checklist items that validate requirements quality (completeness, clarity, consistency), NOT implementation behavior.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap plan --checklist <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
