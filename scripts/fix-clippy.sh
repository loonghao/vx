#!/bin/bash
# Clippy fix script for vx project
# Automatically fixes common clippy warnings and runs checks

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

function log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

function log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

function log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

function log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

function show_help() {
    cat << EOF
Clippy Fix Script for vx

Usage: $0 [OPTIONS]

OPTIONS:
    -h, --help          Show this help message
    -f, --fix           Automatically fix clippy warnings
    -c, --check         Only check for clippy warnings (no fixes)
    -w, --workspace     Run on entire workspace
    -p, --package       Run on specific package
    --pedantic          Use pedantic clippy lints
    --nursery           Use nursery clippy lints
    --all-features      Check with all features enabled
    --no-deps           Don't check dependencies

EXAMPLES:
    $0                          # Basic clippy check
    $0 --fix                    # Fix clippy warnings automatically
    $0 --workspace --fix        # Fix entire workspace
    $0 --package vx-core        # Check specific package
    $0 --pedantic --nursery     # Use strict lints

EOF
}

function run_clippy_check() {
    local args=("$@")
    
    log_info "Running clippy check with args: ${args[*]}"
    
    if cargo clippy "${args[@]}" -- -D warnings; then
        log_success "Clippy check passed!"
        return 0
    else
        log_error "Clippy check failed!"
        return 1
    fi
}

function run_clippy_fix() {
    local args=("$@")
    
    log_info "Running clippy fix with args: ${args[*]}"
    
    if cargo clippy "${args[@]}" --fix --allow-dirty --allow-staged -- -D warnings; then
        log_success "Clippy fix completed!"
        return 0
    else
        log_error "Clippy fix failed!"
        return 1
    fi
}

function check_cargo_config() {
    log_info "Checking Cargo configuration..."
    
    local config_file=".cargo/config.toml"
    
    if [ -f "$config_file" ]; then
        # Check for problematic jobs = 0 setting
        if grep -q "^jobs = 0" "$config_file"; then
            log_warning "Found 'jobs = 0' in $config_file, this may cause issues"
            log_info "Consider removing this line or setting it to a specific number"
        fi
        
        # Check for sccache wrapper
        if grep -q "rustc-wrapper.*sccache" "$config_file"; then
            if ! command -v sccache >/dev/null 2>&1; then
                log_warning "sccache is configured but not installed"
                log_info "Install with: cargo install sccache"
            fi
        fi
    fi
}

function main() {
    local fix_mode=false
    local check_only=false
    local workspace=false
    local package=""
    local pedantic=false
    local nursery=false
    local all_features=false
    local no_deps=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -f|--fix)
                fix_mode=true
                shift
                ;;
            -c|--check)
                check_only=true
                shift
                ;;
            -w|--workspace)
                workspace=true
                shift
                ;;
            -p|--package)
                package="$2"
                shift 2
                ;;
            --pedantic)
                pedantic=true
                shift
                ;;
            --nursery)
                nursery=true
                shift
                ;;
            --all-features)
                all_features=true
                shift
                ;;
            --no-deps)
                no_deps=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    log_info "Starting clippy analysis for vx project..."
    
    # Check Cargo configuration
    check_cargo_config
    
    # Build clippy arguments
    local clippy_args=()
    
    if [ "$workspace" = true ]; then
        clippy_args+=(--workspace)
    elif [ -n "$package" ]; then
        clippy_args+=(--package "$package")
    fi
    
    if [ "$all_features" = true ]; then
        clippy_args+=(--all-features)
    fi
    
    if [ "$no_deps" = false ]; then
        clippy_args+=(--all-targets)
    fi
    
    # Build lint arguments
    local lint_args=()
    
    if [ "$pedantic" = true ]; then
        lint_args+=(-W clippy::pedantic)
    fi
    
    if [ "$nursery" = true ]; then
        lint_args+=(-W clippy::nursery)
    fi
    
    # Add standard strict lints
    lint_args+=(
        -D warnings
        -D clippy::all
        -D clippy::correctness
        -D clippy::suspicious
        -D clippy::complexity
        -D clippy::perf
        -D clippy::style
    )
    
    # Run clippy
    if [ "$fix_mode" = true ] && [ "$check_only" = false ]; then
        log_info "Running clippy in fix mode..."
        
        # First try to fix automatically
        if run_clippy_fix "${clippy_args[@]}"; then
            log_success "Automatic fixes applied successfully"
        else
            log_warning "Some issues couldn't be fixed automatically"
        fi
        
        # Then run check to see remaining issues
        log_info "Checking for remaining issues..."
        run_clippy_check "${clippy_args[@]}"
    else
        log_info "Running clippy in check mode..."
        run_clippy_check "${clippy_args[@]}"
    fi
    
    # Additional checks
    log_info "Running additional code quality checks..."
    
    # Check formatting
    if cargo fmt -- --check >/dev/null 2>&1; then
        log_success "Code formatting is correct"
    else
        log_warning "Code formatting issues found. Run 'cargo fmt' to fix."
    fi
    
    # Check for unused dependencies (if cargo-machete is available)
    if command -v cargo-machete >/dev/null 2>&1; then
        log_info "Checking for unused dependencies..."
        if cargo machete; then
            log_success "No unused dependencies found"
        else
            log_warning "Unused dependencies detected"
        fi
    fi
    
    log_success "Clippy analysis completed!"
}

# Check if we're in a Rust project
if [ ! -f "Cargo.toml" ]; then
    log_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Run main function
main "$@"
