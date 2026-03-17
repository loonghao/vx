#!/bin/bash
# Discover all vx providers and generate test matrix for GitHub Actions
#
# Usage:
#   ./discover-providers.sh [options]
#
# Options:
#   --chunk-size N       Number of runtimes per test job (default: 8)
#   --skip RUNTIMES      Comma-separated list of runtimes to skip
#   --runtimes RUNTIMES  Only test these specific runtimes (comma-separated)
#   --output FILE        Write outputs to file (for GitHub Actions)
#   --summary FILE       Write summary to file (for GitHub step summary)
#
# Environment variables:
#   VX_PROVIDERS_DIR     Path to providers directory (default: crates/vx-providers)
#
# Output format (to stdout or --output file):
#   matrix-linux=["chunk1", "chunk2", ...]
#   matrix-macos=["chunk1", "chunk2", ...]
#   matrix-windows=["chunk1", "chunk2", ...]
#   has-linux=true|false
#   has-macos=true|false
#   has-windows=true|false

set -euo pipefail

# Default values
CHUNK_SIZE=8
SKIP_LIST=""
RUNTIME_FILTER=""
OUTPUT_FILE=""
SUMMARY_FILE=""
PROVIDERS_DIR="${VX_PROVIDERS_DIR:-crates/vx-providers}"

# Known problematic runtimes that can't be tested in CI
# - msbuild, msvc: require Visual Studio installation
# - systemctl, journalctl, etc: Linux system services (can't test in CI)
# - choco: Windows package manager (requires elevation)
# - xcodebuild, xcrun, xcode-select: macOS only, requires full Xcode
# - swift, swiftc: requires Xcode toolchain
# - make: no download URL available (system tool only) - TODO: add system_install
# - awscli, azcli: use MSI installer format, can't be extracted in CI
# - curl: only has manifest, no implementation
# - nasm: not registered in provider registry
# - rust, rustc, cargo, rustup: require system install (winget/brew), not suitable for CI
# - ollama: download URL issues with proxy
# - python: requires system install, network timeout issues
# - brew, homebrew: require script installation, not suitable for CI
SKIP_ALWAYS="msbuild,msvc,systemctl,journalctl,systemd-analyze,loginctl,choco,xcodebuild,xcrun,xcode-select,swift,swiftc,make,awscli,aws,azcli,az,curl,nasm,rust,rustc,cargo,rustup,ollama,python,brew,homebrew"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --chunk-size)
            CHUNK_SIZE="$2"
            shift 2
            ;;
        --skip)
            SKIP_LIST="$2"
            shift 2
            ;;
        --runtimes)
            RUNTIME_FILTER="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --summary)
            SUMMARY_FILE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Combine user skip list with always-skipped
if [ -n "$SKIP_LIST" ]; then
    SKIP_ALWAYS="$SKIP_ALWAYS,$SKIP_LIST"
fi

echo "Discovering providers from $PROVIDERS_DIR..."

if command -v cargo >/dev/null 2>&1; then
    CARGO_CMD=(cargo)
elif command -v cargo.exe >/dev/null 2>&1; then
    CARGO_CMD=(cargo.exe)
else
    echo "cargo is required for provider discovery but was not found on PATH" >&2
    exit 1
fi

DISCOVERY_CMD=("${CARGO_CMD[@]}" run --quiet --release -p vx-star-metadata --bin vx-star-discover-providers --
    --providers-dir "$PROVIDERS_DIR"
    --chunk-size "$CHUNK_SIZE"
    --skip "$SKIP_ALWAYS")

if [ -n "$RUNTIME_FILTER" ]; then
    DISCOVERY_CMD+=(--runtimes "$RUNTIME_FILTER")
fi

DISCOVERY_OUTPUT=$("${DISCOVERY_CMD[@]}")

TOTAL_RUNTIMES="0"
TESTABLE_RUNTIMES="0"
LINUX_COUNT="0"
MACOS_COUNT="0"
WINDOWS_COUNT="0"
LINUX_RUNTIMES=""
MACOS_RUNTIMES=""
WINDOWS_RUNTIMES=""
LINUX_MATRIX="[]"
MACOS_MATRIX="[]"
WINDOWS_MATRIX="[]"
HAS_LINUX="false"
HAS_MACOS="false"
HAS_WINDOWS="false"

while IFS='=' read -r key value; do
    case "$key" in
        total-runtimes) TOTAL_RUNTIMES="$value" ;;
        testable-runtimes) TESTABLE_RUNTIMES="$value" ;;
        linux-count) LINUX_COUNT="$value" ;;
        macos-count) MACOS_COUNT="$value" ;;
        windows-count) WINDOWS_COUNT="$value" ;;
        linux-runtimes) LINUX_RUNTIMES="$value" ;;
        macos-runtimes) MACOS_RUNTIMES="$value" ;;
        windows-runtimes) WINDOWS_RUNTIMES="$value" ;;
        matrix-linux) LINUX_MATRIX="$value" ;;
        matrix-macos) MACOS_MATRIX="$value" ;;
        matrix-windows) WINDOWS_MATRIX="$value" ;;
        has-linux) HAS_LINUX="$value" ;;
        has-macos) HAS_MACOS="$value" ;;
        has-windows) HAS_WINDOWS="$value" ;;
    esac
done <<< "$DISCOVERY_OUTPUT"

echo "Found $TOTAL_RUNTIMES runtimes"

echo ""
echo "Platform-specific runtimes:"
echo "  Linux:   $LINUX_COUNT runtimes"
echo "  macOS:   $MACOS_COUNT runtimes"
echo "  Windows: $WINDOWS_COUNT runtimes"

echo ""
echo "Matrix chunks (chunk_size=$CHUNK_SIZE):"
echo "  Linux:   $LINUX_MATRIX"
echo "  macOS:   $MACOS_MATRIX"
echo "  Windows: $WINDOWS_MATRIX"

# Output results
output_line() {
    if [ -n "$OUTPUT_FILE" ]; then
        echo "$1" >> "$OUTPUT_FILE"
    else
        echo "$1"
    fi
}

output_line "matrix-linux=$LINUX_MATRIX"
output_line "matrix-macos=$MACOS_MATRIX"
output_line "matrix-windows=$WINDOWS_MATRIX"
output_line "has-linux=$HAS_LINUX"
output_line "has-macos=$HAS_MACOS"
output_line "has-windows=$HAS_WINDOWS"
output_line "total-runtimes=$TOTAL_RUNTIMES"
output_line "testable-runtimes=$TESTABLE_RUNTIMES"

# Generate summary if requested
if [ -n "$SUMMARY_FILE" ]; then
    {
        echo "## Provider Discovery Results"
        echo ""
        echo "### Runtimes per Platform"
        echo "| Platform | Count | Runtimes (first 10) |"
        echo "|----------|-------|---------------------|"
        echo "| Linux | $LINUX_COUNT | $(echo $LINUX_RUNTIMES | tr ' ' '\n' | head -10 | tr '\n' ' ')... |"
        echo "| macOS | $MACOS_COUNT | $(echo $MACOS_RUNTIMES | tr ' ' '\n' | head -10 | tr '\n' ' ')... |"
        echo "| Windows | $WINDOWS_COUNT | $(echo $WINDOWS_RUNTIMES | tr ' ' '\n' | head -10 | tr '\n' ' ')... |"
        echo ""
        echo "### Configuration"
        echo "- Chunk size: $CHUNK_SIZE"
        echo "- Total runtimes discovered: $TOTAL_RUNTIMES"
        echo "- Testable runtimes after CI filters: $TESTABLE_RUNTIMES"
        echo ""
        echo "### Skipped Runtimes (CI-incompatible)"
        echo "\`$SKIP_ALWAYS\`"
    } >> "$SUMMARY_FILE"
fi

echo ""
echo "Discovery complete!"
