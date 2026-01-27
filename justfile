# vx - Universal Development Tool Manager
# Just command runner recipes

# Set shell for Windows compatibility
set windows-shell := ["pwsh", "-NoProfile", "-Command"]

# Default recipe - show available commands
default:
    @just --list

# ============================================
# Documentation
# ============================================

# Install docs dependencies
docs-install:
    cd docs && npm ci

# Build documentation site
docs-build:
    cd docs && npx vitepress build

# Start docs dev server
docs-dev:
    cd docs && npx vitepress dev

# Preview built docs
docs-preview:
    cd docs && npx vitepress preview

# ============================================
# Build
# ============================================

# Build debug version
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Build with fast profile
build-fast:
    cargo build --profile dev-fast

# ============================================
# Test
# ============================================

# Run all tests
test:
    cargo test

# Run tests with verbose output
test-verbose:
    cargo test -- --nocapture

# Test all providers in a clean temporary environment
test-providers:
    @echo "Building VX first..."
    @pwsh -NoProfile -Command "Get-Process vx -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue"
    @cargo build
    @echo ""
    ./target/debug/vx test --ci --temp-root --keep-going --detailed

# Test all providers with verbose output
test-providers-verbose:
    @echo "Building VX first..."
    @pwsh -NoProfile -Command "Get-Process vx -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue"
    @cargo build  
    @echo ""
    ./target/debug/vx test --ci --temp-root --keep-going --verbose

# Test all providers and output JSON report
test-providers-json:
    @echo "Building VX first..."
    @cargo build
    @echo ""
    ./target/debug/vx test --ci --temp-root --keep-going --json

# Test specific runtimes (comma-separated)
test-runtimes RUNTIMES:
    @echo "Building VX first..."
    @cargo build
    @echo ""
    ./target/debug/vx test --ci --ci-runtimes {{RUNTIMES}} --temp-root --verbose

# Quick CI test with core runtimes only
test-ci-quick:
    @echo "Building VX first..."
    @cargo build
    @echo ""
    ./target/debug/vx test --ci --ci-runtimes node,go,uv,just,cargo --temp-root --verbose

# ============================================
# Code Quality
# ============================================

# Run linting checks
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
format:
    cargo fmt

# Check code formatting
format-check:
    cargo fmt -- --check

# ============================================
# Development
# ============================================

# Quick development cycle: format, lint, test, build
quick: format lint test build

# Quick video workflow: setup, start, build (for testing changes)
video-quick: video-setup video-start

# Video development workflow: clean, build
video-rebuild: video-clean video-build

# Clean build artifacts
clean:
    cargo clean

# Install vx locally
install:
    cargo install --path .

# ============================================
# Promotional Video (Remotion)
# ============================================

# Setup and install video dependencies
video-setup:
    @echo "Setting up Remotion video project..."
    cd vx-promo-video && npm install

# Start Remotion preview server for video development
video-start:
    @echo "Starting Remotion preview server..."
    cd vx-promo-video && npm start

# Build the promotional video (renders all scenes to MP4)
video-build:
    @echo "Building promotional video..."
    cd vx-promo-video && npm run build

# Render specific video scene
video-render-scene SCENE:
    @echo "Rendering scene: {{SCENE}}..."
    cd vx-promo-video && npx remotion render src/Root.tsx {{SCENE}} --out=out/{{SCENE}}.mp4

# Build video for different platforms (square, vertical, etc.)
video-build-platform PLATFORM:
    @echo "Building video for platform: {{PLATFORM}}..."
    cd vx-promo-video && npx remotion render src/Root.tsx vx-intro --out=out/vx-intro-{{PLATFORM}}.mp4 --size={{PLATFORM}}

# Clean video output directory
video-clean:
    @echo "Cleaning video output..."
    cd vx-promo-video && rm -rf out/ public/

# Upgrade Remotion dependencies
video-upgrade:
    @echo "Upgrading Remotion..."
    cd vx-promo-video && npm run upgrade

# Show video-related commands
video-help:
    @echo "VX Promotional Video Commands:"
    @echo ""
    @echo "  just video-setup         - Install video dependencies"
    @echo "  just video-start         - Start preview server"
    @echo "  just video-build         - Build full video"
    @echo "  just video-render-scene <SCENE>  - Render specific scene"
    @echo "  just video-build-platform <PLATFORM> - Build for platform"
    @echo "  just video-clean         - Clean video output"
    @echo "  just video-upgrade       - Upgrade Remotion"
    @echo ""
    @echo "Available scenes: vx-intro, vx-problem, vx-solution, vx-features, vx-cta"
    @echo "Available platforms: 1080x1080 (Instagram), 1080x1920 (TikTok)"
    @echo ""
