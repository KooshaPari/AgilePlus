# Project Instructions

**This project IS AgilePlus - the core project management platform.**

## Self-Reference

AgilePlus tracks its own work through its own system.

## Branch Discipline

- Feature branches in `repos/worktrees/AgilePlus/<category>/<branch>`
- Canonical repository tracks `main` only
- Return to `main` for merge/integration checkpoints

## Work Requirements

1. **Check for AgilePlus spec before implementing**
2. **Create spec for new work**: `agileplus specify --title "<feature>" --description "<desc>"`
3. **Update work package status**: `agileplus status <feature-id> --wp <wp-id> --state <state>`
4. **No code without corresponding AgilePlus spec**

## Spec Management

Use AgilePlus: `agileplus specify` and `agileplus status`

Specs are stored in `.agileplus/specs/<feature-id>/`:
- `.agileplus/specs/<feature-id>/spec.md` - feature description
- `.agileplus/specs/<feature-id>/meta.json` - id, title, status
- `.agileplus/specs/<feature-id>/tasks.md` - work packages

## Worklog

- AgilePlus/.work-audit/worklog.md
