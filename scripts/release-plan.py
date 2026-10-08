#!/usr/bin/env python3
"""Print the dependency-ordered registry release plan; never publish packages."""
import json
import subprocess
import sys


def release_order(packages, roots):
    by_name = {p["name"]: p for p in packages}
    visiting, visited, ordered = set(), set(), []

    def visit(name):
        if name in visited:
            return
        if name in visiting:
            raise ValueError(f"release dependency cycle at {name}")
        package = by_name[name]
        if package.get("publish") == []:
            raise ValueError(f"required package {name} prohibits publication")
        visiting.add(name)
        for dependency in sorted(package["dependencies"], key=lambda d: d["name"]):
            if dependency["kind"] == "dev":
                continue
            if dependency["name"] in by_name:
                if dependency["req"] == "*":
                    raise ValueError(f"{name}: {dependency['name']} lacks a registry version")
                visit(dependency["name"])
            elif dependency.get("path") or (dependency.get("source") or "").startswith("git+"):
                raise ValueError(f"{name}: external dependency {dependency['name']} needs registry reconciliation")
        visiting.remove(name)
        visited.add(name)
        ordered.append({"name": name, "version": package["version"]})

    for root in roots:
        visit(root)
    return ordered


if __name__ == "__main__":
    metadata = json.loads(subprocess.check_output([
        "cargo", "+stable", "metadata", "--no-deps", "--format-version", "1",
    ]))
    try:
        plan = release_order(metadata["packages"], sys.argv[1:] or ["agileplus-cli", "agileplus-api"])
    except (KeyError, ValueError) as error:
        sys.exit(str(error))
    print(json.dumps({"mode": "plan-only", "packages": plan}, indent=2))
