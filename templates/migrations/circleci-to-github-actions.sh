#!/bin/bash
# CircleCI to GitHub Actions Migration

set -e

if [ ! -f .circleci/config.yml ]; then
    echo "No .circleci/config.yml found"
    exit 1
fi

mkdir -p .github/workflows

cat > .github/workflows/migrated-from-circleci.yml << 'YAML'
name: CI (Migrated from CircleCI)

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: echo "Translate your CircleCI steps here"
YAML

mv .circleci/config.yml .circleci/config.yml.backup.$(date +%Y%m%d)

echo "✅ CircleCI to GitHub Actions migration started"
echo "⚠️ Please manually translate your CircleCI jobs to GitHub Actions"
