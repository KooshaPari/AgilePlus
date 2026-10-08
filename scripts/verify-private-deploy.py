#!/usr/bin/env python3
"""Static safety gate for the tailnet-first AgilePlus API bootstrap.

This intentionally does NOT call a running backend. It validates the Docker
Compose *resolved configuration* without claiming that deployment, DNS, TLS,
auth or remote browser journeys have been verified.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
COMPOSE = ROOT / "deploy/selfhost/docker-compose.selfhost.yml"
CADDY = ROOT / "deploy/selfhost/Caddyfile"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    env = dict(os.environ)
    env["AGILEPLUS_API_KEY"] = "static-contract-test-only"
    env["AGILEPLUS_HOST_PORT"] = "3000"

    try:
        result = subprocess.run(
            ["docker", "compose", "-f", str(COMPOSE), "config", "--format", "json"],
            cwd=ROOT,
            env=env,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as exc:
        print(f"Compose resolution unavailable or invalid: {exc}", file=sys.stderr)
        return 2

    config = json.loads(result.stdout)
    services = config["services"]
    require(set(services) == {"agileplus-api"},
            "private bootstrap must not start per-project Caddy, Traefik or Tunnel")
    api = services["agileplus-api"]
    rust_version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["rust-version"]
    image = api["image"]
    require(image.startswith("rust:"), "bootstrap must use an explicit Rust image")
    image_version = image.split(":", 1)[1].split("-", 1)[0]

    def version(raw: str) -> tuple[int, ...]:
        return tuple(int(part) for part in raw.split("."))

    require(version(image_version) >= version(rust_version),
            f"Rust image {image_version} is older than workspace MSRV {rust_version}")

    published = api.get("ports", [])
    require(len(published) == 1, "API must have exactly one loopback-only host port")
    port = published[0]
    require(port["host_ip"] == "127.0.0.1",
            "the API must not publish on any LAN/public interface")
    require(int(port["target"]) == 3000 and int(port["published"]) == 3000,
            "bootstrap port must agree with the host Caddy upstream")

    mounts = api.get("volumes", [])
    require(any(v.get("target") == "/data" and v.get("type") == "volume" for v in mounts),
            "SQLite data must be stored in a persistent named volume")
    require(api.get("environment", {}).get("AGILEPLUS_API_KEY"),
            "API credential must be provided to backend at runtime")
    require(not api.get("privileged", False), "API container must not be privileged")
    command = api.get("command", "")
    if isinstance(command, list):
        command = " ".join(command)
    require("--locked" in command,
            "production bootstrap should use the exact Cargo.lock")

    caddy = CADDY.read_text()
    require("reverse_proxy 127.0.0.1:3000" in caddy,
            "shared host Caddy must route to the loopback API port")
    require("api.agileplus.pheno.studio" in caddy,
            "Caddy fragment must use the owned product API hostname")

    print("PASS: static private-deploy contract (NOT live-deployment evidence)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, KeyError, ValueError, TypeError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
