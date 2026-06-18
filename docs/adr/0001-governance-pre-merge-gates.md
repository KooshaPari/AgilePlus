# ADR 0001: Governance Pre-Merge Gates

## Status

Accepted

## Context

Pull requests need lightweight checks that keep implementation, specification, and
release evidence aligned before merge.

## Decision

Run dedicated pre-merge jobs for anti-pattern detection, SPEC FR/NFR verification,
and governance document coverage. The jobs delegate policy details to scripts under
`scripts/qa-gates/` so local and CI behavior stay aligned.

## Consequences

PRs that reference FR/NFR IDs must keep `SPEC.md` current. PRs must also touch a
changelog, an ADR, and a QA matrix entry so governance evidence is reviewable.
