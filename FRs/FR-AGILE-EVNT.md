# FR-AGILE-EVNT: Event API Contract Tests

## Overview

Tests contract between agileplus-api and agileplus-events boundary.
Verifies pagination shape, filter application, field completeness, and ordering guarantees.

## Tests (FR-AGILE-001 to FR-AGILE-013)

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-001 | Event has all API required fields | `contract_event_has_all_api_required_fields` |
| FR-AGILE-002 | Event payload is JSON value | `contract_event_payload_is_json_value` |
| FR-AGILE-003 | Query filter by entity_type isolates stream | `contract_query_filter_by_entity_type_isolates_stream` |
| FR-AGILE-004 | Query filter by entity_id | `contract_query_filter_by_entity_id` |
| FR-AGILE-005 | Query limit matches API page size | `contract_query_limit_matches_api_page_size` |
| FR-AGILE-006 | Query sequence range enables offset pagination | `contract_query_sequence_range_enables_offset_pagination` |
| FR-AGILE-007 | Query preserves sequence order | `contract_query_preserves_sequence_order` |
| FR-AGILE-008 | Query filter by event_type | `contract_query_filter_by_event_type` |
| FR-AGILE-009 | Query no match returns empty not error | `contract_query_no_match_returns_empty_not_error` |
| FR-AGILE-010 | Event serializes to API JSON shape | `contract_event_serializes_to_api_json_shape` |
| FR-AGILE-011 | Event timestamp is RFC3339 | `contract_event_timestamp_is_rfc3339` |
| FR-AGILE-012 | Event hash fields are 32 bytes | `contract_event_hash_fields_are_32_bytes` |
| FR-AGILE-013 | Combined filter and limit | `contract_combined_filter_and_limit` |

## Source

- File: `tests/contracts/api_events_contract.rs`
- Traceability: T115
