# ADR-018: MCP Protocol Integration for AI Agent Native Support

**Date**: 2026-04-04
**Status**: Accepted
**Deciders**: AgilePlus Core Team

---

## Context

AgilePlus positions AI agents as first-class participants in the project management workflow. Unlike traditional PM tools where AI is a plugin or external service, AgilePlus agents must be deeply integrated with the core domain model, able to read specs, create work packages, update state, and be held accountable through the audit chain.

The key question is: **how do AI agents interact with AgilePlus?**

### Constraints

- Agents must be able to perform all PM operations (create feature, transition state, etc.)
- Agents must understand context from SPEC.md files, existing code, and history
- Agent actions must be auditable and attributable
- Hidden subcommands (not visible to humans) must be supported for agent efficiency
- Multiple agents must be able to coordinate on the same feature
- Agent must work with standard LLMs (Claude, GPT-4, Codex, etc.)

### The MCP Decision

After evaluating alternatives, we adopt the **Model Context Protocol (MCP)** from Anthropic as the primary agent integration layer.

---

## Decision

### Why MCP?

**MCP (Model Context Protocol)** is an open protocol developed by Anthropic that enables AI models to connect with external tools and data sources in a standardized way. Unlike proprietary APIs, MCP provides:

| Property | MCP Value | Comparison |
|----------|-----------|------------|
| Open standard | ✅ (Anthropic, now CNCF) | Proprietary APIs vary |
| Tool discovery | ✅ Automatic via protocol | Manual API docs |
| Server implementation | ✅ Any language | Custom per-API |
| Streaming | ✅ Built-in | Varies |
| Sampling (LLM calls) | ✅ Native | Not in REST |
| Resource templates | ✅ Dynamic context | Static endpoints |
| Security | ✅ Scope-based auth | API key management |

### Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                         AgilePlus Agent Architecture                     │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                        MCP Client (Rust)                          │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐                │   │
│  │  │   Claude   │  │   GPT-4    │  │   Codex    │                │   │
│  │  │  (native)  │  │  (via SDK) │  │  (via API) │                │   │
│  │  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘                │   │
│  │        │               │               │                         │   │
│  │        └───────────────┼───────────────┘                         │   │
│  │                        │                                          │   │
│  │                        ▼                                          │   │
│  │              ┌──────────────────┐                                │   │
│  │              │  MCP Client SDK   │                                │   │
│  │              │  (Rust + Python) │                                │   │
│  │              └────────┬─────────┘                                │   │
│  └───────────────────────┼──────────────────────────────────────────┘   │
│                          │                                              │
│                          │ stdio / HTTP+SSE                            │
│                          ▼                                              │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      MCP Server (Python)                           │   │
│  │  ┌──────────────────────────────────────────────────────────┐    │   │
│  │  │                    Tool Handlers                           │    │   │
│  │  │  • feature_create    • feature_transition                 │    │   │
│  │  │  • wp_create         • wp_assign                          │    │   │
│  │  │  • spec_read        • spec_validate                       │    │   │
│  │  │  • audit_query      • cycle_list                          │    │   │
│  │  └──────────────────────────────────────────────────────────┘    │   │
│  │                                                                   │   │
│  │  ┌──────────────────────────────────────────────────────────┐    │   │
│  │  │                   Resource Providers                       │    │   │
│  │  │  • feature://{slug}    → Feature state + history        │    │   │
│  │  │  • spec://{slug}       → SPEC.md content                 │    │   │
│  │  │  • workspace://        → Full project context             │    │   │
│  │  │  • audit://{feature}  → Hash-chained audit trail        │    │   │
│  │  └──────────────────────────────────────────────────────────┘    │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                            │                                             │
│                            ▼                                             │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                    AgilePlus Domain (Rust)                         │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐ │  │
│  │  │  Feature   │  │WorkPackage │  │   Cycle    │  │  Governance│ │  │
│  │  │  Entity    │  │  Entity    │  │   Entity   │  │   Rules    │ │  │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘ │  │
│  │                                                                   │  │
│  │  ┌───────────────────────────────────────────────────────────┐  │  │
│  │  │              SQLite Event Store (Hash-Chained)              │  │  │
│  │  └───────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

### Tool Definitions

Agents interact via these tools (defined in MCP schema):

```json
{
  "name": "feature_create",
  "description": "Create a new feature in AgilePlus",
  "inputSchema": {
    "type": "object",
    "properties": {
      "slug": {
        "type": "string",
        "pattern": "^[a-z0-9-]+$",
        "description": "URL-safe identifier (e.g., 'user-auth-flow')"
      },
      "title": {
        "type": "string",
        "minLength": 3,
        "maxLength": 200
      },
      "description": {
        "type": "string",
        "description": "Optional detailed description"
      },
      "priority": {
        "type": "string",
        "enum": ["P0", "P1", "P2", "P3"]
      },
      "module_id": {
        "type": "string",
        "description": "Optional module assignment"
      }
    },
    "required": ["slug", "title", "priority"]
  }
}
```

### Hidden Subcommands

For agent efficiency, we support **hidden subcommands** that are not exposed in human-visible CLI:

```rust
// In agileplus-subcmds
pub struct HiddenSubcommands;

impl HiddenSubcommands {
    /// Hidden: Bulk create work packages from spec decomposition
    /// Not shown in `pheno help`, only visible to agents
    #[command(hidden = true)]
    pub async fn wp_bulk_create(
        &self,
        feature_id: &str,
        spec_content: &str,
    ) -> Result<Vec<WorkPackageId>> {
        // Agent-only: Parse spec and create WPs automatically
    }

    /// Hidden: Query feature history for context
    #[command(hidden = true)]
    pub async fn feature_context(
        &self,
        slug: &str,
        depth: usize,
    ) -> Result<FeatureContext> {
        // Agent-only: Return rich context for LLM
    }

    /// Hidden: Check if feature is ready for state transition
    #[command(hidden = true)]
    pub async fn can_transition(
        &self,
        feature_id: &str,
        target_state: &str,
    ) -> Result<TransitionCheck> {
        // Agent-only: Pre-flight governance check
    }
}
```

### Agent Identity and Attribution

Every agent action is attributed to a specific agent:

```rust
// Agent identity stored with every action
struct AgentIdentity {
    id: AgentId,           // ULID
    name: String,          // "claude-code:production"
    type: AgentType,       // claude_code, gpt4, codex
    instance_id: String,   // Unique per execution
    capabilities: Vec<Capability>,
    config: AgentConfig,   // Model, temperature, etc.
}

struct DomainEvent {
    // ... other fields
    actor_id: Option<ActorId>,     // Some(agent_id) for agent actions
    actor_type: ActorType,        // ActorType::Agent for agent actions
    metadata: Metadata,            // Includes agent_id, model used, tokens consumed
}
```

### Sampling (LLM calls from tools)

MCP supports **sampling** - the server calling the LLM during tool execution:

```rust
// MCP Server can request LLM reasoning
struct SamplingRequest {
    method: "messages/create",
    params: SamplingParams {
        model: "claude-sonnet-4-20250514",
        max_tokens: 1024,
        system: "You are a project management AI. Analyze this spec...",
        messages: [...],
    }
}

// Use case: Auto-review spec before transition to "specified"
async fn spec_review_sampling(spec_content: &str) -> Result<ReviewResult> {
    let request = SamplingRequest {
        params: SamplingParams {
            system: SPEC_REVIEW_SYSTEM_PROMPT,
            messages: vec![Message {
                role: "user",
                content: format!("Review this spec:\n\n{}", spec_content)
            }],
            ..Default::default()
        }
    };
    mcp_client.sample(request).await
}
```

---

## Options Considered

### Option A: REST API Only (Rejected)

**Description**: Agents access AgilePlus via REST API like any other client.

**Pros**:
- Simple to implement
- Works with any HTTP-capable agent

**Cons**:
- ❌ No standardized tool discovery
- ❌ No streaming support
- ❌ No sampling capability
- ❌ Manual API documentation required
- ❌ No resource abstraction

**Assessment**: ❌ Rejected — does not meet L4 Native AI requirement

### Option B: LangChain Tool Interface (Rejected)

**Description**: Implement LangChain tool interface for agent integration.

**Pros**:
- Wide agent framework compatibility
- Python-first (familiar to AI devs)

**Cons**:
- ❌ Proprietary to LangChain
- ❌ No standardized protocol
- ❌ Python-centric (we use Rust)
- ❌ No resource concept

**Assessment**: ❌ Rejected — proprietary lock-in, not a protocol

### Option C: MCP Protocol (Selected)

**Description**: Implement MCP server as the primary agent integration layer.

**Pros**:
- ✅ Open protocol (CNCF)
- ✅ Language-agnostic
- ✅ Built-in tool discovery
- ✅ Streaming + sampling native
- ✅ Resource abstraction
- ✅ Growing ecosystem (Anthropic, Google, Sourcegraph)

**Cons**:
- ⚠️ Newer standard (2024)
- ⚠️ Rust SDK less mature than Python

**Assessment**: ✅ Selected — best balance of openness, capability, and ecosystem

---

## Implementation Plan

### Phase 1: Core MCP Server (2026 Q2)

```python
# agileplus-mcp/mcp_server.py
from mcp.server import Server
from mcp.types import Tool, Resource
import asyncio

server = Server("agileplus")

@server.list_tools()
async def list_tools() -> list[Tool]:
    return [
        Tool(
            name="feature_create",
            description="Create a new feature",
            inputSchema=FeatureCreateInputSchema,
        ),
        # ... more tools
    ]

@server.call_tool()
async def call_tool(name: str, arguments: dict) -> CallToolResult:
    if name == "feature_create":
        return await feature_create(arguments)
    # ...
```

### Phase 2: Resource Providers (2026 Q3)

```python
@server.list_resources()
async def list_resources() -> list[Resource]:
    return [
        Resource(
            uri="feature://{slug}",
            name="Feature",
            description="Feature entity with history",
            mimeType="application/json",
        ),
        Resource(
            uri="spec://{slug}",
            name="Specification",
            description="SPEC.md content",
            mimeType="text/markdown",
        ),
    ]
```

### Phase 3: Sampling Integration (2026 Q4)

Implement LLM-based review and validation using MCP sampling.

---

## Consequences

### Positive

1. **Standard integration**: Any MCP-compatible agent can work with AgilePlus
2. **Tool discovery**: Agents automatically see available operations
3. **Rich context**: Resource providers give agents full project context
4. **Attribution**: All agent actions are auditable
5. **Sampling**: Built-in LLM reasoning during tool execution

### Negative

1. **MCP SDK maturity**: Rust SDK less mature than Python
2. **Protocol evolution**: MCP still evolving (mitigated by versioned protocol)
3. **Additional service**: MCP server adds deployment complexity

### Security Considerations

| Concern | Mitigation |
|---------|------------|
| Agent permissions | Scope-based access via MCP OAuth |
| Audit trail | All agent actions logged with identity |
| Token consumption | Metadata includes token counts |
| Prompt injection | Validate all agent inputs, sanitize SPEC.md |
| Rate limiting | Per-agent rate limits via token bucket |

---

## References

- [AI-001] Anthropic (2024). "Model Context Protocol Specification" - modelcontextprotocol.io
- [AI-003] LangChain - LLM Application Framework - python.langchain.com
- [AI-004] AutoGPT - Autonomous Agent Framework - agpt.co
- [CLI-001] clig.dev - Command Line Interface Guidelines - clig.dev

---

*Decision made 2026-04-04 to adopt MCP as the standard agent integration protocol.*
