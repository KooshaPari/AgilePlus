# AgilePlus Security Threat Model

**Version**: 0.2.1 | **Date**: 2026-09-14 | **Author**: Jcode (automated)
**Scope**: CLI, gRPC server, Tauri desktop app, dashboard, agent adapters

---

## 1. System Overview

AgilePlus is a spec-driven development platform with three runtime contexts:

| Context | Components | Trust Level |
|---------|-----------|-------------|
| **CLI** | `agileplus` binary, SQLite DB, git operations | Local user (full trust) |
| **Server** | gRPC server, API key auth, proxy router | Network-exposed (partial trust) |
| **Desktop** | Tauri 2 app, SQLite read, tray menu | Local user (full trust) |

### Data Flow

```
User → CLI → SQLite DB ← Desktop App
              ↓
         gRPC Server → Agent Adapters (Codex/Claude/Gemini)
              ↓
         Dashboard (HTTP) → Service Restart
```

---

## 2. Trust Boundaries

| Boundary | From → To | Risk |
|----------|-----------|------|
| **TB-1**: User → CLI | Local process arguments | Low (local user) |
| **TB-2**: CLI → SQLite | SQL queries with user input | Medium (injection) |
| **TB-3**: CLI → Git | Shell commands with feature slugs | Medium (injection) |
| **TB-4**: Network → gRPC | API key authentication | High (remote) |
| **TB-5**: gRPC → Agent | Process spawning with user config | High (RCE) |
| **TB-6**: Dashboard → OS | Service restart commands | High (privilege) |
| **TB-7**: Desktop → SQLite | Read-only DB access | Low (local) |

---

## 3. Threat Catalog

### T-01: SQLite Injection
- **Surface**: Feature slugs, work package titles, evidence paths passed to SQL
- **Severity**: Medium
- **Status**: MITIGATED — All queries use parameterized statements (`?1`, `:param`)
- **Evidence**: `commands/*.rs` use `conn.prepare("... WHERE id = ?1")` throughout
- **Residual risk**: Low. Parameterized queries prevent SQL injection.

### T-02: Git Command Injection
- **Surface**: Feature slugs passed to `git` commands via `Command::new("git").arg(slug)`
- **Severity**: Medium
- **Status**: MITIGATED — Uses `std::process::Command` with `.arg()` (not shell interpolation)
- **Evidence**: `agileplus-git/src/lib.rs:127` — `Command::new("git").args([...])`
- **Residual risk**: Low. Rust's `Command::arg()` does not invoke a shell. However, malicious slugs with newlines could cause git argument injection. Recommend slug validation.

### T-03: Agent Adapter RCE
- **Surface**: Agent adapters spawn external processes (Codex, Claude, Gemini CLI)
- **Severity**: High
- **Status**: PARTIAL MITIGATION
- **Current controls**:
  - Agent backend binary path from environment/config
  - Process spawned with `tokio::process::Command`
  - Output captured and parsed
- **Missing controls**:
  - No sandboxing of spawned processes
  - No resource limits (CPU, memory, time)
  - Agent can write arbitrary files to the workspace
- **Recommendation**: Add process sandboxing, filesystem ACLs, and timeout enforcement.

### T-04: API Key Handling
- **Surface**: API key generation, storage, validation
- **Severity**: High
- **Status**: WELL DESIGNED
- **Controls**:
  - 32-byte random key generation via `rand::fill()`
  - SHA-256 hash stored (non-reversible)
  - Plaintext never persisted to disk
  - Key shown to user once, then discarded
  - Prefix `agp_` for identification
- **Residual risk**: Low. Hash-only storage prevents key recovery.

### T-05: Dashboard Service Restart
- **Surface**: Dashboard routes include service restart via `Command::new(program)`
- **Severity**: High
- **Status**: NEEDS REVIEW
- **Evidence**: `agileplus-dashboard/src/routes/health.rs:142` — `Command::new(program)`
- **Risk**: If `program` is derived from user input without validation, arbitrary command execution is possible.
- **Recommendation**: Validate restart command against allowlist. Never pass unsanitized user input to `Command::new()`.

### T-06: Evidence Generation Command Injection
- **Surface**: `feature_evidence_generate` runs `bash -c` with user-influenced paths
- **Severity**: High
- **Status**: NEEDS REVIEW
- **Evidence**: `agileplus-dashboard/src/routes/evidence.rs:279` — `tokio::process::Command::new("bash")`
- **Risk**: If artifact paths contain shell metacharacters, command injection is possible.
- **Recommendation**: Avoid shell invocation. Use `Command::new("bash").arg("-c")` with properly escaped input, or better, avoid bash entirely.

### T-07: Unsafe Code Blocks
- **Surface**: 24 `unsafe` blocks across the codebase
- **Severity**: Medium
- **Status**: ACCEPTABLE (mostly tests)
- **Breakdown**:
  - ~20 blocks are `unsafe { env::set_var/remove_var }` in test code (test-only, not production)
  - 1 block in telemetry: `unsafe impl Send for TelemetryAdapter` (soundness concern)
  - 1 block in dashboard: `unsafe { restart_service_executes_command }`
- **Note**: `traceability-core` uses `#![forbid(unsafe_code)]` — good practice.

### T-08: Credential Store File Fallback
- **Surface**: `FileCredentialStore` writes encrypted credentials to disk
- **Severity**: Medium
- **Status**: MITIGATED
- **Controls**:
  - File-based store uses encryption key from `AGILEPLUS_CREDENTIAL_KEY` env var
  - Keychain backend preferred (macOS/Linux)
  - File permissions should be restricted (0600)
- **Recommendation**: Verify file permissions on creation. Log warning if world-readable.

### T-09: CLI Argument Injection
- **Surface**: Feature slugs, file paths, branch names from CLI args
- **Severity**: Low
- **Status**: MITIGATED
- **Controls**:
  - Clap derive parser handles argument parsing
  - Feature slugs validated by Clap constraints
  - File paths checked for existence before use
- **Residual risk**: Low. Local user context limits impact.

### T-10: Proxy Router SSRF
- **Surface**: `ProxyRouter` forwards gRPC requests to downstream servers
- **Severity**: Medium
- **Status**: NEEDS REVIEW
- **Risk**: If downstream server URLs are user-configurable, SSRF is possible.
- **Recommendation**: Validate downstream URLs against allowlist. Block internal IPs (127.0.0.1, 10.x, 192.168.x).

### T-11: Workspace Directory Traversal
- **Surface**: Desktop app reads files from workspace (ADRs, traces, evidence)
- **Severity**: Low
- **Status**: MITIGATED
- **Controls**:
  - Files read relative to known workspace root
  - No user-controlled path concatenation in file reads
  - SQLite queries use parameterized IDs

### T-12: Denial of Service
- **Surface**: Large number of features/work packages, expensive queries
- **Severity**: Low
- **Status**: PARTIAL MITIGATION
- **Controls**:
  - CLI commands have `--limit` flags
  - Dashboard queries use pagination
- **Missing controls**:
  - No query timeout enforcement
  - No rate limiting on gRPC server
- **Recommendation**: Add query timeouts and gRPC rate limiting.

---

## 4. Recommendations Summary

| Priority | ID | Recommendation | Effort |
|----------|----|----------------|--------|
| **P0** | T-06 | Remove bash invocation in evidence generation | 2h |
| **P0** | T-05 | Validate dashboard restart command against allowlist | 1h |
| **P1** | T-03 | Add process sandboxing for agent adapters | 1d |
| **P1** | T-10 | Validate proxy router downstream URLs | 2h |
| **P2** | T-02 | Add feature slug format validation (alphanumeric + hyphens only) | 1h |
| **P2** | T-08 | Verify file permissions on credential store creation | 30m |
| **P2** | T-12 | Add query timeouts and gRPC rate limiting | 4h |
| **P3** | T-07 | Audit `unsafe impl Send/Sync` in telemetry adapter | 1h |

---

## 5. Positive Findings

1. **Parameterized SQL queries** throughout — no string interpolation in SQL
2. **Non-reversible API key storage** — SHA-256 hashes only, no plaintext persistence
3. **`Command::arg()` usage** — avoids shell injection in most git operations
4. **`#![forbid(unsafe_code)]`** in traceability-core — prevents unsafe in critical path
5. **Test isolation** — `unsafe` blocks confined to test code for env var manipulation
6. **Input validation** — Clap derive parser with constraints on arguments

---

## 6. Audit Checklist

- [ ] T-06: Replace bash invocation with safe process spawning
- [ ] T-05: Add restart command allowlist validation
- [ ] T-03: Evaluate process sandboxing options (landlock, seccomp, containers)
- [ ] T-10: Add URL validation to proxy router configuration
- [ ] T-02: Add regex validation for feature slugs (`^[a-z0-9][a-z0-9-]*$`)
- [ ] T-08: Add file permission check for credential store
- [ ] T-12: Add query timeout configuration
- [ ] T-07: Review `unsafe impl Send/Sync` for soundness
