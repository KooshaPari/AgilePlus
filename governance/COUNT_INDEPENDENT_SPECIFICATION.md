# Count-independent mature-product specification

**Controlling user correction: September 29, 2026.** Conversational numbers are not targets, minimums, maximums, quotas or expected ranges. Requirement counts are outputs only. Do not hit a number, exceed a number to demonstrate diligence, cap scope at a number, or impose equal branch sizes. A larger quota is not a correction.

## Objective and authority

Describe the most comprehensive presently knowable mature AgilePlus first. Its large structured work backlog is intentional. Derive CVP/MVP/other viable forms from that mature contract, not the other way around. Comprehensive means covering justified product obligations; it does not mean inventing adjacent products or exhaustive meaningless permutations.

AgilePlus owns repository-scoped working specifications, plans, execution state, work packages, resource claims, dispatch, review, governance and attributable work receipts. Tracera owns accepted product intent, product graph, dissatisfaction and product-state assessment. Source repositories and producing verifiers retain their fact ownership. Current code is evidence of present behavior, not automatic authorization for desired scope.

## Discover before decomposing

Reconcile user intent with PRD.md, USER_JOURNEYS.md, current domain and port contracts, source-native FR/NFR catalogues, traceability matrices, ADRs, interfaces, commands, supported components, tests and documented gaps. Follow their relevant references; record exact revisions, reviewed ranges, conflicts, coverage gaps and exclusions. An unread or inaccessible source is not completed review. Historical inventories may be consulted, but the separate deleted-repository/full-history recovery effort remains deferred.

Use top-down journeys and bottom-up state/interface/implementation inspection as independent checks. Record every material discovered surface as covered, duplicate, excluded with rationale, deferred within mature scope, or unresolved. Do not silently convert a legacy status such as SHIPPED into current verification.

## Semantic decomposition

Identify each capability's actual actors, preconditions, commands, state transitions, outputs, permissions, persistence, concurrency, cancellation, error and recovery semantics where relevant. Split only independently falsifiable obligations or obligations with materially different stage/ownership decisions. Do not multiply every feature by generic behavior/identity/boundary/continuity/evidence dimensions.

A template enforces structure, not substance. Generic 'provide X correctly' statements are unfinished prompts. Shared invariants and nonfunctional constraints belong once with applicability links unless the product has genuinely distinct functional obligations. Negative cases and test permutations do not automatically create FRs. Merge redundant entries while preserving source lineage.

For example, the claim-engine ADR distinguishes traceability assertions from resource leases. Inspect its actual contract and implementation before writing obligations for exclusion, ownership, lease expiry, heartbeat, release, persistence and execution fencing. Their acceptance outcomes must be individually stated; filling five interchangeable prose slots would miss these semantics.

## Requirement content

An actionable FR needs a distinct observable behavior; authoritative origin or explicit derivation rationale; product/component owner; meaningful feature hierarchy; role; actor/preconditions; trigger/input; expected outcome/postconditions; relevant failure prohibitions; dependencies; reasoned stage applicability; likely work/source surfaces with confidence; growth disposition; and concrete verification intent. Existing source IDs are preserved or crosswalked rather than overwritten by a freshly numbered duplicate catalogue.

## Stages, shape and minimally-changing growth

Choose a viable stage from a closed usable journey and its necessary dependency, safety and operational obligations. Never select the first N features or allocate an equal number per domain. Stages share canonical identities and may branch where justified. Distinguish core completion from auxiliary improvement and distinguish primitives/stubs from features and usable journeys. Empty or invalidated projections are ungradable.

Stub breadth and mature a real execution spine. Prefer stable boundaries and additive enrichment. Temporary adapters must state replacement and migration burden. Forecast reuse and compatibility estimates are not observed evidence; do not set them true by default.

## Verification design, not mass test generation

Concrete product-test generation remains deferred. Verification intent is required now: stimulus, preconditions, exact expected outcomes, meaningful counterexamples, fixtures, reset/isolation, target/environment, assertion needs, evidence identity and invalidation. A reserved test ID plus a generic sentence does not make a requirement test-generation ready.

Keep workflow state, behavioral result, trace completeness and product acceptance separate. Preserve accepted governance policy; this specification correction is not permission to bypass existing runtime gates. A missing required observation must not become a pass.

## Acceptance of the specification pass

Require source-coverage review, coherent hierarchy, duplicate/contradiction resolution, justified stage/journey bindings, resolved references, concrete oracles and an independent review for missing behavior and padding. Counts are computed afterward and never determine acceptance. If any material review remains, report the exact unfinished frontier instead of manufacturing entries to declare completion.

The earlier generated catalogue is rejected. Its source snapshot and Git history are retained; its generic rows, positional stages and unsupported readiness/compatibility claims are not an accepted baseline. The comprehensive replacement is not complete.
