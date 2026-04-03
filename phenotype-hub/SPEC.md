# phenotype-hub Specification

Canonical definition of the system behavior.

## Overview

**phenotype-hub** is the LLM provider abstraction layer that enables agents to work with multiple LLM providers through a unified interface.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Agents                               │
│   (phenotype-agent-core, phenotype-task-engine, etc.)    │
└─────────────────────────┬───────────────────────────────────┘
                          │ Tool Calls
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    phenotype-hub                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Router    │  │   Selector  │  │   Monitor   │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Provider Abstraction Layer              │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────┬───────────────────────────────────┘
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   OpenAI    │  │ Anthropic  │  │   Ollama    │
│   Provider  │  │   Provider  │  │   Provider  │
└─────────────┘  └─────────────┘  └─────────────┘
```

## Data Models

### Message

```go
type Message struct {
    Role    Role   // "user", "assistant", "system"
    Content string
    Name    string // Optional for multi-agent
}
```

### ChatRequest

```go
type ChatRequest struct {
    Provider   string            // "openai", "anthropic", "ollama"
    Model     string            // "gpt-4", "claude-3", "llama2"
    Messages  []Message         // Conversation history
    MaxTokens int               // Token limit
    Temp      float64           // Temperature (0.0-2.0)
    Tools     []ToolDefinition  // Available tools
}
```

### ChatResponse

```go
type ChatResponse struct {
    Content   string            // Text response
    Provider  string            // Provider used
    Model     string            // Model used
    Tokens    int               // Total tokens
    Latency   time.Duration     // Response time
    Cost      float64           // Cost in USD
}
```

### ToolDefinition

```go
type ToolDefinition struct {
    Type        string            // "function"
    Function    FunctionSchema
}

type FunctionSchema struct {
    Name        string            // Tool name
    Description string            // What it does
    Parameters  map[string]any    // JSON schema
}
```

## API Reference

### Initialize

```bash
hub, err := phenotypehub.New(ctx, phenotypehub.Config{
    DefaultProvider: "openai",
    Providers: []ProviderConfig{
        {Name: "openai", APIKey: os.Getenv("OPENAI_API_KEY")},
        {Name: "anthropic", APIKey: os.Getenv("ANTHROPIC_API_KEY")},
        {Name: "ollama", BaseURL: "http://localhost:11434"},
    },
    Timeout: 60 * time.Second,
})
```

### Chat Completion

```go
// Simple chat
resp, err := hub.Chat(ctx, &ChatRequest{
    Provider: "openai",
    Model:   "gpt-4",
    Messages: []Message{
        {Role: "user", Content: "Hello!"},
    },
})

// With tools
resp, err := hub.Chat(ctx, &ChatRequest{
    Provider:  "anthropic",
    Model:     "claude-3-sonnet",
    Messages:  messages,
    MaxTokens: 1024,
    Temp:      0.7,
    Tools:     []ToolDefinition{calculator, weather},
})
```

### Model Selection

```go
// Auto-select based on task
model, err := hub.SelectModel(ctx, &SelectionRequest{
    Task:        "code generation",
    MaxCost:     0.50,
    MaxLatency:  30 * time.Second,
    Capabilities: []string{"code", "reasoning"},
})

// Use selected model
resp, err := hub.Chat(ctx, &ChatRequest{
    Provider: model.Provider,
    Model:    model.Model,
    // ...
})
```

### Streaming

```go
stream, err := hub.ChatStream(ctx, req)
defer stream.Close()

for {
    chunk, err := stream.Recv()
    if err == io.EOF { break }
    fmt.Print(chunk.Content)
}
```

## Providers

### OpenAI

| Model | Context | Max Output | Cost |
|-------|---------|------------|------|
| gpt-4-turbo | 128K | 4K | $0.01/1K |
| gpt-4 | 8K | 8K | $0.03/1K |
| gpt-3.5-turbo | 16K | 4K | $0.001/1K |

### Anthropic

| Model | Context | Max Output | Cost |
|-------|---------|------------|------|
| claude-3-opus | 200K | 4K | $0.015/1K |
| claude-3-sonnet | 200K | 4K | $0.003/1K |
| claude-3-haiku | 200K | 4K | $0.00025/1K |

### Ollama

| Model | Context | Notes |
|-------|---------|-------|
| llama2 | 4K | Local, no cost |
| codellama | 4K | Code-optimized |
| mistral | 8K | Balanced |

## Configuration

```yaml
# hub.yaml
default_provider: openai
timeout: 60s
retry:
  max_attempts: 3
  backoff: exponential

providers:
  openai:
    api_key: ${OPENAI_API_KEY}
    organization: ${OPENAI_ORG}
    default_model: gpt-4-turbo

  anthropic:
    api_key: ${ANTHROPIC_API_KEY}
    default_model: claude-3-sonnet

  ollama:
    base_url: http://localhost:11434
    default_model: llama2
```

## Error Handling

```go
// Typed errors
var errResp *ErrorResponse
if errors.As(err, &errResp) {
    switch errResp.Code {
    case RateLimitError:
        // Implement backoff
    case AuthError:
        // Refresh credentials
    case ContextLengthError:
        // Truncate messages
    }
}
```

## Observability

### Metrics

- `hub_requests_total{provider, model, status}`
- `hub_tokens_total{provider, model, direction}`
- `hub_latency_seconds{provider, model}`
- `hub_cost_usd{provider, model}`

### Tracing

All operations traced with OpenTelemetry:
- Span name: `hub.{operation}`
- Attributes: provider, model, tokens, cost

### Logging

```go
hub, _ := phenotypehub.New(ctx, Config{
    Logger: slog.Default(),
    // Log levels: debug, info, warn, error
})
```

## Security

1. API keys never logged
2. Keys from environment or secret manager
3. Request/response sanitized
4. Rate limiting per provider
5. Cost budgets enforced

## Testing

```go
// Mock provider for tests
mockProvider := &MockProvider{
    Responses: []ChatResponse{
        {Content: "Test response", Tokens: 10},
    },
}

hub, _ := phenotypehub.New(ctx, Config{
    Providers: []ProviderConfig{
        {Name: "mock", Provider: mockProvider},
    },
})
```
