"""Unit tests for MCP boundary validation."""

from __future__ import annotations

import pytest

from agileplus_mcp.validation import (
    InputValidationError,
    validate_batch_size,
    validate_file_path,
    validate_item_type,
    validate_slug,
    validate_text,
    validate_transition,
)


@pytest.mark.parametrize("value", ["feature-1", "a", "with-123"])
def test_validate_slug_accepts_kebab_case(value: str) -> None:
    assert validate_slug(value) == value


@pytest.mark.parametrize("value", ["", "Uppercase", "has_underscore", "-leading-hyphen"])
def test_validate_slug_rejects_invalid_values(value: str) -> None:
    with pytest.raises(InputValidationError):
        validate_slug(value)


def test_validate_transition_accepts_and_rejects_expected_shape() -> None:
    assert validate_transition("specified->planned") == "specified->planned"
    with pytest.raises(InputValidationError):
        validate_transition("specified-planned")


def test_validate_text_and_batch_size_enforce_limits() -> None:
    assert validate_text("ok", max_length=2) == "ok"
    assert validate_batch_size([1, 2], max_size=2) == [1, 2]
    with pytest.raises(InputValidationError):
        validate_text("too long", max_length=2)
    with pytest.raises(InputValidationError):
        validate_batch_size([1, 2, 3], max_size=2)


def test_validate_file_path_blocks_traversal_and_unapproved_roots() -> None:
    assert validate_file_path("kitty-specs/feature/spec.md") == "kitty-specs/feature/spec.md"
    with pytest.raises(InputValidationError):
        validate_file_path("kitty-specs/../secret.txt")
    with pytest.raises(InputValidationError):
        validate_file_path("outside/spec.md")


def test_validate_item_type_enforces_allowlist() -> None:
    assert validate_item_type("story") == "story"
    assert validate_item_type("") == ""
    with pytest.raises(InputValidationError):
        validate_item_type("release")
