# AgilePlus Mature Product Model

**Contract:** `AGP-MATURE-V1`  
**Canonical product:** AgilePlus (`PRD-AGILEPLUS`)  
**Source snapshot:** `f4deadab516293651398fb69b6f5cc38d621d069`

AgilePlus is the repository-scoped governed specification and **work-execution engine**. It turns accepted work intent into specifications, plans, work packages, claims, bounded agent execution, review, governance/evidence checks, shipping and immutable execution history.

## Reciprocal Tracera boundary

- **AgilePlus owns:** working change intent, specifications, plans, work packages, execution transitions, claims/leases, checkpoints, agent dispatch, review loops and work completion.
- **Tracera owns:** product identity, mature product intent, product hierarchy, product assessment, dissatisfaction and product-stage interpretation.

Work completion is an observation available to Tracera; it is not itself product acceptance.

## Mature-first hierarchy

Product -> Pillar -> Feature -> Sub-feature -> Atomic FR -> Acceptance -> Verification -> Implementation/Evidence.

VP stages are projections over one mature contract. Parent nodes aggregate; they do not double-count.

## Stub-first growth

**Stub the breadth; mature the execution spine.** Early stages should keep stable domain/state/authority boundaries while using narrow adapters where replacement is cheap. Avoid stage-specific execution architectures that must later be discarded.

## Working baseline

`spec/product/mature-contract.v1.json` defines 25 pillars, 200 features and 1,000 atomic FR records. The count is a working granularity baseline, not a completion claim.
