"""Boundary tests for MCP input validation."""

from __future__ import annotations

import pytest

from agileplus_mcp.validation import (
    MAX_BATCH_IMPORT_SIZE,
    MAX_SLUG_LENGTH,
    InputValidationError,
    validate_batch_size,
    validate_file_path,
    validate_item_type,
    validate_slug,
    validate_text,
    validate_transition,
)


@pytest.mark.parametrize("value", ["coverage-gate", "a", "a" * MAX_SLUG_LENGTH])
def test_validate_slug_accepts_kebab_case_within_limit(value: str) -> None:
    assert validate_slug(value, "feature") == value


@pytest.mark.parametrize("value", ["", "Coverage-Gate", "coverage_gate", "a" * (MAX_SLUG_LENGTH + 1)])
def test_validate_slug_rejects_empty_non_kebab_and_overlong_values(value: str) -> None:
    with pytest.raises(InputValidationError, match="feature"):
        validate_slug(value, "feature")


@pytest.mark.parametrize("value", ["specified->planned", "in_progress->done"])
def test_validate_transition_accepts_state_arrow_state(value: str) -> None:
    assert validate_transition(value) == value


@pytest.mark.parametrize("value", ["specified-planned", "specified -> planned", "specified->planned->done"])
def test_validate_transition_rejects_non_transition_shapes(value: str) -> None:
    with pytest.raises(InputValidationError, match="transition"):
        validate_transition(value)


def test_validate_text_allows_limit_and_rejects_only_overflow() -> None:
    assert validate_text("x" * 12, "note", max_length=12) == "x" * 12
    with pytest.raises(InputValidationError, match="note exceeds maximum"):
        validate_text("x" * 13, "note", max_length=12)


def test_validate_file_path_normalizes_in_root_and_rejects_traversal() -> None:
    assert validate_file_path("kitty-specs//coverage/spec.md") == "kitty-specs/coverage/spec.md"
    with pytest.raises(InputValidationError, match="must not contain"):
        validate_file_path("kitty-specs/../../outside.md")
    with pytest.raises(InputValidationError, match="must be under"):
        validate_file_path("other/spec.md")
    with pytest.raises(InputValidationError, match="must not be empty"):
        validate_file_path("")


def test_validate_batch_and_item_type_enforce_boundaries() -> None:
    items = list(range(MAX_BATCH_IMPORT_SIZE))
    assert validate_batch_size(items) is items
    with pytest.raises(InputValidationError, match="batch size"):
        validate_batch_size(items + [MAX_BATCH_IMPORT_SIZE])
    assert validate_item_type("bug") == "bug"
    with pytest.raises(InputValidationError, match="invalid item_type"):
        validate_item_type("incident")
