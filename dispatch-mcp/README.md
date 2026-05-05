# dispatch-mcp

MCP server for tier-based dispatch delegation via OmniRoute.

## Tools

### Per-tier dispatch tools

| Tool name | Tier |
|---|---|
| `dispatch_worker` | `worker` |
| `dispatch_main` | `main` |
| `dispatch_codeman` | `codeman` |
| `dispatch_freetier` | `freetier` |
| `dispatch_kimi` | `kimi` |
| `dispatch_kimi_thinking` | `kimi_thinking` |
| `dispatch_minimax` | `minimax` |
| `dispatch_opus` | `opus` |
| `dispatch_haiku` | `haiku` |
| `dispatch_gemini` | `gemini` |

Each accepts a single `message: str` argument and dispatches it to the configured OmniRoute backend under the corresponding tier.

### Custom dispatch

`dispatch_custom(tier: str, message: str)` — dispatch to any tier from `VALID_TIERS` above.

### Health

- `dispatch_health()` — probe the OmniRoute backend health endpoint. Requires `OMNIROUTE_URL` to be set.
- `dispatch_liveness()` — returns server liveness status without contacting OmniRoute.

## Configuration

| Variable | Required | Default | Description |
|---|---|---|---|
| `OMNIROUTE_URL` | Yes | — | Base URL of the OmniRoute dispatch backend (e.g. `http://localhost:8080`) |

### Constraints

- `message` must not exceed **4096 bytes** (UTF-8 encoded).
- `tier` must be one of the known tiers listed above.
- HTTP redirects are **not followed** — only direct requests to `OMNIROUTE_URL` are made.

## Run

```bash
# Set the backend URL
export OMNIROUTE_URL=http://localhost:8080

# Via entry point
dispatch-mcp

# Or directly
python -m dispatch_mcp.server
```
