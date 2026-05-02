# Tasks: 002-org-wide-release-governance-dx-automation

**Spec**: `spec.md` | **Plan**: `plan.md` | **Work Packages**: 15 | **Total Subtasks**: 93

## Overview

Codify the existing 5-tier release channel governance (alpha → canary → beta → rc → prod) into reusable GitHub Actions publishing workflows (npm, PyPI, crates.io), DX tooling (`pheno` CLI), standardized git hooks, and task runner configurations. Covers all ~47 Phenotype org repos. Eliminates per-repo manual publishing toil, enforces gate checks at every channel transition, and provides org-wide release status auditing. Currently only 3/47 repos implement the governance model; this spec drives it to 100%.

---

## Phase 0 — Foundation

### WP01: CLI Scaffold & Adapter Interface
**Phase**: 0 | **Wave**: 0 | **Priority**: P0 | **Dependencies**: none

Create the `pheno-cli` Go project with Cobra skeleton, `RegistryAdapter` interface, and version calculation logic. This is the foundation every other WP depends on.

**Why**: Eliminates per-repo tribal knowledge by providing a single, consistent CLI for all release operations. Without this, each repo's CI/repo setup is bespoke and impossible to audit org-wide.

**Effort**: ~450 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T001: Initialize Go module (`pheno-cli`) with Cobra + Viper + Lipgloss deps (Go 1.23+)
- [ ] T002: Create Cobra root command with subcommand stubs (publish, promote, audit, bootstrap, matrix, config)
- [ ] T003: Define `RegistryAdapter` interface in `internal/adapters/adapter.go` — `Detect()`, `Version()`, `Build()`, `Publish()`, `Verify()`
- [ ] T004: Implement `internal/version/calculator.go` — version suffix logic per registry × per channel (7 registries × 5 channels)
- [ ] T005: Implement `internal/detect/detector.go` — language/manifest auto-detection (parallel with T004)
- [ ] T006: Unit tests for version calculator — exhaustively cover all 35 registry × channel combinations

**Acceptance Criteria**:
- `pheno --help` shows all 6 subcommands
- Adapter interface compiles with all 5 method signatures
- Version calculator tests: 35/35 combinations pass (including PEP 440 edge cases)
- PyPI PEP 440 edge cases (dev vs alpha ordering) covered by T006

---

### WP10: Centralized CI Workflows
**Phase**: 0 | **Wave**: 0 | **Priority**: P1 | **Dependencies**: none

Create reusable GitHub Actions workflows in the `phenotypeActions` repo — publish, gate-check, promote, changelog, and audit — so every repo calls the same proven workflow templates. This is the org-wide consistency anchor.

**Why**: Without centralized workflows, each of the 47 repos has its own CI pipeline, its own publish logic, and its own failure modes. Centralizing into reusable workflows means one bug fix benefits all 47 repos simultaneously, and org-wide audit becomes a single workflow invocation.

**Effort**: ~500 lines | **Subtasks**: 7

**Subtasks**:
- [ ] T057: Create `publish.yml` reusable workflow — registry-specific publish with retry/backoff; handles 429 rate-limit with Retry-After parsing
- [ ] T058: Create `gate-check.yml` reusable workflow — run channel-specific gate criteria (`mise run lint`, `mise run test`, etc.); fail on any gate failure
- [ ] T059: Create `promote.yml` reusable workflow — composite: calls gate-check, then publish on success; supports both auto (tag/branch) and manual (workflow dispatch) triggers
- [ ] T060: Create `changelog.yml` reusable workflow — git-cliff changelog generation on release, attached as CI artifact
- [ ] T061: Create `audit.yml` scheduled workflow — org-wide release status report, scheduled nightly, posts to GitHub Issues or PR comment
- [ ] T062: Add workflow inputs/outputs schema (language, registry, channel, risk_profile, version, credentials)
- [ ] T063: Test workflows with `act` or dry-run mode; verify workflow_call inputs/outputs contract

**Acceptance Criteria**:
- All 5 reusable workflows pass workflow_call contract validation
- `publish.yml` retries on 429 and honors Retry-After header
- `gate-check.yml` returns non-zero exit code when any gate fails
- `audit.yml` runs on schedule without manual intervention

---

## Phase 1 — Registry Adapters (all parallel after WP01)

### WP02: npm Adapter
**Phase**: 1 | **Wave**: 1 | **Priority**: P0 | **Dependencies**: WP01

Implement the npm registry adapter — detect, version, build, publish, verify.

**Why**: npm packages represent a significant portion of Phenotype org's publishable artifacts. Without a standardized adapter, each TypeScript/Node repo has its own CI publish logic that must be manually maintained and audited.

**Effort**: ~350 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T007: Implement `internal/adapters/npm.go` — Detect: parse package.json, check `"private": true`
- [ ] T008: Implement npm Version: SemVer pre-release + dist-tag mapping (alpha→alpha, canary→canary, beta→beta, rc→rc, prod→latest)
- [ ] T009: Implement npm Build: `npm pack` producing tarball
- [ ] T010: Implement npm Publish: `npm publish --tag <channel>` with retry/exponential backoff; handle scoped packages (`@org/name`)
- [ ] T011: Implement npm Verify: check npm registry API for published version
- [ ] T012: Unit + integration tests for npm adapter (mock registry responses, real publish in test environment)

**Acceptance Criteria**:
- Detects both plain and scoped packages correctly
- Correct dist-tag applied per channel
- Private packages skip publish step
- npm 2FA/OTP: CI uses granular access tokens with 2FA bypass (documented in code)

---

### WP03: PyPI Adapter
**Phase**: 1 | **Wave**: 1 | **Priority**: P0 | **Dependencies**: WP01

Implement the PyPI registry adapter with PEP 440 versioning.

**Why**: Python packages are second only to Rust in the Phenotype org. PEP 440 has subtle normalization rules that differ from SemVer — a dedicated adapter prevents the 404-publishing-failures that happen when teams hand-roll versioning without understanding PEP 440 ordering.

**Effort**: ~350 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T013: Implement `internal/adapters/pypi.go` — Detect: parse pyproject.toml, check `Private :: Do Not Upload` classifier
- [ ] T014: Implement PyPI Version: PEP 440 normalization (alpha→aN, canary→devN, beta→bN, rc→rcN); `0.2.0a1` not `0.2.0-alpha.1`
- [ ] T015: Implement PyPI Build: `python -m build` (generic, handles hatchling + setuptools + uv_build backends)
- [ ] T016: Implement PyPI Publish: `twine upload` with retry/exponential backoff
- [ ] T017: Implement PyPI Verify: check PyPI JSON API (`https://pypi.org/pypi/<package>/<version>/json`) for version
- [ ] T018: Unit + integration tests for PyPI adapter (PEP 440 version normalization tests, private package skip)

**Acceptance Criteria**:
- PEP 440 normalization: `0.2.0a1` format generated, not SemVer dashes
- Canary maps to `devN` (sorts before alpha per PEP 440)
- Both hatchling and setuptools build backends handled
- Private packages skip publish step

---

### WP04: crates.io Adapter
**Phase**: 1 | **Wave**: 1 | **Priority**: P0 | **Dependencies**: WP01

Implement the crates.io registry adapter with workspace dependency ordering and rate-limit handling.

**Why**: crates.io rate-limits are aggressive and experienced firsthand. Without automatic retry handling and proper topological ordering, workspace publishes fail silently or publish crates out of dependency order, breaking consumers. A dedicated adapter codifies the retry policy and ordering so all Rust repos benefit.

**Effort**: ~400 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T019: Implement `internal/adapters/crates.go` — Detect: parse Cargo.toml, workspace members, `publish = false` field
- [ ] T020: Implement crates.io Version: SemVer pre-release (`-alpha.N`, `-beta.N`, etc.)
- [ ] T021: Implement topological dependency sorting: parse `[dependencies]` path deps, publish leaves first
- [ ] T022: Implement crates.io Build + Publish: `cargo package` + `cargo publish` with 429 retry + Retry-After header handling; **never `--allow-dirty`**
- [ ] T023: Implement crates.io Verify: check crates.io API for published version
- [ ] T024: Unit + integration tests including workspace ordering tests

**Acceptance Criteria**:
- Workspace crates published in correct topological order
- Dirty working tree blocks publish with clear error
- 429 responses trigger exponential backoff using Retry-After header
- `publish = false` crates skipped

---

### WP05: Go Proxy + Pre-Wired Stub Adapters
**Phase**: 1 | **Wave**: 1 | **Priority**: P1 | **Dependencies**: WP01

Implement the Go module proxy adapter and pre-wired stub adapters for Hex.pm, Zig, and Mojo.

**Why**: Go proxy is architecturally distinct — publishing is a git tag push, not an upload. Stub adapters for future registries (Hex, Zig, Mojo) are pre-wired now to avoid retrofit work later. This reduces toil across the entire registry ecosystem.

**Effort**: ~400 lines | **Subtasks**: 7

**Subtasks**:
- [ ] T025: Implement `internal/adapters/goproxy.go` — Detect: parse go.mod; Version: v-prefix SemVer (`v0.2.0-alpha.1`)
- [ ] T026: Implement Go Publish: git tag create + push — proxy pulls from VCS, no upload step needed
- [ ] T027: Implement Go Verify: poll proxy.golang.org with 5-minute timeout for module version
- [ ] T028: Implement `internal/adapters/hex.go` — Pre-wired stub: Detect from mix.exs, Version from SemVer, Publish/Verify return `ErrNotSupported`; parses package name and version minimally
- [ ] T029: Implement `internal/adapters/zig.go` — Pre-wired stub: Detect from build.zig.zon, git-tag-based versioning
- [ ] T030: Implement `internal/adapters/mojo.go` — Pre-wired stub: Detect from mojoproject.toml, returns `"no registry available"`
- [ ] T031: Unit tests for Go adapter + stub adapter behavior (mock git operations, verify ErrNotSupported paths)

**Acceptance Criteria**:
- Go publish creates and pushes git tag correctly
- Go Verify polls with backoff up to 5 minutes
- All stubs return `ErrNotSupported` for unsupported operations (not silent success)
- Hex adapter minimally parses mix.exs

---

### WP06: Gate Evaluation Engine
**Phase**: 1 | **Wave**: 1 | **Priority**: P1 | **Dependencies**: WP01

Build the channel promotion gate evaluation system — define criteria per channel, evaluate, generate structured reports.

**Why**: The core governance enforcement mechanism. Without gate checks, any developer can promote a package to `rc` or `prod` without running integration tests or documenting a rollback plan. This engine makes the 5-tier governance enforceable, not aspirational — reducing the risk of broken releases reaching production consumers.

**Effort**: ~400 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T032: Define gate criteria data model in `internal/gate/criteria.go` — per-channel requirements (from STACKED_PRS_AND_RELEASE_CHANNELS.md)
- [ ] T033: Implement gate evaluator in `internal/gate/evaluator.go` — run criteria, collect results, return structured output
- [ ] T034: Implement risk-based channel skip logic: high-risk must traverse all intermediates; low-risk can skip; configurable per-package
- [ ] T035: Implement structured report generation: pass/fail per criterion, stdout/stderr capture, duration, total time
- [ ] T036: Implement gate criteria: `lint`, `unit_tests`, `integration_tests`, `security_audit`, `docs_build`, `rollback_plan`
- [ ] T037: Unit tests for evaluator (mock task runner commands, test risk-based skipping)

**Acceptance Criteria**:
- canary: lint + unit tests + security pass
- beta: user flows validated, default behavior unaffected
- rc: API contract freeze, migration/rollback runbook attached, docs synced
- prod: monitoring dashboards configured, rollback RTO met
- Risk-based skip: low-risk packages can skip intermediate channels; high-risk must traverse all
- Gate criteria configurable per-repo via `.pheno.toml`

---

## Phase 2 — CLI Commands

### WP07: CLI Publish & Promote Commands
**Phase**: 2 | **Wave**: 2 | **Priority**: P1 | **Dependencies**: WP01, WP02, WP03, WP04, WP05, WP06

Wire up `pheno publish` and `pheno promote` commands using adapters and gate engine.

**Why**: `pheno promote` is the primary DX command — developers run this to advance a package through channels. It must be frictionless (one command) while being rigorous (gates enforced). Without this, the governance model requires developers to manually orchestrate CI steps, leading to mistakes and shortcuts.

**Effort**: ~400 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T038: Implement `cmd/publish.go` — detect packages, select adapter, build, publish (no gates — direct publish for manual intervention)
- [ ] T039: Implement `cmd/promote.go` — validate channel transition, run gate evaluation, publish on pass, block with report on fail
- [ ] T040: Implement workspace publishing orchestration — topological order, verify between publishes
- [ ] T041: Add Lipgloss-styled progress output — publish progress bars, gate results table with pass/fail indicators
- [ ] T042: Add Viper config loading — registry credentials from env vars, then config file, then GitHub secrets; `~/.config/pheno/config.toml` global, `.pheno.toml` per-repo
- [ ] T043: Integration tests — mock registries, test full publish and promote flows end-to-end

**Acceptance Criteria**:
- `pheno publish` correctly detects all package types and routes to correct adapter
- `pheno promote` blocks promotion when any gate fails, with structured failure report
- Lipgloss table renders pass/fail status clearly
- Credentials loaded in priority order (env → config → GH secrets)
- Workspace crates published in topological order

---

### WP08: CLI Audit & Matrix Commands
**Phase**: 2 | **Wave**: 2 | **Priority**: P2 | **Dependencies**: WP01, WP02, WP03, WP04, WP05

Implement `pheno audit` (org-wide release status) and `pheno matrix` (release matrix generation).

**Why**: Provides org-wide visibility — the single biggest gap in the current state. Without `pheno audit`, release managers must manually check each of 47 repos to understand what's published where. This reduces a multi-hour manual audit to a single CLI invocation, eliminating toil at scale.

**Effort**: ~350 lines | **Subtasks**: 5

**Subtasks**:
- [ ] T044: Implement `cmd/audit.go` — scan configured repos, detect packages, query registries in parallel (one goroutine per repo), collect current versions
- [ ] T045: Implement Lipgloss-styled audit table — columns: package, channel, version, registry URL, blocked-by (failing gate)
- [ ] T046: Implement `cmd/matrix.go` — generate release matrix matching RELEASE_MATRIX_TEMPLATE.md format (markdown table)
- [ ] T047: Add repo discovery — scan `repos_dir` for repos with supported manifests; configurable via `~/.config/pheno/config.toml` → `repos_dir`
- [ ] T048: Unit + integration tests for audit and matrix commands (mock registry responses)

**Acceptance Criteria**:
- `pheno audit` completes org-wide scan in under 30 seconds
- Registry API rate limits handled via throttling
- Blocked packages highlighted with failing gate name
- Matrix output matches governance template format

---

## Phase 3 — DX Tooling

### WP11: Task Runner Evaluation & Standardization
**Phase**: 3 | **Wave**: 3 | **Priority**: P1 | **Dependencies**: WP07

Finalize task runner choice (mise recommended), create standardized task definitions for all 4 active languages.

**Why**: Standardized tasks are the foundation of DX consistency. When `mise run lint` works identically in every Phenotype repo, developer onboarding time drops dramatically and CI gate checks become deterministic. This replaces the current fragmented state (31/47 repos with Taskfile.yml, no standardized targets) with a uniform interface.

**Effort**: ~400 lines | **Subtasks**: 7

**Subtasks**:
- [ ] T064: Final evaluation: validate mise monorepo tasks feature stability; fallback to moon if needed; document decision in ADR
- [ ] T065: Create reference mise.toml for Rust projects — `cargo clippy`, `cargo test`, `cargo build`, `rustfmt`, MSRV pinning
- [ ] T066: Create reference mise.toml for Python projects — `ruff check`, `pytest`, `ruff format`, `python -m build`
- [ ] T067: Create reference mise.toml for TypeScript projects — `eslint`, `vitest`, `tsc --noEmit`, `prettier --check`
- [ ] T068: Create reference mise.toml for Go projects — `golangci-lint run`, `go test ./...`, `go build ./...`, `gofmt`
- [ ] T069: Create reference mise.toml with `release:promote` and `release:status` tasks — calls `pheno promote` and `pheno audit --repo .`
- [ ] T070: Validate all reference configs on sample repos from the org (Rust, Python, TypeScript, Go)

**Acceptance Criteria**:
- All 4 language configs run correctly on sample repos
- `release:promote` calls pheno CLI correctly
- Tool version pinning present in all configs
- If mise monorepo tasks unstable, moon fallback documented

---

### WP12: Pre-Commit & Pre-Push Hooks
**Phase**: 3 | **Wave**: 3 | **Priority**: P2 | **Dependencies**: WP11

Create standardized git hook infrastructure — conventional commit enforcement, fast lint, channel-aware pre-push validation.

**Why**: Currently only 9/47 repos have pre-commit hooks. This creates inconsistent code quality across the org and makes it easy for non-conventional commits to enter the pipeline. Standardized hooks enforce the conventional commit format and run fast checks in <5s, reducing CI rejections and improving changelog quality for git-cliff.

**Effort**: ~350 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T071: Create pre-commit hook script — validate `^(feat|fix|chore|docs|refactor|test|perf|ci|build|style|revert)(\(.+\))?!?: .+`
- [ ] T072: Add fast lint check to pre-commit — format check (`mise run format -- --check`), encoding validation; <5s target
- [ ] T073: Create pre-push hook with channel-aware logic — `feature/*` → fast checks only; `beta/*` → full suite; `rc/*` → full suite + rollback plan check
- [ ] T074: Create `.pre-commit-config.yaml` template — for repos using pre-commit framework
- [ ] T075: Create standalone hook installer script — for repos not using pre-commit framework (POSIX sh)
- [ ] T076: Test hooks — conventional commit rejection, timing validation, channel branching logic

**Acceptance Criteria**:
- Non-conventional commit messages rejected 100% of the time
- Pre-commit completes in under 5 seconds for typical commits
- Pre-push runs appropriate checks based on branch name pattern
- Both pre-commit framework and standalone hook supported

---

## Phase 4 — Bootstrap & Rollout

### WP09: CLI Bootstrap Command
**Phase**: 4 | **Wave**: 4 | **Priority**: P2 | **Dependencies**: WP01, WP10

Implement `pheno bootstrap` — one-command governance onboarding for any repo.

**Why**: Without frictionless bootstrap, the 47-repo rollout requires manual per-repo setup. `pheno bootstrap` makes adoption a one-command operation, reducing rollout toil from weeks of manual work to a single automated pass. This is the primary mechanism for reaching 100% governance coverage.

**Effort**: ~450 lines | **Subtasks**: 8

**Subtasks**:
- [ ] T049: Implement `cmd/bootstrap.go` — orchestrate artifact generation based on detected languages (uses WP01 detector)
- [ ] T050: Create Go template files in `internal/templates/` for all generated artifacts
- [ ] T051: Implement mise.toml template generation — standard tasks (lint, test, build, format, release:promote, release:status) per language
- [ ] T052: Implement pre-commit hook template generation — conventional commit enforcement + fast lint
- [ ] T053: Implement pre-push hook template generation — channel-aware validation
- [ ] T054: Implement CI workflow wrapper templates — `ci.yml`, `release.yml` calling `KooshaPari/phenotypeActions/.github/workflows/<name>.yml@v1`
- [ ] T055: Implement cliff.toml template generation — git-cliff changelog config
- [ ] T056: Integration test — bootstrap a mock repo and validate all generated artifacts

**Acceptance Criteria**:
- `pheno bootstrap` on a bare repo generates all appropriate artifacts in under 1 minute
- Multi-language repos get merged configs
- Private repos skip publishing templates but get lint/test/hook infrastructure
- Template version-pinning references `phenotypeActions` workflows (not inline)
- Bootstrap detects and warns on conflicting existing configs (does not overwrite without confirmation)

---

### WP13: Pilot Rollout — AgilePlus + 3 Repos
**Phase**: 4 | **Wave**: 4 | **Priority**: P2 | **Dependencies**: WP09, WP10, WP11, WP12

Run `pheno bootstrap` on AgilePlus and 3 diverse repos (1 Rust, 1 Python, 1 Go) to validate end-to-end workflow.

**Why**: Pilot validates that templates work in real repos before org-wide rollout. Discovering template bugs during org-wide rollout creates massive rework; discovering them during a 4-repo pilot is cheap. This reduces rollout risk dramatically.

**Effort**: ~350 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T077: Bootstrap AgilePlus (TypeScript/VitePress) — validate mise.toml, hooks, CI workflows; AgilePlus is private (no publish)
- [ ] T078: Bootstrap tokenledger (Rust) — validate Rust-specific artifacts, crates.io publish test to alpha channel
- [ ] T079: Bootstrap thegent (Python) — validate Python-specific artifacts, PyPI publish test to canary channel
- [ ] T080: Bootstrap agentapi-plusplus (Go) — validate Go-specific artifacts, git tag-based publish test
- [ ] T081: Run `pheno audit` across all 4 repos — validate org-wide view works
- [ ] T082: Document pilot findings — adjust templates based on real-repo feedback; capture all fixes

**Acceptance Criteria**:
- All 4 bootstrap attempts succeed without manual intervention
- CI workflows trigger correctly after bootstrap
- At least 1 test publish succeeds (Go module git tag push)
- Pilot findings doc produced with concrete template fixes

---

### WP14: Org-Wide Rollout Automation
**Phase**: 4 | **Wave**: 5 | **Priority**: P3 | **Dependencies**: WP13

Script the remaining ~43 repo rollout and create bulk bootstrap tooling.

**Why**: Scales the pilot from 4 repos to all 47 in an automated pass. Without this, reaching 100% coverage requires 43 manual bootstrap operations, which is infeasible at agent-swarm scale.

**Effort**: ~300 lines | **Subtasks**: 5

**Subtasks**:
- [ ] T083: Create bulk bootstrap script — `pheno bootstrap --all --repos-dir ~/CodeProjects/Phenotype/repos/`
- [ ] T084: Generate repo manifest — CSV/TOML listing all repos, languages, risk profiles, publish targets (derived from audit data)
- [ ] T085: Run bulk bootstrap on remaining repos — with `--dry-run` first, then full execution
- [ ] T086: Create PRs for each bootstrapped repo — via `gh pr create`, one PR per repo, title: "chore: add release governance infrastructure"
- [ ] T087: Validate org-wide `pheno audit` after rollout — confirm all 47 repos visible in audit output

**Acceptance Criteria**:
- `--dry-run` shows what would be generated without writing files
- Bulk operation handles GitHub API rate limits via batching with delays
- All 43 PRs created successfully (or graceful failure with clear error per repo)
- `pheno audit` shows 47/47 repos after completion

---

## Phase 5 — Polish

### WP15: Documentation & Polish
**Phase**: 5 | **Wave**: 5 | **Priority**: P3 | **Dependencies**: WP07, WP11, WP13

Write user-facing documentation for the pheno CLI, governance model, and contributor onboarding.

**Why**: Documentation is what makes the governance model durable — anyone can read the docs and successfully bootstrap a repo, run standard tasks, and publish a pre-release. Without docs, the system depends on oral tradition and breaks when key people leave.

**Effort**: ~300 lines | **Subtasks**: 6

**Subtasks**:
- [ ] T088: Write pheno CLI README.md — installation, all commands, configuration, common workflows
- [ ] T089: Write governance model overview — evolving the 5-tier model, risk profiles, gate criteria, channel definitions
- [ ] T090: Write contributor quickstart — bootstrap → develop → promote → publish, with worked examples
- [ ] T091: Create ADR: task runner selection rationale (mise vs moon decision)
- [ ] T092: Create ADR: registry adapter architecture rationale (interface design decisions)
- [ ] T093: Final cleanup — ensure all error messages are clear and actionable, all help text is complete

**Acceptance Criteria**:
- README.md is complete and accurate for all subcommands
- Contributor quickstart allows a new developer to go from zero to first publish in under 15 minutes
- Both ADRs follow constitution requirements and go in `docs/adr/`

---

## Dependency Summary

| WP | Dependencies | Blocks |
|----|-------------|--------|
| WP01 | — | WP02, WP03, WP04, WP05, WP06, WP07 |
| WP10 | — | WP09, WP13 |
| WP02 | WP01 | WP07 |
| WP03 | WP01 | WP07 |
| WP04 | WP01 | WP07 |
| WP05 | WP01 | WP07 |
| WP06 | WP01 | WP07 |
| WP07 | WP01, WP02, WP03, WP04, WP05, WP06 | WP11 |
| WP08 | WP01, WP02, WP03, WP04, WP05 | — |
| WP11 | WP07 | WP12 |
| WP12 | WP11 | WP13 |
| WP09 | WP01, WP10 | WP13 |
| WP13 | WP09, WP10, WP11, WP12 | WP14 |
| WP14 | WP13 | — |
| WP15 | WP07, WP11, WP13 | — |

**Critical path**: WP01 → WP02/WP03/WP04/WP05 → WP07 → WP11 → WP12 → WP13 → WP14
**Secondary path**: WP01 → WP06 → WP07 → WP11 → WP12 → WP13
**Parallel foundation**: WP01 + WP10 (both Phase 0, no dependencies on each other)

---

## Success Criteria Mapping

| SC | Criterion | WP |
|----|-----------|-----|
| SC-001 | 100% of public packages publishable via automated CI within 10 min of promotion | WP07 |
| SC-002 | All 5 channels correctly formatted and accepted by registries on first attempt | WP02, WP03, WP04, WP05 |
| SC-003 | Any developer runs `lint`, `test`, `build` successfully within 2 min of setup | WP11 |
| SC-004 | Gate checks block 100% of promotions that fail mandatory criteria | WP06 |
| SC-005 | Pre-commit rejects non-conventional messages 100%, completes in <5s | WP12 |
| SC-006 | New repo fully bootstrapped in under 1 minute via single command | WP09 |
| SC-007 | Org-wide audit completes in under 30 seconds | WP08 |
| SC-008 | Manual publishing errors reduced to zero (no dirty-tree publishes, no rate-limit surprises) | WP04, WP07 |

---

## MVP Recommendation

**WP01 + WP10** is the MVP scope. Once the pheno CLI scaffold and centralized CI workflows exist, all Phase 1 adapters (WP02–WP06) can be dispatched in parallel — they only need WP01's adapter interface. The first parallel wave should be: WP02 (npm), WP03 (PyPI), WP04 (crates.io), WP05 (Go/stubs), WP06 (gate engine), WP10 (CI workflows).

Once adapters + CI workflows are ready, WP07 (publish + promote) unblocks WP11 (task runner) and WP09 (bootstrap). WP08 (audit + matrix) is the fastest win for org-wide visibility and requires only the adapters, not the full CLI.
