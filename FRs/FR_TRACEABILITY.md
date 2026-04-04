# FR-AGILE Traceability Matrix

## Overview

This document tracks all Functional Requirements (FRs) and their associated tests in AgilePlus.

## FR Index

### Event API Contract Tests (FR-AGILE-EVNT)
- File: `tests/contracts/api_events_contract.rs`
- Prefix: FR-AGILE-001 to FR-AGILE-013
- Description: Contract tests for agileplus-api ↔ agileplus-events boundary

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

### Dashboard API Contract Tests (FR-AGILE-DASH)
- File: `tests/contracts/dashboard_api_contract.rs`
- Prefix: FR-AGILE-014 to FR-AGILE-024
- Description: Contract tests for agileplus-api ↔ agileplus-dashboard boundary

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-014 | Feature response has required fields | `contract_feature_response_has_required_fields` |
| FR-AGILE-015 | Feature response state is lowercase string | `contract_feature_response_state_is_lowercase_string` |
| FR-AGILE-016 | Feature response serializes to JSON with correct types | `contract_feature_response_serializes_to_json_with_correct_types` |
| FR-AGILE-017 | Feature response deserializes from JSON | `contract_feature_response_deserializes_from_json` |
| FR-AGILE-018 | Work package response has required fields | `contract_work_package_response_has_required_fields` |
| FR-AGILE-019 | Work package response state is lowercase | `contract_work_package_response_state_lowercase` |
| FR-AGILE-020 | API error not found produces JSON error field | `contract_api_error_not_found_produces_json_error_field` |
| FR-AGILE-021 | API error bad request status code | `contract_api_error_bad_request_status_code` |
| FR-AGILE-022 | API error conflict status code | `contract_api_error_conflict_status_code` |
| FR-AGILE-023 | State strings parse to feature states | `contract_state_strings_parse_to_feature_states` |
| FR-AGILE-024 | Timestamps are RFC3339 formatted | `contract_timestamps_are_rfc3339` |

### Events SQLite Contract Tests (FR-AGILE-EVT-SQLite)
- File: `tests/contracts/events_sqlite_contract.rs`
- Prefix: FR-AGILE-025 to FR-AGILE-034
- Description: Contract tests for agileplus-events ↔ agileplus-sqlite boundary

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

### Sync Plane Contract Tests (FR-AGILE-SYNC)
- File: `tests/contracts/sync_plane_contract.rs`
- Prefix: FR-AGILE-035 to FR-AGILE-051
- Description: Contract tests for agileplus-sync ↔ agileplus-plane boundary

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-035 | Feature state created maps to backlog | `contract_feature_state_created_maps_to_backlog` |
| FR-AGILE-036 | Feature state specified maps to unstarted | `contract_feature_state_specified_maps_to_unstarted` |
| FR-AGILE-037 | Feature state implementing maps to started | `contract_feature_state_implementing_maps_to_started` |
| FR-AGILE-038 | Feature state validated maps to completed | `contract_feature_state_validated_maps_to_completed` |
| FR-AGILE-039 | Plane backlog maps to created | `contract_plane_backlog_maps_to_created` |
| FR-AGILE-040 | Plane started maps to implementing | `contract_plane_started_maps_to_implementing` |
| FR-AGILE-041 | Plane completed maps to validated | `contract_plane_completed_maps_to_validated` |
| FR-AGILE-042 | Plane unknown group is handled gracefully | `contract_plane_unknown_group_is_handled_gracefully` |
| FR-AGILE-043 | Plane state group parsing is case insensitive | `contract_plane_state_group_parsing_case_insensitive` |
| FR-AGILE-044 | Plane issue serializes with required name field | `contract_plane_issue_serializes_with_required_name_field` |
| FR-AGILE-045 | Plane issue response has id and name | `contract_plane_issue_response_has_id_and_name` |
| FR-AGILE-046 | Plane label has id name and optional color | `contract_plane_label_has_id_name_and_optional_color` |
| FR-AGILE-047 | Plane label deserializes from API response | `contract_plane_label_deserializes_from_api_response` |
| FR-AGILE-048 | Plane label color is optional | `contract_plane_label_color_is_optional` |
| FR-AGILE-049 | Plane issue built from feature preserves name | `contract_plane_issue_built_from_feature_preserves_name` |
| FR-AGILE-050 | Feature with plane id produces update not create | `contract_feature_with_plane_id_produces_update_not_create` |
| FR-AGILE-051 | State roundtrip is stable | `contract_state_roundtrip_is_stable` |

### BDD Fixtures Tests (FR-AGILE-BDD)
- File: `tests/bdd/fixtures_test.rs`
- Prefix: FR-AGILE-052 to FR-AGILE-058
- Description: Tests that validate fixture files load and parse correctly

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-052 | Sample audit chain verifies | `sample_audit_chain_verifies` |
| FR-AGILE-053 | Sample governance parses | `sample_governance_parses` |
| FR-AGILE-054 | Sample spec contains FRs | `sample_spec_contains_frs` |
| FR-AGILE-055 | Sample plan contains WPs | `sample_plan_contains_wps` |
| FR-AGILE-056 | Sample meta parses | `sample_meta_parses` |
| FR-AGILE-057 | Evidence fixtures parse | `evidence_fixtures_parse` |
| FR-AGILE-058 | Pact fixture parses | `pact_fixture_parses` |

### Test Fixtures Tests (FR-AGILE-FIXTURES)
- File: `tests/fixtures/mod.rs`
- Prefix: FR-AGILE-059 to FR-AGILE-064
- Description: Test fixture helpers and loaders

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-059 | Sample audit chain loads and verifies | `sample_audit_chain_loads_and_verifies` |
| FR-AGILE-060 | Sample governance contract loads | `sample_governance_contract_loads` |
| FR-AGILE-061 | Sample spec loads | `sample_spec_loads` |
| FR-AGILE-062 | Sample plan loads | `sample_plan_loads` |
| FR-AGILE-063 | Sample meta loads | `sample_meta_loads` |
| FR-AGILE-064 | Evidence fixtures load | `evidence_fixtures_load` |

### Integration Tests (FR-AGILE-INTEGRATION)
- File: `tests/integration/test_full_workflow.rs`
- Prefix: FR-AGILE-065 to FR-AGILE-068
- Description: Full workflow integration tests

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-065 | Full specify to ship workflow | `test_full_specify_to_ship_workflow` |
| FR-AGILE-066 | Specify rejects invalid state transition | `test_specify_rejects_invalid_state_transition` |
| FR-AGILE-067 | Validate blocks on missing evidence | `test_validate_blocks_on_missing_evidence` |
| FR-AGILE-068 | Audit chain integrity after full lifecycle | `test_audit_chain_integrity_after_full_lifecycle` |

## Summary

| Category | File | FR Range | Count |
|----------|------|----------|-------|
| Event API | api_events_contract.rs | FR-AGILE-001 to FR-AGILE-013 | 13 |
| Dashboard API | dashboard_api_contract.rs | FR-AGILE-014 to FR-AGILE-024 | 11 |
| Events SQLite | events_sqlite_contract.rs | FR-AGILE-025 to FR-AGILE-034 | 10 |
| Sync Plane | sync_plane_contract.rs | FR-AGILE-035 to FR-AGILE-051 | 17 |
| BDD Fixtures | fixtures_test.rs | FR-AGILE-052 to FR-AGILE-058 | 7 |
| Test Fixtures | mod.rs | FR-AGILE-059 to FR-AGILE-064 | 6 |
| Integration | test_full_workflow.rs | FR-AGILE-065 to FR-AGILE-068 | 4 |
| **Total** | | | **68** |

## FR Files

- `FR-AGILE-EVNT.md` - Event API Contract Tests
- `FR-AGILE-DASH.md` - Dashboard API Contract Tests
- `FR-AGILE-EVT-SQLite.md` - Events SQLite Contract Tests
- `FR-AGILE-SYNC.md` - Sync Plane Contract Tests
- `FR-AGILE-BDD.md` - BDD Fixtures Tests
- `FR-AGILE-FIXTURES.md` - Test Fixtures Tests
- `FR-AGILE-INTEGRATION.md` - Integration Tests
