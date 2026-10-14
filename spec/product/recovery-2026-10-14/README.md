# AgilePlus intent reconciliation dossier

Status: proposed supplemental recovery analysis, 2026-10-14. This is a source
crosswalk and future-work review surface, not a second specification authority.
Start with the [authority index](../AUTHORITY_INDEX.md), then the accepted
[mature contract](../mature-contract.v1.json) and
[semantic decisions](../CANONICAL_SEMANTIC_DECISIONS.md).

## Scope and evidence limits

This review uses [selected supplied owner excerpts](OWNER_EXCERPTS.md) and
canonical repository files at `45bb5c9fde4d15ee698f42a6e5f9304aae336ff5`.
The accompanying source archive could not be retrieved. Its bytes, digest,
105-row cross-product matrix, proposed 14-package plan and full corpus were not
verified here. This dossier reconstructs only AgilePlus mappings supported by
available sources; it does not claim a complete historical or holistic audit.
Source URLs are retained as supplied, not authenticated read-back evidence.

The canonical denominator is **67 requirements, 12 journeys and 92 specified
verification cases**, derived from the current manifests. Case presence is not
execution. All canonical cases remain `specified_not_executed`. The older
25-case finality prose was stale; no identifier or case has been replaced.
The [crosswalk](requirement-crosswalk.v1.json) links every current requirement
to its existing journeys and oracles, a source theme and an explicit mapping
basis. Broad inherited-contract associations do not claim that every detailed
requirement was literally stated by the owner. No missing archive row is
reconstructed as a historical fact.

## Product intent and ownership

Tracera is a persistent multi-view product/project graph across software,
hardware and other declared domains, with RTM relationships across product
entities, requirements, tests and evidence. Audit history supports that product;
it does not define or exhaust its scope. Product satisfaction and evidence
applicability belong to Tracera. Linked UI, API, SBOM, source and spatial views
must share canonical entity and baseline identity rather than duplicate truth.
These are product goals, not evidence that all views currently work.

AgilePlus owns bounded specification and work execution: durable Development,
accepted SpecRevision, WorkPackage, Assignment, Attempt, Candidate, independent
Evaluation, AcceptedWork and promotion handoff. It supports standalone acceptance
without requiring a live Tracera service. Product graph edits, simulation findings
and research experiments can become work inputs; receipts return through the
versioned federation boundary. Worker completion, review, merge and AcceptedWork
cannot independently award product satisfaction.

The methodology synthesis remains OpenSpec fluidity, Spec Kit structure, BMAD
adaptive depth and general engineering practice, with traceability and rigorous
assignment/oracle feedback. Small work can use a light profile; complex work
requires the depth justified by its risk. The accepted milestones remain
IntentKnown, ContractAccepted, Executable, Attempting, CandidateProduced,
Evaluated, AcceptedWork, Promoted and Archived. They form a partial order;
research/design/planning recur as dependencies demand. No mandatory replacement
FSM or framework-specific authority root is introduced.

## Supported gaps and specification proposals

| Area                     | Existing authority                                                | Supported delta and unresolved evidence                                                                                                                                                           |
| ------------------------ | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Method and specification | AP-R-PROFILE, AP-R-IMPORT, AP-R-NO-DUP-AUTH                       | Retain adaptive profiles and source provenance. A source-to-contract association is not an accepted contract revision.                                                                            |
| Graph-originated work    | AP-R-FEDERATION, AP-R-PRODUCT-BOUNDARY, AP-R-STANDALONE           | Propose baseline-bound handoff for graph edits, scenario findings and experiments; executable end-to-end applicability remains unproven here.                                                     |
| Acceptance and recovery  | ADR-0019; exact candidate/evaluation and persistence requirements | Retain correctness plus governance, immutable history and replay behavior. Existing implementation work has separate ownership; this review supplies no new runtime verdict.                      |
| Model-first scenes       | S05, S07; product/work boundary                                   | Planned Tracera scenario/run model and simulation validity; AgilePlus only executes bounded work derived from them. No simulation or ergonomic accuracy witness was executed.                     |
| Truthful interfaces      | AP-R-INTERFACE-PARITY, AP-R-ERROR-TRUTH, AP-R-OBS-ATTRIB          | Preserve unavailable/stale/error states and exact evidence attribution. Domain-specific product lenses belong to Tracera. Rich previews and usability need separate design and runtime witnesses. |
| Research intake          | S08; AP-R-CONTEXT, AP-R-TRIAGE-PROV, AP-R-EVIDENCE                | Proposed experiment inputs bind source, assumptions, comparison baseline and measured outputs; projected benefit is not realized benefit.                                                         |

The [proposed ADR amendments](../../../docs/adr/proposed-product-work-simulation-boundary.md)
and [future work](FUTURE_WORK.md) describe reviewable deltas. They neither accept
new requirements nor allocate or execute a work graph. Existing product scope,
canonical requirement identifiers, journeys and oracle identifiers are preserved.

## Review boundary

Product owners must reconcile product graph identity, scenario applicability and
simulation fidelity with Tracera. AgilePlus semantic reviewers must review envelope
compatibility, standalone behavior and acceptance prerequisites against ADR-0019.
The existing spatial registration owner retains DAG import/registration and native
transition work. Existing core and dashboard implementation drafts retain their
repair scope and integration path. No competing registration is proposed here.

An accepted follow-up should revise the canonical contract explicitly if it finds
independently gradable new behavior. Until then these documents are proposals and
source evidence only. Missing corpus coverage, source read-back, interface/browser
witnesses and runtime federation remain open; this does not certify product
completion, deployment health or ergonomic safety.
