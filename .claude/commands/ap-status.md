---
description: Run AgilePlus work tracking status checks
argument-hint: "[feature-id] [--wp <work-package-id>] [--json]"
---

# ap-status

Quick project status from CLI or shell-friendly JSON for script checks.

## What this does

1. If no args, runs `agileplus status` and prints summary.
2. If a feature id is supplied, scopes output to that feature.
3. If `--json` is supplied, emits JSON if the local CLI supports it.

## Steps

```pwsh
if ($env:PYTHONPATH) { $env:PYTHONPATH = $env:PYTHONPATH }  # keep environment normalization intact
$argsText = $ARGS
if ($argsText.Count -eq 0) {
    agileplus status
    exit $LASTEXITCODE
}

if ($argsText -contains "--json") {
    agileplus status @($argsText)
} else {
    agileplus status @($argsText)
}
```

## Usage

```
/ap-status
/ap-status us-agileplus-epic-1
/ap-status us-agileplus-epic-1 --wp wp-3 --json
```

