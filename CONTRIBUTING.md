# Contributing

## Development

- Python: `3.11+`
- Install editable package:
  - `python -m pip install -e .`
- Run tests:
  - `python -m unittest discover -s tests -p 'test_*.py' -v`

## Change expectations

- Keep runtime behavior deterministic.
- Add or update tests for any CLI, state, or git/materialization change.
- Prefer fixing forward over compatibility shims.

## Pull requests

- Keep scope narrow and behavior-driven.
- Update `README.md` and `CHANGELOG.md` when command surface or operator behavior changes.
