#!/bin/bash
# Publish Python package to PyPI

set -e

cd "$(dirname "$0")"

echo "=== Building phenotype-traceability ==="
python -m build

echo ""
echo "=== Checking distribution ==="
twine check dist/*

echo ""
echo "=== Uploading to PyPI ==="
echo "Command: twine upload dist/*"
echo ""
read -p "Press Enter to publish to PyPI, or Ctrl+C to cancel..."

twine upload dist/*

echo ""
echo "=== Published successfully ==="
echo "Package: phenotype-traceability"
echo "Version: $(grep version pyproject.toml | head -1 | cut -d'"' -f2)"
