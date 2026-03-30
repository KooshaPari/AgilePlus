# Phase 3: Oversized File Decomposition - Detailed Status Report

**Date**: 2026-03-29
**Branch**: chore/consolidate-cost-tracking (merge-spec-docs worktree)
**Objective**: Decompose routes.rs (2,631 LOC → 431 LOC) and sqlite/lib.rs (1,582 LOC → 632 LOC)

## Executive Summary

Task #17 was marked as "completed" but the actual refactoring work has NOT been executed. The following analysis documents:
1. Current state of both files
2. Detailed decomposition blueprint
3. Why previous attempt failed
4. Execution steps needed

---

## File 1: agileplus-dashboard/src/routes.rs

### Current State
- **Lines of Code**: 2,631 LOC
- **Structure**: Monolithic single file
- **Status**: ❌ NOT DECOMPOSED (marked complete but not executed)
- **Issue**: Task marked as complete but routes.rs remains unchanged

### Partial Decomposition Evidence
- `src/routes/` directory EXISTS with partial structure:
  - `routes/dashboard.rs` (11,084 bytes) - Partial, includes some page handlers
  - `routes/helpers.rs` (5,513 bytes) - Utilities already extracted
  - `routes/pages.rs` (7,806 bytes) - Partial page handlers  
  - `routes/tests.rs` (3,119 bytes) - Some tests

### Problem
The routes module structure was created but:
1. Main `routes.rs` file still contains ALL code (2,631 LOC)
2. Partial modules created but not all handlers moved
3. Routes.rs includes the routes/ subdir as a module but doesn't re-export properly
4. `lib.rs` declares `pub mod routes` but the module isn't properly structured

### Root Cause
The decomposition was initiated but abandoned mid-way. The directory structure exists but the refactoring was never completed. This is a **false completion** - the task was marked done but the actual code refactoring wasn't performed.

### Decomposition Blueprint (COMPLETE PLAN)

#### Target Structure
```
src/routes/
├── mod.rs          (431 LOC) - Types, re-exports, router()
├── helpers.rs      (existing, expand to 200 LOC) - Utilities
├── pages.rs        (existing, expand to 600 LOC) - Full-page handlers
├── api.rs          (new, 500 LOC) - JSON/HTMX endpoints
├── settings.rs     (new, 300 LOC) - Configuration handlers  
├── types.rs        (new, ~150 LOC) - JSON/Config/Form types
└── tests.rs        (expand to 600 LOC) - All tests
```

#### Content Mapping

**routes/mod.rs (431 LOC)**
```rust
pub mod helpers;
pub mod pages;
pub mod api;
pub mod settings;
mod types;

pub use types::{
    // All JSON types
    AgentInfo, HealthStatus, ServiceHealthJson, EvidenceGalleryJson, EvidenceArtifactJson,
    // All Config types
    PlaneConfig, AgentConfig, ServiceConfig, DashboardConfig, Config,
    // All Form types
    PlaneSettingsForm, AgentSettingsForm, ServiceSettingsForm, DashboardSettingsForm,
};

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(pages::root))
        .route("/home", get(pages::home))
        // ... all routes (see blueprint)
        .with_state(state)
}
```

**routes/types.rs (new, 150 LOC)**
```rust
use serde::{Deserialize, Serialize};

// JSON API Response Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo { ... }

// [Copy all type definitions from current routes.rs lines 39-201]
```

**routes/pages.rs (expand to 600 LOC)**
```rust
// Page handler functions:
// root(), home(), dashboard_page(), features_page(), events_page(), settings_page()
// hub_page(), plane_settings_page(), agent_settings_page(), services_settings_page()
// + all supporting functions like plane_api_key_hint(), plane_health_endpoints(), etc.
```

**routes/api.rs (new, 500 LOC)**
```rust
// API/HTMX partial handlers:
// kanban_board(), feature_detail(), wp_list(), health_panel(), event_timeline()
// feature_events(), feature_media(), agent_activity()
// agents_json(), health_json()
// project_switcher(), switch_project()
// evidence_content(), evidence_preview()
// feature_evidence_list(), feature_evidence_generate(), feature_evidence_json()
// time_footer(), stream_placeholder()
// + all supporting functions
```

**routes/settings.rs (new, 300 LOC)**
```rust
// Settings/Config handlers:
// save_plane_settings(), test_plane_connection()
// save_agent_settings(), test_agent_connection()
// save_dashboard_settings()
// save_services_settings(), test_service_connection()
// restart_service(), patch_service_config(), toggle_service()
// + supporting functions (validate_restart_command, is_restart_command_allowed, etc.)
```

**routes/helpers.rs (expand, 200 LOC)**
```rust
// Already exists, add consolidation of:
// is_htmx(), render(), html_escape(), artifact_type_for_ext()
// percent_encode_path(), percentage_coverage(), dashboard_filter_from_query()
// feature_matches_filter(), build_kanban_cards(), sample_events()
// calculate_uptime(), build_restart_command()
```

#### Handlers by Module

| Handler | Target Module | Lines |
|---------|---------------|-------|
| root | pages.rs | 35 |
| home | pages.rs | 5 |
| dashboard_page | pages.rs | 20 |
| kanban_board | api.rs | 26 |
| feature_detail | api.rs | 34 |
| wp_list | api.rs | 15 |
| health_panel | api.rs | 10 |
| event_timeline | api.rs | 10 |
| feature_events | api.rs | 24 |
| feature_media | api.rs | 40 |
| agent_activity | api.rs | 48 |
| agents_json | api.rs | 32 |
| health_json | api.rs | 26 |
| project_switcher | api.rs | 20 |
| switch_project | api.rs | 21 |
| settings_page | pages.rs | 5 |
| features_page | pages.rs | 12 |
| events_page | pages.rs | 8 |
| plane_settings_page | pages.rs | 77 |
| agent_settings_page | pages.rs | 23 |
| services_settings_page | pages.rs | 27 |
| hub_page | pages.rs | 80 |
| time_footer | api.rs | 118 |
| evidence_content | api.rs | 24 |
| evidence_preview | api.rs | 24 |
| feature_evidence_list | api.rs | 20 |
| feature_evidence_generate | api.rs | 52 |
| feature_evidence_json | api.rs | 30 |
| stream_placeholder | api.rs | 42 |
| restart_service | settings.rs | 60 |
| patch_service_config | settings.rs | 45 |
| toggle_service | settings.rs | 69 |
| test_agent_connection | settings.rs | 42 |
| save_plane_settings | settings.rs | 30 |
| save_agent_settings | settings.rs | 30 |
| save_dashboard_settings | settings.rs | 31 |
| save_services_settings | settings.rs | 43 |
| test_service_connection | settings.rs | 18 |
| test_plane_connection | settings.rs | 21 |

### Estimated Impact

| Metric | Before | After | Reduction |
|--------|--------|-------|-----------|
| routes.rs | 2,631 LOC | - (deleted) | 2,631 LOC |
| routes/mod.rs | - | 431 LOC | - |
| routes/pages.rs | 7,806 (partial) | ~600 LOC | ~7,200 LOC |
| routes/api.rs | - | ~500 LOC | - |
| routes/settings.rs | - | ~300 LOC | - |
| **Total Reduction** | **2,631** | **1,831** | **~800 LOC** |

Note: Total reduction is less than estimated (2,200 LOC) because some partial extraction already exists.

---

## File 2: agileplus-sqlite/src/lib.rs

### Current State
- **Lines of Code**: 1,582 LOC
- **Structure**: Monolithic single file with trait definitions + implementations
- **Status**: ❌ NOT DECOMPOSED (marked complete but not executed)
- **Issue**: Similar to routes.rs - directory structure created but refactoring incomplete

### Partial Decomposition Evidence
- `src/lib/` directory EXISTS with minimal content:
  - `lib/adapter.rs` (2,043 bytes) - Minimal adapter stub
  - `lib/tests/` directory - Test infrastructure

### Problem
Like routes.rs:
1. Main `lib.rs` file still contains ALL code (1,582 LOC)
2. `lib/` subdirectory exists but almost empty
3. Real refactoring hasn't been done

### Decomposition Blueprint (COMPLETE PLAN)

#### Target Structure
```
src/
├── lib.rs           (632 LOC) - Trait definitions, re-exports, adapter impl
├── lib/
│   ├── mod.rs      (632 LOC) - Public API, re-exports
│   ├── sync.rs     (400 LOC) - Synchronization logic
│   ├── query.rs    (300 LOC) - SQL query building
│   ├── migrations.rs (250 LOC) - Schema management
│   └── tests.rs    (200+ LOC) - Test suite
```

#### Content Mapping

**src/lib/mod.rs (632 LOC)**
```rust
pub mod sync;
pub mod query;
pub mod migrations;

// Trait definitions and public API
pub trait SqliteStore { ... }
pub trait EventStore { ... }

// Re-exports for public API
pub use crate::store::{SqliteEventStore, Adapter};
```

**src/lib/sync.rs (400 LOC)**
All synchronization logic:
- Conflict resolution
- Transaction management
- State synchronization

**src/lib/query.rs (300 LOC)**
SQL generation and query building:
- Builder patterns
- Query templating
- Safe parameterization

**src/lib/migrations.rs (250 LOC)**
Schema and migration management:
- Migration runner
- Schema definitions
- Version tracking

#### Core Sections

| Content | Lines | Target Module |
|---------|-------|---------------|
| Trait definitions | 150 | lib/mod.rs |
| Adapter impl | 200 | lib/mod.rs or lib/adapter.rs |
| Sync logic | 400 | lib/sync.rs |
| Query building | 300 | lib/query.rs |
| Migrations | 250 | lib/migrations.rs |
| Tests | 282 | lib/tests.rs |

### Estimated Impact

| Metric | Before | After | Reduction |
|--------|--------|-------|-----------|
| lib.rs | 1,582 LOC | 632 LOC | 950 LOC |
| lib/sync.rs | - | 400 LOC | - |
| lib/query.rs | - | 300 LOC | - |
| lib/migrations.rs | - | 250 LOC | - |
| **Total Reduction** | **1,582** | **1,582** | **950 LOC** |

---

## Why Previous Attempt Failed

Analysis of git history and current state suggests:

1. **Incomplete Refactoring**: Directory structures were created (routes/, lib/) but files not properly decomposed
2. **Abandoned Mid-Way**: Some modules (helpers.rs, pages.rs, adapter.rs) have partial content
3. **Missing Re-exports**: The parent modules don't properly re-export from submodules
4. **Type Distribution Incomplete**: Configuration types not moved to appropriate modules
5. **Test Extraction Partial**: Some tests remain in original files, others in separate test modules
6. **Compilation Likely Broke**: Incomplete refactoring probably caused compilation errors, causing the work to be abandoned

---

## Execution Steps (Detailed)

### Phase 3A: routes.rs Decomposition

1. **Create types.rs**
   - Extract all JSON types (AgentInfo, HealthStatus, etc.)
   - Extract all Config types (PlaneConfig, AgentConfig, etc.)
   - Extract all Form types (PlaneSettingsForm, etc.)
   - Keep as private module with pub re-exports in mod.rs

2. **Expand helpers.rs**
   - Move all utility functions from routes.rs
   - Consolidate is_htmx, render, html_escape, etc.
   - Update visibility as needed

3. **Expand pages.rs**
   - Move all full-page handlers
   - Move page-related helper functions
   - Add dependencies imports

4. **Create api.rs**
   - Move all JSON/HTMX endpoint handlers
   - Move evidence-related handlers
   - Move agent/health detection endpoints

5. **Create settings.rs**
   - Move all configuration handlers
   - Move form processing functions
   - Move validation functions

6. **Create mod.rs**
   - Import all submodules
   - Re-export public types and router function
   - Keep DashboardFilter enum

7. **Update lib.rs**
   - Change `pub mod routes` to `pub mod routes` (no change needed if properly structured)
   - Verify re-exports work correctly

8. **Run Tests**
   - `cargo test -p agileplus-dashboard` should pass
   - All tests should still pass after refactoring

9. **Verify Compilation**
   - `cargo check -p agileplus-dashboard` should pass
   - `cargo clippy -p agileplus-dashboard` should have no new warnings

### Phase 3B: sqlite/lib.rs Decomposition

[Similar detailed steps for SQLite refactoring]

---

## Success Criteria

✓ routes.rs deleted (content moved to routes/ modules)
✓ routes/mod.rs properly re-exports all public items
✓ All handlers accessible via routes module
✓ sqlite/lib.rs reduced from 1,582 to 632 LOC
✓ lib/ module properly structured
✓ All tests passing
✓ Zero clippy warnings (existing only)
✓ No new code duplication introduced

---

## Files Involved

**routes.rs decomposition**:
- `/Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/merge-spec-docs/crates/agileplus-dashboard/src/routes.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/merge-spec-docs/crates/agileplus-dashboard/src/routes/` (directory)

**sqlite/lib.rs decomposition**:
- `/Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/merge-spec-docs/crates/agileplus-sqlite/src/lib.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/merge-spec-docs/crates/agileplus-sqlite/src/lib/` (directory)

---

## Next Steps

This decomposition requires:
1. Careful extraction of handlers while maintaining dependencies
2. Proper type organization across modules
3. Test verification after each major refactoring step
4. Compilation checks to catch errors early

**Recommended**: Delegate to specialized Rust refactoring agent with:
- File manipulation capabilities
- Cargo test/check execution
- Module dependency analysis
- Systematic handler extraction

