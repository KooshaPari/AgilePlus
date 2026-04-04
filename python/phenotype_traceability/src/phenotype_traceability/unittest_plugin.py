"""unittest plugin for FR traceability."""

import unittest
from typing import Any, Callable

from .decorators import traces_to, describe_fr
from .collector import get_collector

__all__ = ["traces_to", "describe_fr", "TraceabilityTestCase"]


class TraceabilityTestCase(unittest.TestCase):
    """Base TestCase with traceability support."""

    def run(self, result: Any = None) -> Any:
        """Run test and record FR traces."""
        test_method = getattr(self, self._testMethodName)

        # Check for FR annotations
        fr_ids = getattr(test_method, "_fr_ids", [])
        if fr_ids:
            collector = get_collector()
            collector.record_test(
                f"{self.__class__.__name__}.{self._testMethodName}", list(fr_ids)
            )

        return super().run(result)
