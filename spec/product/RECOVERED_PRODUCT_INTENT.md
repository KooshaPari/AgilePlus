# Recovered AgilePlus methodology intent

**Status:** controlling intent input for semantic specification replacement.
**Captured:** 2026-09-29.

AgilePlus is intended to be an optimal agent-first specification and software-work system, not merely a queue or dispatcher.

## Synthesis target

Preserve the strongest properties of multiple methods without inheriting their ceremony wholesale:

- **OpenSpec:** lightweight/fluid/brownfield-first behavior specs, change deltas, intuitive explore→propose→apply/verify flow, requirements plus concrete scenarios.
- **Spec Kit:** explicit intent→specification→plan→tasks→implementation→convergence structure, clarification/checklist/analysis gates, reusable processes/extensions.
- **BMAD:** adaptive depth, comprehensive research/PRD/architecture when complexity warrants it, explicit decisions/context, specialized thinking/building roles.
- **General engineering:** ADRs, contracts, risk analysis, WBS/PERT/DAG, dependency analysis, interface design, verification strategy, change/release governance where justified.
- **Traceability-first additions:** stable identities, bidirectional links, acceptance/oracle contracts, evidence binding, exact work surfaces, provenance, and promotion gates.

The synthesis must be **adaptive**. Small changes should not require heavyweight artifacts merely because large changes do. Complex/high-risk work must not be allowed to skip depth merely for velocity.

## Assignment/autograder mental model

A useful AgilePlus work unit resembles a rigorous interactive assignment:
- objective and why;
- exact scope/exclusions;
- prerequisites/context bundle;
- observable requirements/scenarios;
- constraints/invariants;
- allowed/forbidden work surfaces;
- dependencies;
- acceptance criteria;
- positive and adversarial oracles;
- expected evidence;
- grading feedback;
- retry/revision history;
- completion/promotion rule.

The agent should know what "done" means before implementation and should receive localized actionable feedback when it is not done.

## Lifecycle is iterative, not waterfall

Artifacts have dependencies and authority, not an arbitrary mandatory linear ceremony. Research, specification, design, plan and verification intent may be revisited as learning occurs. Changes to accepted behavior are explicit revisions rather than silent prompt drift.

## Relationship to Tracera

AgilePlus owns local/repository working change intent and execution. Tracera owns persistent product truth across sessions/repos/releases.

AgilePlus can consume a Tracera product delta or dissatisfaction finding as work input and return execution artifacts/receipts/evidence references. Work completion does not itself mutate Tracera's accepted product state.

## Required semantic review areas

The mature AgilePlus specification must cover at least the actually applicable behaviors discovered under:
- exploration/idea shaping and brownfield understanding;
- research and ambiguity resolution;
- behavior specification and scenarios;
- architecture/design/ADR/contract work;
- planning/decomposition/WBS/DAG;
- task/work-package/context-bundle construction;
- claims/worktree/sandbox boundaries;
- agent dispatch and model/tool selection;
- execution feedback/autograding/retry;
- review/convergence/HITL escalation;
- evidence and traceability;
- change/spec evolution;
- promotion/release handoff;
- recovery/resumption/history;
- CLI/API/MCP/human UX;
- Tracera federation.

This list is a review surface, not a fixed feature count or requirement generator.
