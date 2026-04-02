# tests Specification

Canonical definition of the system behavior.

## Overview

This repo contains test infrastructure for the Phenotype ecosystem, including contract tests (Pact), BDD feature tests, and integration tests.

## Components

### Contract Tests (`contract/`)
Pact contract test fixtures for the gRPC boundary between the Python MCP consumer (`AgilePlusMCP`) and the Rust core provider (`AgilePlusCore`).

**Interactions tested:**
- `GetFeature` — retrieve feature by ID
- `DispatchCommand` — dispatch planning/implementation commands
- `GetAuditTrail` — stream audit entries
- `VerifyAuditChain` — verify audit chain integrity
- `CheckGovernanceGate` — validate governance gates

### BDD Tests (`bdd/`)
Gherkin-style BDD features for high-level acceptance testing.

**Features:**
- `specify.feature` — Feature specification behavior
- `implement.feature` — Implementation workflow
- `governance.feature` — Governance validation
- `audit.feature` — Audit trail functionality

### Integration Tests (`integration/`)
End-to-end integration tests including Docker-based test environment.

### Fixtures (`fixtures/`)
Sample data for test scenarios:
- `sample-spec.md` — Sample feature specification
- `sample-plan.md` — Sample implementation plan
- `sample-governance.json` — Sample governance evidence
- `sample-evidence/` — Evidence files for WP01, WP02

### Contracts (`contracts/`)
Solidity contract tests for on-chain components:
- `sync_plane_contract.rs`
- `events_sqlite_contract.rs`
- `dashboard_api_contract.rs`
- `api_events_contract.rs`

## Running Tests

```bash
# Contract tests
cd contract && uv run pytest tests/contract/ -v

# BDD tests
cargo test

# Integration tests
cd integration && docker-compose -f docker-compose.test.yml up
```

## Dependencies

- Rust (for BDD and contract tests)
- Python + pytest (for contract tests)
- Docker (for integration tests)