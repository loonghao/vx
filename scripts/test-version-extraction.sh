#!/usr/bin/env bash
# Test script for version extraction logic
# This validates that the install scripts correctly handle different version formats

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC}: $1"; }
fail() { echo -e "${RED}FAIL${NC}: $1"; exit 1; }

echo "Testing version extraction logic..."
echo ""

# Test version_short extraction (used in linux-packages.yml)
test_version_short() {
    local input="$1"
    local expected="$2"
    local result

    # Simulate the sed command from linux-packages.yml
    result=$(echo "${input}" | sed -E 's/^(vx-)?v//')

    if [[ "$result" == "$expected" ]]; then
        pass "version_short('$input') = '$result'"
    else
        fail "version_short('$input') = '$result', expected '$expected'"
    fi
}

echo "=== Testing version_short extraction (linux-packages.yml) ==="
# Current release-please format
test_version_short "v0.6.7" "0.6.7"
test_version_short "v1.0.0" "1.0.0"
test_version_short "v0.10.0" "0.10.0"

# Legacy format (should still work)
test_version_short "vx-v0.6.7" "0.6.7"
test_version_short "vx-v1.0.0" "1.0.0"

echo ""
echo "=== Testing tag normalization (install scripts) ==="

# Test tag normalization for install scripts
test_tag_normalize() {
    local input="$1"
    local expected="$2"
    local result

    # Simulate the logic from install.sh
    if [[ "$input" =~ ^vx-v ]]; then
        result="${input#vx-}"
    elif [[ "$input" =~ ^v ]]; then
        result="$input"
    else
        result="v$input"
    fi

    if [[ "$result" == "$expected" ]]; then
        pass "normalize('$input') = '$result'"
    else
        fail "normalize('$input') = '$result', expected '$expected'"
    fi
}

# User input formats
test_tag_normalize "0.6.7" "v0.6.7"
test_tag_normalize "v0.6.7" "v0.6.7"
test_tag_normalize "vx-v0.6.7" "v0.6.7"
test_tag_normalize "1.0.0" "v1.0.0"

echo ""
echo "=== All tests passed! ==="
