#!/bin/bash
# fr-check.sh - Check FR coverage across repositories
# Usage: fr-check.sh [repo_name]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGILEPLUS_DIR="${SCRIPT_DIR}/.."
SPECS_DIR="${AGILEPLUS_DIR}/specs"
PTRACE="${AGILEPLUS_DIR}/bin/ptrace"

if [ ! -f "$PTRACE" ]; then
    echo "Error: ptrace CLI not found at $PTRACE"
    exit 1
fi

REPO_NAME="${1:-all}"

echo "=== FR Coverage Check ==="
echo "Specs directory: $SPECS_DIR"
echo "Target: $REPO_NAME"
echo ""

# Count FRs in specs
count_frs() {
    if [ -d "$SPECS_DIR" ]; then
        ls -1 "$SPECS_DIR"/FR-*.md 2>/dev/null | wc -l
    else
        echo "0"
    fi
}

# Check specific repo
check_repo() {
    local repo="$1"
    local repo_path="/Users/kooshapari/CodeProjects/Phenotype/repos/$repo"
    
    if [ ! -d "$repo_path" ]; then
        echo "Repository not found: $repo"
        return 1
    fi
    
    echo "=== $repo ==="
    
    # Check AI file
    if [ -f "$repo_path/.phenotype/ai-traceability.yaml" ]; then
        echo "✓ AI attribution file"
    else
        echo "✗ Missing AI attribution file"
    fi
    
    # Check CI/CD
    if [ -f "$repo_path/.github/workflows/traceability.yml" ]; then
        echo "✓ CI/CD workflow"
    else
        echo "✗ Missing CI/CD workflow"
    fi
    
    # Run ptrace analyze
    echo "Running coverage check..."
    "$PTRACE" analyze --path "$repo_path" --lang all 2>/dev/null || echo "  No FRs found"
    echo ""
}

TOTAL_FRS=$(count_frs)
echo "Total FR specs: $TOTAL_FRS"
echo ""

if [ "$REPO_NAME" = "all" ]; then
    # Check all repos with AI files
    for ai_file in /Users/kooshapari/CodeProjects/Phenotype/repos/*/.phenotype/ai-traceability.yaml; do
        if [ -f "$ai_file" ]; then
            repo=$(basename "$(dirname "$(dirname "$ai_file")")")
            check_repo "$repo"
        fi
    done
else
    check_repo "$REPO_NAME"
fi

echo "=== Summary ==="
echo "FR specs: $TOTAL_FRS"
echo "Repos with traceability: $(ls /Users/kooshapari/CodeProjects/Phenotype/repos/*/.phenotype/ai-traceability.yaml 2>/dev/null | wc -l)"
