#!/bin/bash
# Docker Compose V1 to V2 Migration

set -e

echo "Migrating Docker Compose V1 to V2..."

files=$(find . -maxdepth 1 -name "docker-compose*.yml" -o -name "docker-compose*.yaml" 2>/dev/null)

for file in $files; do
    echo "Processing $file..."
    cp "$file" "$file.backup.$(date +%Y%m%d)"
    sed -i '' '/^version:/d' "$file" 2>/dev/null || sed -i '/^version:/d' "$file"
done

find . -type f \( -name "*.sh" -o -name "*.yml" -o -name "Makefile" \) -exec sed -i '' 's/docker-compose /docker compose /g' {} \; 2>/dev/null || true

echo "✅ Docker Compose V1 to V2 migration complete"
