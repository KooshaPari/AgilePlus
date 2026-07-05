---
description: DEPRECATED — use `ap implement` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.implement` | `ap implement <feature-id>` |

```bash
ap implement <feature-id>
```

## What it does

Drives task-by-task implementation per the plan. Resumes state via the Feature state machine in crates/agileplus-domain/src/domain/spec_state.rs.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap implement <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
