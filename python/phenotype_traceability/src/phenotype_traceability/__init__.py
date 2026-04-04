"""Phenotype Traceability - FR tracking for Python tests.

This package provides decorators and plugins for marking tests as
tracing to Feature Requirements (FRs).

Example:
    import pytest

    @pytest.mark.traces_to("FR-EXAMPLE-001")
    def test_feature():
        assert True
"""

__version__ = "0.1.0"

from .decorators import traces_to, describe_fr
from .collector import TraceCollector, get_collector

__all__ = [
    "traces_to",
    "describe_fr",
    "TraceCollector",
    "get_collector",
]
