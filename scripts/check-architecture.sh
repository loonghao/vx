#!/bin/bash
# check-architecture.sh — Enforce architectural layer dependencies
#
# This script verifies that crate dependencies respect the layered architecture:
#   Layer 4: vx-cli (Application)
#   Layer 3: vx-resolver, vx-setup, vx-migration, vx-extension, vx-project-analyzer (Orchestration)
#   Layer 2: vx-runtime, vx-starlark, vx-installer, vx-version-fetcher, vx-system-pm, vx-ecosystem-pm, vx-shim (Services)
#   Layer 1: vx-config, vx-env, vx-console, vx-metrics, vx-runtime-core, vx-runtime-archive, vx-runtime-http (Infrastructure)
#   Layer 0: vx-core, vx-paths, vx-cache, vx-versions, vx-manifest, vx-args (Foundation)
#
# Rule: Dependencies flow DOWNWARD only (Layer N can depend on Layer 0..N-1, never on Layer N+1..4)

set -euo pipefail

ERRORS=0
WARNINGS=0

# Color output
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "🏗️  Checking architectural layer dependencies..."
echo ""

# Define layers (crate name -> layer number)
declare -A CRATE_LAYER

# Layer 0: Foundation
for crate in vx-core vx-paths vx-cache vx-versions vx-manifest vx-args; do
    CRATE_LAYER[$crate]=0
done

# Layer 1: Infrastructure
for crate in vx-config vx-env vx-console vx-metrics vx-runtime-core vx-runtime-archive vx-runtime-http; do
    CRATE_LAYER[$crate]=1
done

# Layer 2: Services
for crate in vx-runtime vx-starlark vx-installer vx-version-fetcher vx-system-pm vx-ecosystem-pm vx-shim; do
    CRATE_LAYER[$crate]=2
done

# Layer 3: Orchestration
for crate in vx-resolver vx-setup vx-migration vx-extension vx-project-analyzer; do
    CRATE_LAYER[$crate]=3
done

# Layer 4: Application
for crate in vx-cli; do
    CRATE_LAYER[$crate]=4
done

# Check each crate's Cargo.toml for dependency violations
check_crate() {
    local crate_dir="$1"
    local crate_name
    crate_name=$(basename "$crate_dir")
    local cargo_toml="$crate_dir/Cargo.toml"

    if [ ! -f "$cargo_toml" ]; then
        return
    fi

    local layer="${CRATE_LAYER[$crate_name]:-}"
    if [ -z "$layer" ]; then
        # Unknown crate (provider or workspace-hack), skip
        return
    fi

    # Extract vx-* dependencies from Cargo.toml
    local deps
    deps=$(grep -E '^\s*vx-[a-z-]+\s*=' "$cargo_toml" 2>/dev/null | \
           sed 's/\s*=.*//' | \
           sed 's/^\s*//' || true)

    for dep in $deps; do
        local dep_layer="${CRATE_LAYER[$dep]:-}"
        if [ -z "$dep_layer" ]; then
            continue  # Unknown dep (provider crate), skip
        fi

        if [ "$dep_layer" -gt "$layer" ]; then
            echo -e "${RED}❌ VIOLATION${NC}: $crate_name (layer $layer) depends on $dep (layer $dep_layer)"
            echo "   → Higher layers cannot be imported by lower layers"
            ERRORS=$((ERRORS + 1))
        fi
    done
}

# Check all crates
for dir in crates/vx-*/; do
    if [ -d "$dir" ]; then
        check_crate "$dir"
    fi
done

echo ""

# Check for forbidden terminology in source files
echo "📝 Checking terminology conventions..."

check_terminology() {
    local pattern="$1"
    local replacement="$2"
    local context="$3"

    local matches
    matches=$(grep -rn "$pattern" crates/*/src/ \
        --include="*.rs" \
        --exclude-dir=target \
        -l 2>/dev/null || true)

    if [ -n "$matches" ]; then
        echo -e "${YELLOW}⚠️  TERMINOLOGY${NC}: Found '$pattern' (should be '$replacement') in $context"
        echo "$matches" | head -5 | while read -r file; do
            echo "   → $file"
        done
        local count
        count=$(echo "$matches" | wc -l | xargs)
        if [ "$count" -gt 5 ]; then
            echo "   ... and $((count - 5)) more files"
        fi
        WARNINGS=$((WARNINGS + 1))
    fi
}

# Check for forbidden terms (only in struct/type definitions, not comments)
check_terminology "struct.*VxTool" "Runtime" "struct definitions"
check_terminology "struct.*ToolBundle" "Provider" "struct definitions"
check_terminology "struct.*BundleRegistry" "ProviderRegistry" "struct definitions"
check_terminology "struct.*ToolSpec" "RuntimeSpec" "struct definitions"

echo ""

# Summary
echo "========================================"
if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}❌ Architecture check FAILED${NC}"
    echo "   $ERRORS violation(s), $WARNINGS warning(s)"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Architecture check PASSED with warnings${NC}"
    echo "   $WARNINGS warning(s)"
    exit 0
else
    echo -e "${GREEN}✅ Architecture check PASSED${NC}"
    echo "   All layer dependencies are valid"
    exit 0
fi
