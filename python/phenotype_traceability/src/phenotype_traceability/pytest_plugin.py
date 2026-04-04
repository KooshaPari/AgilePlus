"""pytest plugin for FR traceability."""

import pytest
from typing import List, Optional

from .collector import get_collector


def pytest_configure(config):
    """Configure the plugin."""
    config.addinivalue_line(
        "markers",
        "traces_to(fr_id): mark test as tracing to Feature Requirement",
    )


def pytest_collection_modifyitems(config, items):
    """Process collected test items."""
    for item in items:
        # Check for traces_to marker
        marker = item.get_closest_marker("traces_to")
        if marker:
            fr_ids = marker.args
            item._fr_ids = fr_ids
            # Add to docstring
            if item.obj.__doc__:
                traces_doc = f"Traces to: {', '.join(fr_ids)}"
                if traces_doc not in item.obj.__doc__:
                    item.obj.__doc__ += f"\n\n{traces_doc}"


@pytest.hookimpl(tryfirst=True)
def pytest_runtest_setup(item):
    """Setup hook to record FR traces."""
    fr_ids: List[str] = []

    # Check for traces_to marker
    marker = item.get_closest_marker("traces_to")
    if marker:
        fr_ids = list(marker.args)

    # Check for _fr_ids attribute
    elif hasattr(item.obj, "_fr_ids"):
        fr_ids = list(item.obj._fr_ids)

    # Record in collector
    if fr_ids:
        collector = get_collector()
        collector.record_test(item.nodeid, fr_ids)


def pytest_terminal_summary(terminalreporter, exitstatus, config):
    """Print traceability summary."""
    collector = get_collector()
    report = collector.get_coverage([])  # No expected FRs for summary

    terminalreporter.write_sep("=", "FR Traceability Summary")
    terminalreporter.write_line(f"Total tests: {report['total_tests']}")
    terminalreporter.write_line(f"Annotated tests: {report['annotated_tests']}")
    terminalreporter.write_line(f"Unique FRs covered: {len(report['unique_frs'])}")

    if report["unique_frs"]:
        terminalreporter.write_line("\nFR IDs:")
        for fr in report["unique_frs"][:10]:  # Show first 10
            terminalreporter.write_line(f"  - {fr}")
        if len(report["unique_frs"]) > 10:
            terminalreporter.write_line(f"  ... and {len(report['unique_frs']) - 10} more")
