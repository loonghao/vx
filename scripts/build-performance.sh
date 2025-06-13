#!/bin/bash
# Build performance monitoring script for vx
# This script measures and reports build times for different configurations

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

function format_time() {
    local seconds=$1
    local minutes=$((seconds / 60))
    local remaining_seconds=$((seconds % 60))
    
    if [ $minutes -gt 0 ]; then
        echo "${minutes}m ${remaining_seconds}s"
    else
        echo "${remaining_seconds}s"
    fi
}

function benchmark_build() {
    local build_type=$1
    local target=${2:-"x86_64-unknown-linux-gnu"}
    local description=$3
    
    log_info "Benchmarking $description..."
    
    # Clean previous builds
    cargo clean --target "$target" >/dev/null 2>&1 || true
    
    # Measure build time
    local start_time=$(date +%s)
    
    case $build_type in
        "standard")
            cargo build --release --target "$target" >/dev/null 2>&1
            ;;
        "pgo")
            bash scripts/goreleaser-pgo.sh "$target" >/dev/null 2>&1
            ;;
        "dev")
            cargo build --target "$target" >/dev/null 2>&1
            ;;
        *)
            log_error "Unknown build type: $build_type"
            return 1
            ;;
    esac
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_success "$description completed in $(format_time $duration)"
    echo "$duration"
}

function main() {
    log_info "Starting build performance benchmarks..."
    echo ""
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Set optimal build environment
    export CARGO_BUILD_JOBS=0
    export CARGO_INCREMENTAL=1
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    
    # Benchmark different build types
    local target="x86_64-unknown-linux-gnu"
    
    echo "| Build Type | Target | Time | Description |"
    echo "|------------|--------|------|-------------|"
    
    # Development build
    local dev_time
    dev_time=$(benchmark_build "dev" "$target" "Development build")
    echo "| Development | $target | $(format_time $dev_time) | Fast compilation for development |"
    
    # Standard release build
    local standard_time
    standard_time=$(benchmark_build "standard" "$target" "Standard release build")
    echo "| Standard Release | $target | $(format_time $standard_time) | Optimized release build |"
    
    # PGO build (if supported)
    if command -v llvm-profdata >/dev/null 2>&1; then
        local pgo_time
        pgo_time=$(benchmark_build "pgo" "$target" "PGO-optimized build")
        echo "| PGO Release | $target | $(format_time $pgo_time) | Profile-guided optimization |"
        
        # Calculate PGO overhead
        local pgo_overhead=$((pgo_time - standard_time))
        local pgo_overhead_percent=$(( (pgo_overhead * 100) / standard_time ))
        
        echo ""
        log_info "PGO overhead: $(format_time $pgo_overhead) (+${pgo_overhead_percent}%)"
    else
        log_warning "llvm-profdata not found, skipping PGO benchmark"
        echo "| PGO Release | $target | N/A | llvm-profdata not available |"
    fi
    
    echo ""
    log_success "Build performance benchmarks completed!"
    
    # Performance recommendations
    echo ""
    log_info "Performance Recommendations:"
    echo "  • Use 'cargo build --jobs=0' to utilize all CPU cores"
    echo "  • Enable incremental compilation with CARGO_INCREMENTAL=1"
    echo "  • Use lld linker for faster linking: RUSTFLAGS='-C link-arg=-fuse-ld=lld'"
    echo "  • Consider PGO for production builds (adds build time but improves runtime performance)"
    echo "  • Use 'cargo clean' between major changes to avoid incremental compilation issues"
}

# Run the benchmark
main "$@"
