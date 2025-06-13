#!/bin/bash
# Simplified PGO build script for GoReleaser integration
# This script provides a more stable PGO implementation with better error handling

set -euo pipefail

TARGET="$1"
BINARY_NAME="vx"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
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

function log_error() {
    echo -e "${RED}[PGO]${NC} $1"
}

# Check if PGO is supported for this target
function is_pgo_supported() {
    case "$TARGET" in
        x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl|x86_64-apple-darwin)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Fallback to standard build
function fallback_build() {
    local reason="$1"
    log_warning "$reason - falling back to standard build"
    unset RUSTFLAGS 2>/dev/null || true
    cargo build --release --target "$TARGET" --package=vx
}

# Determine binary extension and path
BINARY_EXT=""
if [[ "$TARGET" == *"windows"* ]]; then
    BINARY_EXT=".exe"
fi

BINARY_PATH="target/$TARGET/release/$BINARY_NAME$BINARY_EXT"
PGO_DATA_DIR="pgo-data-$TARGET"

log_step "Starting simplified PGO optimization for $TARGET"

# Check if PGO is supported for this target
if ! is_pgo_supported; then
    log_warning "PGO not supported for target $TARGET"
    fallback_build "Unsupported target"
    exit 0
fi

# Check if llvm-profdata is available
if ! command -v llvm-profdata >/dev/null 2>&1; then
    log_warning "llvm-profdata not found"
    fallback_build "Missing llvm-profdata"
    exit 0
fi

# Create PGO data directory
mkdir -p "$PGO_DATA_DIR"

# Step 1: Build instrumented binary
log_step "Building instrumented binary for profile collection..."
export RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR -Ccodegen-units=1 -Copt-level=3"

if ! cargo build --release --target "$TARGET" --package=vx; then
    fallback_build "Instrumented build failed"
    exit 0
fi

# Step 2: Run simplified training workload
if [[ -f "$BINARY_PATH" ]]; then
    log_step "Collecting profile data with simplified training workload..."
    
    # Make binary executable on Unix systems
    if [[ "$TARGET" != *"windows"* ]]; then
        chmod +x "$BINARY_PATH" || true
    fi
    
    # Simplified training commands (only safe, fast commands)
    TRAINING_COMMANDS=(
        "--version"
        "--help"
    )
    
    # Run training commands with better error handling
    for cmd in "${TRAINING_COMMANDS[@]}"; do
        log_step "  Running: $BINARY_NAME $cmd"
        if timeout 5s "$BINARY_PATH" "$cmd" >/dev/null 2>&1; then
            log_step "    Command succeeded"
        else
            log_warning "    Command failed or timed out (continuing)"
        fi
    done
    
    # Check if any profile data was generated
    PROFILE_COUNT=$(find "$PGO_DATA_DIR" -name "*.profraw" 2>/dev/null | wc -l || echo "0")
    if [[ "$PROFILE_COUNT" -eq 0 ]]; then
        fallback_build "No profile data generated"
        exit 0
    fi
    
    log_success "Profile data collected: $PROFILE_COUNT files"
    
    # Step 3: Merge profile data
    log_step "Merging profile data..."
    MERGED_PROFILE="$PGO_DATA_DIR/merged.profdata"
    if llvm-profdata merge -output="$MERGED_PROFILE" "$PGO_DATA_DIR"/*.profraw 2>/dev/null; then
        log_success "Profile data merged successfully"
    else
        fallback_build "Profile merging failed"
        exit 0
    fi
    
    # Step 4: Build optimized binary
    log_step "Building PGO-optimized binary..."
    # Use simpler RUSTFLAGS without target-cpu=native for better compatibility
    export RUSTFLAGS="-Cprofile-use=$MERGED_PROFILE -Ccodegen-units=1 -Copt-level=3"

    if cargo build --release --target "$TARGET" --package=vx; then
        log_success "PGO-optimized binary built successfully"

        # Quick verification
        if timeout 3s "$BINARY_PATH" --version >/dev/null 2>&1; then
            log_success "Binary verification passed"
        else
            log_warning "Binary verification failed, but build completed"
        fi
    else
        fallback_build "PGO optimization build failed"
        exit 0
    fi
else
    fallback_build "Binary not found at $BINARY_PATH"
    exit 0
fi

# Cleanup
unset RUSTFLAGS 2>/dev/null || true
rm -rf "$PGO_DATA_DIR" 2>/dev/null || true

log_success "Simplified PGO process completed successfully for $TARGET"
