"""Collector for FR traceability data."""

from typing import Dict, List, Set


class TraceCollector:
    """Collects and reports FR traceability data from test runs."""

    def __init__(self):
        self._traces: Dict[str, List[str]] = {}
        self._test_count = 0
        self._annotated_count = 0

    def record_test(self, test_name: str, fr_ids: List[str]) -> None:
        """Record a test and its FR traces.

        Args:
            test_name: Name of the test
            fr_ids: List of FR IDs the test traces to
        """
        self._test_count += 1
        if fr_ids:
            self._annotated_count += 1
            self._traces[test_name] = fr_ids

    def get_coverage(self, expected_frs: List[str]) -> Dict[str, any]:
        """Calculate coverage against expected FRs.

        Args:
            expected_frs: List of expected FR IDs

        Returns:
            Coverage report dictionary.
        """
        covered = set()
        for fr_ids in self._traces.values():
            covered.update(fr_ids)

        expected_set = set(expected_frs)
        covered_expected = covered & expected_set
        missing = expected_set - covered

        return {
            "total_tests": self._test_count,
            "annotated_tests": self._annotated_count,
            "coverage_percent": (
                len(covered_expected) / len(expected_set) * 100 if expected_set else 0
            ),
            "covered_frs": sorted(covered_expected),
            "missing_frs": sorted(missing),
            "unique_frs": sorted(covered),
        }

    def get_report(self) -> Dict[str, List[str]]:
        """Get full trace report.

        Returns:
            Dictionary mapping test names to FR IDs.
        """
        return dict(self._traces)

    def reset(self) -> None:
        """Reset all collected data."""
        self._traces.clear()
        self._test_count = 0
        self._annotated_count = 0


# Global collector instance
_global_collector: TraceCollector = TraceCollector()


def get_collector() -> TraceCollector:
    """Get the global trace collector instance."""
    return _global_collector
