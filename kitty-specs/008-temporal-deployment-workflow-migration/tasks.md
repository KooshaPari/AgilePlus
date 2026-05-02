# Tasks: 008 — Temporal Deployment Workflow Migration

**Status**: SKELETON

## Work Packages

| ID | Description | Status |
|----|-------------|--------|
| WP-008-001 | Assess Temporal vs existing process-compose | 🔄 PLANNING |
| WP-008-002 | Migrate process-compose workflows to Temporal | 🔄 PLANNING |
| WP-008-003 | Temporal workflow definitions | 🔄 PLANNING |
| WP-008-004 | Temporal CLI wiring | 🔄 PLANNING |

## Context

### Current State
- AgilePlus uses `process-compose` for service orchestration
- 18 services defined in process-compose.yml
- Services include: NATS, Dragonfly, Neo4j, MinIO, backend API, frontend, etc.

### Temporal.io Integration
- Temporal is a durable execution engine for long-running workflows
- Would replace process-compose for stateful, fault-tolerant workflows
- Suitable for: spec→plan→implement→review→ship cycle workflows

### Decision Points
- process-compose vs Temporal: process-compose is simpler for container orchestration
- Temporal shines for: human-in-the-loop workflows, long-running business processes
- Recommendation: evaluate if any AgilePlus workflows need Temporal durability
- If no workflow requires Temporal guarantees, keep process-compose

## Next Steps

1. Audit existing process-compose workflows for Temporal candidacy
2. If candidates found: define Temporal namespace and workflow types
3. If no candidates: close spec as N/A (keep process-compose)

## Notes

- 008 spec has only `contracts/` directory (gRPC proto definitions)
- This spec needs further investigation to determine if Temporal migration is warranted
