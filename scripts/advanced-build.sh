#!/bin/bash
# Advanced build script with all optimizations
# Supports: sccache, PGO, UPX compression, cross-compilation, and parallel builds

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="vx"
DEFAULT_TARGET="x86_64-unknown-linux-gnu"
PGO_DATA_DIR="pgo-data"
SCCACHE_ENABLED=${SCCACHE_ENABLED:-true}
UPX_ENABLED=${UPX_ENABLED:-true}
STRIP_ENABLED=${STRIP_ENABLED:-true}
PARALLEL_JOBS=${CARGO_BUILD_JOBS:-}

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

function log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

function show_help() {
    cat << EOF
Advanced Build Script for vx

Usage: $0 [OPTIONS] [TARGET]

OPTIONS:
    -h, --help          Show this help message
    -t, --target        Target platform (default: $DEFAULT_TARGET)
    -p, --pgo           Enable Profile-Guided Optimization
    -s, --strip         Enable symbol stripping (default: enabled)
    -u, --upx           Enable UPX compression (default: enabled)
    -c, --clean         Clean build before starting
    -j, --jobs          Number of parallel jobs (default: auto)
    --no-sccache        Disable sccache
    --no-upx            Disable UPX compression
    --no-strip          Disable symbol stripping
    --benchmark         Run performance benchmark after build
    --size-analysis     Show binary size analysis

TARGETS:
    x86_64-unknown-linux-gnu    Linux x64 (default)
    x86_64-unknown-linux-musl   Linux x64 (musl)
    aarch64-unknown-linux-gnu   Linux ARM64
    x86_64-apple-darwin         macOS x64
    aarch64-apple-darwin        macOS ARM64
    x86_64-pc-windows-msvc      Windows x64 (MSVC)
    x86_64-pc-windows-gnu       Windows x64 (GNU)

EXAMPLES:
    $0                                  # Build for default target
    $0 -p x86_64-apple-darwin          # PGO build for macOS
    $0 --clean --benchmark              # Clean build with benchmark
    $0 -t aarch64-unknown-linux-gnu     # Cross-compile for ARM64

EOF
}

function setup_environment() {
    log_step "Setting up build environment..."
    
    # Set optimal environment variables
    export CARGO_TERM_COLOR=always
    export CARGO_INCREMENTAL=1
    export CARGO_NET_RETRY=10
    export RUST_BACKTRACE=short
    # Only set CARGO_BUILD_JOBS if PARALLEL_JOBS is set and not 0
    if [[ -n "$PARALLEL_JOBS" && "$PARALLEL_JOBS" != "0" ]]; then
        export CARGO_BUILD_JOBS=$PARALLEL_JOBS
    fi
    
    # Setup sccache if enabled
    if [ "$SCCACHE_ENABLED" = "true" ] && command -v sccache >/dev/null 2>&1; then
        export RUSTC_WRAPPER=sccache
        log_info "sccache enabled"
        sccache --show-stats || true
    else
        log_warning "sccache not available or disabled"
    fi
    
    # Setup linker optimizations
    case "$TARGET" in
        x86_64-unknown-linux-*)
            export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-fuse-ld=lld"
            ;;
        x86_64-apple-darwin|aarch64-apple-darwin)
            export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-fuse-ld=lld"
            ;;
        x86_64-pc-windows-gnu)
            export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-fuse-ld=lld"
            ;;
    esac
    
    log_success "Environment configured"
}

function install_target() {
    local target=$1
    log_step "Installing Rust target: $target"
    
    if rustup target list --installed | grep -q "$target"; then
        log_info "Target $target already installed"
    else
        rustup target add "$target"
        log_success "Target $target installed"
    fi
}

function check_dependencies() {
    log_step "Checking build dependencies..."
    
    # Check for required tools
    local missing_tools=()
    
    if [ "$UPX_ENABLED" = "true" ] && ! command -v upx >/dev/null 2>&1; then
        missing_tools+=("upx")
    fi
    
    if [ "$STRIP_ENABLED" = "true" ]; then
        case "$TARGET" in
            aarch64-unknown-linux-gnu)
                if ! command -v aarch64-linux-gnu-strip >/dev/null 2>&1; then
                    missing_tools+=("gcc-aarch64-linux-gnu")
                fi
                ;;
            x86_64-pc-windows-gnu)
                if ! command -v x86_64-w64-mingw32-strip >/dev/null 2>&1; then
                    missing_tools+=("gcc-mingw-w64-x86-64")
                fi
                ;;
        esac
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        log_warning "Missing tools: ${missing_tools[*]}"
        log_info "Install with: sudo apt-get install ${missing_tools[*]}"
    else
        log_success "All dependencies available"
    fi
}

function build_binary() {
    local target=$1
    local use_pgo=$2
    
    log_step "Building binary for $target (PGO: $use_pgo)"
    
    if [ "$use_pgo" = "true" ]; then
        bash scripts/goreleaser-pgo.sh "$target"
    else
        # Build with or without explicit jobs parameter
        if [[ -n "$PARALLEL_JOBS" && "$PARALLEL_JOBS" != "0" ]]; then
            cargo build --release --target "$target" --package vx --jobs="$PARALLEL_JOBS"
        else
            cargo build --release --target "$target" --package vx
        fi
    fi
    
    log_success "Binary built successfully"
}

function optimize_binary() {
    local target=$1
    local binary_path="target/$target/release/$BINARY_NAME"
    
    # Add .exe extension for Windows
    if [[ "$target" == *"windows"* ]]; then
        binary_path="${binary_path}.exe"
    fi
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found: $binary_path"
        return 1
    fi
    
    # Strip symbols
    if [ "$STRIP_ENABLED" = "true" ] && [[ "$target" != *"windows-msvc"* ]]; then
        log_step "Stripping symbols from binary..."
        
        case "$target" in
            aarch64-unknown-linux-gnu)
                aarch64-linux-gnu-strip "$binary_path"
                ;;
            x86_64-pc-windows-gnu)
                x86_64-w64-mingw32-strip "$binary_path"
                ;;
            *)
                strip "$binary_path" 2>/dev/null || log_warning "Strip failed, continuing..."
                ;;
        esac
        
        log_success "Symbols stripped"
    fi
    
    # UPX compression
    if [ "$UPX_ENABLED" = "true" ] && command -v upx >/dev/null 2>&1; then
        log_step "Compressing binary with UPX..."
        
        local original_size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")
        
        if upx --best --lzma "$binary_path" 2>/dev/null; then
            local compressed_size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")
            
            if [ "$original_size" != "unknown" ] && [ "$compressed_size" != "unknown" ]; then
                local reduction=$((100 - (compressed_size * 100 / original_size)))
                log_success "UPX compression completed (${reduction}% reduction)"
            else
                log_success "UPX compression completed"
            fi
        else
            log_warning "UPX compression failed, continuing..."
        fi
    fi
}

function show_binary_info() {
    local target=$1
    local binary_path="target/$target/release/$BINARY_NAME"
    
    if [[ "$target" == *"windows"* ]]; then
        binary_path="${binary_path}.exe"
    fi
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found: $binary_path"
        return 1
    fi
    
    log_step "Binary information:"
    
    # File size
    local size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")
    if [ "$size" != "unknown" ]; then
        local size_mb=$(echo "scale=2; $size / 1024 / 1024" | bc -l 2>/dev/null || echo "unknown")
        echo "  Size: ${size_mb}MB (${size} bytes)"
    fi
    
    # File type
    if command -v file >/dev/null 2>&1; then
        echo "  Type: $(file "$binary_path" | cut -d: -f2- | sed 's/^ *//')"
    fi
    
    # Verify binary works
    if timeout 5s "$binary_path" version >/dev/null 2>&1; then
        echo "  Status: ✅ Working"
    else
        echo "  Status: ❌ Not working or timeout"
    fi
}

function run_benchmark() {
    local target=$1
    local binary_path="target/$target/release/$BINARY_NAME"
    
    if [[ "$target" == *"windows"* ]]; then
        binary_path="${binary_path}.exe"
    fi
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found for benchmark: $binary_path"
        return 1
    fi
    
    log_step "Running performance benchmark..."
    
    echo "| Command | Time (ms) | Status |"
    echo "|---------|-----------|--------|"
    
    # Benchmark version command
    if time_result=$(timeout 10s time -f "%e" "$binary_path" version 2>&1 >/dev/null | tail -1 2>/dev/null); then
        time_ms=$(echo "$time_result * 1000" | bc -l 2>/dev/null | cut -d. -f1)
        echo "| \`$BINARY_NAME version\` | ${time_ms} | ✅ |"
    else
        echo "| \`$BINARY_NAME version\` | timeout | ❌ |"
    fi
    
    # Benchmark help command
    if time_result=$(timeout 10s time -f "%e" "$binary_path" --help 2>&1 >/dev/null | tail -1 2>/dev/null); then
        time_ms=$(echo "$time_result * 1000" | bc -l 2>/dev/null | cut -d. -f1)
        echo "| \`$BINARY_NAME --help\` | ${time_ms} | ✅ |"
    else
        echo "| \`$BINARY_NAME --help\` | timeout | ❌ |"
    fi
}

# Parse command line arguments
TARGET="$DEFAULT_TARGET"
USE_PGO=false
CLEAN_BUILD=false
RUN_BENCHMARK=false
SHOW_SIZE_ANALYSIS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -p|--pgo)
            USE_PGO=true
            shift
            ;;
        -s|--strip)
            STRIP_ENABLED=true
            shift
            ;;
        -u|--upx)
            UPX_ENABLED=true
            shift
            ;;
        -c|--clean)
            CLEAN_BUILD=true
            shift
            ;;
        -j|--jobs)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        --no-sccache)
            SCCACHE_ENABLED=false
            shift
            ;;
        --no-upx)
            UPX_ENABLED=false
            shift
            ;;
        --no-strip)
            STRIP_ENABLED=false
            shift
            ;;
        --benchmark)
            RUN_BENCHMARK=true
            shift
            ;;
        --size-analysis)
            SHOW_SIZE_ANALYSIS=true
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            TARGET="$1"
            shift
            ;;
    esac
done

# Main execution
main() {
    log_info "Starting advanced build for target: $TARGET"
    
    # Clean build if requested
    if [ "$CLEAN_BUILD" = "true" ]; then
        log_step "Cleaning previous builds..."
        cargo clean
        rm -rf "$PGO_DATA_DIR" 2>/dev/null || true
        log_success "Clean completed"
    fi
    
    # Setup environment
    setup_environment
    
    # Install target
    install_target "$TARGET"
    
    # Check dependencies
    check_dependencies
    
    # Build binary
    build_binary "$TARGET" "$USE_PGO"
    
    # Optimize binary
    optimize_binary "$TARGET"
    
    # Show binary information
    show_binary_info "$TARGET"
    
    # Run benchmark if requested
    if [ "$RUN_BENCHMARK" = "true" ]; then
        run_benchmark "$TARGET"
    fi
    
    # Show sccache stats
    if [ "$SCCACHE_ENABLED" = "true" ] && command -v sccache >/dev/null 2>&1; then
        log_step "sccache statistics:"
        sccache --show-stats || true
    fi
    
    log_success "Advanced build completed successfully!"
}

# Run main function
main "$@"
