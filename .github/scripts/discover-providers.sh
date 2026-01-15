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
# - openssl: system library, varies by platform
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
SKIP_ALWAYS="msbuild,msvc,openssl,systemctl,journalctl,systemd-analyze,loginctl,choco,xcodebuild,xcrun,xcode-select,swift,swiftc,make,awscli,aws,azcli,az,curl,nasm,rust,rustc,cargo,rustup,ollama,python"

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

# Storage for discovered runtimes and their platform constraints
declare -A RUNTIME_PLATFORMS
ALL_RUNTIMES=""

echo "Discovering providers from $PROVIDERS_DIR..."

# Parse each provider.toml file
for dir in "$PROVIDERS_DIR"/*/; do
    MANIFEST="${dir}provider.toml"
    if [ -f "$MANIFEST" ]; then
        PROVIDER_NAME=$(basename "$dir")
        
        # Extract provider-level platform constraint
        # Look for: [provider.platforms] followed by os = ["windows"] etc.
        PROVIDER_PLATFORMS=$(grep -A1 '^\[provider\.platforms\]' "$MANIFEST" 2>/dev/null | \
            grep 'os = ' | \
            sed 's/.*\[\([^]]*\)\].*/\1/' | \
            tr -d '"' | tr -d ' ' || echo "")
        
        # Get all runtime names using Python for reliable TOML parsing
        RUNTIME_NAMES=$(python3 << 'PYTHON_EOF'
import re
import sys

manifest = sys.argv[1] if len(sys.argv) > 1 else ""
try:
    with open(manifest, "r") as f:
        content = f.read()
    # Find all [[runtimes]] sections and extract their name fields
    pattern = r'\[\[runtimes\]\][^\[]*?name\s*=\s*"([^"]+)"'
    names = re.findall(pattern, content, re.MULTILINE | re.DOTALL)
    for name in names:
        print(name)
except Exception as e:
    pass
PYTHON_EOF
        "$MANIFEST" 2>/dev/null) || RUNTIME_NAMES=""
        
        # Fallback: use provider directory name if no runtimes found
        if [ -z "$RUNTIME_NAMES" ]; then
            RUNTIME_NAMES="$PROVIDER_NAME"
        fi
        
        # Store each runtime with its platform constraint
        for runtime in $RUNTIME_NAMES; do
            ALL_RUNTIMES="$ALL_RUNTIMES $runtime"
            if [ -n "$PROVIDER_PLATFORMS" ]; then
                RUNTIME_PLATFORMS[$runtime]="$PROVIDER_PLATFORMS"
            else
                RUNTIME_PLATFORMS[$runtime]=""
            fi
        done
    fi
done

# Clean up and sort runtimes
ALL_RUNTIMES=$(echo "$ALL_RUNTIMES" | xargs | tr ' ' '\n' | sort -u | tr '\n' ' ')
RUNTIME_COUNT=$(echo "$ALL_RUNTIMES" | wc -w | xargs)

echo "Found $RUNTIME_COUNT runtimes"

# Filter runtimes for a specific platform
filter_for_platform() {
    local platform=$1
    local result=()
    
    for runtime in $ALL_RUNTIMES; do
        local constraints="${RUNTIME_PLATFORMS[$runtime]:-}"
        
        # Skip if platform constraints exist and don't include this platform
        if [ -n "$constraints" ]; then
            if ! echo "$constraints" | grep -qi "$platform"; then
                continue
            fi
        fi
        
        # Skip always-excluded runtimes
        if echo ",$SKIP_ALWAYS," | grep -qi ",$runtime,"; then
            continue
        fi
        
        # Apply user runtimes filter (if specified)
        if [ -n "$RUNTIME_FILTER" ]; then
            if ! echo ",$RUNTIME_FILTER," | grep -qi ",$runtime,"; then
                continue
            fi
        fi
        
        result+=("$runtime")
    done
    
    echo "${result[@]}"
}

# Chunk runtimes into groups for parallel testing
chunk_runtimes() {
    local runtimes_str="$1"
    local chunk_size="$2"
    local runtimes=($runtimes_str)
    local chunks=()
    local current_chunk=""
    local count=0
    
    for runtime in "${runtimes[@]}"; do
        if [ $count -ge $chunk_size ]; then
            chunks+=("$current_chunk")
            current_chunk="$runtime"
            count=1
        else
            if [ -z "$current_chunk" ]; then
                current_chunk="$runtime"
            else
                current_chunk="$current_chunk,$runtime"
            fi
            ((count++)) || true
        fi
    done
    
    if [ -n "$current_chunk" ]; then
        chunks+=("$current_chunk")
    fi
    
    # Output as JSON array
    if [ ${#chunks[@]} -eq 0 ]; then
        echo "[]"
    else
        printf '%s\n' "${chunks[@]}" | jq -R -s -c 'split("\n") | map(select(length > 0))'
    fi
}

# Get filtered lists for each platform
LINUX_RUNTIMES=$(filter_for_platform "linux")
MACOS_RUNTIMES=$(filter_for_platform "macos")
WINDOWS_RUNTIMES=$(filter_for_platform "windows")

# Trim whitespace
LINUX_RUNTIMES=$(echo "$LINUX_RUNTIMES" | xargs)
MACOS_RUNTIMES=$(echo "$MACOS_RUNTIMES" | xargs)
WINDOWS_RUNTIMES=$(echo "$WINDOWS_RUNTIMES" | xargs)

echo ""
echo "Platform-specific runtimes:"
echo "  Linux:   $(echo $LINUX_RUNTIMES | wc -w | xargs) runtimes"
echo "  macOS:   $(echo $MACOS_RUNTIMES | wc -w | xargs) runtimes"
echo "  Windows: $(echo $WINDOWS_RUNTIMES | wc -w | xargs) runtimes"

# Generate matrix for each platform
LINUX_MATRIX=$(chunk_runtimes "$LINUX_RUNTIMES" "$CHUNK_SIZE")
MACOS_MATRIX=$(chunk_runtimes "$MACOS_RUNTIMES" "$CHUNK_SIZE")
WINDOWS_MATRIX=$(chunk_runtimes "$WINDOWS_RUNTIMES" "$CHUNK_SIZE")

echo ""
echo "Matrix chunks (chunk_size=$CHUNK_SIZE):"
echo "  Linux:   $LINUX_MATRIX"
echo "  macOS:   $MACOS_MATRIX"
echo "  Windows: $WINDOWS_MATRIX"

# Determine if there are any runtimes to test
HAS_LINUX=$( [ "$LINUX_MATRIX" != "[]" ] && [ -n "$LINUX_RUNTIMES" ] && echo "true" || echo "false" )
HAS_MACOS=$( [ "$MACOS_MATRIX" != "[]" ] && [ -n "$MACOS_RUNTIMES" ] && echo "true" || echo "false" )
HAS_WINDOWS=$( [ "$WINDOWS_MATRIX" != "[]" ] && [ -n "$WINDOWS_RUNTIMES" ] && echo "true" || echo "false" )

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

# Generate summary if requested
if [ -n "$SUMMARY_FILE" ]; then
    {
        echo "## Provider Discovery Results"
        echo ""
        echo "### Runtimes per Platform"
        echo "| Platform | Count | Runtimes (first 10) |"
        echo "|----------|-------|---------------------|"
        echo "| Linux | $(echo $LINUX_RUNTIMES | wc -w | xargs) | $(echo $LINUX_RUNTIMES | tr ' ' '\n' | head -10 | tr '\n' ' ')... |"
        echo "| macOS | $(echo $MACOS_RUNTIMES | wc -w | xargs) | $(echo $MACOS_RUNTIMES | tr ' ' '\n' | head -10 | tr '\n' ' ')... |"
        echo "| Windows | $(echo $WINDOWS_RUNTIMES | wc -w | xargs) | $(echo $WINDOWS_RUNTIMES | tr ' ' '\n' | head -10 | tr '\n' ' ')... |"
        echo ""
        echo "### Configuration"
        echo "- Chunk size: $CHUNK_SIZE"
        echo "- Total runtimes discovered: $RUNTIME_COUNT"
        echo ""
        echo "### Skipped Runtimes (CI-incompatible)"
        echo "\`$SKIP_ALWAYS\`"
    } >> "$SUMMARY_FILE"
fi

echo ""
echo "Discovery complete!"
