# Feature Specification: Kilo Gastown Methodology

**Feature Branch**: `kilo-specs-agileplus`
**Created**: 2026-03-31
**Status**: Active
**Input**: Document the Kilo Gastown methodology for orchestrating multi-agent software development using beads, rigs, and convoys.

## Overview

Gastown is Kilo's multi-agent orchestration system for coordinating software development work across specialized agents. It provides a structured approach to breaking down complex projects into discrete, traceable units of work that can be executed in parallel by different agents while maintaining overall coherence and quality.

## Core Concepts

### Towns

A **town** is a top-level organizational unit representing a single project or repository. Each town has:
- A unique town ID (UUID)
- A name derived from the repository name
- Multiple **rigs** for different types of work
- A **mayor** responsible for triage and escalation handling

### Rigs

A **rig** is a working cluster within a town containing agents that collaborate on related work. Rigs are identified by their rig ID (UUID) and contain:
- **Polecat agents**: Execute tasks assigned to them via hooked beads
- **Refinery agents**: Review completed work and merge changes
- Shared context from the town's projects

### Agents

Agents are the fundamental execution units in Gastown:

| Agent Type | Role | Capabilities |
|------------|------|--------------|
| **Polecat** | Task executor | Implements features, fixes bugs, writes tests and docs |
| **Clover** | Quality verifier | Runs linters, type checkers, test suites |
| **Birch** | Documentation specialist | Writes and maintains documentation |
| **Ember** | Research analyst | Investigates options, gathers context |
| **Maple** | Secondary implementer | Parallel implementation work |
| **Refinery** | Code reviewer | Reviews PRs, merges to default branch |
| **Patrol** | Triage coordinator | Routes issues to appropriate agents |

### Beads

**Beads** are the atomic work units in Gastown. Each bead represents:
- A discrete, completable unit of work
- An assigned agent (via `assignee_agent_bead_id`)
- A status lifecycle: `pending` → `in_progress` → `in_review` → `closed`
- Metadata for tracking and coordination

#### Bead Types

| Type | Purpose | Example |
|------|---------|---------|
| `issue` | Task or bug | "Implement feature X" |
| `escalation` | Blocked or problematic work | "Branch does not exist" |
| `merge_request` | Code review request | PR review from agent |
| `triage` | Routing decision needed | "GUPP force stop" |

#### Bead Lifecycle

```
┌──────────┐    assign    ┌─────────────┐   complete   ┌────────────┐
│ pending  │ ──────────► │ in_progress │ ───────────► │ in_review  │
└──────────┘              └─────────────┘              └────────────┘
                                                                │
                                                           merged
                                                                │
                                                          ┌──────┴──────┐
                                                          │   closed    │
                                                          └─────────────┘
```

### Worktrees and Branching

Gastown agents work in **isolated git worktrees** to prevent interference between parallel workstreams:

1. Each bead/feature gets a dedicated worktree branch
2. Worktrees are named using the pattern: `convoy/{convoy-id}/{agent-name}/{bead-id}`
3. Agents NEVER commit directly to the default branch
4. Work is merged via the Refinery after review

### Convoys

**Convoys** group related beads that need to be developed together:
- A convoy has a feature branch: `convoy/{convoy-id}/head`
- All beads in a convoy share this branch
- Individual agent branches are children of the convoy head
- When the convoy head is ready, the Refinery merges to default

Convoy branch naming: `convoy/{feature-spec-id}/{agent-id}/{bead-id}`

## Agent Workflow

### 1. Prime (Orientation)

At the start of each session, agents call `gt_prime` to receive:
- Their agent identity and status
- The currently hooked bead (their assigned task)
- Any undelivered mail from other agents
- All open beads in the rig

### 2. Work (Execution)

Agents execute their hooked bead following the **GUPP principle**:
> **G**rab the bead, **U**nderstand it, **P**rime your context, **P**erform the work

- Implement the solution completely
- Write tests and documentation as needed
- Commit frequently with descriptive messages
- Push commits after each meaningful unit of work

### 3. Checkpoint (Recovery)

Periodically call `gt_checkpoint` to save progress:
```json
{
  "completed": "function X implemented",
  "next": "write tests for function X",
  "blockers": []
}
```

If the container restarts, the agent can resume from the checkpoint.

### 4. Done (Submission)

When the bead is complete:
1. Push the branch to remote
2. Call `gt_done` with the branch name
3. The bead transitions to `in_review`
4. The Refinery picks it up for merge

## Quality Gates

Before calling `gt_done`, agents must verify:

1. **`task quality`** — Run the quality gate command to validate:
   - Linting passes
   - Type checking passes
   - Tests pass
   - Code follows project conventions

If a gate fails:
- Fix the issue and re-run the failing gate
- Repeat until all gates pass
- If unfixable after a few attempts, call `gt_escalate`

## Coordination Mechanisms

### Mail

Agents communicate via typed mail messages:
- `gt_mail_send` — Send a message to another agent
- `gt_mail_check` — Read undelivered mail

Mail is persistent and queued until acknowledged.

### Nudges

For time-sensitive coordination:
- `gt_nudge` — Deliver an immediate wake-up call to an idle agent

### Escalations

When stuck or blocked:
- `gt_escalate` — Create an escalation bead with context
- Escalations are routed to the mayor for resolution

### Triage Requests

Beads with the `gt:triage-request` label require human/mayor input:
- `gt_triage_resolve` — Take action on a triage request

## Gastown Tools Reference

| Tool | Purpose |
|------|---------|
| `gt_prime` | Get full context: identity, hooked bead, mail, open beads |
| `gt_bead_status` | Inspect any bead by ID |
| `gt_bead_close` | Close a completed bead |
| `gt_done` | Signal bead completion and push to review queue |
| `gt_mail_send` | Send a typed message to another agent |
| `gt_mail_check` | Read pending mail |
| `gt_nudge` | Send real-time nudge to wake an agent |
| `gt_escalate` | Create escalation for blocked/problematic work |
| `gt_checkpoint` | Write crash-recovery data |
| `gt_status` | Emit plain-language status for dashboard |
| `gt_mol_current` | Get current molecule step (if applicable) |
| `gt_mol_advance` | Complete molecule step and advance |
| `gt_triage_resolve` | Resolve a triage request |

## Gastown in AgilePlus

### Town Configuration

- **Town ID**: `78a8d430-a206-4a25-96c0-5cd9f5caf984`
- **Town Name**: `convoy__agileplus-kilo-specs-agileplus`
- **Rig ID**: `297c736c-f6b1-43c7-8167-db647ae94c53`

### Rig Structure

The AgilePlus rig contains:
- Polecat agents (Shadow, Clover, Birch, Maple, Ember) for implementation
- Refinery agents for code review and merging
- Patrol for triage coordination

### Integration Points

Gastown integrates with AgilePlus through:

1. **CLAUDE.md** — Agent instructions embedded in the repository
2. **Work Packages** — AgilePlus features map to Gastown beads
3. **Event Ledger** — All bead state changes are recorded for audit

## Benefits

- **Isolation**: Agents work in parallel without interfering
- **Traceability**: Every change is linked to a bead and agent
- **Recovery**: Checkpoints enable resume after failures
- **Quality**: Mandatory gates prevent bad code from merging
- **Coordination**: Mail and nudges enable agent communication

## Status

Active methodology for AgilePlus development. Documented from Gastown orchestration system v2026-03.
