#!/bin/bash
# Test runner script for vx project
# Provides various testing options with proper error handling

set -euo pipefail

# Default values
TYPE="all"
VERBOSE=false
COVERAGE=false
SERIAL=false
PACKAGE=""
TEST=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Usage information
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Test runner for vx project

OPTIONS:
    -t, --type TYPE         Test type: unit, integration, doc, all, clippy, fmt, check (default: all)
    -p, --package PACKAGE   Run tests for specific package
    -T, --test TEST         Run specific test
    -v, --verbose           Enable verbose output
    -c, --coverage          Generate coverage report (requires cargo-tarpaulin)
    -s, --serial            Run tests serially (single-threaded)
    -h, --help              Show this help message

EXAMPLES:
    $0                      # Run all tests
    $0 -t unit              # Run only unit tests
    $0 -t integration -v    # Run integration tests with verbose output
    $0 -p vx-core           # Run tests for vx-core package only
    $0 -c                   # Run all tests and generate coverage report

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            TYPE="$2"
            shift 2
            ;;
        -p|--package)
            PACKAGE="$2"
            shift 2
            ;;
        -T|--test)
            TEST="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        -s|--serial)
            SERIAL=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Ensure we're in the project root
if [[ ! -f "Cargo.toml" ]]; then
    log_error "Must be run from project root directory"
    exit 1
fi

# Build test command based on parameters
build_test_command() {
    local test_type="$1"
    local cmd="cargo test"
    
    # Add package filter if specified
    if [[ -n "$PACKAGE" ]]; then
        cmd+=" -p $PACKAGE"
    fi
    
    # Add test filter if specified
    if [[ -n "$TEST" ]]; then
        cmd+=" $TEST"
    fi
    
    # Add type-specific flags
    case "$test_type" in
        "unit")
            cmd+=" --lib"
            ;;
        "integration")
            cmd+=" --test '*'"
            ;;
        "doc")
            cmd+=" --doc"
            ;;
        "all")
            cmd+=" --all"
            ;;
    esac
    
    # Add verbose flag if requested
    if [[ "$VERBOSE" == true ]]; then
        cmd+=" -- --nocapture"
    fi
    
    # Add serial execution if requested
    if [[ "$SERIAL" == true ]]; then
        cmd+=" -- --test-threads=1"
    fi
    
    echo "$cmd"
}

# Run tests with error handling
run_tests() {
    local command="$1"
    local description="$2"
    
    log_info "Running $description..."
    log_info "Command: $command"
    
    local start_time=$(date +%s)
    
    if eval "$command"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        log_success "$description completed successfully in ${duration} seconds"
        return 0
    else
        log_error "$description failed"
        return 1
    fi
}

# Main execution
log_info "Starting vx test suite..."
log_info "Test type: $TYPE"

success=true

case "${TYPE,,}" in
    "unit")
        cmd=$(build_test_command "unit")
        run_tests "$cmd" "unit tests" || success=false
        ;;
    "integration")
        cmd=$(build_test_command "integration")
        run_tests "$cmd" "integration tests" || success=false
        ;;
    "doc")
        cmd=$(build_test_command "doc")
        run_tests "$cmd" "documentation tests" || success=false
        ;;
    "all")
        # Run all test types
        test_types=("unit:unit tests" "integration:integration tests" "doc:documentation tests")
        
        for test_type in "${test_types[@]}"; do
            IFS=':' read -r type description <<< "$test_type"
            cmd=$(build_test_command "$type")
            if ! run_tests "$cmd" "$description"; then
                success=false
                log_warning "Continuing with remaining tests..."
            fi
        done
        ;;
    "clippy")
        run_tests "cargo clippy --all -- -D warnings" "clippy linting" || success=false
        ;;
    "fmt")
        run_tests "cargo fmt --all -- --check" "format checking" || success=false
        ;;
    "check")
        run_tests "cargo check --all" "compilation check" || success=false
        ;;
    *)
        log_error "Unknown test type: $TYPE"
        log_info "Available types: unit, integration, doc, all, clippy, fmt, check"
        exit 1
        ;;
esac

# Coverage report (if requested and available)
if [[ "$COVERAGE" == true && "$success" == true ]]; then
    log_info "Generating coverage report..."
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        run_tests "cargo tarpaulin --out Html --output-dir target/coverage" "coverage report"
        log_info "Coverage report generated in target/coverage/"
    else
        log_warning "cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
    fi
fi

# Final status
if [[ "$success" == true ]]; then
    log_success "All tests completed successfully!"
    exit 0
else
    log_error "Some tests failed!"
    exit 1
fi
