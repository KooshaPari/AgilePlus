# Phenotype Hexagonal

> Multi-language hexagonal architecture framework

## Overview

Phenotype Hexagonal provides a unified hexagonal architecture (ports and adapters) implementation across multiple programming languages, ensuring consistent patterns and interoperability.

## Languages

| Language | Status | Location |
|----------|--------|----------|
| Rust (Hexacore) | Stable | `rust/` |
| Go (HexaGo) | Stable | `go/` |
| Python (HexaPy) | Stable | `python/` |
| TypeScript (HexaType) | Stable | `typescript/` |

## Architecture

All implementations follow the hexagonal architecture pattern:

```
┌─────────────────────────────────────────┐
│              Application                 │
│  ┌─────────────────────────────────┐   │
│  │         Domain (Core)            │   │
│  │  ┌─────────┐    ┌─────────────┐ │   │
│  │  │ Entities│◄──►│ Value Objects│ │   │
│  │  └─────────┘    └─────────────┘ │   │
│  └─────────────────────────────────┘   │
├─────────────────────────────────────────┤
│               Ports                      │
│  ┌──────────────┐  ┌──────────────┐   │
│  │   Inbound    │  │   Outbound   │   │
│  │  (Driving)   │  │  (Driven)    │   │
│  └──────────────┘  └──────────────┘   │
├─────────────────────────────────────────┤
│              Adapters                    │
│  ┌──────────┐ ┌──────────┐ ┌────────┐│
│  │  HTTP    │ │ Database │ │ Events ││
│  │  CLI     │ │  Cache   │ │ Queue  ││
│  └──────────┘ └──────────┘ └────────┘│
└─────────────────────────────────────────┘
```

## Quick Start

### Rust
```rust
use hexagonal::{Port, Adapter};

// Define your domain
// Implement ports
// Create adapters
```

### Go
```go
import "github.com/phenotype-dev/hexagonal/go"

// Define ports as interfaces
// Implement domain logic
// Create HTTP adapter
```

### Python
```python
from hexagonal import Port, Adapter

# Define abstract base classes
# Implement domain services
# Create FastAPI adapter
```

### TypeScript
```typescript
import { Port, Adapter } from '@phenotype/hexagonal';

// Define interfaces
// Implement services
// Create Express adapter
```

## Documentation

- [Architecture](SPEC.md)
- [Development Plan](PLAN.md)
- [Language-Specific Guides](docs/)

## License

MIT
