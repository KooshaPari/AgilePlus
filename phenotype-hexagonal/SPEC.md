# Phenotype Hexagonal Specification

> Multi-language hexagonal architecture framework

## Overview

Phenotype Hexagonal implements the hexagonal architecture pattern (also known as ports and adapters) consistently across Rust, Go, Python, and TypeScript.

## Core Concepts

### Domain (Inside)
The business logic that is independent of external concerns:
- **Entities:** Objects with identity and lifecycle
- **Value Objects:** Immutable objects without identity
- **Domain Services:** Operations that don't fit in entities
- **Domain Events:** Significant occurrences in the domain

### Ports (Interfaces)
Contracts that define how the domain interacts with the outside world:
- **Inbound Ports (Driving):** How the application is driven (HTTP, CLI, Events)
- **Outbound Ports (Driven):** What the application needs (Database, Cache, External APIs)

### Adapters (Implementation)
Concrete implementations of ports:
- **Primary Adapters:** Call the application (REST API, GraphQL, CLI)
- **Secondary Adapters:** Called by the application (PostgreSQL, Redis, S3)

## Language Implementations

### Rust (`rust/`)
```rust
// Port definition
trait UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
}

// Domain
struct User {
    id: Uuid,
    email: String,
}

// Adapter
struct PostgresUserRepository {
    pool: PgPool,
}

impl UserRepository for PostgresUserRepository {
    // Implementation
}
```

### Go (`go/`)
```go
// Port (interface)
type UserRepository interface {
    FindByID(ctx context.Context, id uuid.UUID) (*User, error)
    Save(ctx context.Context, user *User) error
}

// Domain
type User struct {
    ID    uuid.UUID
    Email string
}

// Adapter
type PostgresUserRepository struct {
    db *sql.DB
}

func (r *PostgresUserRepository) FindByID(ctx context.Context, id uuid.UUID) (*User, error) {
    // Implementation
}
```

### Python (`python/`)
```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
from uuid import UUID

# Port
class UserRepository(ABC):
    @abstractmethod
    async def find_by_id(self, id: UUID) -> User | None:
        pass

# Domain
@dataclass
class User:
    id: UUID
    email: str

# Adapter
class PostgresUserRepository(UserRepository):
    def __init__(self, session: AsyncSession):
        self.session = session
    
    async def find_by_id(self, id: UUID) -> User | None:
        # Implementation
```

### TypeScript (`typescript/`)
```typescript
// Port
interface UserRepository {
  findByID(id: string): Promise<User | null>;
  save(user: User): Promise<void>;
}

// Domain
class User {
  constructor(
    public readonly id: string,
    public readonly email: string
  ) {}
}

// Adapter
class PostgresUserRepository implements UserRepository {
  constructor(private db: Knex) {}
  
  async findByID(id: string): Promise<User | null> {
    // Implementation
  }
}
```

## Shared Patterns

### Dependency Injection
All implementations support constructor injection:
- **Rust:** Generic parameters or trait objects
- **Go:** Interface satisfaction
- **Python:** Constructor injection
- **TypeScript:** Constructor parameters

### Testing
All implementations provide test adapters:
- In-memory repositories
- Mock adapters
- Test containers

### Configuration
Environment-based configuration for all adapters:
- Database connections
- Cache settings
- External API endpoints

## Cross-Language Consistency

| Aspect | Rust | Go | Python | TypeScript |
|--------|------|-----|---------|-----------|
| Port | Trait | Interface | ABC | Interface |
| Domain | Struct | Struct | dataclass | Class |
| Adapter | Impl | Method Impl | Class | Class |
| DI | Generic/Box | Interface | Constructor | Constructor |
| Async | async_trait | goroutines | asyncio | async/await |

## References

- [Hexagonal Architecture (Alistair Cockburn)](https://alistair.cockburn.us/hexagonal-architecture/)
- [Ports and Adapters Pattern](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software))
