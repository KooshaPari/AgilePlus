"""Unit tests for MCP input validation boundaries."""

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


def test_validation_accepts_valid_boundaries() -> None:
    assert validate_slug("feature-1") == "feature-1"
    assert validate_transition("specified->planned") == "specified->planned"
    assert validate_text("ok", max_length=2) == "ok"
    assert validate_file_path("kitty-specs/feature/spec.md") == "kitty-specs/feature/spec.md"
    assert validate_batch_size([1, 2], max_size=2) == [1, 2]
    assert validate_item_type("task") == "task"
    assert validate_item_type("") == ""


@pytest.mark.parametrize("value", ["", "Bad Slug", "-starts-with-hyphen", "x" * 129])
def test_validate_slug_rejects_invalid_values(value: str) -> None:
    with pytest.raises(InputValidationError, match="slug"):
        validate_slug(value)


@pytest.mark.parametrize("value", ["planned", "planned-implementing", "->planned"])
def test_validate_transition_rejects_invalid_values(value: str) -> None:
    with pytest.raises(InputValidationError, match="transition"):
        validate_transition(value)


def test_validation_rejects_unsafe_paths_and_oversized_values() -> None:
    with pytest.raises(InputValidationError, match="exceeds"):
        validate_text("too long", max_length=3)
    with pytest.raises(InputValidationError, match="must not contain"):
        validate_file_path("../secrets.txt")
    with pytest.raises(InputValidationError, match="under one of"):
        validate_file_path("docs/spec.md")
    with pytest.raises(InputValidationError, match="batch size"):
        validate_batch_size([1, 2], max_size=1)
    with pytest.raises(InputValidationError, match="invalid item_type"):
        validate_item_type("incident")
