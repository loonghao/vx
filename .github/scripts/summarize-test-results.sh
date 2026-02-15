#!/bin/bash
# Summarize vx provider test results from multiple platforms
#
# Usage:
#   ./summarize-test-results.sh [options]
#
# Options:
#   --reports-dir DIR    Directory containing report-* subdirectories (default: .)
#   --summary FILE       Write summary to file (for GitHub step summary)
#   --fail-on-error      Exit with code 1 if any tests failed
#
# Expected directory structure:
#   reports-dir/
#   ├── report-linux-0/
#   │   └── test-report.json
#   ├── report-linux-1/
#   │   └── test-report.json
#   ├── report-macos-0/
#   │   └── test-report.json
#   └── report-windows-0/
#       └── test-report.json

set -euo pipefail

# Default values
REPORTS_DIR="."
SUMMARY_FILE=""
FAIL_ON_ERROR="false"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --reports-dir)
            REPORTS_DIR="$2"
            shift 2
            ;;
        --summary)
            SUMMARY_FILE="$2"
            shift 2
            ;;
        --fail-on-error)
            FAIL_ON_ERROR="true"
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Initialize counters
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_SKIPPED=0

# Collect all failed providers for the summary
FAILED_DETAILS=""

echo "Scanning for test reports in $REPORTS_DIR..."

# Process each report directory
for dir in "$REPORTS_DIR"/report-*/; do
    if [ -d "$dir" ] && [ -f "${dir}test-report.json" ]; then
        REPORT="${dir}test-report.json"
        DIR_NAME=$(basename "$dir")

        # Extract platform from directory name (e.g., report-linux-0 -> linux)
        PLATFORM=$(echo "$DIR_NAME" | sed 's/report-//' | sed 's/-[0-9]*$//')
        CHUNK_INDEX=$(echo "$DIR_NAME" | grep -oE '[0-9]+$' || echo "0")

        # Parse test results
        PASSED=$(jq '.passed // 0' "$REPORT" 2>/dev/null || echo "0")
        FAILED=$(jq '.failed // 0' "$REPORT" 2>/dev/null || echo "0")
        SKIPPED=$(jq '.skipped // 0' "$REPORT" 2>/dev/null || echo "0")

        TOTAL_PASSED=$((TOTAL_PASSED + PASSED))
        TOTAL_FAILED=$((TOTAL_FAILED + FAILED))
        TOTAL_SKIPPED=$((TOTAL_SKIPPED + SKIPPED))

        echo "  $DIR_NAME: ✅ $PASSED passed, ❌ $FAILED failed, ⏭️ $SKIPPED skipped"

        # Collect failed provider details
        if [ "$FAILED" -gt 0 ]; then
            FAILURES=$(jq -r '.results[] | select(.overall_passed == false and .platform_supported == true) | "\(.runtime): \(.error // "unknown error")"' "$REPORT" 2>/dev/null || echo "")
            if [ -n "$FAILURES" ]; then
                FAILED_DETAILS="${FAILED_DETAILS}### $PLATFORM (chunk $CHUNK_INDEX)\n$FAILURES\n\n"
            fi
        fi
    fi
done

TOTAL=$((TOTAL_PASSED + TOTAL_FAILED + TOTAL_SKIPPED))

echo ""
echo "========================================"
echo "Overall Results:"
echo "  ✅ Passed:  $TOTAL_PASSED"
echo "  ❌ Failed:  $TOTAL_FAILED"
echo "  ⏭️  Skipped: $TOTAL_SKIPPED"
echo "  📊 Total:   $TOTAL"
echo "========================================"

# Generate summary if requested
if [ -n "$SUMMARY_FILE" ]; then
    {
        echo "## Provider Test Results"
        echo ""
        echo "### Overall Summary"
        echo ""
        echo "| Status | Count |"
        echo "|--------|-------|"
        echo "| ✅ Passed | $TOTAL_PASSED |"
        echo "| ❌ Failed | $TOTAL_FAILED |"
        echo "| ⏭️ Skipped | $TOTAL_SKIPPED |"
        echo "| **Total** | $TOTAL |"
        echo ""

        # Show failed providers if any
        if [ "$TOTAL_FAILED" -gt 0 ]; then
            echo "### ❌ Failed Providers"
            echo ""
            echo -e "$FAILED_DETAILS"
        fi

        # Per-platform breakdown
        echo "### Platform Breakdown"
        echo ""
        echo "| Platform | Chunk | Passed | Failed | Skipped |"
        echo "|----------|-------|--------|--------|---------|"

        for dir in "$REPORTS_DIR"/report-*/; do
            if [ -d "$dir" ] && [ -f "${dir}test-report.json" ]; then
                REPORT="${dir}test-report.json"
                DIR_NAME=$(basename "$dir")
                PLATFORM=$(echo "$DIR_NAME" | sed 's/report-//' | sed 's/-[0-9]*$//')
                CHUNK_INDEX=$(echo "$DIR_NAME" | grep -oE '[0-9]+$' || echo "0")

                PASSED=$(jq '.passed // 0' "$REPORT" 2>/dev/null || echo "0")
                FAILED=$(jq '.failed // 0' "$REPORT" 2>/dev/null || echo "0")
                SKIPPED=$(jq '.skipped // 0' "$REPORT" 2>/dev/null || echo "0")

                echo "| $PLATFORM | $CHUNK_INDEX | $PASSED | $FAILED | $SKIPPED |"
            fi
        done
    } >> "$SUMMARY_FILE"

    echo "Summary written to $SUMMARY_FILE"
fi

# Exit with error if requested and there were failures
if [ "$FAIL_ON_ERROR" = "true" ] && [ "$TOTAL_FAILED" -gt 0 ]; then
    echo "::error::$TOTAL_FAILED provider(s) failed across all platforms"
    exit 1
fi

echo "Done!"
