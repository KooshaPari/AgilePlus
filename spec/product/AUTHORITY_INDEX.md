# AgilePlus Product Authority Index

**Status:** canonical entrypoint for product/specification authority  
**Date:** 2026-10-01

## A0 — direct accepted owner intent

Current owner decisions about mature-first specification, adaptive methodology, MACE/autograding and Tracera boundary.

## A1 — canonical mature semantics

1. `spec/product/mature-contract.v1.json`
2. `spec/product/CANONICAL_SEMANTIC_DECISIONS.md`
3. `spec/product/MATURE_DOMAIN_COVERAGE_MAP.md`
4. `spec/product/requirements.semantic.v1.json`
5. `spec/product/journeys.v1.json`
6. `spec/product/INTERFACE_PROFILE_CONTRACTS.md`
7. `verification/SPECIFICATION_ACCEPTANCE_CONTRACT.md`
8. `verification/semantic-verification-cases.v1.json`

## A2 — accepted decisions

ADR-0019 and other ADRs explicitly marked accepted remain authority only where they do not conflict with A1. A1 records supersession/refinement.

## A3 — historical requirement/methodology evidence

- `PRD.md`;
- `FUNCTIONAL_REQUIREMENTS.md`;
- historical spec sets;
- harmonization analysis;
- Proposed ADR-0012/0013/0014;
- old framework-specific docs.

These are provenance/lineage. Their mature disposition is governed by:
- `HISTORICAL_REQUIREMENT_DISPOSITION.md`;
- `historical-fr-disposition.v1.json`;
- `METHODOLOGY_RECONCILIATION.md`.

A historical document's own `Status: Active` or `Proposed` label does not override A1.

## A4 — implementation/tests

Current code/tests establish runtime behavior/evidence. If they disagree with A1, record implementation debt; do not silently redefine product semantics from code.

## Fresh-context recovery sequence

Read:
1. this authority index;
2. mature-contract manifest;
3. canonical semantic decisions;
4. recovered product intent;
5. mature domain map;
6. journeys and oracle catalogue;
7. non-code finality ledger;
8. runtime mounting/current-state evidence as needed.

This ordering prevents old fixed-FSM, kitty-specs-only, mandatory Neo4j/NATS/MinIO/Plane, or review→Done assumptions from regaining authority accidentally.
