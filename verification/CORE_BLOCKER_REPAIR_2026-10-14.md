# Core blocker repair candidate — 2026-10-14

Candidate extends PR1120 at `a7e706be8bd33ad05345274dabb2dbbbde2f5e9e`.
The original PR branch and draft checkpoint are preserved. Instinct reviews the
candidate first and coordinates the single final integration path; no merge or
deployment is authorized by this evidence.

## Counterexamples and repair

The original head failed both real-process ownership regressions: an in-flight
stale-owner recovery allowed a second successful contender, and a failed Unix
process probe authorized replacing a legacy owner record. The candidate holds a
persistent kernel-locked `core-writer.guard` throughout acquisition, stale-record
quarantine and cleanup. This guard inode is never deleted or renamed. A crash
releases its OS lock. Ambiguous/failed process probes fail closed.

New owner records use version 2, which legacy binaries refuse to recover. Version
1 records are retained and probed before migration. Adoption must be coordinated:
all recovery contenders must use the new protocol; an already-in-flight legacy
recovery process cannot be fenced by an advisory lock it does not acquire. Do not
manually delete the guard inode while a writer is running. PID reuse remains
conservative. This does not migrate direct CLI writers to the core lock.

Standard-library file locking requires Rust 1.89; the workspace declares 1.88.
The candidate therefore pins `fs2 = 0.4.3`. Cargo resolution adds that one package
and its direct gRPC dependency entry, without updating existing versions.
Execution was on nightly-2026-07-31; a Rust 1.88 or native Windows run was not made.

The original SQLite runtime updater also failed regressions for expired-attempt
completion and rewriting a completed candidate. A conditional SQL UPDATE now
permits active progress or an identical terminal callback, rejecting divergent
terminal writes with Conflict. Expired/failed/cancelled history cannot be revived.
The comparison and mutation are one SQL statement. This is terminal-update
fencing, not a complete worker lease/generation or external side-effect fence.

## Executed acceptance

All commands ran in the isolated candidate after sourcing
`/workspace/.onboarding/env.sh`, with two Cargo jobs and dev/test debug disabled.

| Command | Actual result |
| --- | --- |
| `cargo test --locked -p agileplus-grpc --lib --test writer_lock_regressions --test server_process_lifecycle --test atomic_acceptance_rpc -- --test-threads=1` | 79 passed, zero failures/ignored; includes one child-probe harness entry |
| `cargo test --locked -p agileplus-sqlite -- --test-threads=1` | 614 passed, zero failures/ignored |
| `cargo test --locked -p agileplus-cli --test implement_command_flow -- --test-threads=1` | 11 passed |
| `cargo test --locked -p agileplus-cli --lib restart_reconciliation_expires_only_live_attempts` | 1 passed; 765 filtered |
| `cargo clippy --locked -p agileplus-grpc -p agileplus-sqlite --all-targets -- -D warnings` | Exit 0 |
| `cargo fmt --all -- --check` and `git diff --check` | Exit 0 |

Regression sources: `writer_lock_regressions.rs` and
`repository::execution::runtime_update_regressions`. They preserve the rejected
owner/attempt records, check active and idempotent progress, and prove abrupt
owner exit releases the guard while the stale record remains quarantined.

These selected Linux witnesses do not prove universal single writer, duplicate-
free worker execution, product-obligation linkage, tailnet or desktop deployment.
Existing PR1122/1123 frontend/history gate repairs remain separate owned work.
