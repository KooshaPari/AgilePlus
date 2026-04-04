#!/bin/bash
# NPM to Bun Migration Template

set -e

echo "Migrating NPM to Bun..."

# Remove npm lockfiles
rm -f package-lock.json npm-shrinkwrap.json

# Update package.json if it exists
if [ -f package.json ]; then
    echo "Updating package.json..."
    sed -i '' '/"package-lock":/d' package.json 2>/dev/null || true
    sed -i '' '/"npm-shrinkwrap":/d' package.json 2>/dev/null || true
    
    if ! grep -q "bun.lockb" .gitignore 2>/dev/null; then
        echo "bun.lockb" >> .gitignore
    fi
fi

# Install dependencies with bun
if [ -f package.json ]; then
    echo "Installing dependencies with bun..."
    bun install || {
        echo "bun install failed. Install bun: curl -fsSL https://bun.sh/install | bash"
        exit 1
    }
fi

echo "✅ NPM to Bun migration complete"
echo "Replace 'npm run' with 'bun run' in scripts"
