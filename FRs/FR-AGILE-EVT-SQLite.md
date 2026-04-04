# FR-AGILE-EVT-SQLite: Events SQLite Contract Tests

## Overview

Tests contract between agileplus-events and agileplus-sqlite boundary.
Verifies SqliteStorageAdapter satisfies EventStore trait contract.

## Tests (FR-AGILE-025 to FR-AGILE-034)

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-025 | Append returns positive row id | `contract_append_returns_positive_row_id` |
| FR-AGILE-026 | Get events ordered by sequence ascending | `contract_get_events_ordered_by_sequence_ascending` |
| FR-AGILE-027 | Get events scoped to entity | `contract_get_events_scoped_to_entity` |
| FR-AGILE-028 | Get events empty for unknown entity | `contract_get_events_empty_for_unknown_entity` |
| FR-AGILE-029 | Get events since is exclusive | `contract_get_events_since_is_exclusive` |
| FR-AGILE-030 | Get latest sequence zero for empty entity | `contract_get_latest_sequence_zero_for_empty` |
| FR-AGILE-031 | Get latest sequence reflects max | `contract_get_latest_sequence_reflects_max` |
| FR-AGILE-032 | Get events by range inclusive | `contract_get_events_by_range_inclusive` |
| FR-AGILE-033 | Event fields round-trip | `contract_event_fields_roundtrip` |
| FR-AGILE-034 | Entity streams are isolated | `contract_entity_streams_are_isolated` |

## Source

- File: `tests/contracts/events_sqlite_contract.rs`
- Traceability: T112
