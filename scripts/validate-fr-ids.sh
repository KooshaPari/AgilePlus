#!/bin/bash
# validate-fr-ids.sh - Validate FR ID format consistency across specs
# Usage: validate-fr-ids.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGILEPLUS_DIR="${SCRIPT_DIR}/.."
SPECS_DIR="${AGILEPLUS_DIR}/specs"

ERRORS=0

echo "=== FR ID Validation ==="
echo ""

# FR ID format regex
FR_PATTERN="^FR-[A-Z][A-Z0-9]*-[0-9]+(-[A-Z0-9]+)?$"

# Check all spec files
for spec in "$SPECS_DIR"/FR-*.md; do
    if [ ! -f "$spec" ]; then
        continue
    fi
    
    filename=$(basename "$spec" .md)
    
    # Extract FR ID from frontmatter (YAML format: id: VALUE)
    fr_id=$(grep --color=never "^id:" "$spec" | head -1 | cut -d':' -f2- | xargs | tr -d '\r\n')
    
    if [ -z "$fr_id" ]; then
        echo "❌ $filename: Missing 'id:' in frontmatter"
        ERRORS=$((ERRORS + 1))
        continue
    fi
    
    # Check if ID matches filename
    if [ "$fr_id" != "$filename" ]; then
        echo "❌ $filename: ID mismatch (frontmatter: $fr_id, filename: $filename)"
        ERRORS=$((ERRORS + 1))
        continue
    fi
    
    # Check format
    if [[ ! "$fr_id" =~ $FR_PATTERN ]]; then
        echo "❌ $filename: Invalid FR ID format: $fr_id"
        ERRORS=$((ERRORS + 1))
        continue
    fi
    
    # Check required fields
    if ! grep -q "^title:" "$spec"; then
        echo "❌ $fr_id: Missing 'title:' in frontmatter"
        ERRORS=$((ERRORS + 1))
        continue
    fi
    
    if ! grep -q "^status:" "$spec"; then
        echo "❌ $fr_id: Missing 'status:' in frontmatter"
        ERRORS=$((ERRORS + 1))
        continue
    fi
    
    if ! grep -q "^priority:" "$spec"; then
        echo "❌ $fr_id: Missing 'priority:' in frontmatter"
        ERRORS=$((ERRORS + 1))
        continue
    fi
    
    echo "✓ $fr_id"
done

echo ""

if [ $ERRORS -eq 0 ]; then
    echo "=== All FR IDs Valid ==="
    exit 0
else
    echo "=== $ERRORS Errors Found ==="
    exit 1
fi
