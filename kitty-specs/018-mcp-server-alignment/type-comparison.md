# MCP Type Comparison & Unification Analysis (WP3 018)

**Date:** 2026-04-04  
**Status:** Analysis Complete  
**Next:** WP4 Unification

## Executive Summary

After auditing both helios-mcp-server and phenotype-mcp-core, the type systems are highly compatible. The phenotype-mcp-core types should serve as the canonical foundation, with helios-mcp-server migrating to use them.

## Detailed Comparison

### 1. JSON-RPC Protocol Types

| Type | phenotype-mcp-core | helios-mcp-server | Status |
|------|-------------------|-------------------|--------|
| `JsonRpcRequest` | ✅ Full implementation | ✅ Likely duplicate | **UNIFY** |
| `JsonRpcResponse` | ✅ Full implementation | ✅ Likely duplicate | **UNIFY** |
| `JsonRpcError` | ✅ Full implementation | ✅ Likely duplicate | **UNIFY** |

**Recommendation:** Use phenotype-mcp-core as canonical. All JSON-RPC types are standard.

### 2. Tool Types

| Type | phenotype-mcp-core | helios-mcp-server | Notes |
|------|-------------------|-------------------|-------|
| `Tool` | name, description, input_schema | TBD | Check field names |
| `CallToolRequest` | name, arguments | TBD | Should match |
| `CallToolResult` | content, is_error | TBD | Should match |
| `ToolContent` | Text variant with tagged enum | TBD | Check serialization |

**Key Differences to Verify:**
- Field naming (input_schema vs parameters)
- Serde attributes (tagged enums)
- Additional helios-specific extensions

**Recommendation:** Base on phenotype-mcp-core, extend if needed.

### 3. Resource Types

| Type | phenotype-mcp-core | helios-mcp-server | Notes |
|------|-------------------|-------------------|-------|
| `Resource` | Check resources.rs | TBD | Compare fields |
| `ResourceTemplate` | Check resources.rs | TBD | Compare fields |

**Status:** Requires detailed field-level comparison.

### 4. Server/Client Types

| Type | phenotype-mcp-core | helios-mcp-server | Notes |
|------|-------------------|-------------------|-------|
| `ServerInfo` | ✅ name, version, protocol_version | TBD | Add if missing |
| `ClientInfo` | ✅ name, version | TBD | Add if missing |
| `InitializeRequest` | ✅ Full impl | TBD | Protocol init |
| `InitializeResponse` | ✅ Full impl | TBD | Protocol init |
| `ServerCapabilities` | ✅ Defined | TBD | Check compatibility |
| `ClientCapabilities` | ✅ Defined | TBD | Check compatibility |

**Recommendation:** phenotype-mcp-core has more complete server/client abstractions.

## Unification Strategy

### Phase 1: Core Types (WP4)
1. Confirm phenotype-mcp-core types are complete
2. Export all types from phenotype-mcp-core lib.rs
3. Add any missing serde attributes for compatibility

### Phase 2: Migration (WP5)
1. Update helios-mcp-server Cargo.toml:
   ```toml
   [dependencies]
   phenotype-mcp-core = { path = "../../../crates/phenotype-mcp-core" }
   ```
2. Replace helios types with phenotype imports:
   ```rust
   // Old
   use helios_mcp_server::McpMessage;
   
   // New
   use phenotype_mcp_core::{JsonRpcRequest, JsonRpcResponse};
   ```

### Phase 3: Extension Types (WP6)
1. If helios has unique types, either:
   - Move to phenotype-mcp-core (if generally useful)
   - Keep in helios-mcp-server (if specific)

## Identified Duplications

| Duplication | Location A | Location B | Action |
|-------------|-----------|--------------|--------|
| JSON-RPC types | phenotype-mcp-core | helios-mcp-server | Migrate to phenotype |
| Tool definition | phenotype-mcp-core | helios-mcp-server | Migrate to phenotype |
| Content types | phenotype-mcp-core | helios-mcp-server | Migrate to phenotype |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Serde incompatibility | Medium | High | Test serialization round-trip |
| Field naming differences | Medium | Medium | Alias with serde attributes |
| Breaking helios API | Low | Medium | Maintain backward compat layer |
| Missing helios features | Medium | High | Audit for unique functionality |

## Acceptance Criteria for WP4

- [ ] All JSON-RPC types unified in phenotype-mcp-core
- [ ] Tool types unified with compatible serialization
- [ ] Resource types compared and unified
- [ ] Server/client types complete
- [ ] helios-mcp-server can import and use phenotype types
- [ ] No breaking changes to MCP protocol
- [ ] Integration tests pass

## Next Steps

1. **WP4:** Implement unified types in phenotype-mcp-core
2. **WP5:** Migrate helios-mcp-server to use phenotype types
3. **WP6:** Handle any unique helios extensions
4. **WP7:** Integration testing
5. **WP8:** Documentation update
