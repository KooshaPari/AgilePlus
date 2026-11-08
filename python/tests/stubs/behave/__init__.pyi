"""Type stubs for the `behave` BDD framework (v0.11 backlog Lane F3).

Mirrors pheno-harness/Pheno-HexaKit F1/F2 stubs.
"""

from __future__ import annotations

from typing import Any, Callable, TypeVar

F = TypeVar("F", bound=Callable[..., Any])


class Context:
    """Stub for behave.runner.Context."""

    def __getattr__(self, name: str) -> Any: ...
    def __setattr__(self, name: str, value: Any) -> None: ...


def given(step_text: str, **kwargs: Any) -> Callable[[F], F]:
    def decorator(func: F) -> F:
        return func
    return decorator


def when(step_text: str, **kwargs: Any) -> Callable[[F], F]:
    def decorator(func: F) -> F:
        return func
    return decorator


def then(step_text: str, **kwargs: Any) -> Callable[[F], F]:
    def decorator(func: F) -> F:
        return func
    return decorator


def step(step_text: str, **kwargs: Any) -> Callable[[F], F]:
    def decorator(func: F) -> F:
        return func
    return decorator


__all__ = ["Context", "given", "when", "then", "step"]
