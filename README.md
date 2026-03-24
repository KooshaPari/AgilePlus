# phench

Standalone control-plane repository for Phenotype project-state runtime orchestration.

- Materialized runtime root: `/Users/kooshapari/CodeProjects/Phenotype/projects`
- Home mirror root: `~/phench`
- State metadata per target: `.phench/`

`phench` now contains an independent Python CLI/TUI codepath and can run directly from a source checkout.

## Local usage

- help:
  - `python -m phench --help`
- install editable package:
  - `python -m pip install -e .`
- run tests:
  - `python -m unittest discover -s tests -p 'test_*.py' -v`

## Current command surface

- `python -m phench target init <name>`
- `python -m phench target add-repo <name> --repo <path> --ref <ref>`
- `python -m phench target lock <name>`
- `python -m phench target materialize <name>`
- `python -m phench status <name>`
- `python -m phench timeline <name>`
- `python -m phench run <name>`
- `python -m phench env doctor <name>`
- `python -m phench sync <name>`

## Stabilization status

- source-checkout bootstrap is covered
- dual-store state sync is covered
- target lifecycle is covered
- materialization and runtime preconditions are covered
