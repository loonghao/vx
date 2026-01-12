#!/usr/bin/env bash
# Test All VX Providers
# This script tests all VX providers by executing their commands in a clean temporary environment
# Compatible with Bash 3.x (macOS) and Bash 4+ (Linux)

set -eo pipefail

# Parse arguments
KEEP_CACHE=false
VERBOSE=false
FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --keep-cache) KEEP_CACHE=true; shift ;;
        --verbose) VERBOSE=true; shift ;;
        --filter) FILTER="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

log_success() { echo -e "${GREEN}$*${NC}"; }
log_info() { echo -e "${CYAN}$*${NC}"; }
log_warning() { echo -e "${YELLOW}$*${NC}"; }
log_error() { echo -e "${RED}$*${NC}"; }
log_section() { echo -e "\n${MAGENTA}=== $* ===${NC}"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PROVIDERS_DIR="$PROJECT_ROOT/crates/vx-providers"
TEMP_VX_HOME="$(mktemp -d -t vx-test-XXXXXX)"
VX_BINARY="$PROJECT_ROOT/target/debug/vx"

# Check if vx is built
if [[ ! -f "$VX_BINARY" ]]; then
    log_error "❌ VX binary not found at: $VX_BINARY"
    log_info "Run: cargo build"
    exit 1
fi

log_section "VX Provider Test Suite"
log_info "Project Root: $PROJECT_ROOT"
log_info "Providers Dir: $PROVIDERS_DIR"
log_info "Temp VX_HOME: $TEMP_VX_HOME"
log_info "VX Binary: $VX_BINARY"

# Set VX_HOME to temp directory
export VX_HOME="$TEMP_VX_HOME"

# Test statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Parse provider.toml to extract runtime names
get_runtimes_from_toml() {
    local toml_path="$1"
    grep -A 1 '^\[\[runtimes\]\]' "$toml_path" | grep '^name' | sed 's/name = "\(.*\)"/\1/' || true
}

# Test a single command
test_vx_command() {
    local provider="$1"
    local runtime="$2"
    local cmd="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local output
    local exit_code
    
    if output=$("$VX_BINARY" "$runtime" "$cmd" 2>&1); then
        exit_code=0
    else
        exit_code=$?
    fi
    
    if [[ $exit_code -eq 0 ]]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        log_success "  ✓ vx $runtime $cmd"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "    Output: $output" | head -n 1
        fi
        return 0
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        log_error "  ✗ vx $runtime $cmd (exit: $exit_code)"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "    Error: $output" | head -n 3
        fi
        return 1
    fi
}

# Discover providers
log_section "Discovering Providers"

# Use while loop instead of mapfile for Bash 3.x compatibility
ALL_PROVIDERS=()
while IFS= read -r line; do
    ALL_PROVIDERS+=("$line")
done < <(find "$PROVIDERS_DIR" -maxdepth 1 -type d | tail -n +2 | sort)

if [[ -n "$FILTER" ]]; then
    FILTERED_PROVIDERS=()
    for p in "${ALL_PROVIDERS[@]}"; do
        if [[ "$p" == *"$FILTER"* ]]; then
            FILTERED_PROVIDERS+=("$p")
        fi
    done
    ALL_PROVIDERS=("${FILTERED_PROVIDERS[@]}")
    log_info "Filtered to providers matching: $FILTER"
fi

log_info "Found ${#ALL_PROVIDERS[@]} providers"

# Test each provider
for provider_path in "${ALL_PROVIDERS[@]}"; do
    provider_name="$(basename "$provider_path")"
    toml_path="$provider_path/provider.toml"
    
    if [[ ! -f "$toml_path" ]]; then
        continue
    fi
    
    log_section "Testing Provider: $provider_name"
    
    # Use while loop instead of mapfile for Bash 3.x compatibility
    runtimes=()
    while IFS= read -r line; do
        [[ -n "$line" ]] && runtimes+=("$line")
    done < <(get_runtimes_from_toml "$toml_path")
    
    if [[ ${#runtimes[@]} -eq 0 ]]; then
        log_warning "  ⚠ No runtimes found in provider.toml"
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        continue
    fi
    
    log_info "  Runtimes: ${runtimes[*]}"
    
    for runtime in "${runtimes[@]}"; do
        log_info "  Testing: $runtime"
        
        # Test list command
        "$VX_BINARY" list "$runtime" > /dev/null 2>&1 && \
            log_success "    ✓ vx list $runtime" || \
            log_warning "    ⚠ vx list $runtime (expected, may not be installed yet)"
        
        # Test --version (will trigger auto-install)
        test_vx_command "$provider_name" "$runtime" "--version" || true
        
        # Small delay to avoid rate limiting
        sleep 0.1
    done
done

# Generate summary
log_section "Test Summary"
log_info "Total Tests: $TOTAL_TESTS"
log_success "Passed: $PASSED_TESTS"
log_error "Failed: $FAILED_TESTS"
log_warning "Skipped: $SKIPPED_TESTS"

if [[ $TOTAL_TESTS -gt 0 ]]; then
    SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS / $TOTAL_TESTS) * 100}")
    log_info "Success Rate: $SUCCESS_RATE%"
fi

# Cache info
log_section "Cache Contents"
CACHE_SIZE=$(du -sh "$TEMP_VX_HOME" | cut -f1)
log_info "Cache size: $CACHE_SIZE"
log_info "Cache path: $TEMP_VX_HOME"

if [[ "$VERBOSE" == "true" ]]; then
    log_info "\nInstalled versions:"
    find "$TEMP_VX_HOME" -type d -name "versions" -exec find {} -maxdepth 1 -type d \; 2>/dev/null | \
        while read -r version_dir; do
            [[ "$(basename "$version_dir")" != "versions" ]] && echo "  - $(basename "$(dirname "$version_dir")")/$(basename "$version_dir")"
        done
fi

# Cleanup
if [[ "$KEEP_CACHE" != "true" ]]; then
    log_section "Cleaning Up"
    log_info "Removing temporary cache: $TEMP_VX_HOME"
    rm -rf "$TEMP_VX_HOME"
    log_success "✓ Cache cleaned"
else
    log_section "Cache Preserved"
    log_info "Cache kept at: $TEMP_VX_HOME"
    log_info "To clean up manually: rm -rf '$TEMP_VX_HOME'"
fi

# Exit with appropriate code
log_section "Test Result"
if [[ $FAILED_TESTS -eq 0 ]]; then
    log_success "✓ All tests passed!"
    exit 0
else
    log_error "✗ Some tests failed"
    exit 1
fi
