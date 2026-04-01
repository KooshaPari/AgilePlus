# Specification: OrgOps Capital Ledger & Resource Management

**Slug**: eco-012-orgops-capital-ledger | **Date**: 2026-03-31 | **State**: specified

## Problem Statement

Phenotype agents operate with dozens of API keys, cloud credits, free-tier accounts, and browser authentication sessions across 8+ independent repositories. Currently:

1. **No resource awareness** — agents don't know what LLM subscriptions, cloud credits, or free-tier accounts are available to them
2. **Secret fragility** — authkit keys expire every 5 minutes, agents bypass ggshield, rotation is manual and error-prone
3. **No consumption tracking** — token budgets, API call counts, and compute hours are untracked
4. **No profile persistence** — browser user agent profiles lose auth state between sessions
5. **Worktree chaos** — no systematic gix-based worktree management; canonical repo is mixed with development

The result: agents waste cycles on failed API calls, duplicate secrets across worktrees, and have no concept of organizational "capital."

## Target Users

- **AI Agents** (Forge, Sage, Muse, Helios): Need to query available resources before starting work, check secret freshness, allocate budgets
- **Org Operator** (kooshapari): Needs a single place to declare resources, monitor consumption, manage rotations
- **CI/CD**: Needs secrets injected into worktrees without manual .env management

## Functional Requirements

### FR-CAP-001: Capital Registry (Org Level)
The system SHALL parse `repos/capital.toml` to build a registry of all available organizational resources: LLM accounts, cloud credits, free-tier accounts, API keys, browser profiles.

### FR-CAP-002: Resource Allocation (Project Level)
The system SHALL allow projects to declare which resources they need via `.agileplus/capital.toml` (project-level), and the org SHALL track allocations vs. available capacity.

### FR-CAP-003: Consumption Tracking
The system SHALL record token usage, API call counts, and compute hours per agent session in a local SQLite database (`.agileplus/capital.db`).

### FR-CAP-004: Budget Enforcement
The system SHALL support per-project and per-agent budget limits. When a budget is exceeded, the system SHALL return a `BudgetExceeded` error (reusing `phenotype-cost-core::CostError`).

### FR-SEC-001: Secret Inventory
The system SHALL maintain an inventory of all secrets (API keys, service keys, tokens) with metadata: env_var name, rotation interval, last validated timestamp, freshness status.

### FR-SEC-002: Secret Validation
The system SHALL validate secrets on startup by pinging each service endpoint. Stale or invalid secrets SHALL be flagged with a `Stale` or `Invalid` status.

### FR-SEC-003: Secret Rotation
The system SHALL support secret rotation with a write-test-swap pattern: write new secret to staging, test it works, then atomically swap into production `.env`.

### FR-SEC-004: Secret Propagation (SQLite + .env Export)
The system SHALL export validated secrets from SQLite to `.env` files in project worktrees. The `.env` file is generated (gitignored), not hand-maintained.

### FR-PRO-001: Browser Profile Management
The `phenotype-profiles` crate SHALL manage persistent Chrome user-data-dir profiles with auth session detection and refresh-on-failure strategy.

### FR-GIT-001: Worktree Management
The `phenotype-git-core` crate SHALL provide gix-based worktree creation, listing, and pruning. Agents SHALL always work in worktrees; canonical pulls SHALL target release branches only.

### FR-GIT-002: Canonical Release Tracking
The system SHALL track which release branch each project's canonical repo is on, so agents can show users new features without polluting development worktrees.

## Non-Functional Requirements

### NFR-001: Local-First
All persistence SHALL use SQLite (`.agileplus/capital.db`). No external database dependencies (no Supabase, no Postgres). WAL mode for concurrent reads.

### NFR-002: No New Paid Services
The system SHALL only use existing paid subscriptions and free-tier accounts. New services must be free-tier or already subscribed.

### NFR-003: Agent Performance
Capital checks (`agileplus capital check`) SHALL complete within 5 seconds for up to 50 accounts. Secret validation pings SHALL have a 3-second timeout per service.

### NFR-004: File Size Limits
All source files SHALL stay within 500 lines (hard limit). Decompose into modules if approaching limit.

### NFR-005: Test Coverage
All new code SHALL have inline `#[cfg(test)]` modules. Each FR SHALL have at least one test with `// Traces to: FR-CAP-NNN` or `// Traces to: FR-SEC-NNN` comment.

## Constraints & Dependencies

### Existing Crates (Reuse)
- `phenotype-cost-core`: Cost, BudgetManager, CostError — reuse for budget enforcement
- `agileplus-sqlite`: SqliteStorageAdapter, WAL pattern, migrations — reuse for capital.db persistence
- `phenotype-error-core`: Canonical error types
- `gix 0.71`: Already in workspace deps — use for worktree management
- `rusqlite 0.32`: Already in workspace deps

### External Dependencies (None New Required)
All needed deps already in workspace: `serde`, `toml`, `rusqlite`, `gix`, `reqwest`, `tokio`, `chrono`

### Architecture Decisions
1. **Independence preserved**: Each project remains an independent git repo. No submodules.
2. **Worktree-first**: Agents always work in worktrees. Canonical = release branch only.
3. **SQLite + .env export**: Secrets stored in SQLite, exported to .env for tool compatibility.
4. **`phenotype-profiles` as separate crate**: Browser UA profiles owned separately from capital.

## Acceptance Criteria

### AC-001: Capital Status
**Given** a `capital.toml` with 5 LLM accounts and 3 cloud accounts, **When** an agent runs `agileplus capital status`, **Then** a JSON response shows all accounts with remaining budgets, freshness status, and rate limit headroom.

### AC-002: Secret Validation
**Given** a stale Groq API key (expired), **When** `agileplus capital check` runs, **Then** the Groq account is flagged `Stale` with timestamp of last successful validation.

### AC-003: Budget Enforcement
**Given** a project with a daily token budget of 100K, **When** an agent attempts to spend 150K tokens, **Then** a `BudgetExceeded` error is returned before the API call is made.

### AC-004: Secret Propagation
**Given** a fresh SQLite capital.db with valid secrets, **When** `agileplus capital export-env` runs from a project directory, **Then** a `.env` file is generated with all required env vars populated from the database.

### AC-005: Worktree Creation
**Given** an agent needs to implement a feature, **When** `agileplus worktree create --project heliosCLI --branch feat/auth` runs, **Then** a new worktree is created at `.worktrees/heliosCLI/feat/auth` with the correct branch checked out via gix.

### AC-006: Profile Persistence
**Given** a Chrome profile with active GitHub auth, **When** the agent session restarts, **Then** the profile reuses the existing user-data-dir and detects the auth session is still valid without re-authenticating.
