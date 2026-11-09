---
description: Operate local MCP helper services used by AgilePlus tooling
---

# ap-mcp

Install and run dispatch-oriented MCP helpers for agent workflows.

## What this does

1. Optionally installs `dispatch-mcp` editable package.
2. Prints readiness checks for command availability.
3. Runs `dispatch-mcp` in the foreground for manual smoke checks.

## Steps

```pwsh
$root = Resolve-Path $PSScriptRoot/../..
Push-Location (Join-Path $root "dispatch-mcp")
try {
    if ($ARGS.Count -gt 0 -and $ARGS[0] -eq "install") {
        python -m pip install -e .
    }
    if ($ARGS.Count -gt 0 -and $ARGS[0] -eq "run") {
        if (Get-Command dispatch-mcp -ErrorAction SilentlyContinue) {
            dispatch-mcp
        } else {
            Write-Error "dispatch-mcp entrypoint not found. Run: /ap-mcp install"
            exit 1
        }
    } else {
        dispatch-mcp --version
    }
} finally {
    Pop-Location
}
```

## Usage

```
/ap-mcp             # print dispatch-mcp --version
/ap-mcp install     # install editable package
/ap-mcp run         # launch local dispatch MCP process
```

