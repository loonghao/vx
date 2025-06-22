#!/bin/bash

# VX Crates Publishing Order Script
# This script publishes crates in the correct dependency order to avoid version conflicts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Function to check if a crate exists on crates.io
check_crate_exists() {
    local crate_name=$1
    local version=$2
    
    if curl -s "https://crates.io/api/v1/crates/${crate_name}/${version}" | grep -q '"version"'; then
        return 0  # exists
    else
        return 1  # doesn't exist
    fi
}

# Function to publish a crate
publish_crate() {
    local crate_path=$1
    local crate_name=$2
    
    print_status "Publishing ${crate_name}..."
    
    cd "${crate_path}"
    
    # Check if already published
    local version=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/' || echo "unknown")
    if [[ "$version" == *"workspace"* ]]; then
        version="0.4.1"  # fallback to workspace version
    fi
    
    if check_crate_exists "${crate_name}" "${version}"; then
        print_warning "${crate_name} v${version} already exists on crates.io, skipping..."
        cd - > /dev/null
        return 0
    fi
    
    # Dry run first
    print_status "Performing dry run for ${crate_name}..."
    if ! cargo publish --dry-run; then
        print_error "Dry run failed for ${crate_name}"
        cd - > /dev/null
        return 1
    fi
    
    # Actual publish
    print_status "Publishing ${crate_name} to crates.io..."
    if cargo publish; then
        print_success "Successfully published ${crate_name} v${version}"
        
        # Wait for crates.io to update
        print_status "Waiting for crates.io to update..."
        sleep 30
        
        # Verify publication
        local retries=5
        while [ $retries -gt 0 ]; do
            if check_crate_exists "${crate_name}" "${version}"; then
                print_success "${crate_name} v${version} is now available on crates.io"
                break
            else
                print_warning "Waiting for ${crate_name} to appear on crates.io... (${retries} retries left)"
                sleep 10
                ((retries--))
            fi
        done
        
        if [ $retries -eq 0 ]; then
            print_error "Failed to verify ${crate_name} publication on crates.io"
            cd - > /dev/null
            return 1
        fi
    else
        print_error "Failed to publish ${crate_name}"
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
    return 0
}

# Main publishing function
main() {
    print_status "Starting VX crates publication in dependency order..."
    
    # Ensure we're in the project root
    if [[ ! -f "Cargo.toml" ]] || [[ ! -d "crates" ]]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    # Define publishing order (dependencies first)
    declare -a PUBLISH_ORDER=(
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
    
    for entry in "${PUBLISH_ORDER[@]}"; do
        IFS=':' read -r crate_path crate_name <<< "$entry"
        
        print_status "Processing ${crate_name} (${crate_path})..."
        
        if ! publish_crate "${crate_path}" "${crate_name}"; then
            failed_crates+=("${crate_name}")
            print_error "Failed to publish ${crate_name}"
            
            # Ask user if they want to continue
            read -p "Continue with remaining crates? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                print_error "Publication stopped by user"
                exit 1
            fi
        fi
    done
    
    # Summary
    echo
    print_status "Publication Summary:"
    if [ ${#failed_crates[@]} -eq 0 ]; then
        print_success "All crates published successfully!"
    else
        print_warning "Some crates failed to publish:"
        for crate in "${failed_crates[@]}"; do
            echo "  - ${crate}"
        done
        exit 1
    fi
}

# Check if --dry-run flag is provided
if [[ "${1:-}" == "--dry-run" ]]; then
    print_status "Dry run mode - no actual publishing will occur"
    # Override publish_crate function for dry run
    publish_crate() {
        local crate_path=$1
        local crate_name=$2
        print_status "DRY RUN: Would publish ${crate_name} from ${crate_path}"
        return 0
    }
fi

# Run main function
main "$@"
