#!/usr/bin/env python3
"""Validate a k6 summary against repository-owned performance budgets."""

from __future__ import annotations

import argparse
import json
import math
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import TypeAlias

JsonValue: TypeAlias = None | bool | int | float | str | list["JsonValue"] | dict[str, "JsonValue"]

REQUIRED_BUDGETS = {
    "request_error_rate": ("request_errors", "rate"),
    "grpc_req_duration_p95_ms": ("grpc_req_duration", "p(95)"),
    "core_crash_count": ("core_crash_count", "count"),
}


class BudgetError(ValueError):
    """Raised when a summary or budget document violates its schema."""


def _object(value: JsonValue, label: str) -> Mapping[str, JsonValue]:
    if not isinstance(value, dict):
        raise BudgetError(f"{label} must be an object")
    return value


def _text(value: JsonValue, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise BudgetError(f"{label} must be a non-empty string")
    return value


def _number(value: JsonValue, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise BudgetError(f"{label} must be a finite number")
    resolved = float(value)
    if not math.isfinite(resolved):
        raise BudgetError(f"{label} must be a finite number")
    return resolved


def _validate_required_budgets(configured: Mapping[str, JsonValue]) -> None:
    for budget_name, (required_metric, required_stat) in REQUIRED_BUDGETS.items():
        if budget_name not in configured:
            raise BudgetError(f"missing required budget: {budget_name}")
        rule = _object(configured[budget_name], f"budget.{budget_name}")
        metric_name = _text(rule.get("metric"), f"budget.{budget_name}.metric")
        stat_name = _text(rule.get("stat"), f"budget.{budget_name}.stat")
        if (metric_name, stat_name) != (required_metric, required_stat):
            raise BudgetError(
                f"{budget_name} must target {required_metric}.{required_stat}"
            )


def _validate_metric_domain(metric_name: str, stat_name: str, value: float) -> None:
    identity = f"{metric_name}.{stat_name}"
    if (metric_name, stat_name) == ("request_errors", "rate") and not 0 <= value <= 1:
        raise BudgetError(f"{identity} must be between 0 and 1")
    if (metric_name, stat_name) == ("grpc_req_duration", "p(95)") and value < 0:
        raise BudgetError(f"{identity} must be non-negative")
    if (metric_name, stat_name) == ("core_crash_count", "count") and (
        value < 0 or not value.is_integer()
    ):
        raise BudgetError(f"{identity} must be a non-negative integer")


def check_budget(summary: JsonValue, budget: JsonValue) -> list[str]:
    """Return deterministic threshold failures; malformed input raises BudgetError."""
    metrics = _object(_object(summary, "summary").get("metrics"), "summary.metrics")
    configured = _object(budget, "budget")
    _validate_required_budgets(configured)
    failures: list[str] = []

    for budget_name, raw_rule in configured.items():
        rule = _object(raw_rule, f"budget.{budget_name}")
        metric_name = _text(rule.get("metric"), f"budget.{budget_name}.metric")
        stat_name = _text(rule.get("stat"), f"budget.{budget_name}.stat")
        maximum = _number(rule.get("max"), f"budget.{budget_name}.max")
        _validate_metric_domain(metric_name, stat_name, maximum)
        if metric_name not in metrics:
            raise BudgetError(f"missing metric: {metric_name}")
        metric = _object(metrics[metric_name], f"summary.metrics.{metric_name}")
        values = _object(metric.get("values"), f"summary.metrics.{metric_name}.values")
        if stat_name not in values:
            raise BudgetError(f"missing statistic: {metric_name}.{stat_name}")
        observed = _number(values[stat_name], f"summary.metrics.{metric_name}.values.{stat_name}")
        _validate_metric_domain(metric_name, stat_name, observed)
        if observed > maximum:
            failures.append(
                f"{budget_name}: observed {observed} exceeds maximum {maximum}"
            )

    return failures


def _read_json(path: Path) -> JsonValue:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise BudgetError(f"cannot read {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise BudgetError(f"invalid JSON in {path}: {exc.msg}") from exc


def _write_result(path: Path, payload: Mapping[str, JsonValue]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--budget-file", type=Path, required=True)
    parser.add_argument("--result", type=Path, default=Path(".perf-reports/budget-result.json"))
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        failures = check_budget(_read_json(args.summary), _read_json(args.budget_file))
    except BudgetError as exc:
        _write_result(args.result, {"error": str(exc), "failures": [], "passed": False})
        sys.stderr.write(f"performance budget input error: {exc}\n")
        return 2

    result: dict[str, JsonValue] = {"failures": failures, "passed": not failures}
    _write_result(args.result, result)
    if failures:
        sys.stderr.write("performance budget failed:\n" + "\n".join(failures) + "\n")
        return 1
    sys.stdout.write("performance budget passed\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
