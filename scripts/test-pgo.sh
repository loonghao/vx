#!/bin/bash
# Test script to verify PGO optimization effectiveness
# Compares performance between standard and PGO builds

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function log_step() {
    echo -e "${BLUE}[TEST]${NC} $1"
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

TARGET="x86_64-unknown-linux-gnu"
if [[ "$OSTYPE" == "darwin"* ]]; then
    TARGET="x86_64-apple-darwin"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    TARGET="x86_64-pc-windows-gnu"
fi

BINARY_EXT=""
if [[ "$TARGET" == *"windows"* ]]; then
    BINARY_EXT=".exe"
fi

STANDARD_BINARY="target/$TARGET/release/vx$BINARY_EXT"
PGO_BINARY="target-pgo/$TARGET/release/vx$BINARY_EXT"

log_step "Building standard release binary..."
cargo build --release --target "$TARGET"

if [[ ! -f "$STANDARD_BINARY" ]]; then
    log_error "Standard binary not found at $STANDARD_BINARY"
    exit 1
fi

log_step "Building PGO-optimized binary..."
mkdir -p target-pgo
export CARGO_TARGET_DIR="target-pgo"

# Run PGO build
if [[ -f "scripts/goreleaser-pgo.sh" ]]; then
    chmod +x scripts/goreleaser-pgo.sh
    ./scripts/goreleaser-pgo.sh "$TARGET"
else
    log_warning "PGO script not found, using manual PGO build"
    
    # Manual PGO build
    PGO_DATA_DIR="pgo-data-test"
    mkdir -p "$PGO_DATA_DIR"
    
    # Build instrumented
    RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR" cargo build --release --target "$TARGET"
    
    # Run training
    if [[ -f "$PGO_BINARY" ]]; then
        chmod +x "$PGO_BINARY"
        "$PGO_BINARY" version >/dev/null 2>&1 || true
        "$PGO_BINARY" --help >/dev/null 2>&1 || true
        "$PGO_BINARY" list >/dev/null 2>&1 || true
    fi
    
    # Build optimized
    if command -v llvm-profdata >/dev/null 2>&1; then
        llvm-profdata merge -output="$PGO_DATA_DIR/merged.profdata" "$PGO_DATA_DIR"/*.profraw 2>/dev/null || true
        RUSTFLAGS="-Cprofile-use=$PGO_DATA_DIR/merged.profdata" cargo build --release --target "$TARGET"
    else
        RUSTFLAGS="-Cprofile-use=$PGO_DATA_DIR" cargo build --release --target "$TARGET"
    fi
fi

unset CARGO_TARGET_DIR

if [[ ! -f "$PGO_BINARY" ]]; then
    log_error "PGO binary not found at $PGO_BINARY"
    exit 1
fi

log_success "Both binaries built successfully"

# Performance testing
log_step "Running performance comparison..."

# Test commands
TEST_COMMANDS=(
    "version"
    "--help"
    "list"
)

echo ""
echo "📊 Performance Comparison Results"
echo "=================================="
printf "%-15s %-15s %-15s %-10s\n" "Command" "Standard (ms)" "PGO (ms)" "Improvement"
echo "-----------------------------------------------------------"

TOTAL_STANDARD=0
TOTAL_PGO=0
TEST_COUNT=0

for cmd in "${TEST_COMMANDS[@]}"; do
    # Test standard binary
    STANDARD_TIME=$(time -f "%e" "$STANDARD_BINARY" $cmd >/dev/null 2>&1 | tail -1 2>/dev/null || echo "0.000")
    STANDARD_MS=$(echo "$STANDARD_TIME * 1000" | bc 2>/dev/null || echo "0")
    
    # Test PGO binary  
    PGO_TIME=$(time -f "%e" "$PGO_BINARY" $cmd >/dev/null 2>&1 | tail -1 2>/dev/null || echo "0.000")
    PGO_MS=$(echo "$PGO_TIME * 1000" | bc 2>/dev/null || echo "0")
    
    # Calculate improvement
    if [[ $(echo "$STANDARD_MS > 0" | bc 2>/dev/null || echo "0") -eq 1 ]]; then
        IMPROVEMENT=$(echo "scale=1; (($STANDARD_MS - $PGO_MS) / $STANDARD_MS) * 100" | bc 2>/dev/null || echo "0")
        IMPROVEMENT_STR="${IMPROVEMENT}%"
        
        TOTAL_STANDARD=$(echo "$TOTAL_STANDARD + $STANDARD_MS" | bc 2>/dev/null || echo "$TOTAL_STANDARD")
        TOTAL_PGO=$(echo "$TOTAL_PGO + $PGO_MS" | bc 2>/dev/null || echo "$TOTAL_PGO")
        TEST_COUNT=$((TEST_COUNT + 1))
    else
        IMPROVEMENT_STR="N/A"
    fi
    
    printf "%-15s %-15.1f %-15.1f %-10s\n" "$cmd" "$STANDARD_MS" "$PGO_MS" "$IMPROVEMENT_STR"
done

echo "-----------------------------------------------------------"

if [[ $TEST_COUNT -gt 0 && $(echo "$TOTAL_STANDARD > 0" | bc 2>/dev/null || echo "0") -eq 1 ]]; then
    OVERALL_IMPROVEMENT=$(echo "scale=1; (($TOTAL_STANDARD - $TOTAL_PGO) / $TOTAL_STANDARD) * 100" | bc 2>/dev/null || echo "0")
    printf "%-15s %-15.1f %-15.1f %-10s\n" "OVERALL" "$TOTAL_STANDARD" "$TOTAL_PGO" "${OVERALL_IMPROVEMENT}%"
    
    echo ""
    if [[ $(echo "$OVERALL_IMPROVEMENT > 0" | bc 2>/dev/null || echo "0") -eq 1 ]]; then
        log_success "PGO optimization effective: ${OVERALL_IMPROVEMENT}% improvement"
    else
        log_warning "PGO optimization shows minimal or no improvement"
    fi
else
    log_warning "Unable to calculate performance metrics"
fi

# Binary size comparison
log_step "Comparing binary sizes..."

STANDARD_SIZE=$(stat -f%z "$STANDARD_BINARY" 2>/dev/null || stat -c%s "$STANDARD_BINARY" 2>/dev/null || echo "0")
PGO_SIZE=$(stat -f%z "$PGO_BINARY" 2>/dev/null || stat -c%s "$PGO_BINARY" 2>/dev/null || echo "0")

echo ""
echo "📦 Binary Size Comparison"
echo "========================="
echo "Standard binary: $(numfmt --to=iec $STANDARD_SIZE 2>/dev/null || echo "$STANDARD_SIZE bytes")"
echo "PGO binary:      $(numfmt --to=iec $PGO_SIZE 2>/dev/null || echo "$PGO_SIZE bytes")"

if [[ $STANDARD_SIZE -gt 0 && $PGO_SIZE -gt 0 ]]; then
    SIZE_DIFF=$(echo "scale=1; (($PGO_SIZE - $STANDARD_SIZE) / $STANDARD_SIZE) * 100" | bc 2>/dev/null || echo "0")
    if [[ $(echo "$SIZE_DIFF > 0" | bc 2>/dev/null || echo "0") -eq 1 ]]; then
        echo "Size increase:   +${SIZE_DIFF}%"
    else
        echo "Size change:     ${SIZE_DIFF}%"
    fi
fi

echo ""
log_success "Performance testing completed"

# Cleanup
rm -rf pgo-data-test target-pgo
