#!/bin/bash
# add-fr-annotation.sh - Interactive helper to add FR annotations to test files
# Usage: add-fr-annotation.sh <file> <fr-id>

set -e

show_usage() {
    cat << 'EOF'
Usage: add-fr-annotation.sh <file> <fr-id> [options]

Add FR annotation to a test file.

Arguments:
  file        Path to test file
  fr-id       FR ID (e.g., FR-AGILE-001)

Options:
  --python    Force Python format (pytest)
  --rust      Force Rust format
  --go        Force Go format
  --ts        Force TypeScript format
  -h, --help  Show this help

Examples:
  add-fr-annotation.sh tests/test_api.py FR-AGILE-001
  add-fr-annotation.sh src/lib.rs FR-THEGENT-002 --rust
  add-fr-annotation.sh main_test.go FR-HELIOS-003 --go

EOF
}

if [ $# -lt 2 ]; then
    show_usage
    exit 1
fi

FILE="$1"
FR_ID="$2"
shift 2
LANGUAGE=""

# Parse options
while [ $# -gt 0 ]; do
    case "$1" in
        --python)
            LANGUAGE="python"
            ;;
        --rust)
            LANGUAGE="rust"
            ;;
        --go)
            LANGUAGE="go"
            ;;
        --ts|--typescript)
            LANGUAGE="typescript"
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
    shift
done

# Detect language from file extension if not specified
if [ -z "$LANGUAGE" ]; then
    case "$FILE" in
        *.py)
            LANGUAGE="python"
            ;;
        *.rs)
            LANGUAGE="rust"
            ;;
        *_test.go|*.go)
            LANGUAGE="go"
            ;;
        *.ts|*.test.ts)
            LANGUAGE="typescript"
            ;;
        *)
            echo "Cannot detect language from file: $FILE"
            echo "Please specify with --python, --rust, --go, or --ts"
            exit 1
            ;;
    esac
fi

# Validate FR ID format
if [[ ! "$FR_ID" =~ ^FR-[A-Z][A-Z0-9]*-[0-9]+(-[A-Z0-9]+)?$ ]]; then
    echo "Warning: FR ID '$FR_ID' may not match standard format (FR-XXX-NNN)"
    read -p "Continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check file exists
if [ ! -f "$FILE" ]; then
    echo "Error: File not found: $FILE"
    exit 1
fi

echo "Adding FR annotation to: $FILE"
echo "FR ID: $FR_ID"
echo "Language: $LANGUAGE"
echo ""

# Generate annotation based on language
case "$LANGUAGE" in
    python)
        ANNOTATION="@pytest.mark.traces_to(\"$FR_ID\")"
        IMPORT="import pytest"
        
        # Check if pytest is imported
        if ! grep -q "^import pytest\|^from pytest" "$FILE"; then
            echo "Would add import: $IMPORT"
        fi
        echo "Add annotation: $ANNOTATION"
        echo ""
        echo "Example:"
        echo "$ANNOTATION"
        echo "def test_something():"
        echo "    ..."
        ;;
    rust)
        ANNOTATION="#\[trace_to(\"$FR_ID\")\]"
        
        echo "Add annotation: $ANNOTATION"
        echo ""
        echo "Example:"
        echo "$ANNOTATION"
        echo "#[test]"
        echo "fn test_something() {"
        echo "    ..."
        echo "}"
        ;;
    go)
        echo "Add to test function:"
        echo "    gotreqt.TraceTo(t, \"$FR_ID\")"
        echo ""
        echo "Example:"
        echo "func TestSomething(t *testing.T) {"
        echo "    gotreqt.TraceTo(t, \"$FR_ID\")"
        echo "    ..."
        echo "}"
        ;;
    typescript)
        echo "Add import:"
        echo "    import { tracesTo } from '@phenotype/tstreqt';"
        echo ""
        echo "Add to test:"
        echo "    test('description', tracesTo('$FR_ID'), () => {"
        echo "        ..."
        echo "    })"
        ;;
esac

echo ""
echo "Open $FILE to add the annotation?"
read -p "[y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    ${EDITOR:-nano} "$FILE"
fi
