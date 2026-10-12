# Runtime convergence checkpoint — 2026-10-04

This receipt records implementation state that supersedes earlier forward-WBS
items; it is evidence of authored code, not a claim that the newest candidate
has passed native CI.

## Canonical acceptance transports

CLI, HTTP and gRPC now converge on the same application operation and
AtomicAcceptancePort.

The gRPC acceptance decorator:

- intercepts only validate; other commands retain existing behavior;
- requires an authenticated bearer API credential;
- requires matching project scope;
- requires caller-supplied durable request_id;
- permits only request_id and optional expected_governance_version arguments;
- rejects attempts to override authority/grading semantics;
- resolves the Feature through StoragePort;
- invokes accept_feature() over the same atomic storage capability;
- returns the durable acceptance receipt and replay state;
- maps validation/precondition failures, conflicts and infrastructure failures
  to distinct gRPC statuses.

Shipping remains fail-closed over gRPC until promotion owns its own canonical
application service.

## Worker replacement/restart

The implementation path now expires interrupted Attempts before replacement and
has authored witnesses that:

- restart reconciliation preserves prior Attempt history;
- an interrupted Attempt remains historical rather than being overwritten;
- a replacement Attempt can proceed under the same durable Assignment.

This advances the worker-replacement WBS but does not close process-death
recovery until native execution and broader crash-point tests are green.

## Immediate gate

Obtain one non-bot candidate where native CI actually runs:

cargo fmt -> clippy -> compile -> cargo test

and specifically executes the atomic acceptance, real-SQLite HTTP acceptance,
gRPC acceptance and interrupted-attempt replacement witnesses.

Do not begin canonical promotion mutation until this gate is understood; ship
currently contains useful drift protections but still mixes VCS side effects,
cleanup, terminal state mutation, audit and event persistence in CLI code.
