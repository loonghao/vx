# Makefile for vx - Universal Development Tool Manager
# Provides convenient targets for building, testing, and optimizing

.PHONY: help build build-release build-pgo test clean install benchmark lint format check-deps coverage security

# Default target
help:
	@echo "vx - Universal Development Tool Manager"
	@echo ""
	@echo "Available targets:"
	@echo "  build         - Build debug version"
	@echo "  build-release - Build release version"
	@echo "  build-pgo     - Build with Profile-Guided Optimization"
	@echo "  test          - Run all tests"
	@echo "  coverage      - Generate code coverage report"
	@echo "  security      - Run security audit"
	@echo "  clean         - Clean build artifacts"
	@echo "  install       - Install to system"
	@echo "  benchmark     - Run performance benchmarks"
	@echo "  lint          - Run linting checks"
	@echo "  format        - Format code"
	@echo "  check-deps    - Check for dependency updates"
	@echo ""
	@echo "PGO targets:"
	@echo "  pgo-clean     - Clean PGO data and rebuild"
	@echo "  pgo-verbose   - Build PGO with verbose output"
	@echo "  test-pgo      - Test PGO optimization effectiveness"
	@echo ""
	@echo "GoReleaser targets:"
	@echo "  goreleaser-test     - Test GoReleaser configuration"
	@echo "  goreleaser-snapshot - Create snapshot build"
	@echo "  goreleaser-release  - Create PGO-optimized release"

# Basic build targets
build:
	@echo "🔨 Building debug version..."
	cargo build

build-release:
	@echo "🚀 Building release version..."
	cargo build --release

build-fast:
	@echo "⚡ Building with fast profile..."
	cargo build --profile dev-fast

# PGO build targets
build-pgo:
	@echo "🎯 Building with Profile-Guided Optimization..."
ifeq ($(OS),Windows_NT)
	@powershell -ExecutionPolicy Bypass -File scripts/build-pgo.ps1
else
	@chmod +x scripts/build-pgo.sh
	@./scripts/build-pgo.sh
endif

pgo-clean:
	@echo "🧹 Cleaning and rebuilding with PGO..."
ifeq ($(OS),Windows_NT)
	@powershell -ExecutionPolicy Bypass -File scripts/build-pgo.ps1 -Clean
else
	@chmod +x scripts/build-pgo.sh
	@./scripts/build-pgo.sh --clean
endif

pgo-verbose:
	@echo "🎯 Building PGO with verbose output..."
ifeq ($(OS),Windows_NT)
	@powershell -ExecutionPolicy Bypass -File scripts/build-pgo.ps1 -Verbose
else
	@chmod +x scripts/build-pgo.sh
	@./scripts/build-pgo.sh --verbose
endif

# Test targets
test:
	@echo "🧪 Running tests..."
	cargo test

test-verbose:
	@echo "🧪 Running tests with verbose output..."
	cargo test -- --nocapture

# Coverage targets
coverage:
	@echo "📊 Generating code coverage report..."
	@echo "Installing cargo-llvm-cov if not present..."
	@cargo install cargo-llvm-cov --quiet || true
	@echo "Generating coverage..."
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "Coverage report generated: lcov.info"

coverage-html:
	@echo "📊 Generating HTML coverage report..."
	@echo "Installing cargo-llvm-cov if not present..."
	@cargo install cargo-llvm-cov --quiet || true
	@echo "Generating HTML coverage..."
	cargo llvm-cov --all-features --workspace --html
	@echo "HTML coverage report generated in target/llvm-cov/html/"

# Security targets
security:
	@echo "🔒 Running security audit..."
	@echo "Installing cargo-audit if not present..."
	@cargo install cargo-audit --quiet || true
	@echo "Running audit..."
	cargo audit

# Maintenance targets
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf pgo-data

install:
	@echo "📦 Installing vx..."
	cargo install --path .

# Code quality targets
lint:
	@echo "🔍 Running linting checks..."
	cargo clippy -- -D warnings

format:
	@echo "✨ Formatting code..."
	cargo fmt

format-check:
	@echo "🔍 Checking code formatting..."
	cargo fmt -- --check

# Dependency management
check-deps:
	@echo "🔍 Checking for dependency updates..."
	cargo outdated

update-deps:
	@echo "⬆️ Updating dependencies..."
	cargo update

# Performance targets
benchmark:
	@echo "⏱️ Running performance benchmarks..."
	@echo "Testing startup time..."
	@time cargo run --release -- version >/dev/null
	@echo "Testing command parsing..."
	@time cargo run --release -- --help >/dev/null

benchmark-pgo:
	@echo "⏱️ Benchmarking PGO-optimized binary..."
ifeq ($(OS),Windows_NT)
	@powershell -Command "Measure-Command { ./target/release/vx.exe version } | Select-Object TotalMilliseconds"
else
	@time ./target/release/vx version >/dev/null
endif

# Development targets
dev-setup:
	@echo "🛠️ Setting up development environment..."
	rustup component add clippy rustfmt
	cargo install cargo-outdated

# Release targets
release-check:
	@echo "🔍 Pre-release checks..."
	cargo fmt -- --check
	cargo clippy -- -D warnings
	cargo test
	cargo build --release

# Documentation
docs:
	@echo "📚 Building documentation..."
	cargo doc --open

# Size analysis
size-analysis:
	@echo "📊 Analyzing binary size..."
	cargo build --release
	ls -lh target/release/vx*
ifeq ($(OS),Windows_NT)
	@echo "Windows binary size analysis:"
	@dir target\release\vx.exe
else
	@echo "Binary size analysis:"
	@file target/release/vx
	@size target/release/vx
endif

# Quick development cycle
quick: format lint test build

# Full CI pipeline
ci: format-check lint security test coverage build-release

# GoReleaser targets
goreleaser-test:
	@echo "🧪 Testing GoReleaser configuration..."
	goreleaser check
	goreleaser build --snapshot --clean

goreleaser-release:
	@echo "🚀 Creating release with GoReleaser (PGO-optimized)..."
	goreleaser release --clean

goreleaser-snapshot:
	@echo "📸 Creating snapshot build with GoReleaser..."
	goreleaser build --snapshot --clean

# Performance testing
test-pgo:
	@echo "🧪 Testing PGO optimization effectiveness..."
	@chmod +x scripts/test-pgo.sh
	@./scripts/test-pgo.sh

# Performance comparison
perf-compare:
	@echo "🏁 Performance comparison: Debug vs Release vs PGO"
	@echo "Debug build:"
	@time cargo run -- version >/dev/null 2>&1
	@echo "Release build:"
	@time cargo run --release -- version >/dev/null 2>&1
	@echo "Building PGO version..."
	@$(MAKE) build-pgo >/dev/null 2>&1
	@echo "PGO build:"
	@time ./target/release/vx version >/dev/null 2>&1
