# Tasks: 008-temporal-deployment-workflow-migration

**Spec**: `spec.md` | **Plan**: `plan.md` | **Work Packages**: 12 | **Total Subtasks**: 59

## Overview

Big Bang migration of all NATS JetStream workflow logic to Temporal (critical durable workflows) + Hatchet (lightweight CI/cron) on Hetzner AX101. NATS becomes a pure event bus post-migration. 48-hour dual-write observation period before NATS decommission.

---

## Phase 1 — Foundation

### WP01: Temporal Docker Compose Deployment
**Phase**: 1 | **Wave**: 0 | **Priority**: P0 | **Dependencies**: none

Deploy Temporal as a Docker Compose stack on Hetzner AX101. All subsequent WPs depend on this being operational.

**Implementation command**: `spec-kitty implement WP01`

**Subtasks**:
- [ ] T001: Write `infra/temporal/docker-compose.yml` (temporal-auto-setup:1.26.0, postgres:16, elasticsearch:8.14.0, admin-tools; ports 7233 gRPC, 8233 frontend; healthchecks on all services)
- [ ] T002: Write `infra/temporal/dynamicconfig/dynamicconfig.yaml` (search attributes, force refresh)
- [ ] T003: Write `infra/temporal/.env.example` (TEMPORAL_DB_PASSWORD template)
- [ ] T004: Wire Temporal frontend (port 8233) into Caddy as `temporal.internal` with TLS and logging
- [ ] T005: Add Temporal entry to process-compose.yml (depends on PostgreSQL)
- [ ] T006: Run `curl http://localhost:8233/health`, `temporal operator namespace list`, Elasticsearch cluster health check
- [ ] T007: Measure RAM usage of full Temporal stack (Postgres + Elasticsearch + temporal) via `docker stats`, document headroom
- [ ] T008: Test full stack restart: `docker compose -f infra/temporal/docker-compose.yml restart`, verify health returns within 30s
- [ ] T009: Create `docs/infra/temporal/DEPLOY.md` (install prerequisites, env setup, first-start, upgrade path)

**Parallelization**: T001–T005 can run sequentially in one session; T006–T009 are sequential verification.

**Estimated prompt size**: ~400 lines | **Subtask count**: 9 ⚠️ (high but justified — all infra config)

---

### WP02: Temporal Worker SDK Integration
**Phase**: 1 | **Wave**: 0 | **Priority**: P0 | **Dependencies**: WP01

Integrate the Temporal Rust SDK (`temporal-sdk`) into the codebase. Implement a minimal worker that connects to Temporal, registers a skeleton workflow and activity, and confirms end-to-end connectivity. Establishes the foundation for all workflow migrations.

**Implementation command**: `spec-kitty implement WP02 --base WP01`

**Subtasks**:
- [ ] T010: Add `temporal-sdk = "0.8"`, `tokio = { version = "1", features = ["full"] }`, `anyhow`, `tracing`, `serde` dependencies to relevant crate `Cargo.toml`
- [ ] T011: Create `crates/temporal-worker/` module structure (`Cargo.toml`, `src/lib.rs`, `src/client.rs`, `src/workflows.rs`, `src/activities.rs`, `src/main.rs`)
- [ ] T012: Implement `src/client.rs` — `create_client()` using `ClientOptionsBuilder`, reads `TEMPORAL_HOST` / `TEMPORAL_NAMESPACE` env vars
- [ ] T013: Implement `src/workflows.rs` — skeleton `dummy_workflow` with one activity call, start-to-close timeout, error handling
- [ ] T014: Implement `src/activities.rs` — `dummy_activity` with heartbeat, 500ms simulated work, returns processed value
- [ ] T015: Implement `src/main.rs` — worker binary: connects client, registers workflow + activity, polls task queue, graceful shutdown on SIGINT
- [ ] T016: Write unit tests: `cargo test --lib` for temporal-worker module (client init, workflow registration, activity execution)
- [ ] T017: Add temporal-worker to process-compose.yml with TEMPORAL_HOST/TEMPORAL_TASK_QUEUE/TEMPORAL_NAMESPACE env vars
- [ ] T018: End-to-end test: start Temporal (WP01), run worker, trigger `dummy_workflow` via `tctl workflow run`, verify in Temporal Web UI at `:8233`

**Parallelization**: T010–T015 are independent within the module; T016–T018 are sequential verification.

**Estimated prompt size**: ~450 lines | **Subtask count**: 9 ⚠️ (high but cohesive — all SDK setup)

---

## Phase 2 — Core Workflow Migration

### WP03: Agent Dispatch Workflow Migration
**Phase**: 2 | **Wave**: 1 | **Priority**: P1 | **Dependencies**: WP02

Migrate the primary agent task dispatch workflow from NATS JetStream to Temporal. The most critical WP — agent dispatch must be durable, survive crashes, and be fully observable. After this, agent dispatch is fully Temporal-powered.

**Implementation command**: `spec-kitty implement WP03 --base WP02`

**Subtasks**:
- [ ] T019: Audit existing NATS agent dispatch implementation (grep for `nats`, `jetstream`, `NatsClient`, `subscribe`, `queue_subscribe`; document current flow, triggers, steps, failure handling)
- [ ] T020: Implement `crates/temporal-worker/src/workflows/agent_dispatch.rs` — full workflow: validate_task → dispatch_to_agent → collect_agent_result → notify_completion, with exponential backoff retry (max 3), heartbeat timeouts
- [ ] T021: Implement `activities/validate_task.rs` — validates task_id and prompt non-empty, returns `ValidationResult`
- [ ] T022: Implement `activities/dispatch_to_agent.rs` — dispatches to SGLang/vLLM (primary, env AGENT_RUNTIME_URL) with Groq Cloud API fallback; includes heartbeat
- [ ] T023: Implement `activities/collect_agent_result.rs` — polls agent runtime for completion status every 30s, handles completed/failed/pending states, returns `AgentOutput`
- [ ] T024: Implement `activities/notify_completion.rs` — POSTs to notification webhook (NOTIFICATION_WEBHOOK_URL env), non-fatal on failure
- [ ] T025: Add saga compensation: `saga_compensate()` emits failure signal on final workflow failure
- [ ] T026: Crash recovery test: start agent_dispatch workflow, kill Temporal (`docker kill temporal`), restart, verify workflow resumes and completes with full history
- [ ] T027: Query handler test: `get_task_status` returns current step + step history with durations via `tctl workflow query`

**Parallelization**: T019 is research (before coding); T020–T027 are sequential within one implementation session.

**Estimated prompt size**: ~550 lines | **Subtask count**: 9 ⚠️

---

### WP04: Hatchet Deployment + CI Pipeline Migration
**Phase**: 2 | **Wave**: 1 | **Priority**: P1 | **Dependencies**: WP01

Deploy Hatchet on Hetzner alongside Temporal. Migrate CI pipeline trigger workflows from NATS JetStream to Hatchet. Hatchet handles: GitHub webhook → CI pipeline with retry, concurrency limits, and dashboard for non-technical visibility.

**Implementation command**: `spec-kitty implement WP04 --base WP01`

**Subtasks**:
- [ ] T028: Write `infra/hatchet/docker-compose.yml` (Hatchet server + worker, postgres:16 backend, ports 8080 API / 8081 dashboard, mem_limit 1g, healthchecks)
- [ ] T029: Generate self-signed TLS certificates in `infra/hatchet/certs/` (server.crt/key, worker.crt/key) for internal mutual TLS
- [ ] T030: Write `infra/hatchet/.env` (HATCHET_DB_PASSWORD, HATCHET_PUBLIC_URL=https://hatchet.internal)
- [ ] T031: Wire Hatchet dashboard (port 8081) into Caddy as `hatchet.internal` with TLS and logging
- [ ] T032: Add Hatchet + worker entries to process-compose.yml (depends on Hatchet container health)
- [ ] T033: Create CI pipeline workflow in `infra/hatchet/workflows/ci-pipeline.yaml`: trigger via GitHub webhook, steps checkout→lint→test→build→notify, concurrency limit 5, exponential backoff retries, Slack notification on failure
- [ ] T034: End-to-end CI test: send test webhook to Hatchet endpoint, observe workflow in dashboard within 30s, verify all 5 steps execute, failed step retries, concurrency enforced

**Parallelization**: T028–T032 are sequential infra setup; T033–T034 are sequential testing.

**Estimated prompt size**: ~380 lines | **Subtask count**: 7 ✓

---

### WP05: Data Sync Workflow Migration to Hatchet
**Phase**: 2 | **Wave**: 1 | **Priority**: P1 | **Dependencies**: WP04

Audit all NATS JetStream data sync consumers and migrate them to Hatchet cron workflows. Each sync pipeline: scheduled automatically, retried on transient failure, alerted on permanent failure, visible in Hatchet dashboard.

**Implementation command**: `spec-kitty implement WP05 --base WP04`

**Subtasks**:
- [ ] T035: Audit existing NATS data sync consumers (grep for sync/etl/pipeline/consume; document source, destination, interval, failure behavior, data loss tolerance per pipeline)
- [ ] T036: Create generic data sync Hatchet workflow pattern: source→transform→load→verify→notify-success with exponential backoff retries, concurrency limit 1 per pipeline
- [ ] T037: Implement each discovered sync pipeline as a Hatchet workflow (Postgres→Postgres, API→DB, etc.) with cron schedule, retry policy, Slack alert on failure after all retries
- [ ] T038: Remove NATS JetStream consumer for data sync (mark NATS stream/consumer for removal in WP09)
- [ ] T039: Write integration tests: run each sync workflow against test data, verify source/destination record counts match, verify retry and alert behavior

**Parallelization**: T035 is research; T036–T038 are sequential per pipeline; T039 is sequential verification.

**Estimated prompt size**: ~350 lines | **Subtask count**: 5 ✓

---

## Phase 3 — Observability and Reliability

### WP06: Distributed Tracing Integration
**Phase**: 3 | **Wave**: 2 | **Priority**: P2 | **Dependencies**: WP03

Emit OpenTelemetry traces from Temporal workflows to Jaeger. Every workflow execution produces a trace. Full waterfall (workflow → activity → agent runtime → database) visible in Jaeger. Makes debugging production issues a trace query, not a log scavenger hunt.

**Implementation command**: `spec-kitty implement WP06 --base WP03`

**Subtasks**:
- [ ] T040: Deploy Jaeger all-in-one:1.59 in `infra/jaeger/docker-compose.yml` (OTLP gRPC 4317, HTTP 4318, UI 16686, badger storage, 7d retention, 10k trace limit), wire to Caddy as `jaeger.internal`
- [ ] T041: Add Jaeger to process-compose.yml (depends on Temporal)
- [ ] T042: Enable OpenTelemetry in Temporal worker: add `opentelemetry-sdk`, `opentelemetry-otlp`, `tracing-opentelemetry` deps; init OTLP exporter to `http://jaeger:4317`; configure service.name and deployment.environment attributes
- [ ] T043: Add custom spans to activities (`dispatch_to_agent`, `collect_agent_result`): span attributes include task_id, agent_type, attempt_number; use `FutureExt::with_context`
- [ ] T044: End-to-end trace test: run complete agent_dispatch workflow, query Jaeger by service="temporal-worker", verify waterfall: root workflow span → activity child spans → HTTP client spans

**Parallelization**: T040–T041 are independent; T042–T043 sequential; T044 sequential verification.

**Estimated prompt size**: ~320 lines | **Subtask count**: 5 ✓

---

### WP07: SLO Monitoring Dashboard
**Phase**: 3 | **Wave**: 2 | **Priority**: P2 | **Dependencies**: WP06

Create Grafana dashboards displaying workflow-level SLOs for all Temporal and Hatchet workflows. Define alert rules for SLO breach. Alert fires within 5 minutes of threshold violation.

**Implementation command**: `spec-kitty implement WP07 --base WP06`

**Subtasks**:
- [ ] T045: Update `infra/prometheus/prometheus.yml` to scrape Temporal metrics endpoint (`temporal:9233/metrics`) and Hatchet (`hatchet:8080`)
- [ ] T046: Create Temporal Workflow SLO dashboard in Grafana (provisioned YAML): completion rate stat (target 99.9%), p50/p95/p99 latency timeseries per workflow type, error rate by workflow type, in-progress count, activity retry rate
- [ ] T047: Create Hatchet Job Health dashboard (provisioned YAML): job success rate, step duration heatmap, concurrency utilization gauge, failed jobs table (last 24h)
- [ ] T048: Define and provision Grafana alerting rules: completion rate < 99.9% for 5min → critical alert; p99 latency > 5min for 5min → warning alert; wire alerts to Slack webhook via Grafana notification channel
- [ ] T049: Verify SLO dashboard shows live data; verify alert fires within 5 minutes when completion rate drops (inject test failure to trigger alert)

**Parallelization**: T045 is independent; T046–T047 sequential dashboard creation; T048–T049 sequential testing.

**Estimated prompt size**: ~400 lines | **Subtask count**: 5 ✓

---

### WP08: Rollback Capability
**Phase**: 3 | **Wave**: 2 | **Priority**: P0 | **Dependencies**: WP03, WP04, WP05

Ensure NATS can be restored as the primary workflow engine within 10 minutes of a critical Temporal/Hatchet failure. This is the safety net that makes the Big Bang migration acceptable.

**Implementation command**: `spec-kitty implement WP08 --base WP03`

**Subtasks**:
- [ ] T050: Implement `WorkflowEngine` enum (`Temporal`, `Hatchet`, `Nats`) with `WORKFLOW_ENGINE` env var; implement `dispatch_workflow()` router in `crates/temporal-worker/src/config.rs`
- [ ] T051: Implement `crates/temporal-worker/src/nats_fallback.rs` — NATS fallback dispatch preserving current NATS queue publish behavior (rollback path, kept from current implementation)
- [ ] T052: Create git branch `rollback/nats-workflow-logic` preserving current NATS workflow implementation (all queue consumers and dispatch logic); verify branch builds cleanly
- [ ] T053: Write `docs/ROLLBACK.md`: when to rollback, 5-step procedure with time budgets, rollback test script, post-rollback investigation steps
- [ ] T054: Execute rollback test: toggle `WORKFLOW_ENGINE=nats`, verify NATS accepts work, toggle back, measure total time; must be < 10 minutes

**Parallelization**: T050–T051 independent; T052 sequential; T053 sequential; T054 sequential test.

**Estimated prompt size**: ~350 lines | **Subtask count**: 5 ✓

---

## Phase 4 — Cutover

### WP09: NATS Workflow Logic Removal
**Phase**: 4 | **Wave**: 3 | **Priority**: P1 | **Dependencies**: WP07, WP08

Remove all JetStream workflow/queue consumers and durable stream configurations from NATS. Retain only pub/sub for non-critical events. This is the irreversible step — only done after WP01–WP08 complete and WP08 rollback confirmed.

**Implementation command**: `spec-kitty implement WP09 --base WP08`

**Subtasks**:
- [ ] T055: Audit current NATS JetStream configuration: `nats stream list`, `nats consumer list`; document each stream/consumer purpose and disposition; create `docs/NATS_Streams_Audit.md`
- [ ] T056: Remove JetStream workflow streams (`nats stream rm`) for agent dispatch, CI pipeline, data sync; verify `nats stream list` returns only event-bus streams
- [ ] T057: Update `infra/nats/nats-server.conf` to reduce JetStream store limits (256MB mem, 1GB file — event bus only, not workflow queuing); remove NATS worker services from process-compose.yml
- [ ] T058: Verify pure pub/sub behavior: publish test event to `events.>`, verify subscribers receive it; confirm no workflow action occurs; update `docs/NATS_ROLE.md`
- [ ] T059: Code audit: grep for `queue_subscribe`, `jetstream`, `NatsClient.*dispatch` across all crates; confirm zero workflow dispatch references remain

**Parallelization**: T055 is independent; T056–T059 are sequential verification.

**Estimated prompt size**: ~320 lines | **Subtask count**: 5 ✓

---

### WP10: 48-Hour Dual-Write Observation
**Phase**: 4 | **Wave**: 3 | **Priority**: P1 | **Dependencies**: WP09

Run Temporal + Hatchet in production for 48 hours monitoring all SLOs continuously. Declare success if zero critical failures in 48 hours. If critical failure occurs, execute rollback immediately.

**Implementation command**: `spec-kitty implement WP10 --base WP09`

**Subtasks**:
- [ ] T060: Create observation runbook (`docs/observation/RUNBOOK.md`): start time recording, Grafana dashboard link, on-call contacts, checkpoint schedule (T+4h, T+12h, T+24h, T+36h, T+48h), critical failure triggers, success procedure
- [ ] T061: Set `OBSERVATION_START_TIME` env var; implement and deploy `scripts/observation-checkpoint.sh` (queries Prometheus for completion rate, error rate, p99 latency; logs checkpoint; notifies at milestones; creates `v1.0.0-temporal` Git tag at T+48h)
- [ ] T062: Set up cron: `*/15 * * * *` observation checkpoint script, `0 * * * *` rollback health test
- [ ] T063: On success (48h zero critical failures): create Git tag `v1.0.0-temporal`, archive rollback branch (rename to `archived/rollback/nats-workflow-logic`), notify team, close Plane.so incident
- [ ] T064: On failure (critical SLO breach): execute rollback procedure (WP08), halt observation, open incident report, investigate root cause before re-migration

**Parallelization**: T060 independent; T061 sequential; T062–T064 sequential.

**Estimated prompt size**: ~350 lines | **Subtask count**: 5 ✓

---

## Phase 5 — Handoff

### WP11: Documentation and Runbooks
**Phase**: 5 | **Wave**: 4 | **Priority**: P2 | **Dependencies**: WP10

Create all operational documentation for Temporal + Hatchet infrastructure. Every operational procedure must have a runbook. Archive all legacy NATS JetStream workflow documentation.

**Implementation command**: `spec-kitty implement WP11 --base WP10`

**Subtasks**:
- [ ] T065: Write `docs/infra/README.md` — complete service index with all services, ports, dashboards, and quick links
- [ ] T066: Write `docs/infra/temporal/README.md` — architecture (Mermaid diagram), key concepts (workflow, activity, namespace, task queue), SLO table, environment variables, common operations (`tctl` commands)
- [ ] T067: Write `docs/infra/temporal/AUTHORING.md` — how to write new Temporal workflows: struct definitions, workflow function, retry policy, heartbeat patterns, local activities, testing
- [ ] T068: Write `docs/infra/temporal/TROUBLESHOOT.md` — common failure scenarios: stuck workflows, high latency, worker disconnection, Elasticsearch OOM, reset vs cancel decision tree
- [ ] T069: Write `docs/infra/hatchet/README.md` — architecture, use cases, key metrics, environment variables, health check commands
- [ ] T070: Write `docs/infra/hatchet/DEPLOY.md` — initial deploy, first-start checklist, TLS cert generation, upgrade path
- [ ] T071: Write `docs/infra/hatchet/AUTHORING.md` — writing new Hatchet workflows and cron triggers with worked example
- [ ] T072: Write `docs/infra/nats/README.md` — pure event bus role, retained subjects table, delivery guarantees, health check commands
- [ ] T073: Archive legacy NATS JetStream workflow documentation to `.archive/`

**Parallelization**: T065–T068 can run in parallel (Temporal docs); T069–T071 parallel (Hatchet docs); T072–T073 sequential.

**Estimated prompt size**: ~450 lines | **Subtask count**: 9 ⚠️

---

### WP12: AX101 Capacity Planning and Resource Audit
**Phase**: 5 | **Wave**: 4 | **Priority**: P3 | **Dependencies**: WP01

Audit actual AX101 resource usage with Temporal + Hatchet deployed. Confirm all services fit within headroom. Document scaling triggers and 6-month growth projection.

**Implementation command**: `spec-kitty implement WP12 --base WP01`

**Subtasks**:
- [ ] T074: Write and run `scripts/capacity-audit.sh` — measures CPU (nproc), RAM (`free -h`), disk (`df -h`, `iostat`), Docker container stats, Docker volume sizes; output to `docs/infra/CAPACITY.md`
- [ ] T075: Document `docs/infra/CAPACITY.md`: server specs, per-service resource allocation table (CPU, RAM, disk per container), total utilization vs capacity, bottleneck analysis (Elasticsearch is primary bottleneck)
- [ ] T076: Define scaling triggers with clear thresholds: Worker CPU > 80% for 5min, Elasticsearch CPU > 70%, PostgreSQL connections > 80%, disk usage > 70%, RAM > 85%; document remediation actions
- [ ] T077: Produce 6-month growth projection table (M0/M1/M3/M6: agent dispatches/day, CI runs/day, data syncs/day, estimated CPU/RAM); document M3 scaling plan (second Temporal worker node)
- [ ] T078: Document backup and restore procedure (Temporal history from PostgreSQL PITR, Hatchet from PostgreSQL, Jaeger traces acceptable loss); configure weekly Docker system prune cron job

**Parallelization**: T074 independent; T075–T077 sequential; T078 sequential.

**Estimated prompt size**: ~350 lines | **Subtask count**: 5 ✓

---

## Dependency Summary

| WP | Dependencies | Blocks |
|----|-------------|--------|
| WP01 | — | WP02, WP03, WP12 |
| WP02 | WP01 | WP03 |
| WP03 | WP02 | WP06, WP08 |
| WP04 | WP01 | WP05, WP08 |
| WP05 | WP04 | WP08 |
| WP06 | WP03 | WP07 |
| WP07 | WP06 | WP08 |
| WP08 | WP03, WP04, WP05 | WP09 |
| WP09 | WP07, WP08 | WP10 |
| WP10 | WP09 | WP11 |
| WP11 | WP10 | — |
| WP12 | WP01 | — |

**Critical path**: WP01 → WP02 → WP03 → WP06 → WP07 → WP08 → WP09 → WP10 → WP11

---

## Success Criteria Mapping

| SC | Criterion | WP |
|----|-----------|-----|
| SC-001 | Workflows survive Temporal restart (100% resume) | WP03 |
| SC-002 | Agent dispatch p99 latency < 5 min | WP07 |
| SC-003 | 99.9% completion rate | WP07 |
| SC-004 | Full trace retrievable in 60s | WP06 |
| SC-005 | Zero workflow logic in NATS | WP09 |
| SC-006 | Hatchet first step < 30s from webhook | WP04 |
| SC-007 | Rollback in < 10 minutes | WP08 |
| SC-008 | SLO alert fires within 5 min of breach | WP07 |
| SC-009 | 100% trace coverage to Jaeger | WP06 |
| SC-010 | 48h zero discrepancies | WP10 |

---

## MVP Recommendation

**WP01** is the MVP scope. Once Temporal is deployed and verified healthy, all subsequent WPs can be dispatched in parallel agents along the dependency graph (WP02, WP04, WP12 all depend only on WP01). The first parallel wave should be: WP02 (Rust SDK), WP04 (Hatchet deploy), WP12 (capacity baseline).
