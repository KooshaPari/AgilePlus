---
id: FR-TEMPLATE-001
title: Template Platform Completion
status: draft
priority: P2
created: 2026-04-02
category: platform
owner: phenotype-org
source: kitty-specs/feature-specification-template-platform-completion
---

# FR-TEMPLATE-001: Template Platform Completion

## Description

Complete all 12 template repositories from Foundation (v0.1.0) to Alpha (v0.2.0) with core functionality implemented. Each template provides minimal scaffolding; add production-ready templates.

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

## User Stories

### US-1: Complete Python Template (P0)
**Given** a Python developer uses the template,  
**When** they scaffold a new API project,  
**Then** they get FastAPI + pytest + pyright strict mode configured.

## Acceptance Criteria

- [ ] `phenotype-py-api` template generates complete FastAPI project
- [ ] pytest configuration with async support
- [ ] pyright strict mode configuration
- [ ] All smoke tests pass for each template
- [ ] 12 templates at v0.2.0 with core functionality

## Notes

Original: `kitty-specs/feature-specification-template-platform-completion/`
