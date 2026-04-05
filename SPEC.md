# AgilePlus Specification

**Version**: 2.0  
**Status**: Draft  
**Date**: 2026-04-02  

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [State of the Art Comparison](#state-of-the-art-comparison)
3. [System Architecture](#system-architecture)
4. [Data Model](#data-model)
5. [CLI Command Reference](#cli-command-reference)
6. [Git Integration](#git-integration)
7. [Sync Strategies](#sync-strategies)
8. [Configuration](#configuration)
9. [Performance Targets](#performance-targets)
10. [Security Model](#security-model)
11. [Deployment Patterns](#deployment-patterns)
12. [Monitoring & Observability](#monitoring--observability)
13. [References](#references)

---

## Executive Summary

### Project Overview

AgilePlus is a **local-first, spec-driven development engine** that harmonizes the best practices from modern agile tools into a streamlined CLI-centric workflow. It operates as a sidecar alongside AI coding agents (Claude Code, Codex), orchestrating the entire feature lifecycle from specification through implementation to validation.

### Core Principles

1. **Local-First**: All operational state lives in SQLite on the developer's machine. No cloud dependency for core functionality.
2. **Git-Native**: All artifacts (specs, plans, evidence) are stored in git, making them versioned and reviewable.
3. **Spec-Driven**: Every feature begins with a structured specification that drives work package generation and validation.
4. **Agent-Orchestrated**: AI agents are dispatched to worktrees for implementation, not replaced by custom engines.
5. **Governance-Backed**: Hash-chained audit logs and policy-driven quality gates ensure compliance.

### The "7 Command" Workflow

AgilePlus reduces the idea-to-shipment gap to exactly 7 commands:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  specify    │ -> │  research   │ -> │    plan     │
│  (spec)     │    │  (analyze)  │    │  (decompose)│
└─────────────┘    └─────────────┘    └─────────────┘
       │                                    │
       └────────────────────────────────────┘
                       │
                       ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    ship     │ <- │  validate   │ <- │  implement  │
│  (deliver)  │    │  (verify)   │    │  (execute)  │
└─────────────┘    └─────────────┘    └─────────────┘
       │
       ▼
┌─────────────┐
│    retro    │
│  (learn)    │
└─────────────┘
```

### Key Differentiators

| Aspect | Traditional Tools | AgilePlus |
|--------|------------------|-----------|
| Location | Cloud/SaaS required | Local-first, offline-capable |
| Specs | Unstructured or rigid templates | YAML-structured, machine-readable |
| Agent Integration | Manual coordination | First-class agent orchestration |
| Validation | Manual checklists | Automated policy gates |
| Audit | Ad-hoc or manual | SHA-256 hash-chained |
| Git Correlation | Commit message discipline | Automatic with worktree isolation |

### Success Metrics

- **Time to Plan**: Idea to work packages in < 10 minutes of active input
- **Validation Coverage**: 100% of functional requirements have traceable evidence
- **Governance Compliance**: Zero violations reach production branches
- **Audit Integrity**: Every state transition is tamper-verifiable

---

## State of the Art Comparison

### Market Landscape

The project management tool space is crowded but has clear gaps that AgilePlus fills:

```
                        ┌─────────────────────────────────────┐
                        │        COMPETITIVE LANDSCAPE         │
                        └─────────────────────────────────────┘

   Enterprise Depth
          │
          │  ┌─────────┐
          │  │  Jira   │ ◄── Heavy, complex, cloud-only
          │  │         │     Poor agent integration
          │  └─────────┘
          │
          │       ┌──────────┐
          │       │   bmad   │ ◄── Deep governance, but
          │       │          │     requires 8+ commands,
          │       └──────────┘     steep learning curve
          │
          │  ┌─────────┐
          │  │AgilePlus│ ◄── Sweet spot: deep governance
          │  │   ★     │     with streamlined UX
          │  └─────────┘
          │
          │    ┌────────┐
          │    │ Linear │ ◄── Beautiful, opinionated
          │    │        │     limited customization
          │    └────────┘
          │
          │      ┌──────────┐
          │      │ Plane.so │ ◄── Open source, flexible
          │      │          │     but manual workflows
          │      └──────────┘
          │
          │    ┌─────────────┐
          └────┤  spec-kitty  │ ◄── Structured, but
               │  OpenSpec   │     fragmented tools
               └─────────────┘

                     Lightweight ────────────► Complex
```

### Detailed Comparison

#### Jira (Atlassian)

| Aspect | Evaluation |
|--------|-----------|
| **Strengths** | Enterprise features, extensive plugins, mature ecosystem |
| **Weaknesses** | Slow, complex UI, cloud-only for modern features, poor agent integration |
| **Spec Support** | Limited - no structured spec format |
| **Agent Integration** | REST API only, no first-class agent support |
| **Local-First** | No - cloud required |
| **Governance** | Workflows exist but no hash-chained audit |
| **Pricing** | $7.75-15.25/user/month |

**Verdict**: Too heavy for developer-centric workflows. Enterprise sales-driven, not developer experience-driven.

#### Linear

| Aspect | Evaluation |
|--------|-----------|
| **Strengths** | Beautiful UI, fast, keyboard-driven, GitHub integration |
| **Weaknesses** | Opinionated (follows Linear's workflow), cloud-only, limited customization |
| **Spec Support** | Issue templates, but not structured spec format |
| **Agent Integration** | API exists but not agent-centric |
| **Local-First** | No |
| **Governance** | Cycles and roadmaps, but no policy gates |
| **Pricing** | $8/user/month (free for small teams) |

**Verdict**: Excellent for product teams, not designed for AI agent orchestration or deep technical governance.

#### Plane.so

| Aspect | Evaluation |
|--------|-----------|
| **Strengths** | Open source, self-hostable, flexible, good GitHub integration |
| **Weaknesses** | Still requires infrastructure, manual workflow setup |
| **Spec Support** | Pages feature for docs, but not structured specs |
| **Agent Integration** | API exists, but not designed for agent dispatch |
| **Local-First** | Can be self-hosted, but not local-first by design |
| **Governance** | Basic, no policy engine |
| **Pricing** | Free self-hosted, $20/month hosted |

**Verdict**: Good foundation, but AgilePlus treats Plane.so as a sync target, not the primary system.

#### spec-kitty / OpenSpec

| Aspect | Evaluation |
|--------|-----------|
| **Strengths** | Structured specs, worktree isolation, Kanban tracking |
| **Weaknesses** | Fragmented tools, manual coordination between steps |
| **Spec Support** | Good - markdown-based specs |
| **Agent Integration** | Agent-friendly, but not orchestrated |
| **Local-First** | Yes |
| **Governance** | Limited |
| **Pricing** | Free (open source) |

**Verdict**: AgilePlus builds on spec-kitty patterns but adds orchestration and governance.

#### bmad

| Aspect | Evaluation |
|--------|-----------|
| **Strengths** | Deep governance, role-based agents, enterprise depth |
| **Weaknesses** | Requires 8+ commands for full workflow, steep learning curve |
| **Spec Support** | Comprehensive |
| **Agent Integration** | Built-in agent framework |
| **Local-First** | Yes |
| **Governance** | Excellent - smart contracts, evidence chains |
| **Pricing** | Unknown |

**Verdict**: Excellent governance model, but too complex for daily use. AgilePlus streamlines bmad concepts into 7 commands.

### Feature Matrix

| Feature | Jira | Linear | Plane.so | spec-kitty | bmad | AgilePlus |
|---------|------|--------|----------|------------|------|-----------|
| Local-first | No | No | Partial | Yes | Yes | Yes |
| Structured specs | No | No | Partial | Yes | Yes | Yes |
| Worktree isolation | No | No | No | Yes | Yes | Yes |
| Agent orchestration | No | No | No | Partial | Yes | Yes |
| Hash-chained audit | No | No | No | No | Yes | Yes |
| Policy gates | Partial | No | No | No | Yes | Yes |
| Git correlation | Manual | Partial | Partial | Yes | Yes | Yes |
| Open source | No | No | Yes | Yes | Unknown | Yes |
| CLI-first | No | No | No | Yes | Yes | Yes |
| 7-command workflow | No | No | No | No | No | Yes |

### AgilePlus Positioning

```
┌─────────────────────────────────────────────────────────────────────┐
│                         AgilePlus Positioning                       │
│                                                                       │
│  For: Solo developers and small teams who:                            │
│  • Want structured specs without enterprise complexity                │
│  • Use AI coding agents (Claude Code, Codex)                          │
│  • Need governance without ceremony                                   │
│  • Prefer local-first with optional cloud sync                        │
│                                                                       │
│  Not for:                                                             │
│  • Large enterprises needing complex permissions                      │
│  • Teams wanting fully-managed SaaS                                   │
│  • Organizations requiring SOC2/ISO27001 out of box                   │
│                                                                       │
│  Competitive moat:                                                    │
│  • Spec-driven development with agent orchestration                   │
│  • Hash-chained governance without blockchain complexity              │
│  • Local-first with P2P sync (no central server)                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            System Architecture                                │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                           Interface Layer                                │ │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌─────────────────────┐  │ │
│  │  │   CLI     │  │    API    │  │   gRPC    │  │    MCP Server       │  │ │
│  │  │ (Primary) │  │  (REST)   │  │ (Internal)│  │  (Agent Interface)  │  │ │
│  │  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └──────────┬──────────┘  │ │
│  └────────┼───────────────┼──────────────┼───────────────────┼─────────────┘ │
│           │               │              │                   │               │
│  ┌────────┴───────────────┴──────────────┴───────────────────┴─────────────┐ │
│  │                        Application Layer                                  │ │
│  │  ┌─────────────────────────────────────────────────────────────────┐    │ │
│  │  │                    Command Handlers (7 Commands)                   │    │ │
│  │  │  specify │ research │ plan │ implement │ validate │ ship │ retro  │    │ │
│  │  └─────────────────────────────────────────────────────────────────┘    │ │
│  │                                    │                                       │ │
│  │  ┌─────────────────────────────────┴─────────────────────────────────┐    │ │
│  │  │                    Application Services                            │    │ │
│  │  │  FeatureService │ BacklogService │ CycleService │ GovernanceService│    │ │
│  │  └─────────────────────────────────────────────────────────────────┘    │ │
│  └────────────────────────────────────┬────────────────────────────────────┘ │
│                                       │                                       │
│  ┌────────────────────────────────────┴────────────────────────────────────┐ │
│  │                           Domain Layer                                    │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │ │
│  │  │   Feature    │  │ WorkPackage  │  │    Audit     │  │   Policy     │  │ │
│  │  │   Entity     │  │   Entity     │  │    Entry     │  │    Rule      │  │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  │ │
│  │                                                                           │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐   │ │
│  │  │                     Port Traits (Hexagonal)                       │   │ │
│  │  │  StoragePort │ EventPort │ VcsPort │ AgentPort │ ObservabilityPort│   │ │
│  │  └──────────────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────┬────────────────────────────────────┘ │
│                                       │                                       │
│  ┌────────────────────────────────────┴────────────────────────────────────┐ │
│  │                        Infrastructure Layer                                 │ │
│  │                                                                           │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │ │
│  │  │   SQLite     │  │    NATS      │  │     Git      │  │    Neo4j     │  │ │
│  │  │   (State)    │  │   (Events)   │  │   (Artifacts)│  │   (Graph)    │  │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  │ │
│  │                                                                           │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │ │
│  │  │  Dragonfly   │  │    MinIO     │  │   Plane.so   │  │   GitHub     │  │ │
│  │  │   (Cache)    │  │  (Archive)   │  │    (Sync)    │  │    (Sync)    │  │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘  │ │
│  │                                                                           │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐   │ │
│  │  │                     P2P Replication Layer                           │   │ │
│  │  │           mDNS Discovery │ Vector Clocks │ State Merge            │   │ │
│  │  └──────────────────────────────────────────────────────────────────┘   │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### Domain Layer (`agileplus-domain`)

The core business logic with zero external dependencies:

```rust
// crates/agileplus-domain/src/lib.rs
pub mod entities {
    pub mod feature;
    pub mod work_package;
    pub mod audit;
    pub mod policy;
}

pub mod ports {
    pub mod storage;
    pub mod event;
    pub mod vcs;
    pub mod agent;
    pub mod observability;
}

pub mod services {
    pub mod feature_service;
    pub mod backlog_service;
    pub mod governance_service;
}
```

#### Application Layer

Orchestrates domain logic through ports:

```rust
// crates/agileplus-domain/src/services/feature_service.rs
pub struct FeatureService<S: StoragePort, E: EventPort, V: VcsPort> {
    storage: Arc<S>,
    events: Arc<E>,
    vcs: Arc<V>,
}

impl<S, E, V> FeatureService<S, E, V>
where
    S: StoragePort,
    E: EventPort,
    V: VcsPort,
{
    pub async fn specify(&self, input: SpecificationInput) -> Result<Feature> {
        // 1. Create feature entity
        let feature = Feature::new(input)?;
        
        // 2. Persist to storage
        self.storage.save_feature(&feature).await?;
        
        // 3. Write spec to git
        self.vcs.write_spec(&feature).await?;
        
        // 4. Emit event
        self.events.publish(FeatureSpecified {
            id: feature.id.clone(),
            timestamp: Utc::now(),
        }).await?;
        
        Ok(feature)
    }
}
```

#### Infrastructure Adapters

Implement port traits for specific technologies:

```
crates/
├── agileplus-sqlite/          # StoragePort implementation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── adapter.rs         # SqliteStorage implementation
│   │   ├── schema.rs          # DDL
│   │   └── migrations/
│   └── Cargo.toml
│
├── agileplus-git/             # VcsPort implementation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── adapter.rs         # GitVcs implementation
│   │   ├── correlation.rs     # Commit correlation
│   │   └── worktree.rs        # Worktree management
│   └── Cargo.toml
│
├── agileplus-nats/            # EventPort implementation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── adapter.rs         # NatsEventBus implementation
│   │   └── subjects.rs        # Subject naming
│   └── Cargo.toml
│
├── agileplus-neo4j/           # Graph relationships
│   ├── src/
│   │   ├── lib.rs
│   │   ├── adapter.rs         # Neo4jGraph implementation
│   │   └── queries.rs         # Cypher queries
│   └── Cargo.toml
│
└── agileplus-p2p/             # P2P replication
    ├── src/
    │   ├── lib.rs
    │   ├── discovery.rs        # mDNS discovery
    │   ├── vector_clock.rs     # Vector clock logic
    │   └── merge.rs            # State merging
    └── Cargo.toml
```

### Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Typical Data Flow                             │
│                                                                       │
│  1. SPECIFY                                                           │
│  ┌──────┐    ┌──────────────┐    ┌────────────┐    ┌──────────────┐  │
│  │ User │ -> │   specify    │ -> │  Domain    │ -> │   SQLite     │  │
│  │      │    │   command    │    │  Service   │    │  (CREATE)    │  │
│  └──────┘    └──────────────┘    └────────────┘    └──────────────┘  │
│                                                            │          │
│                                                            ▼          │
│                                                   ┌──────────────┐   │
│                                                   │     Git      │   │
│                                                   │  (spec.md)   │   │
│                                                   └──────────────┘   │
│                                                                       │
│  2. IMPLEMENT                                                         │
│  ┌──────────┐    ┌──────────────┐    ┌────────────┐    ┌──────────┐  │
│  │implement │ -> │   Domain     │ -> │   Agent    │ -> │Worktree  │  │
│  │ command  │    │   Service    │    │   Port     │    │created   │  │
│  └──────────┘    └──────────────┘    └────────────┘    └──────────┘  │
│                                                           │           │
│                                                           ▼           │
│                                                ┌─────────────────┐   │
│                                                │  Claude Code /  │   │
│                                                │     Codex       │   │
│                                                │  (in worktree)  │   │
│                                                └─────────────────┘   │
│                                                           │           │
│                                                           ▼           │
│                                                ┌─────────────────┐   │
│                                                │  GitHub PR      │   │
│                                                │  (with context) │   │
│                                                └─────────────────┘   │
│                                                                       │
│  3. VALIDATE                                                          │
│  ┌──────────┐    ┌──────────────┐    ┌────────────┐    ┌──────────┐  │
│  │ validate │ -> │  Governance  │ -> │   Policy   │ -> │  Report  │  │
│  │ command  │    │   Service    │    │   Rules    │    │ generated│  │
│  └──────────┘    └──────────────┘    └────────────┘    └──────────┘  │
│                                              │                        │
│                                              ▼                        │
│                                   ┌──────────────────┐               │
│                                   │   Evidence Check   │               │
│                                   │ (FR-001 -> tests) │               │
│                                   └──────────────────┘               │
└─────────────────────────────────────────────────────────────────────┘
```

### Multi-Repo Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Multi-Repository Architecture                    │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                    agileplus-proto                              │ │
│  │  • Protocol Buffer definitions (.proto)                         │ │
│  │  • Single source of truth for contracts                         │ │
│  │  • Generates Rust (tonic) and Python (grpcio) stubs            │ │
│  │  • Versioned independently (semver)                             │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                              ▲                                        │
│           ┌──────────────────┼──────────────────┐                    │
│           │                  │                  │                     │
│           ▼                  ▼                  ▼                     │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐       │
│  │  agileplus-core │ │ agileplus-mcp    │ │agileplus-agents │       │
│  │                 │ │                  │ │                 │       │
│  │ • Domain logic  │ │ • MCP server     │ │ • Agent dispatch│       │
│  │ • CLI           │ │ • FastMCP 3.0    │ │ • Review loop   │       │
│  │ • API server    │ │ • Python         │ │ • Worktrees     │       │
│  │ • SQLite        │ │                  │ │                 │       │
│  │ • gRPC server   │ │                  │ │                 │       │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘       │
│           │                  │                  │                     │
│           │         ┌────────┴──────────────────┘                    │
│           │         │                                               │
│           │         ▼                                               │
│           │  ┌─────────────────┐                                  │
│           │  │agileplus-integrations│                             │
│           │  │                     │                              │
│           │  │ • Plane.so sync     │                              │
│           │  │ • GitHub sync       │                              │
│           │  │ • Triage queue      │                              │
│           └──┤ • gRPC client       │                              │
│              └─────────────────┘                                  │
└─────────────────────────────────────────────────────────────────────┘

All repos consume agileplus-proto as a git submodule or cargo dependency.
Cross-repo communication exclusively via gRPC.
```

---

## Data Model

### Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Entity Relationships                          │
│                                                                       │
│  ┌─────────────┐         ┌─────────────┐         ┌─────────────┐      │
│  │   Project   │◄────────┤   Feature   │◄────────┤ WorkPackage │      │
│  │             │  1:M    │             │  1:M    │             │      │
│  │ - id        │         │ - id        │         │ - id        │      │
│  │ - name      │         │ - slug      │         │ - feature_id│      │
│  │ - path      │         │ - title     │         │ - title     │      │
│  └─────────────┘         │ - status    │         │ - status    │      │
│                          │ - priority  │         │ - ordinal   │      │
│                          │ - spec_hash │         │ - assignee  │      │
│                          └──────┬──────┘         └──────┬──────┘      │
│                                 │                        │            │
│                                 │ 1:M                    │ 1:M         │
│                                 ▼                        ▼            │
│                          ┌─────────────┐          ┌─────────────┐      │
│                          │  AuditEntry │          │  Subtask    │      │
│                          │             │          │             │      │
│                          │ - id        │          │ - id        │      │
│                          │ - feature_id│          │ - wp_id     │      │
│                          │ - actor     │          │ - title     │      │
│                          │ - transition│          │ - status    │      │
│                          │ - hash      │          └─────────────┘      │
│                          │ - prev_hash │                               │
│                          └─────────────┘                               │
│                                 ▲                                      │
│                                 │ 1:1                                  │
│                          ┌─────────────┐                               │
│                          │    Event    │                               │
│                          │             │                               │
│                          │ - id        │                               │
│                          │ - type      │                               │
│                          │ - payload   │                               │
│                          │ - timestamp │                               │
│                          └─────────────┘                               │
│                                                                       │
│  ┌─────────────┐         ┌─────────────┐         ┌─────────────┐      │
│  │GitCorrelation│◄───────┤   Commit    │◄───────┤    Repo     │      │
│  │             │  M:1    │             │  M:1    │             │      │
│  │ - id        │         │ - hash      │         │ - id        │      │
│  │ - commit_id │         │ - message   │         │ - remote_url│      │
│  │ - feature_id│         │ - author    │         │ - local_path│      │
│  │ - wp_id     │         │ - timestamp │         └─────────────┘      │
│  └─────────────┘         └─────────────┘                               │
│                                                                       │
│  ┌─────────────┐         ┌─────────────┐                             │
│  │  SyncMapping│◄───────┤ ExternalItem│                             │
│  │             │  1:1    │             │                             │
│  │ - id        │         │ - id        │                             │
│  │ - entity_type│        │ - source    │                             │
│  │ - entity_id │         │ - external_id│                            │
│  │ - external_id│        │ - url       │                             │
│  │ - content_hash│       └─────────────┘                             │
│  └─────────────┘                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Core Entities

#### Feature

A feature is the top-level unit of work, from idea to shipment.

```rust
pub struct Feature {
    /// Internal database ID
    pub id: FeatureId,
    
    /// URL-friendly identifier (e.g., "user-authentication")
    pub slug: String,
    
    /// Human-readable title
    pub title: String,
    
    /// Current state in lifecycle
    pub status: FeatureStatus,
    
    /// Business priority
    pub priority: Priority,
    
    /// SHA-256 of spec content (for integrity)
    pub spec_hash: [u8; 32],
    
    /// Git branch for implementation
    pub target_branch: String,
    
    /// Parent feature (for hierarchical specs)
    pub parent_id: Option<FeatureId>,
    
    /// Associated work packages
    pub work_packages: Vec<WorkPackageId>,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum FeatureStatus {
    Draft,           // Initial creation
    Specified,       // spec.yaml complete
    Researched,      // research.md complete
    Planned,         // plan.md + WPs created
    Implementing,    // Agents working
    Validated,       // Quality gates passed
    Shipped,         // Merged to main
    Archived,        // Historical record
}
```

#### WorkPackage

A work package is a decomposed unit of implementation within a feature.

```rust
pub struct WorkPackage {
    /// Internal database ID
    pub id: WorkPackageId,
    
    /// Parent feature
    pub feature_id: FeatureId,
    
    /// Display title
    pub title: String,
    
    /// Ordering within feature (WP-001, WP-002, etc.)
    pub ordinal: u32,
    
    /// Detailed description
    pub description: String,
    
    /// Current state
    pub status: WpStatus,
    
    /// Who is working on this (agent or human)
    pub assignee: Option<Assignee>,
    
    /// Linked functional requirements
    pub requirements: Vec<RequirementId>,
    
    /// Blocking dependencies
    pub blocked_by: Vec<WorkPackageId>,
    
    /// Work estimates
    pub estimated_hours: Option<u32>,
    pub actual_hours: Option<u32>,
    
    /// Git references
    pub worktree_path: Option<PathBuf>,
    pub branch_name: Option<String>,
    pub pr_url: Option<String>,
    
    /// Evidence for validation
    pub evidence: Vec<EvidenceRef>,
    
    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum WpStatus {
    Planned,     // Created but not started
    Doing,       // Active development
    ForReview,   // PR open, awaiting review
    Done,        // Merged, completed
    Blocked,     // Dependencies blocking
}

pub enum Assignee {
    Human(String),           // Username
    Agent(AgentType),        // Claude, Codex, etc.
    Unassigned,
}
```

#### AuditEntry

Immutable, hash-chained record of all state transitions.

```rust
pub struct AuditEntry {
    /// Internal ID
    pub id: AuditId,
    
    /// What feature this relates to
    pub feature_id: FeatureId,
    
    /// Optional: what WP this relates to
    pub work_package_id: Option<WorkPackageId>,
    
    /// Who made the change
    pub actor: Actor,
    
    /// What happened
    pub transition: String,
    
    /// Evidence references
    pub evidence_refs: Vec<EvidenceRef>,
    
    /// Chain integrity
    pub prev_hash: [u8; 32],
    pub hash: [u8; 32],
    
    /// Event correlation
    pub event_id: Option<EventId>,
    
    /// If archived to MinIO
    pub archived_to: Option<String>,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl AuditEntry {
    /// Compute hash for chain integrity
    pub fn compute_hash(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&self.prev_hash);
        hasher.update(self.feature_id.as_bytes());
        hasher.update(&self.timestamp.to_rfc3339());
        hasher.update(&self.transition);
        
        for evidence in &self.evidence_refs {
            hasher.update(evidence.as_bytes());
        }
        
        hasher.finalize().into()
    }
}
```

#### Event

Domain events for event sourcing and pub/sub.

```rust
pub struct Event {
    pub id: EventId,
    pub entity_type: EntityType,
    pub entity_id: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub prev_hash: [u8; 32],
    pub hash: [u8; 32],
    pub sequence: i64,  // Monotonic per entity
}

pub enum EntityType {
    Feature,
    WorkPackage,
    Governance,
    Sync,
}

// Example events
pub enum DomainEvent {
    FeatureSpecified { id: FeatureId, title: String },
    WorkPackageStarted { id: WorkPackageId, assignee: Assignee },
    EvidenceSubmitted { id: EvidenceId, wp_id: WorkPackageId },
    StateTransitioned { from: String, to: String, reason: String },
}
```

### SQLite Schema

```sql
-- Core feature table
CREATE TABLE features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK (status IN (
        'draft', 'specified', 'researched', 'planned',
        'implementing', 'validated', 'shipped', 'archived'
    )),
    priority TEXT CHECK (priority IN ('p0', 'p1', 'p2', 'p3')),
    spec_hash BLOB NOT NULL,  -- 32 bytes SHA-256
    target_branch TEXT DEFAULT 'main',
    parent_id INTEGER REFERENCES features(id),
    plane_issue_id TEXT,
    labels TEXT,  -- JSON array
    created_at TEXT NOT NULL,  -- ISO 8601
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    
    UNIQUE(slug)
);

CREATE INDEX idx_features_status ON features(status);
CREATE INDEX idx_features_parent ON features(parent_id);

-- Work packages
CREATE TABLE work_packages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    ordinal INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('planned', 'doing', 'for_review', 'done', 'blocked')),
    assignee_type TEXT CHECK (assignee_type IN ('human', 'agent', 'unassigned')),
    assignee_value TEXT,  -- username or agent type
    estimated_hours INTEGER,
    actual_hours INTEGER,
    worktree_path TEXT,
    branch_name TEXT,
    pr_url TEXT,
    evidence TEXT,  -- JSON array of EvidenceRef
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    
    UNIQUE(feature_id, ordinal)
);

CREATE INDEX idx_work_packages_feature ON work_packages(feature_id);
CREATE INDEX idx_work_packages_status ON work_packages(status);

-- WP dependencies (for DAG tracking)
CREATE TABLE wp_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wp_id INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    depends_on_wp_id INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    dependency_type TEXT DEFAULT 'blocks',  -- blocks, requires
    
    UNIQUE(wp_id, depends_on_wp_id)
);

-- Audit trail with hash chain
CREATE TABLE audit_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL REFERENCES features(id),
    work_package_id INTEGER REFERENCES work_packages(id),
    actor_type TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    transition TEXT NOT NULL,
    evidence_refs TEXT NOT NULL,  -- JSON array
    prev_hash BLOB NOT NULL,  -- 32 bytes
    hash BLOB NOT NULL,       -- 32 bytes
    event_id INTEGER,
    archived_to TEXT,
    timestamp TEXT NOT NULL,
    
    FOREIGN KEY (event_id) REFERENCES events(id)
);

CREATE INDEX idx_audit_feature ON audit_entries(feature_id);
CREATE INDEX idx_audit_timestamp ON audit_entries(timestamp);

-- Event store (append-only)
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,  -- JSON
    actor TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    prev_hash BLOB NOT NULL,
    hash BLOB NOT NULL,
    sequence INTEGER NOT NULL,
    
    UNIQUE(entity_type, entity_id, sequence)
);

CREATE INDEX idx_events_entity ON events(entity_type, entity_id, sequence);
CREATE INDEX idx_events_type ON events(event_type);

-- Git correlations
CREATE TABLE git_correlations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER REFERENCES features(id),
    work_package_id INTEGER REFERENCES work_packages(id),
    commit_hash TEXT NOT NULL,
    commit_message TEXT,
    commit_author TEXT,
    commit_date TEXT,
    files_changed TEXT,  -- JSON array
    lines_added INTEGER,
    lines_deleted INTEGER,
    correlation_method TEXT,  -- message_pattern, git_notes, manual
    created_at TEXT NOT NULL
);

CREATE INDEX idx_git_correlations_commit ON git_correlations(commit_hash);
CREATE INDEX idx_git_correlations_feature ON git_correlations(feature_id);

-- Sync mappings for external systems
CREATE TABLE sync_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('feature', 'work_package')),
    entity_id INTEGER NOT NULL,
    external_system TEXT NOT NULL,  -- plane, github, etc.
    external_id TEXT NOT NULL,
    external_url TEXT,
    content_hash TEXT NOT NULL,
    last_synced_at TEXT NOT NULL,
    sync_direction TEXT DEFAULT 'bidirectional',
    conflict_count INTEGER DEFAULT 0,
    
    UNIQUE(entity_type, entity_id, external_system)
);

CREATE INDEX idx_sync_mappings_external ON sync_mappings(external_system, external_id);

-- Full-text search for specs
CREATE VIRTUAL TABLE spec_search USING fts5(
    title,
    content,
    content_rowid=rowid
);

-- Triggers for updated_at
CREATE TRIGGER features_updated_at 
AFTER UPDATE ON features
BEGIN
    UPDATE features SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER work_packages_updated_at 
AFTER UPDATE ON work_packages
BEGIN
    UPDATE work_packages SET updated_at = datetime('now') WHERE id = NEW.id;
END;
```

### Graph Model (Neo4j)

For complex dependency queries and relationship analysis:

```cypher
// Node types
(:Feature {id: "FEAT-001", title: "User Auth", status: "implementing"})
(:WorkPackage {id: "WP-001", title: "OAuth Core", status: "done"})
(:Agent {name: "claude", type: "ai"})
(:Label {name: "backend"})
(:Project {name: "AgilePlus", path: "/path/to/repo"})

// Relationships
(Feature)-[:OWNS]->(WorkPackage)
(WorkPackage)-[:ASSIGNED_TO]->(Agent)
(WorkPackage)-[:BLOCKS]->(WorkPackage)
(Feature)-[:DEPENDS_ON]->(Feature)
(Feature)-[:TAGGED]->(Label)
(Feature)-[:IN_PROJECT]->(Project)
(WorkPackage)-[:IMPLEMENTS]->(Requirement)
```

---

## CLI Command Reference

### Core Workflow Commands

#### `specify` - Create Feature Specification

```bash
# Interactive mode (discovery interview)
agileplus specify

# Quick mode with flags
agileplus specify \
  --title "User Authentication" \
  --priority p1 \
  --description "OAuth and SAML support"

# Re-run to refine existing spec
agileplus specify FEAT-001

# Output options
agileplus specify --format json
agileplus specify --format yaml
agileplus specify --output kitty-specs/custom-name/
```

**Arguments**:

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--title` | `-t` | Feature title | Prompt |
| `--priority` | `-p` | Priority (p0-p3) | Prompt |
| `--description` | `-d` | Description | Prompt |
| `--quick` | `-q` | Skip interview | false |
| `--yes` | `-y` | Non-interactive | false |
| `--format` | `-f` | Output format | table |
| `--output` | `-o` | Output directory | auto |

**Exit Codes**:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation error |
| 2 | Git error |
| 3 | Spec already exists (use --force to overwrite) |

#### `research` - Technical Analysis

```bash
# Pre-specify research (codebase scan)
agileplus research --mode pre-specify

# Post-specify technical deep-dive
agileplus research FEAT-001

# Specific research areas
agileplus research FEAT-001 --areas security,performance

# Include external sources
agileplus research FEAT-001 --web-search --github-search
```

**Modes**:

| Mode | Purpose | Output |
|------|---------|--------|
| `pre-specify` | Codebase architecture analysis | `research/codebase-analysis.md` |
| `post-specify` | Technical feasibility | `kitty-specs/FEAT-XXX/research.md` |
| `competitor` | Competitive analysis | `research/competitor-analysis.md` |

#### `plan` - Generate Work Packages

```bash
# Generate plan from spec
agileplus plan FEAT-001

# Update plan (refinement loop)
agileplus plan FEAT-001 --update

# Custom template
agileplus plan FEAT-001 --template minimal

# Include estimates
agileplus plan FEAT-001 --with-estimates
```

**Plan Templates**:

| Template | Description | WPs Generated |
|----------|-------------|---------------|
| `full` | Complete plan with all sections | 5-10 WPs |
| `minimal` | Essential WPs only | 2-4 WPs |
| `research` | Research-focused | 1-2 WPs |
| `spike` | Proof of concept | 1 WP |

#### `implement` - Execute Work Packages

```bash
# Implement all WPs for a feature
agileplus implement FEAT-001

# Implement specific WP
agileplus implement FEAT-001 WP-002

# Limit concurrent agents
agileplus implement FEAT-001 --max-agents 2

# Dry run (show what would happen)
agileplus implement FEAT-001 --dry-run

# Skip CI checks (for hotfixes)
agileplus implement FEAT-001 --skip-ci
```

**Implementation Options**:

| Flag | Description | Default |
|------|-------------|---------|
| `--max-agents` | Concurrent agents | 3 |
| `--agent` | Specific agent type | auto |
| `--reviewer` | Code reviewer | coderabbit |
| `--timeout` | Per-WP timeout | 4h |
| `--dry-run` | Preview only | false |

#### `validate` - Quality Gates

```bash
# Validate feature against governance
agileplus validate FEAT-001

# Validate specific aspects
agileplus validate FEAT-001 --scope tests,governance

# Strict mode (warnings as errors)
agileplus validate FEAT-001 --strict

# Generate report file
agileplus validate FEAT-001 --output validation-report.md
```

**Validation Scopes**:

| Scope | Checks |
|-------|--------|
| `all` | Everything |
| `tests` | FR-to-evidence tracing |
| `governance` | Policy gates |
| `coverage` | Test coverage thresholds |
| `security` | Vulnerability scan |
| `lint` | Code quality |

#### `ship` - Deliver Feature

```bash
# Ship validated feature
agileplus ship FEAT-001

# Ship with custom merge strategy
agileplus ship FEAT-001 --strategy squash

# Skip validation (use with caution)
agileplus ship FEAT-001 --force

# Keep worktrees (don't clean up)
agileplus ship FEAT-001 --keep-worktrees
```

#### `retro` - Post-Hoc Analysis

```bash
# Generate retrospective
agileplus retro FEAT-001

# Include metrics
agileplus retro FEAT-001 --with-metrics

# Suggest governance updates
agileplus retro FEAT-001 --update-constitution
```

### Utility Commands

#### `status` - Project Dashboard

```bash
# Overall status
agileplus status

# Feature-specific status
agileplus status FEAT-001

# Watch mode (live updates)
agileplus status --watch

# Filter by status
agileplus status --status implementing

# JSON for scripting
agileplus status --format json | jq '.features[] | select(.status == "implementing")'
```

**Status Output**:

```
┌──────────┬────────────────────────┬──────────────┬──────────┬─────────┐
│ ID       │ Title                  │ Status       │ Progress │ Blocked │
├──────────┼────────────────────────┼──────────────┼──────────┼─────────┤
│ FEAT-001 │ User Authentication    │ implementing │ 2/5 WPs  │ WP-003  │
│ FEAT-002 │ API Rate Limiting      │ planned      │ 0/3 WPs  │ -       │
│ FEAT-003 │ Audit Logging          │ validated    │ 4/4 WPs  │ -       │
└──────────┴────────────────────────┴──────────────┴──────────┴─────────┘
```

#### `config` - Settings Management

```bash
# View all config
agileplus config

# Get value
agileplus config get sync.plane.url

# Set value
agileplus config set sync.plane.enabled true

# Edit config file
agileplus config edit

# Validate config
agileplus config validate
```

#### `init` - Project Bootstrap

```bash
# Initialize in current directory
agileplus init

# Initialize with specific template
agileplus init --template rust-workspace

# Force re-initialization
agileplus init --force
```

### Git Commands

```bash
# Scan commits for correlations
agileplus git scan
agileplus git scan --since 2024-01-01
agileplus git scan --feature FEAT-001

# Show correlations for feature
agileplus git show FEAT-001

# Manually link commit to feature
agileplus git link FEAT-001 abc123def

# Sync git notes
agileplus git sync-notes

# Create worktree for WP
agileplus git worktree FEAT-001 WP-002

# Clean up worktrees
agileplus git cleanup-worktrees
```

### Sync Commands

```bash
# Push to Plane.so
agileplus sync push-plane FEAT-001

# Pull from Plane.so
agileplus sync pull-plane

# Bidirectional sync
agileplus sync sync-plane

# Push to GitHub
agileplus sync push-github FEAT-001

# Check sync status
agileplus sync status
```

### Hidden Subcommands (Agent-Only)

These commands are hidden from help but available for agent orchestration:

```bash
# Triage
agileplus triage classify "fix the login bug"
agileplus triage file-bug --title "Auth fails" --description "..."
agileplus triage queue-idea "Add dark mode"

# Governance
agileplus governance check-gates FEAT-001
agileplus governance verify-chain FEAT-001
agileplus governance evaluate-policy --policy security

# DevOps
agileplus devops lint-and-format
agileplus devops run-ci-checks
agileplus devops conventional-commit "feat: add OAuth"

# Escape hatches
agileplus escape hotfix --title "Critical fix"
agileplus escape quick-fix --file src/auth.rs
agileplus escape skip-with-warning FEAT-001 WP-002
```

---

## Git Integration

### Multi-Layer Git Correlation

AgilePlus uses multiple mechanisms to correlate work items with code changes:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Git Correlation Strategy                          │
│                                                                       │
│  Layer 1: Commit Message Parsing (Primary)                            │
│  ─────────────────────────────────────────                            │
│  Pattern: "FEAT-001: Implement OAuth flow"                          │
│           "WP-001: Add token validation"                            │
│                                                                       │
│  Regex: (FEAT|WP)-([0-9]+)[\s:]+(.*)                                 │
│                                                                       │
│  Layer 2: Git Notes (Secondary)                                     │
│  ────────────────────────────────                                     │
│  git notes add -m "Feature: FEAT-001, WP: WP-002" <commit>          │
│                                                                       │
│  Layer 3: Branch Naming (Tertiary)                                  │
│  ─────────────────────────────────                                    │
│  Pattern: feat/FEAT-001-oauth-core                                    │
│           fix/WP-003-token-refresh                                    │
│                                                                       │
│  Layer 4: Worktree Isolation (Default)                                │
│  ───────────────────────────────────                                  │
│  Each WP gets isolated worktree:                                    │
│  .worktrees/FEAT-001/WP-001/                                         │
│  .worktrees/FEAT-001/WP-002/                                         │
└─────────────────────────────────────────────────────────────────────┘
```

### Worktree Architecture

```
project-root/
├── .agileplus/                    # State directory
│   └── worktrees/                 # Worktree registry
│
├── .worktrees/                    # Git worktrees
│   └── FEAT-001/
│       ├── WP-001/                # Isolated worktree
│       │   ├── src/               # Modified files
│       │   ├── tests/
│       │   └── .agileplus/        # WP-specific state
│       │
│       ├── WP-002/                # Another isolated worktree
│       │   └── ...
│       │
│       └── shared/                # Shared dependencies (symlinks)
│
├── kitty-specs/                   # Specifications
│   └── FEAT-001/
│       ├── spec.yaml
│       ├── plan.md
│       └── tasks/
│
└── src/                           # Main codebase
```

### Commit Correlation Implementation

```rust
// crates/agileplus-git/src/correlation.rs
pub struct CommitCorrelator {
    repo: Repository,
    feature_pattern: Regex,
    wp_pattern: Regex,
}

impl CommitCorrelator {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        
        // Flexible patterns for different conventions
        let feature_pattern = Regex::new(
            r"(?i)(?:^|\s)(?:FEAT|feat|Feature|#)-?\s*(\d+)"
        )?;
        
        let wp_pattern = Regex::new(
            r"(?i)(?:^|\s)(?:WP|wp|Work-?Package)-?\s*(\d+)"
        )?;
        
        Ok(Self {
            repo,
            feature_pattern,
            wp_pattern,
        })
    }
    
    /// Scan repository history for correlations
    pub fn scan(&self, options: ScanOptions) -> Result<Vec<Correlation>> {
        let mut walk = self.repo.revwalk()?;
        
        // Configure walk
        if let Some(since) = options.since {
            walk.push_range(&format!("{}..HEAD", since))?;
        } else {
            walk.push_head()?;
        }
        
        let mut correlations = Vec::new();
        
        for oid in walk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let message = commit.message().unwrap_or("");
            
            // Extract feature references
            for cap in self.feature_pattern.captures_iter(message) {
                let feature_num = &cap[1];
                let feature_id = format!("FEAT-{}", feature_num);
                
                // Look for WP references in same message
                let mut wp_id = None;
                for wp_cap in self.wp_pattern.captures_iter(message) {
                    wp_id = Some(format!("WP-{}", &wp_cap[1]));
                    break; // Usually one WP per commit
                }
                
                correlations.push(Correlation {
                    commit_hash: oid.to_string(),
                    feature_id,
                    work_package_id: wp_id,
                    commit_message: message.to_string(),
                    author: commit.author().name().unwrap_or("").to_string(),
                    timestamp: DateTime::from_timestamp(commit.time().seconds(), 0)
                        .unwrap_or_else(|| Utc::now()),
                    correlation_method: CorrelationMethod::MessagePattern,
                });
            }
        }
        
        Ok(correlations)
    }
    
    /// Write correlation to git notes
    pub fn add_note(&self, commit_hash: &str, feature_id: &str, wp_id: Option<&str>) -> Result<()> {
        let oid = Oid::from_str(commit_hash)?;
        let commit = self.repo.find_commit(oid)?;
        
        let note_content = if let Some(wp) = wp_id {
            format!("Feature: {}, WP: {}", feature_id, wp)
        } else {
            format!("Feature: {}", feature_id)
        };
        
        let sig = self.repo.signature()?;
        self.repo.note(
            &sig, &sig, None,
            commit.id(),
            &note_content,
            false,  // Don't force
        )?;
        
        Ok(())
    }
}
```

### Worktree Management

```rust
// crates/agileplus-git/src/worktree.rs
pub struct WorktreeManager {
    repo: Repository,
    base_path: PathBuf,
}

impl WorktreeManager {
    /// Create isolated worktree for a work package
    pub fn create_worktree(
        &self,
        feature: &Feature,
        work_package: &WorkPackage,
    ) -> Result<PathBuf> {
        let worktree_path = self.base_path
            .join(&feature.slug)
            .join(format!("WP-{:03}", work_package.ordinal));
        
        // Create parent directories
        fs::create_dir_all(&worktree_path)?;
        
        // Generate branch name
        let branch_name = format!("feat/{}/{}", feature.slug, work_package.slug());
        
        // Create worktree
        let mut cmd = Command::new("git");
        cmd.arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(&branch_name)
            .arg(&worktree_path)
            .arg(&feature.target_branch);
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(GitError::WorktreeCreationFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Write WP context file
        let context_path = worktree_path.join(".agileplus/WP_CONTEXT.md");
        let context = format!(
            "# Work Package Context\n\n\
             Feature: {}\n\
             WP: {}\n\
             Goal: {}\n\
             Acceptance Criteria:\n{}\n",
            feature.title,
            work_package.id,
            work_package.title,
            work_package.acceptance_criteria.join("\n")
        );
        fs::write(&context_path, context)?;
        
        Ok(worktree_path)
    }
    
    /// Clean up worktree after WP completion
    pub fn remove_worktree(&self, worktree_path: &Path) -> Result<()> {
        // Remove worktree
        let mut cmd = Command::new("git");
        cmd.arg("worktree").arg("remove").arg("--force").arg(worktree_path);
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(GitError::WorktreeRemovalFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        Ok(())
    }
    
    /// List all active worktrees
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let mut cmd = Command::new("git");
        cmd.arg("worktree").arg("list").arg("--porcelain");
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // Parse porcelain output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let worktrees = self.parse_worktree_list(&stdout)?;
        
        Ok(worktrees)
    }
}
```

### Git Correlation CLI

```bash
# Scan and store correlations
$ agileplus git scan
Scanning commits since 2024-01-01...
Found 42 correlations:
  FEAT-001: 15 commits
  FEAT-002: 18 commits
  FEAT-003: 9 commits

# Show correlations for feature
$ agileplus git show FEAT-001
Feature: User Authentication

Commits:
  abc1234 WP-001: Implement OAuth core
  def5678 WP-001: Add token validation
  ghi9012 WP-002: SAML integration setup
  ...

Stats:
  Total commits: 15
  Files changed: 23
  Lines added: +1,247
  Lines deleted: -89

# Manual correlation
$ agileplus git link FEAT-001 abc1234
Linked commit abc1234 to FEAT-001

# Sync notes to remote
$ agileplus git sync-notes
Pushing git notes to origin...
Done.
```

---

## Sync Strategies

### Sync Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Sync Architecture                            │
│                                                                       │
│  ┌─────────────┐         ┌─────────────┐         ┌─────────────┐   │
│  │   SQLite    │◄───────►│   NATS      │◄───────►│  Sync       │   │
│  │  (Source)   │  events │  (Event Bus)│  route  │  Workers    │   │
│  └─────────────┘         └─────────────┘         └──────┬──────┘   │
│                                                         │            │
│                              ┌──────────────────────────┤            │
│                              │                          │            │
│                              ▼                          ▼            │
│                       ┌─────────────┐              ┌─────────────┐   │
│                       │  Plane.so   │              │   GitHub    │   │
│                       │  (Mirror)   │              │   (Mirror)  │   │
│                       └─────────────┘              └─────────────┘   │
│                                                                       │
│  Sync Modes:                                                          │
│  ───────────                                                          │
│  • Push: SQLite → External (unidirectional)                          │
│  • Pull: External → SQLite (unidirectional)                            │
│  • Bidirectional: Two-way with conflict detection                      │
│                                                                       │
│  Conflict Resolution:                                                 │
│  ───────────────────                                                  │
│  • Content hash comparison                                            │
│  • Last-write-wins for non-critical fields                            │
│  • Manual resolution required for conflicts                           │
└─────────────────────────────────────────────────────────────────────┘
```

### Plane.so Sync

```rust
// crates/agileplus-integrations/src/plane.rs
pub struct PlaneSyncAdapter {
    client: PlaneClient,
    project_id: String,
}

#[async_trait]
impl SyncAdapter for PlaneSyncAdapter {
    async fn push_feature(&self, feature: &Feature) -> Result<SyncResult> {
        // Check if already synced
        let existing = self.find_existing(feature).await?;
        
        if let Some(plane_issue) = existing {
            // Update existing
            let update = PlaneIssueUpdate {
                title: feature.title.clone(),
                description: self.format_description(feature),
                state: self.map_status(&feature.status),
                priority: self.map_priority(&feature.priority),
            };
            
            let result = self.client.update_issue(&plane_issue.id, update).await?;
            
            Ok(SyncResult::Updated {
                local_id: feature.id.to_string(),
                external_id: result.id,
                url: result.url,
            })
        } else {
            // Create new
            let create = PlaneIssueCreate {
                title: feature.title.clone(),
                description: self.format_description(feature),
                state: self.map_status(&feature.status),
                priority: self.map_priority(&feature.priority),
                labels: feature.labels.clone(),
            };
            
            let result = self.client.create_issue(&self.project_id, create).await?;
            
            // Store mapping
            self.store_mapping(feature, &result).await?;
            
            Ok(SyncResult::Created {
                local_id: feature.id.to_string(),
                external_id: result.id,
                url: result.url,
            })
        }
    }
    
    async fn pull_updates(&self, since: DateTime<Utc>) -> Result<Vec<SyncChange>> {
        let updates = self.client.get_updates(&self.project_id, since).await?;
        
        let mut changes = Vec::new();
        for update in updates {
            if let Some(mapping) = self.find_mapping(&update.id).await? {
                let change = SyncChange {
                    entity_type: EntityType::Feature,
                    entity_id: mapping.local_id,
                    field_changes: self.diff_changes(&mapping, &update)?,
                    external_timestamp: update.updated_at,
                };
                changes.push(change);
            }
        }
        
        Ok(changes)
    }
}
```

### Sync Mappings

```sql
-- Track sync state for each external system
CREATE TABLE sync_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    
    -- External system info
    external_system TEXT NOT NULL,  -- 'plane', 'github', etc.
    external_id TEXT NOT NULL,
    external_url TEXT,
    
    -- Content tracking for conflict detection
    content_hash TEXT NOT NULL,  -- SHA-256 of synced content
    
    -- Timestamps
    last_synced_at TEXT NOT NULL,
    external_updated_at TEXT,
    
    -- Sync configuration
    sync_direction TEXT DEFAULT 'bidirectional',
    auto_sync BOOLEAN DEFAULT 1,
    
    -- Conflict tracking
    conflict_count INTEGER DEFAULT 0,
    last_conflict_at TEXT,
    
    UNIQUE(entity_type, entity_id, external_system)
);

-- Conflict detection query
SELECT 
    sm.entity_type,
    sm.entity_id,
    sm.external_system,
    sm.conflict_count,
    CASE 
        WHEN sm.content_hash != :current_hash THEN 'local_changed'
        WHEN sm.external_updated_at < :external_updated THEN 'remote_changed'
        ELSE 'in_sync'
    END as conflict_status
FROM sync_mappings sm
WHERE sm.entity_id = :entity_id;
```

### Sync CLI

```bash
# Push feature to Plane.so
$ agileplus sync push-plane FEAT-001
Syncing FEAT-001 to Plane.so...
Created Plane issue: https://plane.example.com/projects/PROJ/issues/F-42
Stored mapping in SQLite.

# Pull updates from Plane.so
$ agileplus sync pull-plane
Checking for updates since 2024-03-01T00:00:00Z...
Found 3 updates:
  FEAT-001: Status changed "planned" -> "implementing"
  FEAT-003: Priority changed "p2" -> "p1"
  FEAT-005: Labels updated (+urgent)
Apply changes? [Y/n] y
Applied 3 changes.

# Bidirectional sync
$ agileplus sync sync-plane
Analyzing conflicts...
No conflicts detected.

Pushing local changes:
  FEAT-002: Title updated

Pulling remote changes:
  None

Sync complete.

# Check sync status
$ agileplus sync status
┌──────────┬─────────────┬─────────────┬──────────────┬───────────┐
│ Feature  │ Plane.so    │ GitHub      │ Last Sync    │ Conflicts │
├──────────┼─────────────┼─────────────┼──────────────┼───────────┤
│ FEAT-001 │ ✓ synced    │ ✓ synced    │ 2m ago       │ 0         │
│ FEAT-002 │ ✓ synced    │ ○ pending   │ 1h ago       │ 0         │
│ FEAT-003 │ ✗ conflict  │ ✓ synced    │ 5m ago       │ 1         │
└──────────┴─────────────┴─────────────┴──────────────┴───────────┘
```

### P2P Sync (Multi-Device)

```rust
// crates/agileplus-p2p/src/sync.rs
pub struct P2PSyncManager {
    device_id: String,
    discovery: mDNSDiscovery,
    vector_clock: VectorClock,
}

impl P2PSyncManager {
    /// Discover peers on local network
    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        self.discovery.discover().await
    }
    
    /// Initiate sync with discovered peer
    pub async fn sync_with_peer(&self, peer: &PeerInfo) -> Result<SyncReport> {
        // 1. Exchange vector clocks to determine divergence
        let remote_clock = self.exchange_clock(peer).await?;
        let divergence = self.vector_clock.divergence(&remote_clock);
        
        // 2. Request missing events from peer
        let missing_events = self.request_events(peer, divergence.remote_missing).await?;
        
        // 3. Send our missing events to peer
        self.send_events(peer, divergence.local_missing).await?;
        
        // 4. Merge received events
        for event in missing_events {
            self.apply_event(event).await?;
        }
        
        // 5. Update vector clock
        self.vector_clock.merge(&remote_clock);
        
        Ok(SyncReport {
            sent: divergence.local_missing.len(),
            received: divergence.remote_missing.len(),
            conflicts: 0, // Resolved via vector clock
        })
    }
}
```

---

## Configuration

### Configuration Hierarchy

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Configuration Hierarchy                          │
│                                                                       │
│  Priority 1: CLI Flags (highest)                                      │
│  ─────────────────────────────                                        │
│  --config /path/to/config.toml                                        │
│  --verbose                                                            │
│                                                                       │
│  Priority 2: Environment Variables                                    │
│  ────────────────────────────────                                     │
│  AGILEPLUS_CONFIG=/path/to/config                                     │
│  AGILEPLUS_VERBOSE=1                                                  │
│  AGILEPLUS_SYNC_PLANE_ENABLED=true                                    │
│                                                                       │
│  Priority 3: Project Configuration                                    │
│  ─────────────────────────────────                                    │
│  ./.agileplus/config.toml                                             │
│                                                                       │
│  Priority 4: User Configuration                                       │
│  ─────────────────────────────                                        │
│  ~/.config/agileplus/config.toml                                      │
│  (or platform equivalent)                                             │
│                                                                       │
│  Priority 5: System Defaults (lowest)                                 │
│  ─────────────────────────────────                                    │
│  Embedded defaults in binary                                            │
└─────────────────────────────────────────────────────────────────────┘
```

### Configuration Schema

```toml
# .agileplus/config.toml

[project]
name = "MyProject"
description = "A sample project using AgilePlus"
version = "1.0.0"

# Feature naming conventions
[project.naming]
feature_prefix = "FEAT"
wp_prefix = "WP"
specs_dir = "kitty-specs"

[git]
# Correlation settings
correlation.enabled = true
correlation.auto_scan = true
correlation.scan_on_sync = true

# Patterns for feature ID extraction
correlation.patterns = [
    "FEAT-{id}",
    "#{id}",
    "[FEAT-{id}]"
]

# Worktree settings
worktree.enabled = true
worktree.base_path = ".worktrees"
worktree.cleanup_on_ship = true

# Hooks
hooks.pre_commit = ".agileplus/hooks/pre-commit"
hooks.post_implement = ".agileplus/hooks/post-implement"

[sync]
# Global sync settings
sync.enabled = true
sync.auto_sync = false
sync.interval_seconds = 300

# Conflict resolution
sync.conflict_resolution = "manual"  # or "last_write_wins"

[sync.plane]
enabled = true
url = "https://plane.example.com"
api_key = "${PLANE_API_KEY}"  # Reference to env var
project_id = "83c93c22-9da2-4ef8-9847-05f7e00b9947"

[sync.github]
enabled = true
token = "${GITHUB_TOKEN}"
repo = "myorg/myproject"

[agents]
# Default agent settings
agents.default = "claude"
agents.max_concurrent = 3
agents.timeout_minutes = 240

# Agent-specific settings
[agents.claude]
enabled = true
capabilities = ["rust", "python", "typescript"]

[agents.codex]
enabled = true
capabilities = ["python", "javascript"]

[governance]
# Quality gates
governance.quality_gates.enabled = true
governance.quality_gates.required = [
    "test_coverage",
    "lint_pass",
    "type_check"
]

# Coverage thresholds
governance.thresholds.test_coverage = 80
governance.thresholds.lint_severity = "error"

# Auto-enforcement
governance.auto_block_on_failure = true
governance.allow_force_skip = false

[observability]
# Logging
observability.log_level = "info"
observability.log_format = "json"  # or "pretty"
observability.log_file = ".agileplus/logs/agileplus.log"

# Metrics
observability.metrics.enabled = true
observability.metrics.export_interval_seconds = 60

# Tracing
observability.tracing.enabled = false
observability.tracing.endpoint = "http://localhost:4317"

[nats]
# NATS configuration
nats.enabled = true
nats.url = "nats://localhost:4222"
nats.store_dir = ".agileplus/nats-data"

[cache]
# Dragonfly/Redis cache
cache.enabled = true
cache.url = "redis://localhost:6379"
cache.ttl_seconds = 300

[storage]
# MinIO object storage
storage.enabled = true
storage.endpoint = "http://localhost:9000"
storage.access_key = "${MINIO_ACCESS_KEY}"
storage.secret_key = "${MINIO_SECRET_KEY}"
storage.bucket = "agileplus"
```

### Secrets Management

```rust
// crates/agileplus-domain/src/config/secrets.rs
pub enum SecretStorage {
    /// OS keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager)
    Keychain,
    /// Environment variables
    Environment,
    /// Encrypted file
    EncryptedFile { path: PathBuf },
}

pub struct SecretManager {
    storage: SecretStorage,
}

impl SecretManager {
    /// Store credential securely
    pub fn store(&self, key: &str, value: &str) -> Result<()> {
        match &self.storage {
            SecretStorage::Keychain => {
                let entry = keyring::Entry::new("agileplus", key)?;
                entry.set_password(value)?;
            }
            SecretStorage::Environment => {
                // Environment variables can't be stored, only retrieved
                return Err(ConfigError::EnvironmentReadOnly);
            }
            SecretStorage::EncryptedFile { path } => {
                self.store_encrypted(path, key, value)?;
            }
        }
        Ok(())
    }
    
    /// Retrieve credential
    pub fn retrieve(&self, key: &str) -> Result<Option<String>> {
        match &self.storage {
            SecretStorage::Keychain => {
                let entry = keyring::Entry::new("agileplus", key)?;
                match entry.get_password() {
                    Ok(password) => Ok(Some(password)),
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
            SecretStorage::Environment => {
                let env_key = format!("AGILEPLUS_{}", key.to_uppercase());
                Ok(env::var(&env_key).ok())
            }
            SecretStorage::EncryptedFile { path } => {
                self.retrieve_encrypted(path, key)
            }
        }
    }
}
```

---

## Performance Targets

### CLI Performance

| Metric | Target | p95 | Measurement |
|--------|--------|-----|-------------|
| Cold start | < 100ms | < 150ms | `time agileplus --help` |
| Warm start | < 50ms | < 75ms | Second invocation |
| Command dispatch | < 20ms | < 30ms | Parsing and routing |
| Tab completion | < 100ms | < 150ms | First suggestion |
| Status query | < 50ms | < 100ms | `agileplus status` |
| Feature creation | < 200ms | < 500ms | `agileplus specify` |

### Database Performance

| Operation | Target | p95 | Measurement |
|-----------|--------|-----|-------------|
| Feature read | < 1ms | < 5ms | Single row by ID |
| Feature list | < 10ms | < 50ms | 100 features |
| WP read | < 1ms | < 5ms | Single row by ID |
| WP list (per feature) | < 5ms | < 20ms | 20 WPs |
| Audit scan | < 50ms | < 200ms | 1000 entries |
| Event append | < 1ms | < 5ms | Single event |
| Search (FTS5) | < 20ms | < 100ms | Full-text query |

### Git Operations

| Operation | Target | p95 | Measurement |
|-----------|--------|-----|-------------|
| Worktree creation | < 500ms | < 2s | Single worktree |
| Worktree removal | < 200ms | < 1s | Cleanup |
| Commit scan | < 100ms | < 500ms | 1000 commits |
| Correlation query | < 50ms | < 200ms | Feature correlations |

### Sync Performance

| Operation | Target | p95 | Measurement |
|-----------|--------|-----|-------------|
| Plane.so push | < 1s | < 3s | Single feature |
| GitHub push | < 1s | < 3s | Single feature |
| Conflict detection | < 50ms | < 200ms | Compare hashes |
| Full sync | < 5s | < 15s | 50 features |

### Resource Usage

| Resource | Target | Limit | Measurement |
|----------|--------|-------|-------------|
| Memory (idle) | < 50MB | < 100MB | RSS after startup |
| Memory (active) | < 200MB | < 500MB | During implementation |
| Binary size | < 50MB | < 100MB | Release build |
| SQLite size | < 100MB | < 1GB | Per project |
| Concurrent features | 10 | 50 | Active features |
| Concurrent WPs | 20 | 100 | Active work packages |

---

## Security Model

### Threat Model

| Threat | Severity | Mitigation |
|--------|----------|------------|
| Credential exposure | Critical | OS keychain integration, encrypted storage |
| Audit log tampering | Critical | SHA-256 hash chaining, append-only |
| Unauthorized state changes | High | Governance contracts, evidence requirements |
| Data exfiltration | Medium | Local-first, no cloud dependency |
| Worktree isolation breach | Medium | Git worktrees, filesystem permissions |
| P2P sync MITM | Medium | TLS for gRPC, device authentication |

### Security Controls

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Security Controls                            │
│                                                                       │
│  Data at Rest                                                         │
│  ────────────                                                         │
│  • SQLite: Filesystem permissions (0600)                              │
│  • Credentials: OS keychain or encrypted file                         │
│  • Specs: Git-committed, versioned                                    │
│  • Audit log: Hash-chained, tamper-evident                            │
│                                                                       │
│  Data in Transit                                                      │
│  ────────────────                                                     │
│  • gRPC: TLS 1.3 required                                             │
│  • P2P sync: mTLS with device certificates                            │
│  • External APIs: HTTPS only                                          │
│                                                                       │
│  Access Control                                                       │
│  ───────────────                                                      │
│  • API keys: SHA-256 hashed, revocable                                │
│  • CLI: OS user permissions                                           │
│  • Worktrees: Filesystem isolation                                    │
│                                                                       │
│  Audit & Compliance                                                   │
│  ───────────────────                                                  │
│  • All state changes logged with actor attribution                    │
│  • Evidence required for sensitive transitions                        │
│  • Hash chain integrity verifiable offline                            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Deployment Patterns

### Local Development

```bash
# Single command startup
agileplus init
agileplus status
```

### CI/CD Integration

```yaml
# .github/workflows/agileplus.yml
name: AgilePlus Validation

on:
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install AgilePlus
        run: |
          curl -fsSL https://agileplus.io/install.sh | sh
          agileplus --version
      
      - name: Correlate commits
        run: |
          agileplus git scan
      
      - name: Validate governance
        run: |
          # Find feature from branch name
          FEATURE=$(echo "${{ github.head_ref }}" | grep -oE 'FEAT-[0-9]+' || echo "")
          if [ -n "$FEATURE" ]; then
            agileplus validate "$FEATURE" --strict
          fi
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p agileplus-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y git sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agileplus /usr/local/bin/
ENTRYPOINT ["agileplus"]
```

---

## Monitoring & Observability

### Three Pillars

| Pillar | Implementation | Metrics |
|--------|---------------|---------|
| **Logs** | `tracing` with structured JSON | All operations |
| **Metrics** | OpenTelemetry + Prometheus | Performance, counts |
| **Traces** | OpenTelemetry with W3C context | Distributed operations |

### Key Metrics

```rust
// crates/agileplus-telemetry/src/metrics.rs
lazy_static! {
    pub static ref COMMAND_DURATION: HistogramVec = register_histogram_vec!(
        "agileplus_command_duration_seconds",
        "Command execution time",
        &["command"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();
    
    pub static ref ACTIVE_FEATURES: Gauge = register_gauge!(
        "agileplus_active_features",
        "Number of non-archived features"
    ).unwrap();
    
    pub static ref WP_COMPLETION_RATE: CounterVec = register_counter_vec!(
        "agileplus_wp_completions_total",
        "Work package completions",
        &["status"]
    ).unwrap();
}
```

### SLOs

| SLO | Target | Alert Threshold |
|-----|--------|-----------------|
| Command availability | 99.9% | < 99% |
| p95 command latency | < 1s | > 2s |
| Sync success rate | > 99% | < 95% |
| Audit log integrity | 100% | Any failure |

---

## References

### Internal Documentation

| Document | Purpose |
|----------|---------|
| `ADR.md` | Architecture Decision Records |
| `docs/adr/` | Detailed ADRs |
| `FUNCTIONAL_REQUIREMENTS.md` | Complete FR list |
| `FR_TRACEABILITY.md` | Requirement tracing |
| `GOVERNANCE.md` | Governance principles |
| `PLAN.md` | Project roadmap |

### External References

| Resource | URL |
|----------|-----|
| SQLite | https://www.sqlite.org/ |
| libsql/Turso | https://github.com/tursodatabase/libsql |
| NATS | https://nats.io/ |
| Neo4j | https://neo4j.com/ |
| Protocol Buffers | https://protobuf.dev/ |
| gRPC | https://grpc.io/ |
| MCP (Model Context Protocol) | https://modelcontextprotocol.io/ |
| Plane.so | https://plane.so/ |
| Tonic (Rust gRPC) | https://github.com/hyperium/tonic |
| sqlx | https://github.com/launchbadge/sqlx |
| git2-rs | https://github.com/rust-lang/git2-rs |
| OpenTelemetry | https://opentelemetry.io/ |
| FastMCP | https://github.com/jlowin/fastmcp |

### Inspiration

- **bmad**: Spec-driven development with deep governance
- **spec-kitty**: Worktree isolation and Kanban tracking
- **OpenSpec**: Simplicity in specification
- **GSD**: Automation and parallel execution
- **thegent**: Smart contracts and hash-chained audit

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-27 | Initial specification |
| 1.1 | 2026-03-15 | Added P2P sync, Neo4j graph |
| 1.2 | 2026-03-27 | Added import subsystem, MinIO |
| 2.0 | 2026-04-02 | Comprehensive rewrite with nanovms style |

---

*This specification is a living document. Updates are tracked in git and require governance approval for significant changes.*
