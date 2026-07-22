# Delivery DAG and Work Breakdown

## Dependency graph

`B0 clean build` -> `R1 runtime contract` -> `R2 launcher/live API` -> `R6 MCP/API stream`

`B0 clean build` -> `R3 credential protection`

`R1 runtime contract` -> `R5 MinIO artifact adapter`

`R2 live API` + `R3 credentials` -> `R4 persistent events/query`

`R4 events` + `R5 artifacts` + `R6 streaming` -> `D1 AgilePlus evidence journey`

`D1` -> `D2 Tracera dogfood` -> `D3 Grapheon recovery and dogfood` -> `D4 portfolio rollout`

## Work packages

| ID | Outcome | Depends on | Acceptance evidence |
|---|---|---|---|
| B0 | Clean reproducible release build | none | `cargo build --release` generates services; no runnable stub fallback; workspace tests pass. |
| R1 | One runtime resolver and endpoint contract | B0 | unit tests for precedence/conflicts; launcher and status report identical endpoints. |
| R2 | Real API/gRPC launch lifecycle | R1 | process start, health, gRPC read, graceful shutdown, failure-log tests. |
| R3 | Keychain-first, encrypted fail-closed credential and API-key store | B0 | secret scan/redaction; keychain and AES-256-GCM/Argon2id fallback store/read/rotate/delete; unavailable/decrypt-failure tests. |
| R4 | Persistent event repository/query/cursor | R2,R3 | restart persistence; type/filter/page/cursor and chain verification tests. |
| R5 | MinIO artifact adapter | R1 | isolated MinIO put/get/digest/authorization/failure tests. |
| R6 | Authenticated API/MCP stream | R2,R4 | live stream, resume/no-duplicate, heartbeat, authorization tests. |
| D1 | AgilePlus self-dogfood evidence pack | R3-R6 | immutable manifest verifies every required evidence reference. |
| D2 | Tracera dogfood | D1 | full journey and project-scoped evidence pack pass. |
| D3 | Grapheon recovery plus dogfood | D2 | no conflict markers, clean build/test, full journey evidence pack. |
| D4 | Remaining portfolio | D3 | one evidence pack and explicit go/no-go record per project. |

## Parallelization boundary

R3 can proceed independently after B0. R1 must finish before R2/R5. R4 and R5 can run
in parallel after their listed prerequisites. R6 begins only after a real R2 server and
R4 cursor contract exist. Dogfood never begins until R1-R6 are green.
