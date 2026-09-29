# AgilePlus Autograder Contract

**Mass test generation:** deferred.  
**Verification design:** required now.

The grader evaluates the accepted AgilePlus work contract, not test counts.

It must distinguish behavioral state from trace completeness and must fail closed on missing evidence, expired claims, skipped/empty checks, wrong revisions, unauthorized transitions, mismatched worktree/branch identities, weakened acceptance, or externally closed work without required proof.

Required outputs include mature completeness, VP readiness, feature/journey closure, structural shape, survivability, transition burden, blockers and scope delta.

Each FR in `spec/product/mature-contract.v1.json` reserves acceptance and positive/negative future test IDs. Later test generation fills those concrete assertions without redefining behavior.

The grader must preserve the reciprocal boundary: AgilePlus may prove execution/work state; Tracera independently interprets product acceptance.
