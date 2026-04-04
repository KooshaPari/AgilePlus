#!/bin/bash
# Publish TypeScript package to npm

set -e

cd "$(dirname "$0")"

echo "=== Building @phenotype/tstreqt ==="
npm run build

echo ""
echo "=== Running tests ==="
npm test

echo ""
echo "=== Publishing to npm ==="
echo "Command: npm publish --access public"
echo ""
read -p "Press Enter to publish to npm, or Ctrl+C to cancel..."

npm publish --access public

echo ""
echo "=== Published successfully ==="
echo "Package: @phenotype/tstreqt"
echo "Version: $(grep version package.json | head -1 | cut -d'"' -f4)"
