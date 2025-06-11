#!/bin/bash
# PGO build script for GoReleaser integration
# This script is called by GoReleaser to build PGO-optimized binaries

set -euo pipefail

TARGET="$1"
BINARY_NAME="vx"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function log_step() {
    echo -e "${BLUE}[PGO]${NC} $1"
}

function log_success() {
    echo -e "${GREEN}[PGO]${NC} $1"
}

function log_warning() {
    echo -e "${YELLOW}[PGO]${NC} $1"
}

# Determine binary extension
BINARY_EXT=""
if [[ "$TARGET" == *"windows"* ]]; then
    BINARY_EXT=".exe"
fi

BINARY_PATH="target/$TARGET/release/$BINARY_NAME$BINARY_EXT"
PGO_DATA_DIR="pgo-data-$TARGET"

log_step "Starting PGO optimization for $TARGET"

# Create PGO data directory
mkdir -p "$PGO_DATA_DIR"

# Step 1: Build instrumented binary
log_step "Building instrumented binary for profile collection..."
export RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR"

if ! cargo build --release --target "$TARGET"; then
    log_warning "Failed to build instrumented binary, falling back to standard build"
    unset RUSTFLAGS
    cargo build --release --target "$TARGET"
    exit 0
fi

# Step 2: Run training workload (if binary exists and is executable)
if [[ -f "$BINARY_PATH" ]]; then
    log_step "Collecting profile data with training workload..."
    
    # Make binary executable on Unix systems
    if [[ "$TARGET" != *"windows"* ]]; then
        chmod +x "$BINARY_PATH"
    fi
    
    # Define training commands
    TRAINING_COMMANDS=(
        "version"
        "--help"
        "list"
        "plugin list"
        "plugin stats"
        "config"
    )
    
    # Run training commands
    for cmd in "${TRAINING_COMMANDS[@]}"; do
        echo "  Running: $BINARY_NAME $cmd"
        if [[ "$cmd" == "--help" ]]; then
            timeout 10s "$BINARY_PATH" --help >/dev/null 2>&1 || true
        else
            # shellcheck disable=SC2086
            timeout 10s "$BINARY_PATH" $cmd >/dev/null 2>&1 || true
        fi
    done
    
    # Check if profile data was generated
    PROFILE_COUNT=$(find "$PGO_DATA_DIR" -name "*.profraw" 2>/dev/null | wc -l)
    if [[ "$PROFILE_COUNT" -eq 0 ]]; then
        log_warning "No profile data generated, falling back to standard build"
        unset RUSTFLAGS
        cargo build --release --target "$TARGET"
        exit 0
    fi
    
    log_success "Profile data collected: $PROFILE_COUNT files"
    
    # Step 3: Merge profile data
    MERGED_PROFILE="$PGO_DATA_DIR"
    if command -v llvm-profdata >/dev/null 2>&1; then
        log_step "Merging profile data..."
        MERGED_PROFILE="$PGO_DATA_DIR/merged.profdata"
        if llvm-profdata merge -output="$MERGED_PROFILE" "$PGO_DATA_DIR"/*.profraw 2>/dev/null; then
            log_success "Profile data merged successfully"
        else
            log_warning "Profile merging failed, using raw data"
            MERGED_PROFILE="$PGO_DATA_DIR"
        fi
    else
        log_warning "llvm-profdata not found, using raw profile data"
    fi
    
    # Step 4: Build optimized binary
    log_step "Building PGO-optimized binary..."
    export RUSTFLAGS="-Cprofile-use=$MERGED_PROFILE -Cllvm-args=-pgo-warn-missing-function"
    
    if cargo build --release --target "$TARGET"; then
        log_success "PGO-optimized binary built successfully"
        
        # Verify the optimized binary
        if timeout 5s "$BINARY_PATH" version >/dev/null 2>&1; then
            log_success "Binary verification passed"
        else
            log_warning "Binary verification failed, but continuing"
        fi
    else
        log_warning "PGO build failed, falling back to standard build"
        unset RUSTFLAGS
        cargo build --release --target "$TARGET"
    fi
else
    log_warning "Binary not found at $BINARY_PATH, falling back to standard build"
    unset RUSTFLAGS
    cargo build --release --target "$TARGET"
fi

# Cleanup
unset RUSTFLAGS

log_success "PGO process completed for $TARGET"
