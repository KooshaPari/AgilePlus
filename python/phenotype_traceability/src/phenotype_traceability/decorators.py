"""Decorators for FR traceability."""

import functools
import re
from typing import Callable, Optional, TypeVar

F = TypeVar("F", bound=Callable)


def _validate_fr_id(fr_id: str) -> bool:
    """Validate FR ID format: FR-XXXX-NNN or FR-XXXX-NNN-YYY."""
    pattern = r"^FR-[A-Z][A-Z0-9]*-\d{3,}(-[A-Z0-9]+)?$"
    return bool(re.match(pattern, fr_id))


def traces_to(*fr_ids: str) -> Callable[[F], F]:
    """Decorator to mark a test as tracing to Feature Requirement(s).

    Args:
        *fr_ids: One or more FR IDs (e.g., "FR-EXAMPLE-001")

    Returns:
        Decorated function with traceability metadata.

    Example:
        @traces_to("FR-EXAMPLE-001")
        def test_feature():
            assert True

        @traces_to("FR-EXAMPLE-001", "FR-EXAMPLE-002")
        def test_multiple_frs():
            assert True
    """
    for fr_id in fr_ids:
        if not _validate_fr_id(fr_id):
            raise ValueError(f"Invalid FR ID format: {fr_id}")

    def decorator(func: F) -> F:
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            # Store FR IDs on the function
            wrapper._fr_ids = fr_ids  # type: ignore
            return func(*args, **kwargs)

        # Also store on the original function for introspection
        func._fr_ids = fr_ids  # type: ignore
        return wrapper  # type: ignore

    return decorator


def describe_fr(fr_id: str, description: Optional[str] = None) -> Callable[[F], F]:
    """Decorator to describe a test group for a specific FR.

    Args:
        fr_id: The FR ID
        description: Optional description

    Returns:
        Decorated function.

    Example:
        @describe_fr("FR-EXAMPLE-001", "User authentication")
        class TestAuth:
            def test_login(self):
                pass
    """
    if not _validate_fr_id(fr_id):
        raise ValueError(f"Invalid FR ID format: {fr_id}")

    def decorator(func: F) -> F:
        func._fr_id = fr_id  # type: ignore
        func._fr_description = description  # type: ignore
        return func

    return decorator
