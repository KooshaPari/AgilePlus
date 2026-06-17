# QA Matrix

| Gate | Evidence | Command |
| --- | --- | --- |
| Anti-pattern detection | Changed source files reject `unwrap()`, `expect()`, `panic!`, and SQL string concatenation patterns (test modules excluded). | `scripts/qa-gates/antipattern.sh` |
| SPEC verification | PR-body `FR-*` and `NFR-*` references must exist in `SPEC.md`. | `scripts/qa-gates/spec-verify.sh` |
| Governance spec-first | PRs must touch CHANGELOG, ADR, and QA matrix documentation. | `scripts/qa-gates/governance-spec-first.sh` |
| eco-029 CI consolidation | phenoShared clone added to all cargo workflows; buf proto deduplicated; taplo format applied; commitlint + domain-deps gates fixed. | CI / PR #729 |
