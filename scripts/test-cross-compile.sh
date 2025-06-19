#!/bin/bash
# Test cross-compilation for different targets
# This script helps verify that our OpenSSL fixes work

set -e

echo "🔧 Testing cross-compilation fixes..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to test a target
test_target() {
    local target=$1
    local description=$2
    
    echo -e "\n${YELLOW}Testing $description ($target)...${NC}"
    
    # Add the target if not already installed
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Adding target $target..."
        rustup target add "$target"
    fi
    
    # Try to build
    if cargo build --target "$target" --release --bin vx; then
        echo -e "${GREEN}✅ $description build successful${NC}"
        return 0
    else
        echo -e "${RED}❌ $description build failed${NC}"
        return 1
    fi
}

# Test different targets
echo "Testing various cross-compilation targets..."

# Test native target first
echo -e "\n${YELLOW}Testing native target...${NC}"
if cargo build --release --bin vx; then
    echo -e "${GREEN}✅ Native build successful${NC}"
else
    echo -e "${RED}❌ Native build failed${NC}"
    exit 1
fi

# Test targets that commonly have OpenSSL issues
TARGETS=(
    "x86_64-unknown-linux-musl:Linux musl (static)"
    "aarch64-unknown-linux-gnu:Linux ARM64"
    "x86_64-pc-windows-gnu:Windows GNU"
)

SUCCESS_COUNT=0
TOTAL_COUNT=${#TARGETS[@]}

for target_info in "${TARGETS[@]}"; do
    IFS=':' read -r target description <<< "$target_info"
    
    if test_target "$target" "$description"; then
        ((SUCCESS_COUNT++))
    fi
done

echo -e "\n${YELLOW}Cross-compilation test summary:${NC}"
echo -e "Successful: ${GREEN}$SUCCESS_COUNT${NC}/$TOTAL_COUNT"

if [ $SUCCESS_COUNT -eq $TOTAL_COUNT ]; then
    echo -e "${GREEN}🎉 All cross-compilation tests passed!${NC}"
    echo -e "${GREEN}OpenSSL dependency issues have been resolved.${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some cross-compilation tests failed.${NC}"
    echo -e "${YELLOW}This might be due to missing system dependencies.${NC}"
    echo -e "${YELLOW}Check the CI configuration for required packages.${NC}"
    exit 1
fi
