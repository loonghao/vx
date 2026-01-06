#!/bin/bash
# Check for inline tests in source files
# 
# This script enforces the project convention that tests should be placed
# in separate tests/ directories, not inline in source files.
#
# Usage: ./scripts/check-inline-tests.sh [--fix]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Files that are temporarily allowed to have inline tests (whitelist)
# These should be migrated over time
WHITELIST=(
    # Add files here that are temporarily allowed
    # Example: "crates/vx-config/src/inheritance.rs"
)

# Check if a file is in the whitelist
is_whitelisted() {
    local file="$1"
    for allowed in "${WHITELIST[@]}"; do
        if [[ "$file" == *"$allowed" ]]; then
            return 0
        fi
    done
    return 1
}

# Find all Rust source files with inline tests
find_inline_tests() {
    local found_issues=0
    
    # Search for #[cfg(test)] in src/ directories (not tests/)
    while IFS= read -r file; do
        # Skip files in tests/ directories
        if [[ "$file" == */tests/* ]]; then
            continue
        fi
        
        # Skip whitelisted files
        if is_whitelisted "$file"; then
            echo "⚠️  WHITELISTED: $file"
            continue
        fi
        
        echo "❌ INLINE TEST: $file"
        found_issues=1
    done < <(grep -rl '#\[cfg(test)\]' "$PROJECT_ROOT/crates" --include="*.rs" 2>/dev/null || true)
    
    return $found_issues
}

# Main
echo "🔍 Checking for inline tests in source files..."
echo ""

if find_inline_tests; then
    echo ""
    echo "✅ No inline tests found (or all are whitelisted)"
    exit 0
else
    echo ""
    echo "❌ Found inline tests in source files!"
    echo ""
    echo "Project convention requires tests to be in separate tests/ directories."
    echo "Please move inline tests to: crates/<crate>/tests/<module>_tests.rs"
    echo ""
    echo "If a file must temporarily keep inline tests, add it to the whitelist"
    echo "in scripts/check-inline-tests.sh"
    exit 1
fi
