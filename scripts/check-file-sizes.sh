#!/bin/bash
# check-file-sizes.sh — Enforce file size limits
#
# Prevents code bloat by enforcing maximum line counts:
#   - Source files (*.rs): max 500 lines
#   - Test files (tests/*.rs): max 800 lines
#   - Provider files (provider.star): max 300 lines
#
# This is a "soft" linter — it warns but doesn't block CI by default.
# Use --strict to make it fail on violations.

set -euo pipefail

STRICT=false
SRC_LIMIT=500
TEST_LIMIT=800
STAR_LIMIT=300

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --strict)
            STRICT=true
            shift
            ;;
        --src-limit)
            SRC_LIMIT="$2"
            shift 2
            ;;
        --test-limit)
            TEST_LIMIT="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Color output
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
NC='\033[0m'

VIOLATIONS=0
WARNINGS=0

echo "📏 Checking file size limits..."
echo "   Source: max $SRC_LIMIT lines | Tests: max $TEST_LIMIT lines | Starlark: max $STAR_LIMIT lines"
echo ""

# Check Rust source files
check_rust_files() {
    local dir_pattern="$1"
    local limit="$2"
    local label="$3"

    while IFS= read -r -d '' file; do
        local lines
        lines=$(wc -l < "$file" | xargs)
        if [ "$lines" -gt "$limit" ]; then
            local pct=$((lines * 100 / limit))
            if [ "$lines" -gt $((limit * 2)) ]; then
                echo -e "${RED}❌ $file${NC}: $lines lines (${pct}% of limit) — $label"
                VIOLATIONS=$((VIOLATIONS + 1))
            else
                echo -e "${YELLOW}⚠️  $file${NC}: $lines lines (${pct}% of limit) — $label"
                WARNINGS=$((WARNINGS + 1))
            fi
        fi
    done < <(find $dir_pattern -name "*.rs" -print0 2>/dev/null)
}

# Check source files (exclude tests/)
echo "### Rust Source Files (limit: $SRC_LIMIT lines)"
while IFS= read -r -d '' file; do
    # Skip test directories
    if echo "$file" | grep -q "/tests/"; then
        continue
    fi
    lines=$(wc -l < "$file" | xargs)
    if [ "$lines" -gt "$SRC_LIMIT" ]; then
        pct=$((lines * 100 / SRC_LIMIT))
        if [ "$lines" -gt $((SRC_LIMIT * 2)) ]; then
            echo -e "${RED}❌ $file${NC}: $lines lines (${pct}% of limit)"
            VIOLATIONS=$((VIOLATIONS + 1))
        else
            echo -e "${YELLOW}⚠️  $file${NC}: $lines lines (${pct}% of limit)"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi
done < <(find crates/*/src -name "*.rs" -print0 2>/dev/null)

echo ""

# Check test files
echo "### Rust Test Files (limit: $TEST_LIMIT lines)"
while IFS= read -r -d '' file; do
    lines=$(wc -l < "$file" | xargs)
    if [ "$lines" -gt "$TEST_LIMIT" ]; then
        pct=$((lines * 100 / TEST_LIMIT))
        if [ "$lines" -gt $((TEST_LIMIT * 2)) ]; then
            echo -e "${RED}❌ $file${NC}: $lines lines (${pct}% of limit)"
            VIOLATIONS=$((VIOLATIONS + 1))
        else
            echo -e "${YELLOW}⚠️  $file${NC}: $lines lines (${pct}% of limit)"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi
done < <(find crates/*/tests -name "*.rs" -print0 2>/dev/null)

echo ""

# Check provider.star files
echo "### Provider Starlark Files (limit: $STAR_LIMIT lines)"
while IFS= read -r -d '' file; do
    lines=$(wc -l < "$file" | xargs)
    if [ "$lines" -gt "$STAR_LIMIT" ]; then
        pct=$((lines * 100 / STAR_LIMIT))
        echo -e "${YELLOW}⚠️  $file${NC}: $lines lines (${pct}% of limit)"
        WARNINGS=$((WARNINGS + 1))
    fi
done < <(find crates/vx-providers -name "provider.star" -print0 2>/dev/null)

echo ""

# Summary
echo "========================================"
echo "Results: $VIOLATIONS violation(s), $WARNINGS warning(s)"

if [ $VIOLATIONS -gt 0 ] && [ "$STRICT" = true ]; then
    echo -e "${RED}❌ File size check FAILED (strict mode)${NC}"
    exit 1
elif [ $VIOLATIONS -gt 0 ]; then
    echo -e "${YELLOW}⚠️  File size check found violations (non-strict mode)${NC}"
    echo "   Run with --strict to fail CI on violations"
    exit 0
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠️  File size check PASSED with warnings${NC}"
    exit 0
else
    echo -e "${GREEN}✅ All files within size limits${NC}"
    exit 0
fi
