---
description: DEPRECATED — use `ap plan` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.plan` | `ap plan <feature-id>` |

```bash
ap plan <feature-id>
```

## What it does

Breaks the spec into atomic tasks with FR-ID traceability, deps, and effort estimates.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap plan <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
