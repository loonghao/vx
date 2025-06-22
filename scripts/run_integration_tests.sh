#!/bin/bash
# Bash script to run vx integration tests
# Usage: ./scripts/run_integration_tests.sh [test_type] [tool]
# test_type: all, quick, single, cdn, versions

set -e

TEST_TYPE=${1:-"quick"}
TOOL=${2:-"uv"}
VERBOSE=${VERBOSE:-false}

echo "🚀 VX Integration Test Runner"
echo "============================="

# Get project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "📁 Project root: $PROJECT_ROOT"
echo "🧪 Test type: $TEST_TYPE"

# Build the project first
echo "🔨 Building vx project..."
if cargo build --release; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Prepare test arguments
TEST_ARGS=("test" "--test" "comprehensive_test")
if [[ "$VERBOSE" == "true" ]]; then
    TEST_ARGS+=("--" "--nocapture")
fi

# Run specific test based on type
case "${TEST_TYPE,,}" in
    "all")
        echo "🔄 Running comprehensive test suite..."
        TEST_ARGS+=("test_all_vx_tools_comprehensive")
        ;;
    "quick")
        echo "⚡ Running quick tests..."
        TEST_ARGS+=("quick_tests")
        ;;
    "single")
        echo "🎯 Running single tool test for: $TOOL"
        TEST_ARGS+=("test_single_tool_${TOOL}")
        ;;
    "cdn")
        echo "⚡ Running CDN performance tests..."
        TEST_ARGS+=("test_cdn_performance")
        ;;
    "versions")
        echo "📋 Running version listing tests..."
        TEST_ARGS+=("test_version_listing_only")
        ;;
    *)
        echo "❌ Unknown test type: $TEST_TYPE"
        echo "Available types: all, quick, single, cdn, versions"
        exit 1
        ;;
esac

# Run the tests
echo "🏃 Executing: cargo ${TEST_ARGS[*]}"
if cargo "${TEST_ARGS[@]}"; then
    echo "✅ Tests completed successfully!"
else
    echo "❌ Some tests failed (exit code: $?)"
    exit $?
fi

echo "🎉 Integration test run completed!"
