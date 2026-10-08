#!/usr/bin/env python3
"""T118: real loopback HTTP measurements; local SQLite, no production target."""
import argparse
import concurrent.futures
import hashlib
import http.client
import json
import math
import os
from pathlib import Path
import platform
import secrets
import signal
import socket
import subprocess
import tempfile
import threading
import time

SOURCE = "kitty-specs/003-agileplus-platform-completion/tasks/WP21-performance-benchmarks.md#t118"
BUDGETS = {
    "list": {"p95_ms": 100, "p99_ms": 200},
    "detail": {"p95_ms": 50, "p99_ms": 100},
    "transition": {"p95_ms": 100, "p99_ms": 200},
    "health": {"p95_ms": 10, "p99_ms": 20},
}
CLIENTS = 10
FEATURES = 100


def percentile(values, fraction):
    """Nearest-rank empirical percentile; no interpolation or fake samples."""
    if not values:
        raise ValueError("cannot calculate percentile without measurements")
    return sorted(values)[math.ceil(len(values) * fraction) - 1]


def distribution(samples, name):
    values = [sample["elapsed_ms"] for sample in samples]
    if len(values) != FEATURES:
        raise ValueError(f"{name}: expected {FEATURES} measurements, got {len(values)}")
    result = {"samples": len(values), "p50_ms": percentile(values, .50),
              "p95_ms": percentile(values, .95), "p99_ms": percentile(values, .99),
              "max_ms": max(values), "budget": BUDGETS[name], "gate_type": "warning"}
    result["warnings"] = [metric for metric, budget in BUDGETS[name].items()
                          if result[metric] >= budget]
    return result


class Client:
    def __init__(self, port, key):
        self.connection = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
        self.key = key

    def close(self):
        self.connection.close()

    def request(self, method, path, payload=None, expected=200):
        headers = {"Authorization": f"Bearer {self.key}"}
        body = None
        if payload is not None:
            body = json.dumps(payload)
            headers["Content-Type"] = "application/json"
        start = time.perf_counter_ns()
        self.connection.request(method, path, body=body, headers=headers)
        response = self.connection.getresponse()
        data = response.read()
        elapsed_ms = (time.perf_counter_ns() - start) / 1e6
        if response.status != expected:
            raise RuntimeError(f"{method} {path}: expected HTTP {expected}, got {response.status}")
        return json.loads(data), {"method": method, "path": path,
                                  "status": response.status, "elapsed_ms": elapsed_ms}


def validate_response(name, body, index=None):
    if name == "list":
        if len(body) != FEATURES or {row["slug"] for row in body} != {
                f"perf-feature-{i:03}" for i in range(FEATURES)}:
            raise RuntimeError("feature list does not match the seeded 100 features")
    elif name == "health":
        if body.get("status") != "healthy" or body.get("service") != "agileplus-api":
            raise RuntimeError("health did not identify the actual healthy AgilePlus API")
    elif name == "detail":
        if body.get("slug") != f"perf-feature-{index:03}":
            raise RuntimeError("feature detail returned the wrong feature")
    elif name == "transition":
        if body.get("from_state") != "created" or body.get("to_state") != "specified":
            raise RuntimeError("transition did not execute created -> specified")


def measure(client, name, index=0):
    slug = f"perf-feature-{index:03}"
    if name == "list":
        body, sample = client.request("GET", "/api/v1/features/")
    elif name == "detail":
        body, sample = client.request("GET", f"/api/v1/features/{slug}")
    elif name == "transition":
        body, sample = client.request("POST", f"/api/v1/features/{slug}/transition",
                                      {"target_state": "specified"})
    else:
        body, sample = client.request("GET", "/health")
    validate_response(name, body, index)
    return sample


def concurrent_measurements(port, key, name):
    barrier = threading.Barrier(CLIENTS)

    def worker(worker_id):
        client = Client(port, key)
        try:
            barrier.wait(timeout=15)
            return [measure(client, name, i)
                    for i in range(worker_id, FEATURES, CLIENTS)]
        finally:
            client.close()

    with concurrent.futures.ThreadPoolExecutor(max_workers=CLIENTS) as executor:
        groups = list(executor.map(worker, range(CLIENTS)))
    return [sample for group in groups for sample in group]


def wait_ready(process, port, key):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"API exited before readiness: {process.returncode}")
        client = Client(port, key)
        try:
            body, _ = client.request("GET", "/health")
            validate_response("health", body)
            return
        except (OSError, http.client.HTTPException):
            time.sleep(.1)
        finally:
            client.close()
    raise RuntimeError("API did not become ready within 30 seconds")


def write_reports(directory, report):
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "http-results.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = ["### AgilePlus T118 loopback HTTP measurements", "",
             "Real release API + temporary SQLite; 100 features, 10 simultaneous closed-loop clients.",
             "Request time includes connection setup when needed and the complete HTTP response body.",
             "Targets are warnings; protocol, data, auth and incomplete measurements fail the job.", "",
             "| Endpoint | Samples | p50 ms | p95 ms | p99 ms | Target status |",
             "|---|---:|---:|---:|---:|---|"]
    for name, row in report.get("distributions", {}).items():
        lines.append(f"| {name} | {row['samples']} | {row['p50_ms']:.3f} | "
                     f"{row['p95_ms']:.3f} | {row['p99_ms']:.3f} | "
                     f"{'Warning' if row['warnings'] else 'Within targets'} |")
    lines += ["", f"Execution status: **{report['status']}**.",
              "Initial requests follow seeding; they are not an OS cold-cache measurement.",
              "This measures bounded local HTTP behavior, not production capacity, external services, "
              "long-duration stress, adoption or accepted end-user value."]
    if report.get("error"):
        lines += ["", f"Execution error: {report['error']}"]
    text = "\n".join(lines) + "\n"
    (directory / "summary.md").write_text(text)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
            summary.write(text)


def run(binary, directory):
    binary = binary.resolve()
    directory = directory.resolve()
    report = {"schema_version": 1, "source": SOURCE, "status": "failed",
              "source_commit": subprocess.run(["git", "rev-parse", "HEAD"],
                                              capture_output=True, text=True).stdout.strip(),
              "workload": {"features": FEATURES, "clients": CLIENTS,
                           "requests_per_client_per_endpoint": FEATURES // CLIENTS,
                           "model": "bounded closed-loop; barrier start per endpoint"},
              "environment": {"platform": platform.platform(), "python": platform.python_version()},
              "distributions": {}, "samples": {}, "initial_requests": {},
              "after_state_change": [], "limitations": [
                  "Readiness and seeding precede initial measured reads; not cold OS cache.",
                  "No external NATS/Redis/Neo4j/MinIO, production traffic or capacity claim."]}
    process = None
    directory.mkdir(parents=True, exist_ok=True)
    try:
        if not binary.is_file():
            raise RuntimeError(f"API binary does not exist: {binary}")
        report["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
        with tempfile.TemporaryDirectory(prefix="agileplus-http-perf-") as temporary:
            runtime = Path(temporary)
            subprocess.run(["git", "init", "-q", str(runtime)], check=True)
            with socket.socket() as listener:
                listener.bind(("127.0.0.1", 0))
                port = listener.getsockname()[1]
            key = secrets.token_urlsafe(32)
            config = runtime / "config.toml"
            config.write_text('[credentials]\nbackend = "file"\nfile_path = '
                              + json.dumps(str(runtime / "credentials.enc")) + "\n")
            environment = {key: value for key, value in os.environ.items()
                           if not key.startswith("AGILEPLUS_") and key not in
                           {"API_HOST", "API_PORT", "DATABASE_URL"}}
            environment.update({"AGILEPLUS_CONFIG_PATH": str(config),
                                "AGILEPLUS_API_HOST": "127.0.0.1",
                                "AGILEPLUS_API_PORT": str(port),
                                "AGILEPLUS_CORE_DB_PATH": str(runtime / "perf.db"),
                                "AGILEPLUS_CREDENTIAL_KEY": secrets.token_urlsafe(32),
                                "AGILEPLUS_API_KEY": key})
            with open(directory / "api.log", "w") as log:
                process = subprocess.Popen([str(binary)], cwd=runtime, env=environment,
                                           stdout=log, stderr=subprocess.STDOUT)
                try:
                    wait_ready(process, port, key)
                    client = Client(port, key)
                    try:
                        # Auth is part of the real flow, not bypassed for benchmarking.
                        unauthenticated = Client(port, "invalid-perf-key")
                        try:
                            unauthenticated.request("GET", "/api/v1/features/", expected=401)
                        finally:
                            unauthenticated.close()
                        for i in range(FEATURES):
                            body, _ = client.request("POST", "/api/v1/features/",
                                                      {"title": f"perf feature {i:03}"},
                                                      expected=201)
                            if body["slug"] != f"perf-feature-{i:03}" or body["state"] != "created":
                                raise RuntimeError("seeded feature has incorrect identity/state")
                        for name in ["health", "list", "detail"]:
                            report["initial_requests"][name] = measure(client, name)
                        for name in ["health", "list", "detail", "transition"]:
                            samples = concurrent_measurements(port, key, name)
                            report["samples"][name] = samples
                            report["distributions"][name] = distribution(samples, name)
                        for i in range(FEATURES):
                            body, sample = client.request("GET", f"/api/v1/features/perf-feature-{i:03}")
                            validate_response("detail", body, i)
                            if body["state"] != "specified":
                                raise RuntimeError("state change was not visible on subsequent read")
                            report["after_state_change"].append(sample)
                        report["status"] = "completed"
                    finally:
                        client.close()
                finally:
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                    report["cleanup"] = {"api_process_stopped": process.poll() is not None,
                                         "temporary_runtime_removed": False}
        report["cleanup"]["temporary_runtime_removed"] = True
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError,
            http.client.HTTPException, KeyError, TypeError, KeyboardInterrupt) as error:
        report["error"] = str(error)
    finally:
        write_reports(directory, report)
    for name, row in report["distributions"].items():
        if row["warnings"]:
            print(f"::warning::{name} exceeds T118 warning targets: {', '.join(row['warnings'])}")
    return 0 if report["status"] == "completed" else 1


if __name__ == "__main__":
    def stop_on_signal(_signum, _frame):
        raise KeyboardInterrupt("HTTP performance run interrupted")

    signal.signal(signal.SIGTERM, stop_on_signal)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--report-dir", type=Path, default=Path(".perf-reports"))
    options = parser.parse_args()
    raise SystemExit(run(options.binary, options.report_dir))

