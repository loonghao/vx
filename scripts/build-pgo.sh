#!/bin/bash
# PGO (Profile-Guided Optimization) build script for vx
# This script automates the PGO build process for optimal performance

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
CLEAN=false
VERBOSE=false
TARGET=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--clean] [--verbose] [--target TARGET]"
            echo "  --clean    Clean previous builds"
            echo "  --verbose  Enable verbose output"
            echo "  --target   Specify target triple"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Auto-detect target if not specified
if [[ -z "$TARGET" ]]; then
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        TARGET="x86_64-unknown-linux-gnu"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        TARGET="x86_64-apple-darwin"
    else
        echo -e "${RED}[ERROR]${NC} Unsupported OS: $OSTYPE"
        exit 1
    fi
fi

function log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
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

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    log_error "Must be run from the project root directory"
    exit 1
fi

# Clean previous builds if requested
if [[ "$CLEAN" == true ]]; then
    log_step "Cleaning previous builds..."
    cargo clean
    rm -rf pgo-data
fi

# Create PGO data directory
PGO_DATA_DIR="pgo-data"
mkdir -p "$PGO_DATA_DIR"

log_step "Starting PGO optimization process..."

# Step 1: Build with PGO instrumentation
log_step "Building instrumented binary for profile collection..."
export RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR"

if [[ "$VERBOSE" == true ]]; then
    cargo build --release --target "$TARGET"
else
    cargo build --release --target "$TARGET" 2>/dev/null
fi

log_success "Instrumented binary built successfully"

# Step 2: Run training workload to collect profile data
log_step "Collecting profile data with training workload..."

BINARY_PATH="target/$TARGET/release/vx"
if [[ ! -f "$BINARY_PATH" ]]; then
    log_error "Binary not found at $BINARY_PATH"
    exit 1
fi

# Define training commands that represent typical usage
TRAINING_COMMANDS=(
    "version"
    "list"
    "plugin list"
    "plugin stats"
    "config"
    "--help"
)

log_step "Running training workload..."
for cmd in "${TRAINING_COMMANDS[@]}"; do
    echo "  Running: vx $cmd"
    if [[ "$cmd" == "--help" ]]; then
        "$BINARY_PATH" --help >/dev/null 2>&1 || log_warning "Command 'vx $cmd' failed, continuing..."
    else
        # shellcheck disable=SC2086
        "$BINARY_PATH" $cmd >/dev/null 2>&1 || log_warning "Command 'vx $cmd' failed, continuing..."
    fi
done

# Check if profile data was generated
PROFILE_FILES=$(find "$PGO_DATA_DIR" -name "*.profraw" 2>/dev/null | wc -l)
if [[ "$PROFILE_FILES" -eq 0 ]]; then
    log_error "No profile data generated. Check if the binary was built correctly."
    exit 1
fi

log_success "Profile data collected: $PROFILE_FILES files"

# Step 3: Merge profile data
log_step "Merging profile data..."
MERGED_PROFILE="$PGO_DATA_DIR/merged.profdata"

if command -v llvm-profdata >/dev/null 2>&1; then
    llvm-profdata merge -output="$MERGED_PROFILE" "$PGO_DATA_DIR"/*.profraw
    log_success "Profile data merged successfully"
else
    log_warning "llvm-profdata not found, using rustc's built-in merging"
    # Rust will automatically merge .profraw files
    MERGED_PROFILE="$PGO_DATA_DIR"
fi

# Step 4: Build optimized binary using profile data
log_step "Building PGO-optimized binary..."
export RUSTFLAGS="-Cprofile-use=$MERGED_PROFILE -Cllvm-args=-pgo-warn-missing-function"

if [[ "$VERBOSE" == true ]]; then
    cargo build --release --target "$TARGET"
else
    cargo build --release --target "$TARGET" 2>/dev/null
fi

log_success "PGO-optimized binary built successfully"

# Step 5: Verify the optimized binary
log_step "Verifying optimized binary..."
if "$BINARY_PATH" version >/dev/null 2>&1; then
    log_success "Optimized binary verification passed"
else
    log_error "Optimized binary verification failed"
    exit 1
fi

# Cleanup
unset RUSTFLAGS

log_success "PGO optimization completed successfully!"
echo ""
echo -e "${GREEN}Optimized binary location:${NC} $BINARY_PATH"
echo -e "${GREEN}Profile data location:${NC} $PGO_DATA_DIR"
echo ""
echo -e "${YELLOW}Performance improvements:${NC}"
echo "  • Faster startup time"
echo "  • Better branch prediction"
echo "  • Optimized hot code paths"
echo "  • Reduced instruction cache misses"
