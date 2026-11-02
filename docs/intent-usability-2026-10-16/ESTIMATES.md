# Effort estimates and confidence

Planning estimate made 2026-10-16 from existing source/audit findings; no execution
benchmark or team throughput study. Units are productive engineering hours, including
focused research, integration, tests and evidence. Confidence is LOW. These ranges
are neither calendar deadlines nor a claim of measured percent complete.

| Package | Productive hours | Main uncertainty |
|---|---:|---|
| A: candidate/profile reconciliation | 8-16 | Existing PR convergence and supported runtime ownership |
| B: install/status/MCP truth | 16-40 | Actual mounted implementation and core adapter gaps |
| C: durable core lifecycle closure | 24-64 | Recovery/concurrency findings and existing fix quality |
| D: operator pages and usability | 24-64 | Supported page inventory and real integration depth |
| E: assurance and packaging | 24-56 | Cross-language enforcement, negative controls, provenance |
| F: CVP pilot and repair cycle | 32-72 | Clean subject failures, repeatability and environment friction |
| G: MVP everyday workflow closure | 40-96 | Supported workflow breadth and usability defects |
| H: deployment/telemetry/recovery | 40-104 | Target identity, desktop/device and restore qualification |
| I: remaining mature journey closure | 112-320 | Unexecuted cases, semantic defects and integration rework |

| Milestone | Included packages | Cumulative range |
|---|---|---:|
| CVP | A+B+C+E+F | 104-248 h |
| MVP | CVP+D+G | 168-408 h |
| Finite 12-journey maturity | MVP+H+I | 320-832 h |

The arithmetic is additive; milestones overlap and must not be added together.
I includes remaining case execution, failure diagnosis, repair and release review;
it is not estimated by multiplying 92 cases by a uniform test duration.

## Assumptions and exclusions

- Existing canonical semantics and substantial recovery implementations are reusable.
- Existing PRs can be reconciled without a foundational redesign or data migration crisis.
- A capable engineer has a working development environment and access to required targets.
- Targeted verification and repairs are included; open-ended feature expansion is excluded.
- Competitive superiority, fleet-wide rollout, new external integrations and a rewrite are excluded.
- No paid CI capacity, credential material or production data is assumed necessary for planning.
- A remote/device target must be qualified before deployment work; unavailable access adds waiting.

## Capacity conversion and updates

Calendar duration = dependency-constrained effort / observed productive capacity,
plus external waits and review/integration time. Do not divide by agent count: shared
state, reviewers and runtime targets constrain concurrency. Plan parallel B/C/E only
after A assigns one writer per mutable area; D and pilots depend on usable integration.

Re-estimate after A, the first clean-install/core journey and the first adversarial
recovery cycle. Record old range -> new range with actual defects, measured task
completion and surviving assumptions. If core semantics fail, widen the range
before promising a date. Friday or other historical target language is intent,
not a current defensible delivery commitment.
