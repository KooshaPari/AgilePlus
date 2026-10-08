# T118 HTTP measurement

From the workspace root:

```sh
python3 -m unittest discover -s scripts/perf -p 'test_*.py' -v
cargo build --locked --release -p agileplus-api --bin agileplus-api
python3 scripts/perf/http_benchmark.py --binary target/release/agileplus-api --report-dir .perf-reports
```

The harness starts the real API on loopback with a temporary Git repository,
SQLite database, explicit configuration and encrypted file credentials. It seeds
100 features through authenticated HTTP. Ten barrier-started clients issue 100
requests for each of health, feature list, feature detail and state transition.
It verifies every transition through a subsequent HTTP read. An invalid key must
receive HTTP 401. The child process stops and temporary state is removed on success,
failure or SIGTERM. API logs and complete request samples survive in the report directory.

T118 targets come from `kitty-specs/003-agileplus-platform-completion/tasks/WP21-performance-benchmarks.md`:

| Endpoint | p95 target | p99 target |
|---|---:|---:|
| List 100 features | <100 ms | <200 ms |
| Feature detail | <50 ms | <100 ms |
| Feature transition | <100 ms | <200 ms |
| Health | <10 ms | <20 ms |

These are warning targets. Missing measurements, invalid responses, failed
authentication checks or absent state changes fail the command. Percentiles use
the empirical nearest-rank method on 100 successful HTTP measurements per endpoint;
failed requests are not silently removed from a distribution.

Durations include connection setup when needed and reading the full response body.
The first measured requests follow readiness and seeding; they are not cold OS-cache
measurements. This is a bounded closed-loop local workload, not a production capacity
or external-service benchmark. It does not prove long-running stress behavior, adoption,
human usability, or complete WP21 acceptance. The fixture server used by harness tests
provides correctness evidence only; its latency is never product performance evidence.

`AGILEPLUS_CONFIG_PATH` selects the temporary configuration without replacing HOME or
modifying the user's normal configuration or OS keychain. There is deliberately no URL
argument: this command cannot aim its workload at a production service.
