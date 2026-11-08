---
description: Explore and validate AgilePlus specs layout
argument-hint: "[feature-id]"
---

# ap-specs

Inspect `kitty-specs/` and quickly validate feature tree shape.

## What this does

1. Lists top-level spec folders.
2. Optionally opens the docs index for a single feature.
3. Runs a quick file presence check (`README.md`, `tasks.md`) before edits.

## Steps

```pwsh
$root = Resolve-Path $PSScriptRoot/../..
$base = Join-Path $root "kitty-specs"

if ($ARGS.Count -gt 0) {
    $feature = $ARGS[0]
    $featureDir = Join-Path $base $feature
    if (-not (Test-Path $featureDir)) {
        Write-Error "Feature not found: $feature"
        exit 1
    }
    Get-ChildItem $featureDir -Depth 2
    exit 0
}

Get-ChildItem $base | Select-Object -First 20 Name
```

## Usage

```
/ap-specs
/ap-specs 021-polyrepo-ecosystem-stabilization
```

