#!/bin/bash
# Project Analyze Script for Unix
# Usage: ./analyze.sh <project-name-or-url> [temp-dir]

set -e

PROJECT="$1"
TEMP_DIR="${2:-/tmp}"

if [ -z "$PROJECT" ]; then
    echo "Usage: $0 <project-name-or-url> [temp-dir]"
    exit 1
fi

# Known project mappings
declare -A KNOWN_PROJECTS=(
    ["codex"]="https://github.com/openai/codex"
    ["kubectl"]="https://github.com/kubernetes/kubectl"
    ["deno"]="https://github.com/denoland/deno"
    ["ripgrep"]="https://github.com/BurntSushi/ripgrep"
    ["uv"]="https://github.com/astral-sh/uv"
    ["nextjs"]="https://github.com/vercel/next.js"
    ["next.js"]="https://github.com/vercel/next.js"
    ["vite"]="https://github.com/vitejs/vite"
    ["ruff"]="https://github.com/astral-sh/ruff"
    ["httpx"]="https://github.com/encode/httpx"
    ["auroraview"]="https://github.com/loonghao/auroraview"
    ["docker-cli"]="https://github.com/docker/cli"
)

# Resolve project URL
if [[ "$PROJECT" =~ ^https?:// ]]; then
    REPO_URL="$PROJECT"
    PROJECT_NAME=$(basename "$PROJECT" .git)
elif [ -n "${KNOWN_PROJECTS[$PROJECT]}" ]; then
    REPO_URL="${KNOWN_PROJECTS[$PROJECT]}"
    PROJECT_NAME="$PROJECT"
else
    echo "Unknown project: $PROJECT"
    echo "Attempting GitHub search..."
    REPO_URL="https://github.com/$PROJECT"
    PROJECT_NAME=$(basename "$PROJECT")
fi

TEST_DIR="$TEMP_DIR/${PROJECT_NAME}-test"
VX_ROOT="${VX_ROOT:-$(dirname "$0")/../../..}"

echo ""
echo "=== Project Analyze ==="
echo "Project: $PROJECT_NAME"
echo "URL: $REPO_URL"
echo "Test Dir: $TEST_DIR"
echo ""

# Step 1: Clone
if [ -d "$TEST_DIR" ]; then
    echo "Removing existing test directory..."
    rm -rf "$TEST_DIR"
fi

echo "Cloning repository..."
git clone --depth 1 "$REPO_URL" "$TEST_DIR"

# Step 2: Analyze
echo ""
echo "Running analysis..."
cargo run --manifest-path "$VX_ROOT/Cargo.toml" -p vx-project-analyzer --example analyze_project -- "$TEST_DIR"

# Step 3: Prompt for cleanup
echo ""
echo "=== Analysis Complete ==="
echo "Test directory: $TEST_DIR"
echo ""
read -p "Clean up test directory? (y/N) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Cleaning up..."
    rm -rf "$TEST_DIR"
    echo "Done!"
else
    echo "Test directory preserved at: $TEST_DIR"
fi
