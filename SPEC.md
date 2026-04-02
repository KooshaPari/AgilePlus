# AgilePlus Specification

## Architecture
```
┌──────────────────────────────────────────────────────────────────────┐
│                   AgilePlus (Monorepo)                   │
├──────────────────────────────────────────────────────────────────────┤
│  apps/                                                     │
│  ├─ space    (Frontend - Next.js)                        │
│  ├─ api     (Backend - Node.js)                          │
│  └─ worker  (Background jobs)                          │
│                                                         │
│  packages/                                               │
│  ├─ ui        (Shared React components)               │
│  ├─ config    (Configuration)                         │
│  ├─ database  (Database layer)                         │
│  └─ events   (Event system)                          │
└──────────────────────────────────────────────────────┘
```

## Components

| Package | Responsibility | Tech |
|---------|----------------|------|
| space | Web UI | Next.js, React |
| api | REST + WebSocket API | Fastify |
| worker | Async jobs | BullMQ |
| database | Prisma ORM | PostgreSQL |

## Data Models

```typescript
interface Issue {
  id: string;
  title: string;
  status: 'open' | 'in_progress' | 'closed';
  workspace_id: string;
  assignee_id: string | null;
  labels: string[];
}

interface Workspace {
  id: string;
  name: string;
  slug: string;
  members: Member[];
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| API response | <200ms |
| Page load | <1s |
| Worker job | <30s |
| DB query | <50ms |