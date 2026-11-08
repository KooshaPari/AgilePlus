---
work_package_id: WP16
title: 'trace: Code Intelligence Library Adoption'
lane: planned
dependencies: []
subtasks: [T048, T049, T050]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP16: trace — Code Intelligence Library Adoption

**Implementation command**: `spec-kitty implement WP16`

## Objective

Replace custom code intelligence infrastructure in the trace repo with battle-tested OSS libraries, reducing 7-23.5K LOC of custom parsers, embedding services, and client wrappers while gaining feature coverage (incremental parsing, SCIP cross-references, persistent embeddings).

## Context

- The trace repo (Go+Python+React) provides code-semantic + requirement traceability with graph storage
- No OSS replacement exists for trace itself — its differentiation is the combination of code semantics, requirement traceability, and graph storage
- However, the underlying infrastructure (parsers, embeddings, clients) can be replaced with specialized libraries
- trace already uses NATS for messaging; JetStream adoption is a natural extension
- See `research/external-library-leverage.md` for full analysis

## Subtasks

### T048: Replace custom code parsers with tree-sitter + scip-go

**LOC saved**: 5-14K (2-6K from tree-sitter replacing custom parsers, 3-8K from scip-go replacing custom cross-reference indexing)

**Steps**:
1. Add `github.com/tree-sitter/go-tree-sitter` to Go module dependencies
   - Note: CGo build requirement — ensure CI has C toolchain
2. Replace custom language parsers with tree-sitter grammars:
   - Identify all custom AST parsing code (likely in `internal/indexer/` or similar)
   - Replace with tree-sitter `Parser` + language-specific grammars (Go, Python, TypeScript, Rust, etc.)
   - Use tree-sitter's incremental parsing for file-change re-indexing
3. Add `github.com/sourcegraph/scip-go` for deep Go cross-reference indexing:
   - Replace custom Go symbol resolution with SCIP index generation
   - SCIP provides definition, reference, and hover data in a standard format
   - Note: Sourcegraph has commercial interests; pin version and evaluate license terms
4. Update index storage to consume tree-sitter node types and SCIP occurrences
5. Ensure backward compatibility with existing graph data (migration path for stored indexes)

**Validation**:
- tree-sitter parses all supported languages correctly
- SCIP indexes match or exceed current cross-reference quality for Go
- Incremental parsing works on file-change events
- Existing graph queries still return correct results after migration

### T049: Replace custom embedding service with chromem-go

**LOC saved**: 1-3K

**Steps**:
1. Add `github.com/philippgille/chromem-go` to Go module dependencies
2. Replace custom embedding storage/retrieval:
   - Identify custom embedding service code (likely vector storage, similarity search)
   - Replace with chromem-go's in-process embedding store
   - chromem-go uses brute-force search — acceptable for per-repo scale; evaluate if cross-org scale needs a different solution later
3. Configure persistent storage (chromem-go supports file-based persistence)
4. Migrate existing embeddings or re-index (re-index is simpler if embedding model unchanged)

**Validation**:
- Embedding storage and retrieval works end-to-end
- Similarity search returns equivalent results to current implementation
- Persistence survives process restart

### T050: Replace custom Neo4j client wrappers + optimize NATS JetStream

**LOC saved**: 1-3.5K (0.5-1.5K from Neo4j driver, 0.5-2K from NATS JetStream)

**Steps**:
1. Replace custom Neo4j client wrappers with `github.com/neo4j/neo4j-go-driver/v5`:
   - Identify custom Neo4j client code (connection pooling, query builders, result mappers)
   - Replace with official driver's session management, result helpers, and built-in connection pooling
   - Use driver's `ExecuteRead`/`ExecuteWrite` managed transaction APIs
2. Optimize NATS usage with JetStream:
   - trace already uses `github.com/nats-io/nats.go` for messaging
   - Replace custom message routing/persistence with JetStream streams and consumers
   - Use JetStream for durable event delivery (code change events, index update events)
   - Eliminate custom retry/ack logic that JetStream provides natively

**Validation**:
- Neo4j queries execute correctly with official driver
- Connection pooling and transaction management work under load
- JetStream durable consumers receive all events without loss
- No regression in message delivery latency

## Definition of Done

- [ ] tree-sitter replaces custom language parsers
- [ ] scip-go provides Go cross-reference indexing
- [ ] chromem-go replaces custom embedding storage
- [ ] neo4j-go-driver v5 replaces custom Neo4j wrappers
- [ ] NATS JetStream replaces custom message routing/persistence
- [ ] All existing graph queries and searches return correct results
- [ ] CI builds with CGo toolchain (for tree-sitter)

## Risks

- **CGo build complexity**: tree-sitter requires C compilation. Ensure CI images have gcc/clang. Consider static linking for deployment.
- **chromem-go scale limits**: Brute-force search is O(n). Acceptable for per-repo embeddings (~10K-100K vectors); may need replacement (e.g., Qdrant) for cross-organization scale.
- **scip-go Sourcegraph coupling**: Monitor for license changes or API instability. Pin to specific version, maintain ability to revert to custom indexing.
- **JetStream operational overhead**: JetStream requires stream configuration and monitoring. Ensure proper stream retention policies to avoid unbounded storage growth.

## Reviewer Guidance

- Verify tree-sitter grammar coverage matches current language support
- Check that SCIP index quality matches or exceeds custom Go cross-references
- Ensure chromem-go persistence is properly configured (not in-memory only)
- Verify Neo4j driver connection pooling settings are appropriate for workload
- Check JetStream stream retention and consumer acknowledgment policies
