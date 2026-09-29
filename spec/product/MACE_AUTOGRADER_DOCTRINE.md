# MACE / Autograder Control-Loop Doctrine

**Status:** controlling product intent and blocking architecture requirement.
**Recovered intent:** June–September 2026 conversations plus 2026-09-29 clarification.

## Core thesis

The strongest shared inspiration for AgilePlus and Tracera is not a conventional project dashboard. It is the **ZyBooks/Gradescope/autograder control loop** applied to agentic product development.

An oracle is useful precisely because it gives the worker a measurable target. The design goal is not to hide the target from the agent; it is to make the target truthful, multidimensional, difficult to game, and aligned with the accepted product outcome.

The environment should let an agent observe:
- where it is;
- what remains;
- which dimension is failing;
- what changed after its last action;
- whether it is converging;
- whether it is merely moving the score by weakening/changing the grader;
- whether progress is verified and durable.

## MACE loop

```text
accepted assignment / product state
          ↓
bounded worker context
          ↓
worker action
          ↓
independent grader/oracle
          ↓
multidimensional result + evidence
          ↓
compressed localized failure/delta
          ↓
next minimal context bundle
          ↓
retry / replan / spec clarification / escalation
```

Attempts are restartable. Verified progress should survive when its dependencies remain valid.

## Progress is a vector and a trajectory

Do not collapse progress to one score.

Candidate dimensions include:
- accepted functional behavior;
- negative/error behavior;
- acceptance/oracle coverage;
- trace completeness;
- evidence freshness/admissibility;
- journey/feature closure;
- regression confidence;
- quality families (performance, reliability, security, accessibility, usability, etc.);
- unexplained/unmapped implementation;
- transition debt/survivability;
- scope uncertainty;
- human-judgment debt.

For each applicable dimension track, where meaningful:
- current position;
- delta from prior attempt;
- slope/velocity;
- confidence/uncertainty;
- blocker distance;
- evidence age;
- regressions;
- oscillation/thrashing.

The "asymptote" is the accepted outcome under the current contract, not a permanently fixed numeric score.

## Anti-Goodhart rules

The worker may see the rubric and feedback. The system protects alignment by:
- separating accepted grader policy from ordinary implementation authority;
- retaining adversarial/negative controls;
- binding results to exact candidate/environment/verifier;
- keeping scope changes separate from engineering progress;
- preventing empty/skipped checks from producing green;
- preserving independent quality-family denominators;
- detecting score increases caused by weakened criteria;
- recording grader revisions as their own events;
- periodically using hidden/held-out or independently generated adversarial checks where appropriate;
- rewarding durable verified progress rather than raw attempt count or rubric-point accumulation.

## Failure classification

A failed grade should localize the problem when possible:
- implementation defect;
- missing behavior;
- regression;
- specification ambiguity/conflict;
- environment/tool failure;
- verifier/grader failure;
- stale evidence;
- authority/policy block;
- dependency block;
- human-judgment requirement.

This classification determines the next context bundle and whether retry is appropriate.

## Human vs machine judgment

Machines should grade what can be made objective: correctness, constraints, completeness, consistency, traceability, regressions, measurable quality targets.

Humans remain disproportionately responsible for irreducibly subjective product judgment: taste, vibe, novelty, strategic choice, creative direction, and explicit exceptions.

## AgilePlus scope

AgilePlus applies MACE at the **bounded work/assignment level**:
- compile assignment/rubric/context;
- execute attempt;
- run applicable graders;
- produce localized feedback;
- preserve verified sub-results;
- choose retry/replan/spec-loop/HITL;
- emit attributable receipts/evidence.

## Tracera scope

Tracera applies MACE at the **persistent product level**:
- maintain the accepted multidimensional product contract;
- aggregate requirement/feature/journey/quality evidence;
- show product position and trajectory;
- detect dissatisfaction, regressions and stalled/thrashing areas;
- invalidate evidence when dependencies/contract/candidate change;
- expose product/stage distance without confusing scaffolding with usable closure.

Tracera does not own AgilePlus's attempt/execution state machine; AgilePlus does not award itself product acceptance.

## Architecture consequences

Both products need first-class identities for:
- rubric/criterion/oracle;
- grader/verifier version;
- attempt/evaluation;
- score vector/result;
- evidence;
- failure classification;
- progress delta/trajectory;
- contract/grader revision;
- dependency/invalidation relation.

The final SOTA study must explicitly compare educational autograders, benchmark/evaluation harnesses, CI/test systems, agent evaluation systems, and reward/control-loop research—not only PM/ALM competitors.
