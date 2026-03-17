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
    cd docs && vx node::npm ci

# Build documentation site
docs-build:
    cd docs && vx node::npx vitepress build

# Start docs dev server
docs-dev:
    cd docs && vx node::npx vitepress dev

# Preview built docs
docs-preview:
    cd docs && vx node::npx vitepress preview


# ============================================
# Build
# ============================================

# Build debug version
build:
    vx cargo build

# Build release version
build-release:
    vx cargo build --release

# Build release version for target
build-release-target TARGET:
    vx cargo build --release --target {{TARGET}}

# Build with fast profile
build-fast:
    vx cargo build --profile dev-fast


# ============================================
# Test
# ============================================

# Run all tests (use nextest for faster execution)
test:
    vx cargo nextest run --workspace --no-fail-fast

# Run tests with verbose output
test-verbose:
    vx cargo nextest run --workspace --no-fail-fast --no-capture

# Run tests with cargo test (fallback if nextest not available)
test-cargo:
    vx cargo test --workspace --no-fail-fast

# Run fast unit tests only (skip slow integration tests)
test-fast:
    vx cargo nextest run --workspace --no-fail-fast -E 'not test(e2e)'

# Run tests for specific packages
# Usage: just test-pkgs "-p vx-provider-* -p vx-cli"
test-pkgs PKGS:
    vx cargo nextest run --no-fail-fast {{PKGS}}

# Fast static checks for provider.star logic and provider unit tests
test-providers-static:
    vx cargo test -p vx-starlark --test lint_all_providers_test -- --nocapture
    vx cargo nextest run --no-fail-fast -p 'vx-provider-*'


# Test all providers in a clean temporary environment
test-providers:
    @echo "Building VX first..."
    @pwsh -NoProfile -Command "Get-Process vx -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue"
    @vx cargo build
    @echo ""
    ./target/debug/vx test --ci --temp-root --keep-going --detailed

# Test all providers with verbose output
test-providers-verbose:
    @echo "Building VX first..."
    @pwsh -NoProfile -Command "Get-Process vx -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue"
    @vx cargo build
    @echo ""
    ./target/debug/vx test --ci --temp-root --keep-going --verbose

# Test all providers and output JSON report
test-providers-json:
    @echo "Building VX first..."
    @vx cargo build
    @echo ""
    ./target/debug/vx test --ci --temp-root --keep-going --json

# Test specific runtimes (comma-separated)
test-runtimes RUNTIMES:
    @echo "Building VX first..."
    @vx cargo build
    @echo ""
    ./target/debug/vx test --ci --ci-runtimes {{RUNTIMES}} --temp-root --verbose

# Quick CI test with core runtimes only
test-ci-quick:
    @echo "Building VX first..."
    @vx cargo build
    @echo ""
    ./target/debug/vx test --ci --ci-runtimes node,go,uv,just,cargo --temp-root --verbose


# ============================================
# Code Quality
# ============================================

# Run linting checks
lint:
    vx run clippy

# Format code
format:
    cargo fmt

# Verify workspace-hack is up-to-date (CI check)
hakari-verify:
    cargo hakari generate --diff
    cargo hakari manage-deps --dry-run

# Regenerate workspace-hack after dependency changes
hakari-generate:
    cargo hakari generate
    cargo hakari manage-deps

# Check code formatting
format-check:
    vx run fmt-check

# CI documentation build (no deps download)
ci-docs:
    vx run ci-docs

# Check for inline tests
check-inline-tests:
    ./scripts/check-inline-tests.sh

# Validate release version parsing scripts
validate-version-scripts:
    ./scripts/test-version-extraction.sh && ./scripts/test-winget-version.sh

# Run E2E benchmark tests (local)
benchmark-run:
    vx cargo test --release --test e2e_benchmark_tests -- --nocapture

# Run E2E benchmark tests and capture output (CI)
# Note: output redirection is handled by the CI workflow shell, not here,
# because justfile on Windows uses pwsh which doesn't support bash syntax.
benchmark-run-ci:
    vx cargo test --release --test e2e_benchmark_tests -- --nocapture

# Security audit (CI)
security-audit-ci:
    -vx cargo generate-lockfile
    -vx cargo audit --deny warnings

# Coverage (CI)
coverage-ci:
    vx cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Cross build (CI)
cross-build TARGET:
    vx cross build --release --target {{TARGET}} --no-default-features


# ============================================
# Architecture & Quality Gates
# ============================================

# Check architectural layer dependencies (custom linter)
check-architecture:
    bash scripts/check-architecture.sh

# Check file size limits (warns on bloat)
check-file-sizes:
    bash scripts/check-file-sizes.sh

# Check file size limits strictly (fails on violations)
check-file-sizes-strict:
    bash scripts/check-file-sizes.sh --strict

# Run all architecture checks
check-all: check-architecture check-file-sizes check-inline-tests
    @echo "✅ All architecture checks passed"

# Diagnose development environment
doctor:
    @echo "🏥 VX Development Environment Diagnosis"
    @echo "========================================"
    @echo ""
    @echo "## Rust Toolchain"
    @rustc --version 2>/dev/null || echo "❌ rustc not found"
    @cargo --version 2>/dev/null || echo "❌ cargo not found"
    @echo ""
    @echo "## Build Tools"
    @sccache --version 2>/dev/null || echo "⚠️  sccache not found (optional, speeds up builds)"
    @cargo nextest --version 2>/dev/null || echo "⚠️  cargo-nextest not found (optional, faster tests)"
    @cargo hakari --version 2>/dev/null || echo "⚠️  cargo-hakari not found (optional, workspace-hack)"
    @echo ""
    @echo "## VX"
    @vx --version 2>/dev/null || echo "⚠️  vx not found in PATH"
    @echo ""
    @echo "## Git"
    @git --version 2>/dev/null || echo "❌ git not found"
    @echo ""
    @echo "## Workspace"
    @echo "Crates: $(ls -d crates/vx-*/ 2>/dev/null | wc -l | xargs)"
    @echo "Providers: $(ls -d crates/vx-providers/*/ 2>/dev/null | wc -l | xargs)"
    @echo "provider.star files: $(find crates/vx-providers -name 'provider.star' 2>/dev/null | wc -l | xargs)"
    @echo ""
    @echo "========================================"
    @echo "🏥 Diagnosis complete"

# Fast pre-merge check (what CI will run on your PR)
pre-merge: format-check lint check-architecture test-fast
    @echo "✅ Pre-merge checks passed — ready for PR"

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
    vx cargo clean

# Install vx locally
install:
    vx cargo install --path .


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
