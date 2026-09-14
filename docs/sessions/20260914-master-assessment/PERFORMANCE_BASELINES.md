# AgilePlus Performance Baselines

**Version**: 0.2.1 | **Date**: 2026-09-14 | **Machine**: macOS aarch64 (Apple Silicon)
**Tool**: criterion 0.5 (100 samples, 3s warmup)

---

## 1. Event System

### Event Append Throughput

| Benchmark | Latency | Notes |
|-----------|---------|-------|
| `append_single_event` | **46.3 µs** | Single event to in-memory store |
| `append_1000_events_100_entities` | **63.6 ms** | Batch: 10 events/entity avg |
| `sequential_events/5000` | **391.8 ms** | 5K sequential events |

**Analysis**: Single-event append at ~46µs is excellent for a local event store. Batch throughput of ~15K events/sec (1000/63.6ms) is production-ready.

### Event Replay

| Benchmark | Latency | Notes |
|-----------|---------|-------|
| `apply_1000_events_in_memory` | **73.1 µs** | In-memory replay (no I/O) |
| `snapshot_replay_900_plus_100` | **184.1 µs** | Snapshot + 100 delta events |
| `full_replay_1000_events` | **1.68 ms** | Full replay from storage |

**Analysis**: Snapshot-based replay (184µs) is 9x faster than full replay (1.68ms). Snapshotting is critical for performance at scale.

---

## 2. Graph / Entity System

| Benchmark | Latency | Notes |
|-----------|---------|-------|
| `graph_seed_features/100` | **197.9 µs** | Insert 100 features |
| `graph_create_owns_relationship` | **4.7 µs** | Single relationship creation |
| `graph_dependency_chain_query` | **86.1 ns** | Dependency chain traversal |

**Analysis**: Relationship creation at ~5µs and dependency queries at ~86ns are extremely fast. The graph layer is not a bottleneck.

---

## 3. Storage Layer

### SQLite Adapter (from transport tests)

| Operation | Latency | Notes |
|-----------|---------|-------|
| Open DB (WAL mode) | <1ms | One-time setup |
| Single row query | <1ms | `WHERE id = ?` |
| Aggregation query | <1ms | `GROUP BY state` |
| Full feature list (100 rows) | <5ms | `ORDER BY created_at DESC` |

**Analysis**: All SQLite operations are sub-5ms. The database layer is not a bottleneck for CLI or desktop operations.

### gRPC Transport (from transport tests)

| Operation | Latency | Notes |
|-----------|---------|-------|
| `domain_error_to_status` | <1µs | Error mapping (pure function) |
| `parse_evidence_requirement` | <1µs | String parsing (pure function) |
| ProxyRouter construction | <1ms | One-time setup |

---

## 4. CLI Command Latency

| Command | Latency | Notes |
|---------|---------|-------|
| `--version` | <10ms | Binary metadata |
| `--help` | <10ms | Clap help rendering |
| `list` (empty) | <50ms | DB query + table render |
| `list` (10 features) | <100ms | DB query + table render |
| `specify --from-file` | <200ms | File read + DB write + spec artifact |
| `plan --feature` | <500ms | Spec parse + WP generation + DB write |
| `dashboard` | <200ms | DB aggregation + ASCII render |

---

## 5. Desktop App

| Operation | Latency | Notes |
|-----------|---------|-------|
| `open_project` | <100ms | DB discovery + connection |
| `list_features` | <50ms | SQL query + JSON serialization |
| `get_feature_with_details` | <100ms | 3 SQL queries (feature + WPs + evidence) |
| `get_dashboard_stats` | <50ms | 2 aggregation queries |

---

## 6. Scaling Projections

### Feature Count Scaling

| Features | Expected `list` latency | Expected `plan` latency |
|----------|------------------------|------------------------|
| 10 | <100ms | <500ms |
| 100 | <200ms | <1s |
| 1,000 | <500ms | <2s |
| 10,000 | <1s | <5s (index-dependent) |

### Work Package Scaling

| WPs per Feature | Expected `plan` latency |
|-----------------|------------------------|
| 5 | <500ms |
| 20 | <1s |
| 100 | <2s |

---

## 7. Regression Thresholds

These thresholds trigger CI failures if exceeded:

| Metric | Threshold | Action |
|--------|-----------|--------|
| Single event append | >100µs | Investigate |
| Full replay (1K events) | >5ms | Check I/O |
| Graph dependency query | >1µs | Check index |
| CLI list (10 features) | >500ms | Check query plan |
| Desktop open_project | >500ms | Check DB discovery |

---

## 8. Memory Profiles

| Operation | Peak Memory | Notes |
|-----------|-------------|-------|
| CLI binary | ~15MB RSS | Minimal footprint |
| Desktop app (idle) | ~80MB RSS | Tauri + WebView |
| Event replay (1K) | ~2MB delta | In-memory state |
| Graph (100 features) | ~1MB delta | Entity + relationship cache |

---

## 9. Future Benchmarks Needed

| Benchmark | Priority | Status |
|-----------|----------|--------|
| gRPC server request latency | P1 | Not implemented |
| Desktop Tauri IPC roundtrip | P1 | Not implemented |
| Agent adapter spawn latency | P2 | Not implemented |
| Credential store read/write | P2 | Not implemented |
| Concurrent read performance | P2 | Not implemented |
