# SOTA / Alternatives / Bootstrap Gate

**Status:** BLOCKING for specification completion.
**Captured:** 2026-09-29.

AgilePlus cannot reach 100% design/spec verification until its methodology, execution architecture, and grader are benchmarked against current alternatives. The post-build pilot remains a second empirical gate.

## Required comparison families

### Spec-driven / agent-development methods
- OpenSpec
- GitHub Spec Kit
- BMAD Method
- Kiro
- Tessl
- Spec Kitty
- GSD and Ralph-style execution loops
- Superpowers / agent-skills approaches
- other credible current SDD systems discovered during the review

### Execution/orchestration
- coding-agent harnesses and multi-agent orchestration systems;
- worktree/branch/claim isolation systems;
- issue/DAG/task execution systems;
- CI/review/convergence loops;
- agent context compilation and repository-intelligence approaches.

### Verification/grading
- SWE-bench-style executable task evaluation;
- coding-agent benchmark harnesses;
- mutation/property/contract testing approaches;
- educational autograder concepts where transferable;
- evidence/provenance and reproducible execution systems.

### Established engineering methods
- requirements engineering and EARS/Gherkin where useful;
- ADR/design/architecture practices;
- WBS/DAG/PERT and dependency planning;
- risk/threat/failure analysis;
- change/configuration/release management;
- traceability and V&V practices.

## Questions this gate must answer

1. What should AgilePlus reuse directly instead of reimplementing?
2. Which OpenSpec/Spec Kit/BMAD/Kiro/Tessl concepts should be retained, adapted, conditional, or rejected?
3. Is adaptive process depth actually better than selecting one of these methods directly?
4. Does the assignment/ZyBooks-style oracle model materially improve agent outcomes?
5. Can existing tools already provide enough context compilation, worktree isolation, execution, review, and convergence?
6. What is the minimum unique AgilePlus kernel after commodity capabilities are delegated?
7. What alternative stack would we deploy if AgilePlus did not exist?

## Preliminary findings — not final decisions

- OpenSpec already supplies lightweight change folders, delta specs, fluid artifact editing, and broad agent compatibility.
- Spec Kit now supplies multiple independent processes, Specify→Plan→Tasks→Implement→Converge, brownfield guidance, extensions/presets/workflows, and dozens of integrations.
- BMAD supplies deeper methodology and adversarial review.
- Kiro/Tessl and other systems cover integrated spec workflows or spec-as-source/test linkage.
- Therefore "OpenSpec + Spec Kit + BMAD" is not itself sufficient product differentiation. AgilePlus must justify its adaptive synthesis, executable assignment contract, traceability/evidence semantics, bounded context compiler, claims/execution engine, and autograder.

## Deliverables

- feature/method comparison matrix using primary sources;
- workflow-state and artifact-model comparison;
- architecture/reuse map;
- build-vs-bootstrap decisions;
- adaptive-depth decision table backed by evidence;
- alternative-stack baseline;
- benchmark plan comparing velocity, correctness, cost, retries, spec drift, trace completeness, and human intervention;
- requirement/architecture amendments caused by the study.

No claim of optimal architecture is permitted before these are reviewed.
