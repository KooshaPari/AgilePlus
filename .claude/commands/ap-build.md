---
description: Build the entire AgilePlus workspace or a specific crate
argument-hint: [--release] [--package <crate>]
---

# ap-build

Build AgilePlus workspace crates and catch compile breakages before status updates.

## What this does

1. Runs `cargo build --workspace` by default.
2. Optionally targets a single crate with `--package`.
3. Supports release builds with `--release`.

## Steps

```pwsh
param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$ARGS
)

$root = Resolve-Path $PSScriptRoot/../..
$arguments = @("build","--workspace")

if ($ARGS.Count -gt 0) {
    $arguments = @()
    $arguments += "build"
    $arguments += $ARGS
}

Push-Location $root
try {
    cargo @arguments
} finally {
    Pop-Location
}
```

## Usage

```
/ap-build
/ap-build --release
/ap-build --workspace --package agileplus-cli
```
