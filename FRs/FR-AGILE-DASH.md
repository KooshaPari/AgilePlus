# FR-AGILE-DASH: Dashboard API Contract Tests

## Overview

Tests contract between agileplus-api and agileplus-dashboard boundary.
Verifies FeatureResponse JSON schema, state string format, and error response format.

## Tests (FR-AGILE-014 to FR-AGILE-024)

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

## Source

- File: `tests/contracts/dashboard_api_contract.rs`
- Traceability: T114
