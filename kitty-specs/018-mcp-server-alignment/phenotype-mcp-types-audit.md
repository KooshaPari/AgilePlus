# phenotype-mcp-core Type Audit (WP2 018)

**Date:** 2026-04-04  
**Auditor:** infrakit-team  
**Scope:** phenotype-mcp-core crate types

## Summary

This document inventories all MCP-related types in phenotype-mcp-core for comparison with helios-mcp-server.

## Type Inventory

### Protocol Types (src/protocol.rs)

| Type | Fields | Serde | Purpose |
|------|--------|-------|---------|
| `JsonRpcRequest` | jsonrpc, id, method, params | yes | JSON-RPC 2.0 request |
| `JsonRpcResponse` | jsonrpc, id, result, error | yes | JSON-RPC 2.0 response |
| `JsonRpcError` | code, message, data | yes | JSON-RPC error object |
| `PROTOCOL_VERSION` | const | - | MCP protocol version "2024-11-05" |

### Tool Types (src/tools.rs)

| Type | Fields | Serde | Purpose |
|------|--------|-------|---------|
| `Tool` | name, description, input_schema | yes | Tool definition |
| `CallToolRequest` | name, arguments | yes | Tool call request |
| `CallToolResult` | content, is_error | yes | Tool call result |
| `ToolContent` | Text { text } (enum) | yes (tag=type) | Tool result content |

### Core Types (src/lib.rs)

| Type | Fields | Serde | Purpose |
|------|--------|-------|---------|
| `ServerInfo` | name, version, protocol_version | no | Server identification |
| `ClientInfo` | name, version | no | Client identification |
| `RequestMeta` | progress_token | no | Request metadata |
| `InitializeRequest` | protocol_version, capabilities, client_info | no | Init request |
| `InitializeResponse` | protocol_version, capabilities, server_info | no | Init response |
| `ClientCapabilities` | roots | no | Client capability flags |
| `RootCapabilities` | list_changed | no | Root list change support |

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `MCP_PROTOCOL_VERSION` | "2024-11-05" | MCP spec version |
| `JSONRPC_VERSION` | "2.0" | JSON-RPC spec version |

## Module Structure

```
phenotype-mcp-core/
├── src/
│   ├── lib.rs          # Core types: ServerInfo, ClientInfo, Initialize*
│   ├── protocol.rs     # JSON-RPC types
│   ├── tools.rs        # Tool types
│   ├── resources.rs    # Resource types
│   ├── server.rs       # Server implementation
│   ├── client.rs       # Client implementation
│   ├── transport.rs    # Transport abstractions
│   ├── handlers.rs     # Protocol handlers
│   └── error.rs        # Error types
```

## Design Patterns

1. **JSON-RPC 2.0:** Standard JSON-RPC protocol for MCP
2. **Serde serialization:** All protocol types derive Serialize/Deserialize
3. **Tagged enums:** `ToolContent` uses `#[serde(tag = "type")]`
4. **Builder pattern:** `ServerInfo::new()`, `ClientInfo::new()`

## Comparison with helios-mcp-server

### Confirmed Duplicates
- JSON-RPC types (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`)
- Tool definition structure

### Potential Differences
- Field naming conventions
- Additional helios-specific extensions
- Error handling approaches

## Canonical Types (Recommended for Unification)

These phenotype-mcp-core types should be the canonical versions:

1. ✅ `JsonRpcRequest/Response/Error` — Standard JSON-RPC
2. ✅ `Tool` — MCP spec compliant
3. ✅ `CallToolRequest/Result` — MCP spec compliant
4. ✅ `ToolContent` — MCP spec compliant
5. ✅ `ServerInfo/ClientInfo` — Standard metadata

## Action Items

- [ ] Complete field-level comparison with helios-mcp-server
- [ ] Identify any helios-specific extensions needed
- [ ] Create unified type definitions if gaps found
- [ ] Plan migration strategy for helios-mcp-server
