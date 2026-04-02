#!/usr/bin/env python3
"""
Traceability Checker for Phenotype Ecosystem.
Supports mass verification across 170+ repositories.
"""

import os
import re
import json
import sys
import argparse
from typing import Dict, List, Set, Tuple

# Regex patterns
SPEC_MARKERS = {
    "FR": re.compile(r"FR-[A-Z0-9]+-\d+"),
    "TRACE": re.compile(r"@trace\s+([A-Z0-9-]+\d+)"),
    "TEST_ID": re.compile(r"TEST-\d+"),
}

def find_markers_in_dir(directory: str, extensions: Tuple[str, ...] = (".rs", ".ts", ".py", ".yaml", ".yml", ".md", ".zig", ".go", ".proto")) -> Set[str]:
    found_frs = set()
    for root, _, files in os.walk(directory):
        if any(d in root for d in ["target", "node_modules", ".git", "vendor", "__pycache__"]):
            continue
        for file in files:
            if file.endswith(extensions):
                path = os.path.join(root, file)
                try:
                    with open(path, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                        found_frs.update(SPEC_MARKERS["FR"].findall(content))
                        found_frs.update(SPEC_MARKERS["TRACE"].findall(content))
                except Exception:
                    pass
    return found_frs

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", required=True)
    parser.add_argument("--repos-file", help="File containing list of repo paths")
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    with open(args.json, "r") as f:
        data = json.load(f)

    repos_to_check = []
    if args.repos_file:
        with open(args.repos_file, "r") as f:
            repos_to_check = [line.strip() for line in f if line.strip()]
    else:
        repos_to_check = ["."]

    print(f"--- Traceability Validation for {len(repos_to_check)} repositories ---")
    
    global_missing = []
    for repo_path in repos_to_check:
        repo_name = os.path.basename(repo_path)
        # Find spec config for this repo
        repo_spec = next((r for r in data["repositories"] if r["name"] == repo_name), None)
        
        if not repo_spec:
            continue

        markers = find_markers_in_dir(repo_path)
        implemented = [s["id"] for s in repo_spec["specsList"] if s["status"] == "implemented"]
        missing = [sid for sid in implemented if sid not in markers]
        
        if missing:
            print(f"❌ {repo_name}: Missing {len(missing)}: {', '.join(missing)}")
            global_missing.extend([(repo_name, sid) for sid in missing])
        else:
            print(f"✅ {repo_name}: Verified.")

    print(f"\nSummary: {len(global_missing)} total missing markers.")
    if args.strict and global_missing:
        sys.exit(1)

if __name__ == "__main__":
    main()
