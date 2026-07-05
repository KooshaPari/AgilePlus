---
description: DEPRECATED — use `ap specify --clarify` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.clarify` | `ap specify --clarify <feature-id>` |

```bash
ap specify --clarify <feature-id>
```

## What it does

Surfaces up to 5 clarification questions for ambiguous areas of the spec; answer inline to refine.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap specify --clarify <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
