#!/bin/bash

# Test script to validate publishing order without actually publishing
# This script performs dry-run packaging to detect dependency issues

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to test package a crate
test_package_crate() {
    local crate_path=$1
    local crate_name=$2
    
    print_status "Testing package for ${crate_name}..."
    
    cd "${crate_path}"
    
    # Perform dry run packaging
    if cargo package --dry-run > /dev/null 2>&1; then
        print_success "${crate_name} packages successfully"
        cd - > /dev/null
        return 0
    else
        print_error "${crate_name} packaging failed"
        # Show the actual error
        cargo package --dry-run
        cd - > /dev/null
        return 1
    fi
}

main() {
    print_status "Testing VX crates packaging in dependency order..."
    
    # Ensure we're in the project root
    if [[ ! -f "Cargo.toml" ]] || [[ ! -d "crates" ]]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    # Define test order (same as publishing order)
    declare -a TEST_ORDER=(
        # Layer 1: No internal dependencies
        "crates/vx-dependency:vx-dependency"
        "crates/vx-paths:vx-paths"
        
        # Layer 2: Depend on Layer 1
        "crates/vx-plugin:vx-plugin"
        "crates/vx-version:vx-version"
        
        # Layer 3: Depend on Layer 1-2
        "crates/vx-config:vx-config"
        "crates/vx-installer:vx-installer"
        "crates/vx-download:vx-download"
        
        # Layer 4: Depend on Layer 1-3
        "crates/vx-core:vx-core"
        "crates/vx-benchmark:vx-benchmark"
        
        # Layer 5: Tool standard
        "crates/vx-tool-standard:vx-tool-standard"
        
        # Layer 6: Tool implementations
        "crates/vx-tools/vx-tool-npm:vx-tool-npm"
        "crates/vx-tools/vx-tool-uv:vx-tool-uv"
        "crates/vx-tools/vx-tool-python:vx-tool-python"
        "crates/vx-tools/vx-tool-rust:vx-tool-rust"
        "crates/vx-tools/vx-tool-go:vx-tool-go"
        "crates/vx-tools/vx-tool-bun:vx-tool-bun"
        "crates/vx-tools/vx-tool-node:vx-tool-node"
        "crates/vx-tools/vx-tool-pnpm:vx-tool-pnpm"
        "crates/vx-tools/vx-tool-yarn:vx-tool-yarn"
        
        # Layer 7: CLI and main package
        "crates/vx-cli:vx-cli"
        ".:vx"
    )
    
    local failed_crates=()
    local total_crates=${#TEST_ORDER[@]}
    local current=0
    
    for entry in "${TEST_ORDER[@]}"; do
        IFS=':' read -r crate_path crate_name <<< "$entry"
        ((current++))
        
        print_status "[$current/$total_crates] Testing ${crate_name} (${crate_path})..."
        
        if ! test_package_crate "${crate_path}" "${crate_name}"; then
            failed_crates+=("${crate_name}")
        fi
    done
    
    # Summary
    echo
    print_status "Packaging Test Summary:"
    if [ ${#failed_crates[@]} -eq 0 ]; then
        print_success "All crates can be packaged successfully!"
        print_status "Publishing order is correct and ready for crates.io"
    else
        print_error "Some crates failed packaging tests:"
        for crate in "${failed_crates[@]}"; do
            echo "  - ${crate}"
        done
        print_error "Fix these issues before attempting to publish"
        exit 1
    fi
}

# Run main function
main "$@"
