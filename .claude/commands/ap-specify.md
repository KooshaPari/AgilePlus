---
description: Seed a new feature spec quickly
argument-hint: <title> <description>
---

# ap-specify

Create a new AgilePlus feature spec and attach baseline metadata.

## What this does

1. Validates CLI availability.
2. Runs `agileplus specify --title <title> --description <description>`.
3. Prints the newly created feature identifier for immediate follow-up.

## Steps

```pwsh
if ($ARGS.Count -lt 2) {
  Write-Error "Usage: /ap-specify <title> <description>"
  exit 1
}

$title = $ARGS[0]
$description = $ARGS[1]
agileplus specify --title $title --description $description
```

## Usage

```
/ap-specify "Backlog reorder" "Add drag-to-priority for untriaged work packages"
```

