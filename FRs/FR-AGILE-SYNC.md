# FR-AGILE-SYNC: Sync Plane Contract Tests

## Overview

Tests contract between agileplus-sync and agileplus-plane boundary.
Verifies PlaneStateMapper, OutboundSync mapping logic, and PlaneLabel types.

## Tests (FR-AGILE-035 to FR-AGILE-051)

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

## Source

- File: `tests/contracts/sync_plane_contract.rs`
- Traceability: T113
