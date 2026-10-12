"""Harness correctness tests. The fixture server is not product performance evidence."""

import http.server
import importlib.util
import io
import json
import tempfile
import threading
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import ClassVar

spec = importlib.util.spec_from_file_location(
    "http_benchmark", Path(__file__).with_name("http_benchmark.py")
)
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


class FixtureHandler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    states: ClassVar[dict[int, str]] = {}
    auth_key = "fixture-key"

    def log_message(self, *_):
        pass

    def send(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path == "/health":
            self.send(200, {"status": "healthy", "service": "agileplus-api"})
        elif self.headers.get("Authorization") != f"Bearer {self.auth_key}":
            self.send(401, {"error": "unauthorized"})
        elif self.path == "/api/v1/features":
            self.send(
                200,
                [
                    {"slug": f"perf-feature-{i:03}", "state": self.states[i]}
                    for i in range(100)
                ],
            )
        else:
            index = int(self.path.rsplit("-", 1)[1])
            self.send(
                200, {"slug": f"perf-feature-{index:03}", "state": self.states[index]}
            )

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        if self.path == "/api/v1/features":
            index = int(payload["title"].split()[-1])
            self.states[index] = "created"
            self.send(201, {"slug": f"perf-feature-{index:03}", "state": "created"})
            return
        if payload["target_state"] != "specified":
            self.send(400, {})
            return
        index = int(self.path.split("/")[-2].rsplit("-", 1)[1])
        prior = self.states[index]
        self.states[index] = "specified"
        self.send(200, {"from_state": prior, "to_state": "specified"})


class HarnessTests(unittest.TestCase):
    def test_percentiles_require_measurements_and_use_nearest_rank(self):
        with self.assertRaises(ValueError):
            benchmark.percentile([], 0.95)
        self.assertEqual(benchmark.percentile(list(range(1, 101)), 0.95), 95)
        self.assertEqual(benchmark.percentile(list(range(1, 101)), 0.99), 99)

    def test_budget_exceedance_is_reported_as_warning_and_incomplete_samples_fail(self):
        row = benchmark.distribution([{"elapsed_ms": 500}] * 100, "list")
        self.assertEqual(row["gate_type"], "warning")
        self.assertEqual(row["warnings"], ["p95_ms", "p99_ms"])
        with self.assertRaises(ValueError):
            benchmark.distribution([{"elapsed_ms": 1}] * 99, "list")

    def test_real_http_fixture_checks_responses_and_transitions_for_all_ten_clients(
        self,
    ):
        FixtureHandler.states = dict.fromkeys(range(100), "created")
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), FixtureHandler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            port = server.server_address[1]
            for name in ["health", "list", "detail", "transition"]:
                samples = benchmark.concurrent_measurements(port, "fixture-key", name)
                self.assertEqual(len(samples), 100)
                self.assertTrue(all(sample["status"] == 200 for sample in samples))
            self.assertEqual(set(FixtureHandler.states.values()), {"specified"})
            client = benchmark.Client(port, "bad-key")
            try:
                with self.assertRaises(RuntimeError):
                    benchmark.measure(client, "list")
            finally:
                client.close()
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    def test_invalid_response_and_missing_binary_fail_instead_of_publishing_success(
        self,
    ):
        with self.assertRaises(RuntimeError):
            benchmark.validate_response("list", [])
        with self.assertRaises(RuntimeError):
            benchmark.validate_response(
                "health", {"status": "healthy", "service": "stub"}
            )
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with redirect_stdout(io.StringIO()):
                self.assertEqual(benchmark.run(root / "missing", root / "reports"), 1)
            report = json.loads((root / "reports/http-results.json").read_text())
            self.assertEqual(report["status"], "failed")
            self.assertEqual(report["distributions"], {})

    def test_spawned_fixture_runs_complete_harness_and_stops_its_process(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            executable = root / "fixture-api"
            executable.write_text(
                "#!/usr/bin/env python3\nimport os, sys\n"
                + f"sys.path.insert(0, {str(Path(__file__).parent.resolve())!r})\n"
                + "from test_http_benchmark import FixtureHandler\n"
                + "from http.server import ThreadingHTTPServer\n"
                + "FixtureHandler.auth_key = os.environ['AGILEPLUS_API_KEY']\n"
                + "ThreadingHTTPServer(('127.0.0.1', int(os.environ['AGILEPLUS_API_PORT'])), "
                + "FixtureHandler).serve_forever()\n"
            )
            executable.chmod(0o700)
            output = io.StringIO()
            with redirect_stdout(output):
                self.assertEqual(benchmark.run(executable, root / "reports"), 0)
            metrics_line = next(
                line
                for line in output.getvalue().splitlines()
                if line.startswith("HTTP_MEASUREMENTS ")
            )
            metrics = json.loads(metrics_line.removeprefix("HTTP_MEASUREMENTS "))
            self.assertEqual(metrics["server_binary"], "fixture-api")
            self.assertEqual(metrics["post_transition_verified"], 100)
            report = json.loads((root / "reports/http-results.json").read_text())
            self.assertEqual(report["status"], "completed")
            self.assertEqual(len(report["after_state_change"]), 100)
            self.assertTrue(report["cleanup"]["api_process_stopped"])
            self.assertTrue(report["cleanup"]["temporary_runtime_removed"])


if __name__ == "__main__":
    unittest.main()
