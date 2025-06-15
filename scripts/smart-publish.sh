#!/bin/bash

# VX Smart Publishing Script
# Intelligently publishes workspace packages to crates.io
# - Checks if packages already exist on crates.io
# - Only publishes new/updated versions
# - Handles dependency order automatically
# - Provides detailed logging and error handling

set -euo pipefail

# Configuration
DRY_RUN=${DRY_RUN:-true}
WAIT_TIME=${WAIT_TIME:-30}
FORCE_PUBLISH=${FORCE_PUBLISH:-false}
SKIP_TESTS=${SKIP_TESTS:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_step() { echo -e "${CYAN}🔄 $1${NC}"; }

echo -e "${BLUE}🚀 VX Smart Publishing Script${NC}"
echo -e "${BLUE}==============================${NC}"

if [ "$DRY_RUN" = "true" ]; then
    log_warning "DRY RUN MODE - No actual publishing"
    log_info "Set DRY_RUN=false to actually publish"
else
    log_error "LIVE MODE - Will actually publish to crates.io"
fi

if [ "$FORCE_PUBLISH" = "true" ]; then
    log_warning "FORCE MODE - Will attempt to publish even if version exists"
fi

if [ "$SKIP_TESTS" = "true" ]; then
    log_warning "SKIP TESTS MODE - Will skip running tests"
fi

echo ""

# Publishing order based on dependencies
declare -a packages=(
    "crates/vx-shim"                # Base dependency for vx-core
    "crates/vx-core"                # Core library
    "crates/vx-tools/vx-tool-go"
    "crates/vx-tools/vx-tool-rust"
    "crates/vx-tools/vx-tool-uv"
    "crates/vx-package-managers/vx-pm-npm"
    "crates/vx-tools/vx-tool-node"  # Depends on vx-pm-npm
    "crates/vx-cli"                 # Depends on all tools
    "."                             # Main package depends on everything
)

# Function to check if package exists on crates.io
check_package_exists() {
    local package_name=$1
    local version=$2
    
    log_step "Checking if $package_name@$version exists on crates.io..."
    
    # Use cargo search to check if package exists
    local search_result=$(cargo search "$package_name" --limit 1 2>/dev/null || echo "")
    
    if echo "$search_result" | grep -q "^$package_name = "; then
        local published_version=$(echo "$search_result" | grep "^$package_name = " | sed 's/.*= "\([^"]*\)".*/\1/')
        if [ "$published_version" = "$version" ]; then
            log_warning "$package_name@$version already exists on crates.io"
            return 0
        else
            log_info "$package_name exists but with different version: $published_version (local: $version)"
            return 1
        fi
    else
        log_success "$package_name not found on crates.io - ready to publish"
        return 1
    fi
}

# Function to get package metadata
get_package_metadata() {
    local package_dir=$1
    local manifest_path="Cargo.toml"
    
    if [ "$package_dir" != "." ]; then
        manifest_path="$package_dir/Cargo.toml"
    fi
    
    if [ ! -f "$manifest_path" ]; then
        log_error "Cargo.toml not found at $manifest_path"
        return 1
    fi
    
    # Use cargo metadata to get package info
    local metadata=$(cargo metadata --no-deps --format-version 1 --manifest-path "$manifest_path" 2>/dev/null)
    
    # Extract name and version using grep and sed (no jq dependency)
    local name=$(echo "$metadata" | grep -o '"name":"[^"]*"' | head -1 | sed 's/"name":"\([^"]*\)"/\1/')
    local version=$(echo "$metadata" | grep -o '"version":"[^"]*"' | head -1 | sed 's/"version":"\([^"]*\)"/\1/')
    
    if [ -z "$name" ] || [ -z "$version" ]; then
        log_error "Failed to extract package name or version from $manifest_path"
        return 1
    fi
    
    echo "$name:$version"
}

# Function to validate package before publishing
validate_package() {
    local package_dir=$1
    local package_name=$2
    
    log_step "Validating $package_name..."
    
    # Change to package directory
    local original_dir=$(pwd)
    if [ "$package_dir" != "." ]; then
        cd "$package_dir"
    fi
    
    # Check if Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found in $package_dir"
        cd "$original_dir"
        return 1
    fi
    
    # Build the package
    log_step "Building $package_name..."
    if ! cargo build --release; then
        log_error "Build failed for $package_name"
        cd "$original_dir"
        return 1
    fi
    
    # Run tests (unless skipped)
    if [ "$SKIP_TESTS" != "true" ]; then
        log_step "Testing $package_name..."
        if ! cargo test; then
            log_error "Tests failed for $package_name"
            cd "$original_dir"
            return 1
        fi
    fi
    
    # Dry run publish
    log_step "Dry run publish for $package_name..."
    if ! cargo publish --dry-run; then
        log_error "Dry run failed for $package_name"
        cd "$original_dir"
        return 1
    fi
    
    cd "$original_dir"
    log_success "Validation passed for $package_name"
    return 0
}

# Function to publish a package
publish_package() {
    local package_dir=$1
    local package_info=$(get_package_metadata "$package_dir")
    local package_name=$(echo "$package_info" | cut -d: -f1)
    local package_version=$(echo "$package_info" | cut -d: -f2)
    
    echo -e "${PURPLE}📦 Processing $package_name@$package_version${NC}"
    echo -e "${PURPLE}   Directory: $package_dir${NC}"
    
    # Check if package already exists (unless force mode)
    if [ "$FORCE_PUBLISH" != "true" ]; then
        if check_package_exists "$package_name" "$package_version"; then
            log_warning "Skipping $package_name (already published)"
            echo ""
            return 0
        fi
    fi
    
    # Validate package
    if ! validate_package "$package_dir" "$package_name"; then
        log_error "Validation failed for $package_name"
        return 1
    fi
    
    # Publish if not dry run
    if [ "$DRY_RUN" = "false" ]; then
        local original_dir=$(pwd)
        if [ "$package_dir" != "." ]; then
            cd "$package_dir"
        fi
        
        log_step "Publishing $package_name to crates.io..."
        if cargo publish; then
            log_success "Successfully published $package_name@$package_version"
            
            log_info "Waiting ${WAIT_TIME} seconds for crates.io to update..."
            sleep "$WAIT_TIME"
        else
            log_error "Failed to publish $package_name"
            cd "$original_dir"
            return 1
        fi
        
        cd "$original_dir"
    else
        log_info "Dry run completed for $package_name"
    fi
    
    echo ""
    return 0
}

# Main execution
log_info "Analyzing workspace packages..."
echo ""

# Display publishing plan
echo -e "${BLUE}📋 Publishing Plan:${NC}"
for package in "${packages[@]}"; do
    if package_info=$(get_package_metadata "$package"); then
        package_name=$(echo "$package_info" | cut -d: -f1)
        package_version=$(echo "$package_info" | cut -d: -f2)
        echo -e "  ${GREEN}$package_name@$package_version${NC} ($package)"
    else
        log_error "Failed to get metadata for $package"
        exit 1
    fi
done
echo ""

# Confirmation for live mode
if [ "$DRY_RUN" = "false" ]; then
    echo -e "${RED}⚠️  This will publish packages to crates.io!${NC}"
    read -p "Continue with publishing? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Publishing cancelled by user"
        exit 1
    fi
    echo ""
fi

# Publish each package
failed_packages=()
for package in "${packages[@]}"; do
    if ! publish_package "$package"; then
        package_info=$(get_package_metadata "$package")
        package_name=$(echo "$package_info" | cut -d: -f1)
        failed_packages+=("$package_name")
        
        if [ "$DRY_RUN" = "false" ]; then
            log_error "Failed to publish $package_name - stopping here"
            break
        fi
    fi
done

# Summary
echo -e "${BLUE}📊 Summary:${NC}"
if [ ${#failed_packages[@]} -eq 0 ]; then
    if [ "$DRY_RUN" = "true" ]; then
        log_success "All packages passed validation!"
        log_info "To actually publish, run: DRY_RUN=false $0"
    else
        log_success "All packages published successfully!"
        log_success "Users can now install with: cargo install vx"
    fi
else
    log_error "Failed packages: ${failed_packages[*]}"
    exit 1
fi
