#!/usr/bin/env python3
"""Behavioural tests for the benchmark regression gate.

Run with:
    python3 -m unittest discover -s scripts/ci -p 'test_*.py'

The centrepiece is the recorded slow-runner case: the exact per-benchmark
medians that the `bench-thresholds` job measured on 2026-11-08 on a runner
~1.61x slower than the machine `.bench-baseline.json` was captured on. Under
the old flat 50 % absolute threshold that run reported 19 regressions and
failed the gate on main; the noise budget must not report any of them, while
still failing a genuine per-benchmark regression.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from check_bench_regressions import (
    DEFAULT_NOISE_BUDGET_PCT,
    evaluate,
    noise_budget_pct,
)

REPO_ROOT = Path(__file__).resolve().parents[2]
BASELINE_PATH = REPO_ROOT / ".bench-baseline.json"
EXCLUDED_PATH = REPO_ROOT / ".bench-baseline-excluded.txt"

# Medians measured by the failing `bench-thresholds` job on 2026-11-08
# (run 42444184730, head SHA 0ed90973 on main). Reconstructed from the job log
# because the job skipped its artifact upload after the gate failed.
RECORDED_SLOW_RUNNER_MEDIANS = {
    "api_list_features/list_features/10": 21619,
    "api_list_features/list_features/100": 154570,
    "append_1000_events_100_entities": 19222000,
    "append_single_event": 14899,
    "apply_1000_events_in_memory": 57851,
    "claim_engine_issue/sequential/100": 61128,
    "claim_engine_issue/sequential/1000": 670110,
    "claim_engine_issue/sequential/10000": 7819100,
    "claim_engine_lookup/lookup_existing_10k": 231070,
    "claim_engine_reap/expired_sweep/100": 80503,
    "claim_engine_reap/expired_sweep/1000": 842420,
    "claim_engine_reap/expired_sweep/10000": 9885300,
    "event_append_throughput/sequential_events/100": 7742600,
    "event_append_throughput/sequential_events/1000": 19241000,
    "event_append_throughput/sequential_events/5000": 74795000,
    "feature_state_transition": 19086,
    "full_replay_1000_events": 1513400,
    "get_feature_by_id": 6108,
    "graph_create_feature_node": 1146,
    "graph_create_owns_relationship": 2385,
    "graph_dependency_chain_query": 97,
    "graph_get_feature_node": 232,
    "graph_seed_features/seed_features/10": 12913,
    "graph_seed_features/seed_features/100": 137330,
    "graph_seed_features/seed_features/50": 69855,
    "health_check_db_ping": 975,
    "heartbeat_1000_rounds": 91756,
    "parse_fr_list_200": 13801,
    "pull_single_feature": 452,
    "push_single_feature": 783,
    "snapshot_replay_900_plus_100": 198210,
    "sync_100_features_5_conflicts": 145980,
    "sync_roundtrip_n_features/sync_n_features_no_conflicts/1": 1431,
    "sync_roundtrip_n_features/sync_n_features_no_conflicts/10": 14573,
    "sync_roundtrip_n_features/sync_n_features_no_conflicts/100": 156770,
    "trace_matrix_derivation/cold_derive/200": 165940,
    "trace_matrix_derivation/cold_derive/50": 41235,
    "trace_matrix_derivation/cold_derive/500": 408200,
}

# The 19 benchmarks the old flat-threshold gate reported on that run.
LEGACY_REGRESSIONS = {
    "append_single_event",
    "parse_fr_list_200",
    "full_replay_1000_events",
    "event_append_throughput/sequential_events/100",
    "get_feature_by_id",
    "claim_engine_issue/sequential/10000",
    "claim_engine_reap/expired_sweep/100",
    "api_list_features/list_features/10",
    "api_list_features/list_features/100",
    "claim_engine_reap/expired_sweep/1000",
    "snapshot_replay_900_plus_100",
    "claim_engine_reap/expired_sweep/10000",
    "event_append_throughput/sequential_events/1000",
    "append_1000_events_100_entities",
    "claim_engine_issue/sequential/1000",
    "claim_engine_issue/sequential/100",
    "heartbeat_1000_rounds",
    "event_append_throughput/sequential_events/5000",
    "feature_state_transition",
}


def load_baseline() -> dict:
    return json.loads(BASELINE_PATH.read_text(encoding="utf-8"))


def load_excluded() -> set[str]:
    return {
        line.strip()
        for line in EXCLUDED_PATH.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def synthetic_suite(count: int = 12, base_ns: int = 100_000) -> dict[str, int]:
    """A baseline of `count` benchmarks with distinguishable values."""
    return {f"bench_{i:02d}": base_ns * (i + 1) for i in range(count)}


class NoiseBudgetTest(unittest.TestCase):
    def test_default_budget_applies_without_a_measured_stddev(self):
        base = {"median_ns": 1000, "stddev_ns": 0}
        self.assertEqual(noise_budget_pct(base), DEFAULT_NOISE_BUDGET_PCT)

    def test_missing_stddev_key_also_falls_back_to_the_default(self):
        self.assertEqual(noise_budget_pct({"median_ns": 1000}), DEFAULT_NOISE_BUDGET_PCT)

    def test_measured_stddev_widens_the_budget(self):
        # 10 % relative dispersion -> 3 sigma -> 30 %; the default floor of
        # 40 % still wins because the stored baseline also carries
        # machine-to-machine transfer error.
        base = {"median_ns": 1000, "stddev_ns": 100}
        self.assertEqual(noise_budget_pct(base), DEFAULT_NOISE_BUDGET_PCT)
        # 20 % relative dispersion -> 60 %, above the default floor.
        noisy = {"median_ns": 1000, "stddev_ns": 200}
        self.assertAlmostEqual(noise_budget_pct(noisy), 60.0)

    def test_default_is_a_floor_for_measured_entries(self):
        # A tight measurement cannot tighten the budget below the default.
        self.assertEqual(
            noise_budget_pct({"median_ns": 1_000_000, "stddev_ns": 1}),
            DEFAULT_NOISE_BUDGET_PCT,
        )

    def test_budget_is_never_below_the_floor_or_above_the_cap(self):
        self.assertEqual(noise_budget_pct({"median_ns": 1000, "stddev_ns": 1}, 5.0), 10.0)
        self.assertEqual(noise_budget_pct({"median_ns": 1000, "stddev_ns": 10_000}), 150.0)


class RecordedSlowRunnerTest(unittest.TestCase):
    """The case that used to fail the gate on main for environmental reasons."""

    def setUp(self):
        self.baseline = load_baseline()
        self.excluded = load_excluded()

    def test_legacy_flat_threshold_reproduces_the_ci_failure(self):
        report = evaluate(
            RECORDED_SLOW_RUNNER_MEDIANS,
            self.baseline,
            exclude=self.excluded,
            max_regress=50.0,
        )
        self.assertFalse(report["ok"])
        self.assertEqual(
            {r["name"] for r in report["regressions"]},
            LEGACY_REGRESSIONS,
            "the recorded fixture must reproduce the 19 regressions CI reported",
        )

    def test_noise_budget_reports_no_regression_for_the_recorded_run(self):
        report = evaluate(
            RECORDED_SLOW_RUNNER_MEDIANS,
            self.baseline,
            exclude=self.excluded,
        )
        self.assertTrue(report["ok"], report["regressions"])
        self.assertEqual(report["regressions"], [])
        self.assertTrue(report["common_mode_applied"])
        # The runner really was slower than the baseline machine.
        self.assertGreater(report["runner_factor"], 1.5)
        self.assertLess(report["runner_factor"], 1.7)
        # Every benchmark is still inside its own noise budget.
        self.assertLess(report["worst_deviation_pct"], DEFAULT_NOISE_BUDGET_PCT)

    def test_a_targeted_regression_on_the_slow_runner_is_still_caught(self):
        current = dict(RECORDED_SLOW_RUNNER_MEDIANS)
        victim = "full_replay_1000_events"
        current[victim] = int(current[victim] * 1.60)
        report = evaluate(current, self.baseline, exclude=self.excluded)
        self.assertFalse(report["ok"])
        self.assertEqual([r["name"] for r in report["regressions"]], [victim])
        self.assertGreater(report["regressions"][0]["runner_normalized_delta_pct"], 40.0)


class SyntheticSuiteTest(unittest.TestCase):
    def setUp(self):
        self.baseline = {
            name: {"median_ns": ns, "stddev_ns": 0} for name, ns in synthetic_suite().items()
        }

    def test_uniform_slowdown_is_absorbed_as_common_mode(self):
        current = {name: int(ns * 1.7) for name, ns in synthetic_suite().items()}
        report = evaluate(current, self.baseline)
        self.assertTrue(report["ok"], report["regressions"])
        self.assertAlmostEqual(report["runner_factor"], 1.7, places=6)

    def test_one_slow_benchmark_is_reported(self):
        current = {name: int(ns * 1.0) for name, ns in synthetic_suite().items()}
        current["bench_03"] = int(current["bench_03"] * 1.60)
        report = evaluate(current, self.baseline)
        self.assertFalse(report["ok"])
        self.assertEqual([r["name"] for r in report["regressions"]], ["bench_03"])

    def test_small_speedups_are_not_reported(self):
        current = {name: int(ns * 1.05) for name, ns in synthetic_suite().items()}
        report = evaluate(current, self.baseline)
        self.assertTrue(report["ok"], report["regressions"])

    def test_global_slowdown_is_normalised_but_surfaced_by_the_runner_factor(self):
        # KNOWN LIMITATION: a regression that slows more than half the suite
        # equally is indistinguishable from a slower runner. The gate must not
        # silently swallow it - the runner factor has to show it.
        current = {name: int(ns * 1.5) for name, ns in synthetic_suite().items()}
        report = evaluate(current, self.baseline)
        self.assertTrue(report["ok"])
        self.assertAlmostEqual(report["runner_factor"], 1.5, places=6)

    def test_excluded_benchmarks_are_not_compared(self):
        baseline = dict(self.baseline)
        baseline["excluded_bench"] = {"median_ns": 1000, "stddev_ns": 0}
        current = {name: int(ns * 3.0) for name, ns in synthetic_suite().items()}
        current["excluded_bench"] = 3_000_000
        report = evaluate(current, baseline, exclude={"excluded_bench"})
        self.assertTrue(report["ok"], report["regressions"])
        self.assertEqual(report["excluded"], 1)

    def test_too_few_benchmarks_skips_common_mode_normalisation(self):
        baseline = {
            "a": {"median_ns": 1000, "stddev_ns": 0},
            "b": {"median_ns": 1000, "stddev_ns": 0},
        }
        report = evaluate({"a": 2000, "b": 2000}, baseline)
        self.assertFalse(report["common_mode_applied"])
        self.assertEqual(report["runner_factor"], 1.0)
        self.assertFalse(report["ok"])

    def test_max_regress_override_restores_a_flat_threshold(self):
        current = {name: int(ns * 1.2) for name, ns in synthetic_suite().items()}
        report = evaluate(current, self.baseline, max_regress=10.0)
        self.assertFalse(report["ok"])
        self.assertEqual(len(report["regressions"]), len(current))

    def test_bookkeeping_keys_in_the_baseline_are_ignored(self):
        baseline = dict(self.baseline)
        baseline["_meta"] = {"noise_budget_pct": 40.0, "note": "not a benchmark"}
        current = {name: int(ns * 1.0) for name, ns in synthetic_suite().items()}
        report = evaluate(current, baseline)
        self.assertTrue(report["ok"], report["regressions"])
        self.assertEqual(report["baseline_size"], len(self.baseline))


if __name__ == "__main__":
    unittest.main()
