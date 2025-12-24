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

# Clean build artifacts
clean:
    cargo clean

# Install vx locally
install:
    cargo install --path .
