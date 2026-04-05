# helios-mcp-server Type Audit (WP1 018)

**Date:** 2026-04-04  
**Auditor:** infrakit-team  
**Scope:** helios-mcp-server crate types

## Summary

This document inventories all MCP-related types in helios-mcp-server for comparison with phenotype-mcp-core.

## Type Inventory

### Message Types

| Type | Location | Fields | Purpose |
|------|----------|--------|---------|
| `McpMessage` | TBD | TBD | Top-level MCP protocol message |
| `McpRequest` | TBD | TBD | Request envelope |
| `McpResponse` | TBD | TBD | Response envelope |

### Tool Types

| Type | Location | Fields | Purpose |
|------|----------|--------|---------|
| `Tool` | TBD | name, description, parameters | Tool definition |
| `ToolCall` | TBD | name, arguments | Tool invocation |
| `ToolResult` | TBD | content, is_error | Tool execution result |

### Resource Types

| Type | Location | Fields | Purpose |
|------|----------|--------|---------|
| `Resource` | TBD | uri, name, mime_type, text | Resource definition |
| `ResourceTemplate` | TBD | uri_template, name | Resource template |
| `ResourceContent` | TBD | uri, mime_type, text/blob | Resource content |

### Content Types

| Type | Location | Fields | Purpose |
|------|----------|--------|---------|
| `Content` | TBD | type discriminator | Content union |
| `TextContent` | TBD | text | Text content |
| `ImageContent` | TBD | data, mime_type | Image content |

### Protocol Types

| Type | Location | Fields | Purpose |
|------|----------|--------|---------|
| `JsonRpcRequest` | TBD | jsonrpc, id, method, params | JSON-RPC 2.0 request |
| `JsonRpcResponse` | TBD | jsonrpc, id, result, error | JSON-RPC 2.0 response |
| `JsonRpcError` | TBD | code, message, data | JSON-RPC error |

## Comparison with phenotype-mcp-core

### Exact Matches
- `JsonRpcRequest` ↔ phenotype `JsonRpcRequest`
- `JsonRpcResponse` ↔ phenotype `JsonRpcResponse`
- `JsonRpcError` ↔ phenotype `JsonRpcError`
- `Tool` ↔ phenotype `Tool`
- `CallToolRequest` ↔ likely matches helios `ToolCall`
- `CallToolResult` ↔ likely matches helios `ToolResult`

### Semantic Equivalents (Different Names)
- `ToolContent::Text` ↔ helios `TextContent`
- `Tool` structure similar but schema field naming may differ

### Unique to helios-mcp-server
- TBD after full audit

### Unique to phenotype-mcp-core
- `ServerInfo`, `ClientInfo`
- `InitializeRequest`, `InitializeResponse`
- `ServerCapabilities`, `ClientCapabilities`
- `Server` struct

## Recommendations

1. **Unify JSON-RPC types:** Use phenotype-mcp-core `JsonRpcRequest/Response/Error`
2. **Unify Tool types:** Align helios `Tool` with phenotype `Tool`
3. **Consider adopting:** phenotype server framework for common functionality

## Action Items

- [ ] Complete detailed field-level audit of helios-mcp-server
- [ ] Map all semantic equivalents
- [ ] Identify breaking changes for migration
- [ ] Create migration plan
