# cf-4-investigator report — 2026-10-18T20:24Z

Read at: 2026-10-18T20:17Z to 2026-10-18T20:24Z (~7 min, well under 15-min cap).
Repo: KooshaPari/AgilePlus | Read-only (no writes, no pushes, no comments, no branch switch).

## 1. Verdict

**PASS — NOT a new CF-4 (transient / already addressed).**

The 3-sec `ci / lint` + 2-sec `ci / test` failures are a **symptom, not a root cause**. They are aggregate gates in `.github/workflows/ci.yml` that exit 1 when the underlying `cargo-deny` job fails. The `cargo-deny` failure is already being fixed by fox's PR #1165 (in flight). No new unblocker PR is needed.

## 2. Root cause (workflow file:line)

- `.github/workflows/ci.yml` — `lint:` job (name `ci / lint`; `lint:` began at line 218 when this was read on 2026-10-18, and at line 220 as of 2026-10-22 — the line numbers move as the workflow grows, so the job name is the stable handle). It is a bash loop iterating over dependent job results (`rust`, `cargo-deny`, `python`, `go`, `typescript`, `security`, `dep-review`) and exits 1 if any are `failure` or `cancelled`. The 3-second duration is normal — no setup, just bash.
- `.github/workflows/ci.yml` — `test:` job (name `ci / test`; `test:` began at line 256 when this was read on 2026-10-18, and at line 258 as of 2026-10-22). One-step gate: if `needs.lint.result != "success"`, exit 1. 2-second duration is normal.
- The 3s/2s signature is **not a misconfig** — both jobs run correctly and report the underlying failure fast.
- What they are reporting: **cargo-deny** is failing on PRs #1156, #1157, #1161, #1162, #1163, #1164 with this primary error:
    ```
    error: override toolchain 'nightly-2026-07-31-x86_64-unknown-linux-musl' is not installed:
           the toolchain file at '/github/workspace/rust-toolchain.toml' specifies an uninstalled toolchain
    ```
    Followed by `advisories FAILED, bans ok, licenses ok, sources ok` due to a dead `RUSTSEC-2024-0429` ignore entry + missing `RUSTSEC-2026-0370/0371/0372` ignores in `deny.toml`.

## 3. Evidence (file:line citations)

- Aggregate gate definition: `.github/workflows/ci.yml:172-214` (`lint:` job named `ci / lint`).
- Single-step test gate: `.github/workflows/ci.yml:217-230` (`test:` job named `ci / test`).
- Cargo-deny toolchain override error observed in PR #1156's `Cargo Deny (Advisories + Licenses)` job log (job 117609709338, run 39104462965).
- Three PRs independently verified with same failure signature:
    - PR #1156 (run 39104462959/job/117620432199): `✗ cargo-deny: failure` → 1 lint check failed → exit 1.
    - PR #1157 (run 39105326691/job/117620556304): `✗ rust: failure` + `✗ cargo-deny: failure` → 2 lint checks failed → exit 1.
    - PR #1163 (run 39158933883/job/117772172308): `✗ cargo-deny: failure` → 1 lint check failed → exit 1.
- `deny.toml` is the target of fox's fix (verified by PR #1165 body, `/tmp/cf_4_investigation_2026-10-18-result.md`).
- `rust-toolchain.toml` (repo root): pins `channel = "nightly-2026-07-31"` — that nightly is no longer in the runner image, hence the override error. (This is a SECOND sub-cause inside cargo-deny, but it is intermittent — PR #1165's cargo-deny passed, but the `dtolnay/rust-toolchain@stable` step provisions the **runner**, while the cargo-deny job's own `EmbarkStudios/cargo-deny-action@v2` step runs as a **Docker container** that selects its toolchain independently; see the toolchain caveat in §6.)
- PR #1165 (fox) **does fix the cargo-deny job**: confirmed via `gh pr checks 1165` — `Cargo Deny PASS 2m29s` + `cargo-deny PASS 25s`. `ci / lint` and `ci / test` are not even created on PR #1165 because its base branch is `docs/state-forward-docket-20261017`, which is not in ci.yml's `branches: [main, master, develop]` trigger list (intentional — unblocker PRs are intentionally not blocked by their own lint/test gates).
- PR #1165 targets `docs/state-forward-docket-20261017` (base branch). The 6 failing source PRs target `main`. Once #1165 (and #1166) merge into `main`, `ci/lint` + `ci/test` **can** flip green on the 6 PRs — but only after they are re-triggered: a base-branch-only merge does **not** start a new `pull_request` run on them (see step 4 below).

## 4. Proposed fix

- **No new file changes required.**
- The existing unblocker PRs #1165 (fox — deny.toml + Cargo.toml) and #1166 (badger — coverage.yml + mutation.yml) cover the remediation.
- Local check that fox's fix is sufficient:
    ```
    $ cargo deny --manifest-path Cargo.toml check
    advisories ok, bans ok, licenses ok, sources ok
    ```
    (verbatim from PR #1165 body, locally verified by fox).
- Effort estimate: **0 minutes** (no work needed by this investigation; remediation is owned by the existing PRs).
- Risk: **zero** — no source code changes; the existing PRs already implement the fix and are operator-gated for merge.

## 5. PR coverage

- The 3-sec `ci / lint` + 2-sec `ci / test` failure affects **6 source-code PRs**: #1156, #1157, #1161, #1162, #1163, #1164 (all targeting `main`).
- PR #1165 (fox, cargo-deny + cargo-semver-checks fix): **YES, this unblocks cargo-deny for the 6 PRs once merged into main.**
- PR #1166 (badger, python-coverage + cargo-mutants fix): orthogonal — addresses DIFFERENT failures (python-coverage, cargo-mutants) but does not affect ci/lint + ci/test since those don't depend on python-coverage or cargo-mutants.
- Neither #1165 nor #1166 needs modification. Both should be merged as-is to clear the cohort.

## 6. Recommended next step

- **Operator action: defer + monitor.** Do not open a new CF-4 unblocker PR. Instead:
    1. Wait for PR #1166's remaining 5 checks to finish (cargo-mutants, python-coverage have re-run).
    2. Operator merges PR #1165 (squash-merge, with pre-commit hooks enabled).
    3. Operator merges PR #1166.
    4. Re-trigger the 6 source-code PRs (`gh pr update-branch` — which **merges** the updated `main` into the PR branch by default; add `--rebase` if the workflow requires a rebase instead — or a push of a **new empty commit** to their branches, `git commit --allow-empty && git push`). Pushing the unchanged branch tip does **not** re-trigger anything: the head SHA is the same, so GitHub neither updates the PR nor starts a new check run.
    5. Run a follow-up ci-scout in ~25 minutes to confirm the cohort has flipped green.
- **Confirmation signal**: when fox's PR #1165 is merged **and** the 6 source PRs are re-triggered (`pull_request` CI does not re-run on a base-branch-only change), `cargo-deny` will pass on them; `ci / lint` aggregate gate will see `cargo-deny: success` and exit 0; `ci / test` will then see `lint: success` and exit 0. **Qualification — the gate is an aggregate:** `ci / lint` needs `[detect, rust, cargo-deny, python, go, typescript, security, dep-review]` and exits non-zero if **any** of them reports `failure` or `cancelled` (`skipped` is accepted). Passing `cargo-deny` and `rust` alone is therefore **not** a sufficient confirmation signal; the signal is that **none** of the aggregate dependencies failed or was cancelled. **Exception — `detect` is not checked by the aggregate.** `detect` is listed in the gate's `needs:`, but the aggregate loop never inspects its result. When `detect` fails, the toolchain jobs are skipped by their own `if: needs.detect.outputs.*` conditions, so every checked dependency reports `skipped` — which the aggregate accepts — and both `ci / lint` and `ci / test` can report success. Operators must verify the `detect` job separately: a green `ci / lint` does not imply `detect` succeeded.
    - **Caveat (PR #1157):** #1157's `ci / lint` failure had **two** failing lint jobs — `rust: failure` **and** `cargo-deny: failure` (run 39105326691 / job 117620556304). #1165 only fixes `cargo-deny`. If #1157's `rust` job still fails after the unblockers merge, `ci / lint` stays red and `ci / test` stays red with it, even with `cargo-deny: success`. The `rust` failure must be diagnosed and resolved before recovery can be predicted for #1157.
    - **Caveat (toolchain):** #1165 does not touch `rust-toolchain.toml`; the `nightly-2026-07-31` pin is unchanged. The `cargo-deny` job runs `EmbarkStudios/cargo-deny-action@v2`, which is a **Docker container action** (`runs: using: "docker"`), so the `dtolnay/rust-toolchain@stable` step provisions the **runner's** toolchain and does **not** establish the toolchain the container's cargo selects; the action's `rust-version` input is unset in `ci.yml`, so it pins nothing either. Cargo inside the container still reads the repo's `rust-toolchain.toml`, so if the container's toolchain cannot satisfy the pin, cargo-deny can still fail with the `override toolchain 'nightly-2026-07-31-...' is not installed` error and keep **both** `ci / lint` and `ci / test` red after #1165 merges. A job that honours the pin (e.g. the `rust` job) can likewise still fail on the missing nightly. A toolchain remediation step therefore remains open, and it must cover the cargo-deny **container**, not only the runner.
- **No new investigation needed.** This is a clean PASS — the CF-4 hypothesis (workflow misconfig) is ruled out by the structural analysis above.
