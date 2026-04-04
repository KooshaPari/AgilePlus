#!/bin/bash
# Pip to UV Migration Template

set -e

echo "Migrating Pip to UV..."

if ! command -v uv &> /dev/null; then
    echo "Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
fi

if [ -f requirements.txt ]; then
    echo "Migrating requirements.txt..."
    uv add -r requirements.txt || echo "Some packages may need manual migration"
    mv requirements.txt requirements.txt.backup.$(date +%Y%m%d)
fi

if ! grep -q "uv.lock" .gitignore 2>/dev/null; then
    echo "uv.lock" >> .gitignore
fi

echo "✅ Pip to UV migration complete"
