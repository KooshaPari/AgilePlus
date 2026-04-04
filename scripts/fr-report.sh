#!/bin/bash
# fr-report.sh - Generate consolidated FR report
# Usage: fr-report.sh [--output FILE]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGILEPLUS_DIR="${SCRIPT_DIR}/.."
SPECS_DIR="${AGILEPLUS_DIR}/specs"
WORKLOGS_DIR="${AGILEPLUS_DIR}/../worklogs"

OUTPUT_FILE="${1:--}"
if [ "$OUTPUT_FILE" = "--output" ] || [ "$OUTPUT_FILE" = "-o" ]; then
    OUTPUT_FILE="${2:-/dev/stdout}"
fi

generate_report() {
    cat << 'EOF'
# FR Traceability Report

Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

## Overview

EOF

    # Count specs by status
    echo "### FR Status Summary"
    echo ""
    echo "| Status | Count |"
    echo "|--------|-------|"
    
    for status in draft specified implementing implemented verified; do
        count=$(grep -l "status: $status" "$SPECS_DIR"/FR-*.md 2>/dev/null | wc -l)
        echo "| $status | $count |"
    done
    echo ""
    
    # Count specs by priority
    echo "### FR Priority Summary"
    echo ""
    echo "| Priority | Count |"
    echo "|----------|-------|"
    
    for priority in P0 P1 P2 P3; do
        count=$(grep -l "priority: $priority" "$SPECS_DIR"/FR-*.md 2>/dev/null | wc -l)
        echo "| $priority | $count |"
    done
    echo ""
    
    # Repository coverage
    echo "### Repository Coverage"
    echo ""
    echo "| Repository | AI File | CI/CD | Test FRs |"
    echo "|------------|---------|-------|----------|"
    
    for ai_file in /Users/kooshapari/CodeProjects/Phenotype/repos/*/.phenotype/ai-traceability.yaml; do
        if [ -f "$ai_file" ]; then
            repo=$(basename "$(dirname "$(dirname "$ai_file")")")
            
            # Check CI/CD
            ci="❌"
            if [ -f "/Users/kooshapari/CodeProjects/Phenotype/repos/$repo/.github/workflows/traceability.yml" ]; then
                ci="✅"
            fi
            
            # Count FR annotations in tests
            test_frs=$(find "/Users/kooshapari/CodeProjects/Phenotype/repos/$repo" -type f \( -name "*.py" -o -name "*.rs" -o -name "*.go" -o -name "*.ts" \) 2>/dev/null | xargs grep -h "FR-[A-Z][A-Z0-9]*-[0-9]" 2>/dev/null | wc -l)
            
            echo "| $repo | ✅ | $ci | $test_frs |"
        fi
    done
    echo ""
    
    # Active FRs needing implementation
    echo "### Specified FRs (Ready for Implementation)"
    echo ""
    for spec in "$SPECS_DIR"/FR-*.md; do
        if grep -q "status: specified" "$spec"; then
            id=$(basename "$spec" .md)
            title=$(grep "^# " "$spec" | head -1 | sed 's/^# //')
            priority=$(grep "^priority:" "$spec" | cut -d: -f2 | tr -d ' ')
            echo "- **$id** ($priority): $title"
        fi
    done
    echo ""
    
    echo "### Next Steps"
    echo ""
    echo "1. Implement specified FRs with P0 priority"
    echo "2. Add FR annotations to test files"
    echo "3. Run traceability checks in CI/CD"
    echo ""
}

# Generate report
if [ "$OUTPUT_FILE" = "/dev/stdout" ] || [ -z "$OUTPUT_FILE" ] || [ "$OUTPUT_FILE" = "-" ]; then
    generate_report
else
    generate_report > "$OUTPUT_FILE"
    echo "Report written to: $OUTPUT_FILE"
fi
