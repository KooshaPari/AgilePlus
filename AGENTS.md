# AgilePlus agent instructions

## Repository identity

This repository is the **AgilePlus tooling/product codebase**. The repository root is the working source root.

It is **not**:

- a parent shelf containing many independent projects;
- an organization or personal planning repository;
- a destination for sibling-repository checkouts, worktrees, specifications, or product documents;
- the canonical product model owned by Tracera;
- the canonical home of Plane-derived product forks.

AgilePlus may register and operate on external Git project roots. Runtime references to an external repository do not transfer source or specification ownership into AgilePlus.

## Scope and authority invariants

- AgilePlus owns governed code-project specifications, work packages, execution records, evidence, evaluation, audit, and development acceptance.
- AgilePlus `AcceptedWork` is not Tracera `Satisfied`.
- Tracera owns persistent product/system records and product-state interpretation.
- Plane or Paperclip, if adopted, owns organization/personal planning one level above code projects.
- No external state-transition transport may award AgilePlus terminal acceptance.
- A test not executed is not a test passed.
- Documentation, badges, code presence, compilation, or a successful projection is not proof of an accepted end-to-end journey.
- Current `main` is evidence, not automatically the complete intended product.

When requirements, code, and documents disagree, preserve the contradiction and use the accepted authority chain rather than choosing the newest or largest artifact.

## Operational authority

For mature-product and release work, begin with:

- `spec/product/AUTHORITY_INDEX.md`
- `spec/product/mature-contract.v1.json`
- `spec/product/CANONICAL_SEMANTIC_DECISIONS.md`
- `verification/FORWARD_WBS_2026-10-04.md`
- `verification/FORWARD_DELTA_2026-10-04.md`
- `verification/NEXT_DEPLOYABLE_RELEASE_2026-10-04.md`
- `STABLE.md`
- applicable records in `docs/adr/`

Relevant boundary decisions include:

- `docs/adr/0018-tracera-agileplus-sibling-runtimes.md`
- `docs/adr/0019-acceptance-authority-terminal-gates.md`

Historical documents remain evidence. They do not override accepted human intent or active authority records merely because they are tracked here.

## Repository boundary rules

1. Work from this repository root or an isolated AgilePlus worktree.
2. Do not create `AgilePlus/`, `agileplus/`, `projects/`, or other nested containers as alternate source roots.
3. Do not clone or place sibling repositories inside this repository.
4. Do not copy a sibling project's canonical specifications into AgilePlus. Store a typed reference or handoff instead.
5. Do not treat embedded historical/sibling-looking paths as active dependencies until provenance, build reachability, and ownership are verified.
6. Do not mass-delete historical or foreign-looking material. Classify and migrate it through issue #1128 and the forensic disposition checkpoint.
7. New Planify/Plane-derived product work belongs in Planify, not in an AgilePlus client directory.
8. Tracera changes are handled by the Tracera repository/recovery process; AgilePlus may provide contract handoffs only unless separately authorized.

## Current implementation surfaces

- Rust workspace: `crates/`
- Agent adapters/services: `agileplus-agents/`
- Python FastMCP bridge: `python/`
- Native Askama/Axum web application: `crates/agileplus-dashboard/`
- Active desktop/tray client: `desktop/` using Tauri
- Product contracts: `spec/product/`
- Verification/evidence records: `verification/`
- Architecture and decisions: `docs/`

Electrobun and other frontend/client lineages are historical candidates unless an explicit recovery decision says otherwise. Do not create another frontend to avoid reconciling the existing web and Tauri paths.

## Common commands

```bash
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo deny check
ruff check python
```

Entry points:

```bash
cargo run -p agileplus-cli -- --help
cargo run -p agileplus-dashboard
cargo run -p agileplus-grpc
uv run --project python agileplus-mcp
```

Use the repository's pinned toolchains and workflow commands when they differ. Record exact revision, environment, command, exit status, and relevant logs.

## Change discipline

- Use a focused branch or isolated worktree.
- Keep unrelated fixes in separate commits/PRs, especially baseline CI repairs versus product work.
- Preserve dirty files and concurrent work; never reset, clean, stash, prune, or switch another agent's active worktree without authority.
- Do not rewrite history, delete branches, merge, deploy, publish, change billing/DNS/authentication, or rotate credentials unless explicitly authorized.
- Pull requests remain draft while executable evidence or required review is incomplete.
- A green aggregate check is not sufficient if a required subcheck was skipped, waived, or tested the wrong artifact.
- A red aggregate check is not automatically a product defect; inspect the failing job and separate baseline/tooling failure from branch-local behavior.

## Work and acceptance rules

- Stable IDs and accepted lifecycle transitions come from the canonical AgilePlus application/domain boundary, not raw client SQL.
- CLI, MCP, web, and tray should converge on one writer/application service.
- WorkPackage `Done` and Feature `Validated` require the acceptance and governance conditions in ADR-0019.
- Review approval, external task completion, evidence presence alone, or caller-supplied booleans are insufficient.
- Preserve exact-candidate, SpecRevision, Assignment, Attempt, Evaluation, evidence, audit, event, and receipt identity.
- External projections are asynchronous and non-authoritative; their failure must not roll back committed local acceptance.

## Desktop and OS safety

- Never blind-test on the user's foreground operating system.
- Never capture the user's live desktop merely to prove a visual flow.
- Prefer CI, disposable users/VMs/containers, process/window enumeration, structured accessibility output, and logs.
- Any actual clean-account install, startup registration, signing/notarization, tray, or rollback test must identify the machine/platform and preserve user data.
- Do not claim a hard timeout unless descendant processes and inherited output pipes are bounded and reaped on each supported platform.

## Documentation and evidence

- Mark facts as observed, historically demonstrated, inferred, proposed, blocked, or unknown.
- Preserve stable requirement/ADR/work IDs and source provenance.
- Do not reconstruct missing human wording and call it verbatim.
- Do not turn a historical prototype into a permanent requirement by default.
- Do not bless an accidental regression by rewriting the requirement to match it.
- Root authority contamination and shelf-era material are tracked by issue #1128 and `docs/forensics/ROOT_SCOPE_CONTAMINATION_CHECKPOINT.md`.

## Agent authority

Agents may investigate, prepare isolated changes, run bounded tests, and open reviewable draft pull requests when authorized. Agents do not merge or deploy by default. Human/operator rulings control product scope and final authority.
