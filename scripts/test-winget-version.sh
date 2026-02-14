#!/usr/bin/env bash
# Test script for winget version normalization
# This validates the version normalization logic in package-managers.yml

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC}: $1"; }
fail() { echo -e "${RED}FAIL${NC}: $1"; exit 1; }

echo "Testing winget version normalization..."
echo ""

# Test winget version normalization
test_winget_normalize() {
    local input="$1"
    local expected="$2"
    local result

    # Simulate the normalization from package-managers.yml
    # This removes 'vx-' prefix if present (vx-v0.1.0 -> v0.1.0)
    result="${input#vx-}"
    # Remove 'v' prefix for WinGet (v0.1.0 -> 0.1.0)
    result="${result#v}"

    if [[ "$result" == "$expected" ]]; then
        pass "winget_normalize('$input') = '$result'"
    else
        fail "winget_normalize('$input') = '$result', expected '$expected'"
    fi
}

echo "=== Testing winget version normalization ==="

# Test with vx- prefix (should remove both vx- and v)
test_winget_normalize "vx-v0.6.7" "0.6.7"
test_winget_normalize "vx-v0.6.8" "0.6.8"
test_winget_normalize "vx-v1.0.0" "1.0.0"
test_winget_normalize "vx-v0.1.0" "0.1.0"

# Test without vx- prefix (should remove only v)
test_winget_normalize "v0.6.7" "0.6.7"
test_winget_normalize "v0.6.8" "0.6.8"
test_winget_normalize "v1.0.0" "1.0.0"
test_winget_normalize "v0.1.0" "0.1.0"

echo ""
echo "=== All winget version normalization tests passed! ==="
