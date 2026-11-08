# Feature Specification: Template Platform Completion

**Feature Branch**: 012-template-platform-completion
**Created**: 2026-04-02
**Status**: Draft
**Mission**: platform

## Overview

Complete all 12 template repositories from Foundation (v0.1.0) to Alpha (v0.2.0) with core functionality implemented. Each template currently provides minimal scaffolding; this spec defines the work to add production-ready templates.

## Templates in Scope

| Template | Current State | Target v0.2.0 |
|----------|---------------|---------------|
| template-lang-python | pyproject.toml only | FastAPI + pytest + uv |
| template-lang-rust | Cargo.toml only | Hexagonal workspace + Tokio |
| template-lang-go | go.mod only | chi router + hexagonal |
| template-lang-kotlin | build.gradle.kts only | Koin DI + Coroutines |
| template-lang-typescript | package.json only | Express/Hono + full types |
| template-lang-elixir-hex | mix.exs only | Phoenix + Ecto |
| template-lang-swift | Package.swift only | XcodeGen + MVVM |
| template-lang-zig | build.zig only | Comptime + hexagonal |
| template-lang-mojo | main.mojo only | MAX + ML patterns |
| template-domain-webapp | Compose only | React + auth |
| template-domain-service-api | Compose only | REST API + domain |
| template-program-ops | shell only | Typer + logging |

## User Scenarios

### User Story 1 — Complete Python Template (Priority: P0)

A Python developer uses the template to scaffold a new API project with FastAPI, pytest, and proper tooling.

**Acceptance Criteria**:
1. `phenotype-py-api` template generates complete FastAPI project
2. pytest configuration with async support
3. pyright strict mode configuration
4. All smoke tests pass

### User Story 2 — Complete Rust Template (Priority: P0)

A Rust developer uses the template to scaffold a new microservice with hexagonal architecture.

**Acceptance Criteria**:
1. Hexagonal workspace layout with domain/application/adapters/ports
2. Tokio runtime configuration
3. thiserror + anyhow templates
4. Full clippy.toml with pedantic lints

### User Story 3 — Complete Go Template (Priority: P0)

A Go developer uses the template to scaffold a new HTTP service.

**Acceptance Criteria**:
1. HTTP service template with chi router
2. Middleware patterns (logging, auth)
3. Repository interface patterns
4. golangci-lint integration

### User Story 4 — Complete Kotlin Template (Priority: P1)

A Kotlin developer uses the template to scaffold a new domain service.

**Acceptance Criteria**:
1. Hexagonal architecture template
2. Koin dependency injection
3. Coroutines support
4. JUnit 5 + Kotest setup

## Functional Requirements

### FR-PYTHON-001: FastAPI Template
- Generate FastAPI application with Pydantic models
- Include health check endpoints
- Async request handlers
- OpenAPI documentation

### FR-PYTHON-002: Testing Infrastructure
- pytest configuration
- pytest-asyncio for async
- pytest-cov for coverage
- Test fixtures

### FR-PYTHON-003: Type System
- pyright strict mode configuration
- All public APIs typed
- Type stubs for external libraries

### FR-RUST-001: Hexagonal Workspace
- Cargo workspace with domain/application/adapters/ports crates
- Shared workspace dependencies
- Inter-crate dependency rules

### FR-RUST-002: Async Runtime
- Tokio configuration
- async-trait for port traits
- `#[tokio::main]` entrypoint

### FR-RUST-003: Error Handling
- thiserror for domain errors
- anyhow for application errors
- No-unwrap policy

### FR-GO-001: HTTP Service
- chi router integration
- Middleware patterns
- Request validation
- Error handling

### FR-GO-002: Architecture
- Repository interfaces
- Service layer patterns
- Dependency injection

### FR-KOTLIN-001: Hexagonal Architecture
- Domain/application/adapters/ports structure
- Repository interfaces
- Service interfaces

### FR-KOTLIN-002: DI & Async
- Koin module configuration
- Coroutines integration
- Flow for streaming

## Success Criteria

- All 12 templates compile/generate successfully
- All templates pass their smoke tests
- FR/PRDs accurately reflect implementation
- CI/CD passes on all templates

## Out of Scope

- Phase 2 features (v0.3.0+)
- Domain template full implementation
- Deployment patterns
