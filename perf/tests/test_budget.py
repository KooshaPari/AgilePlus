"""Contract tests for the JSON-driven k6 budget checker."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

from scripts.perf.check_budget import BudgetError, check_budget


BUDGET = {
    "request_error_rate": {"metric": "request_errors", "stat": "rate", "max": 0.01},
    "grpc_req_duration_p95_ms": {
        "metric": "grpc_req_duration",
        "stat": "p(95)",
        "max": 250.0,
    },
    "core_crash_count": {"metric": "core_crash_count", "stat": "count", "max": 0},
}


def summary(*, error_rate: float = 0.0, p95: float = 80.0, crashes: int = 0) -> dict:
    return {
        "metrics": {
            "request_errors": {"values": {"rate": error_rate}},
            "grpc_req_duration": {"values": {"p(95)": p95}},
            "core_crash_count": {"values": {"count": crashes}},
        }
    }


def test_all_metrics_within_budget_pass() -> None:
    assert check_budget(summary(), BUDGET) == []


def test_empty_budget_fails_closed() -> None:
    with pytest.raises(BudgetError, match="missing required budget: request_error_rate"):
        check_budget(summary(), {})


def test_missing_required_budget_rule_fails_closed() -> None:
    budget = dict(BUDGET)
    del budget["core_crash_count"]

    with pytest.raises(BudgetError, match="missing required budget: core_crash_count"):
        check_budget(summary(), budget)


def test_required_budget_cannot_be_renamed_to_an_unrelated_metric() -> None:
    budget = {name: dict(rule) for name, rule in BUDGET.items()}
    budget["request_error_rate"]["metric"] = "http_req_failed"

    with pytest.raises(
        BudgetError,
        match=r"request_error_rate must target request_errors\.rate",
    ):
        check_budget(summary(), budget)


def test_missing_metric_fails_closed() -> None:
    document = summary()
    del document["metrics"]["core_crash_count"]

    with pytest.raises(BudgetError, match="missing metric: core_crash_count"):
        check_budget(document, BUDGET)


def test_malformed_summary_fails_closed() -> None:
    with pytest.raises(BudgetError, match="summary.metrics must be an object"):
        check_budget({"metrics": []}, BUDGET)


def test_p95_breach_reports_observed_and_limit() -> None:
    failures = check_budget(summary(p95=250.01), BUDGET)

    assert failures == [
        "grpc_req_duration_p95_ms: observed 250.01 exceeds maximum 250.0"
    ]


def test_error_rate_breach_reports_observed_and_limit() -> None:
    failures = check_budget(summary(error_rate=0.02), BUDGET)

    assert failures == ["request_error_rate: observed 0.02 exceeds maximum 0.01"]


def test_nonzero_crash_count_fails() -> None:
    failures = check_budget(summary(crashes=1), BUDGET)

    assert failures == ["core_crash_count: observed 1.0 exceeds maximum 0.0"]


@pytest.mark.parametrize("error_rate", [-0.01, 1.01])
def test_error_rate_must_be_between_zero_and_one(error_rate: float) -> None:
    with pytest.raises(BudgetError, match="request_errors.rate must be between 0 and 1"):
        check_budget(summary(error_rate=error_rate), BUDGET)


def test_latency_must_not_be_negative() -> None:
    with pytest.raises(BudgetError, match=r"grpc_req_duration.p\(95\) must be non-negative"):
        check_budget(summary(p95=-0.01), BUDGET)


@pytest.mark.parametrize("crashes", [-1, 0.5])
def test_crash_count_must_be_a_nonnegative_integer(crashes: float) -> None:
    with pytest.raises(
        BudgetError,
        match="core_crash_count.count must be a non-negative integer",
    ):
        check_budget(summary(crashes=crashes), BUDGET)


def test_cli_reads_summary_and_budget_json(tmp_path: Path) -> None:
    summary_path = tmp_path / "summary.json"
    budget_path = tmp_path / "budget.json"
    result_path = tmp_path / "budget-result.json"
    summary_path.write_text(json.dumps(summary()), encoding="utf-8")
    budget_path.write_text(json.dumps(BUDGET), encoding="utf-8")

    result = subprocess.run(
        [
            sys.executable,
            "scripts/perf/check_budget.py",
            "--summary",
            str(summary_path),
            "--budget-file",
            str(budget_path),
            "--result",
            str(result_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    assert json.loads(result_path.read_text(encoding="utf-8")) == {
        "failures": [],
        "passed": True,
    }


def run_cli(
    tmp_path: Path,
    summary_document: object,
    *,
    budget_document: object = BUDGET,
    summary_exists: bool = True,
) -> tuple[subprocess.CompletedProcess[str], Path]:
    summary_path = tmp_path / "summary.json"
    budget_path = tmp_path / "budget.json"
    result_path = tmp_path / "budget-result.json"
    if summary_exists:
        summary_path.write_text(json.dumps(summary_document), encoding="utf-8")
    budget_path.write_text(json.dumps(budget_document), encoding="utf-8")
    result = subprocess.run(  # noqa: S603
        [
            sys.executable,
            "scripts/perf/check_budget.py",
            "--summary",
            str(summary_path),
            "--budget-file",
            str(budget_path),
            "--result",
            str(result_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    return result, result_path


def test_cli_breach_exits_one_and_writes_failure_result(tmp_path: Path) -> None:
    result, result_path = run_cli(tmp_path, summary(p95=251.0))

    assert result.returncode == 1
    assert "performance budget failed:" in result.stderr
    assert "grpc_req_duration_p95_ms" in result.stderr
    assert json.loads(result_path.read_text(encoding="utf-8")) == {
        "failures": [
            "grpc_req_duration_p95_ms: observed 251.0 exceeds maximum 250.0"
        ],
        "passed": False,
    }


def test_cli_malformed_summary_exits_two_and_writes_error_result(tmp_path: Path) -> None:
    result, result_path = run_cli(tmp_path, {"metrics": []})

    assert result.returncode == 2
    assert result.stderr.startswith("performance budget input error: ")
    assert json.loads(result_path.read_text(encoding="utf-8")) == {
        "error": "summary.metrics must be an object",
        "failures": [],
        "passed": False,
    }


def test_cli_missing_summary_exits_two_and_writes_error_result(tmp_path: Path) -> None:
    result, result_path = run_cli(tmp_path, {}, summary_exists=False)

    assert result.returncode == 2
    assert "performance budget input error: cannot read" in result.stderr
    payload = json.loads(result_path.read_text(encoding="utf-8"))
    assert payload["passed"] is False
    assert payload["failures"] == []
    assert payload["error"].startswith("cannot read ")
