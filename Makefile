# Makefile for vx - Universal Development Tool Manager
# Provides convenient targets for building, testing, and optimizing

.PHONY: help build build-release build-pgo test clean install benchmark lint format check-deps coverage security advanced-build sccache-setup upx-install

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
	@echo "  lint-fix      - Fix clippy warnings automatically"
	@echo "  lint-strict   - Run strict linting with pedantic rules"
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
	@echo ""
	@echo "Advanced optimization targets:"
	@echo "  advanced-build      - Build with all optimizations"
	@echo "  sccache-setup       - Install and configure sccache"
	@echo "  upx-install         - Install UPX compression tool"
	@echo "  build-matrix        - Build for multiple targets"
	@echo "  build-optimized     - Build with UPX and stripping"

# Basic build targets
build:
	@echo "🔨 Building debug version..."
	CARGO_BUILD_JOBS=0 cargo build

build-release:
	@echo "🚀 Building release version..."
	CARGO_BUILD_JOBS=0 CARGO_INCREMENTAL=1 cargo build --release

build-fast:
	@echo "⚡ Building with fast profile..."
	CARGO_BUILD_JOBS=0 cargo build --profile dev-fast

# Optimized parallel build
build-parallel:
	@echo "⚡ Building with maximum parallelization..."
	CARGO_BUILD_JOBS=0 CARGO_INCREMENTAL=1 RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build --release

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
	cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
	@echo "🔧 Fixing clippy warnings..."
	@chmod +x scripts/fix-clippy.sh
	@./scripts/fix-clippy.sh --fix

lint-strict:
	@echo "🔍 Running strict linting checks..."
	@chmod +x scripts/fix-clippy.sh
	@./scripts/fix-clippy.sh --pedantic --nursery --all-features

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

# Advanced optimization targets
advanced-build:
	@echo "🚀 Building with all optimizations..."
	@chmod +x scripts/advanced-build.sh
	@./scripts/advanced-build.sh --pgo --benchmark --size-analysis

sccache-setup:
	@echo "⚡ Setting up sccache..."
	@if ! command -v sccache >/dev/null 2>&1; then \
		echo "Installing sccache..."; \
		cargo install sccache --locked; \
	else \
		echo "sccache already installed"; \
	fi
	@echo "sccache version: $$(sccache --version)"
	@echo "sccache stats:"
	@sccache --show-stats

upx-install:
	@echo "🗜️ Installing UPX compression tool..."
ifeq ($(OS),Windows_NT)
	@choco install upx
else ifeq ($(shell uname),Darwin)
	@brew install upx
else
	@sudo apt-get update && sudo apt-get install -y upx-ucl
endif
	@echo "UPX version: $$(upx --version | head -1)"

build-matrix:
	@echo "🌐 Building for multiple targets..."
	@for target in x86_64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin x86_64-pc-windows-msvc; do \
		echo "Building for $$target..."; \
		./scripts/advanced-build.sh --target $$target || echo "Failed to build $$target"; \
	done

build-optimized:
	@echo "🎯 Building optimized release..."
	@$(MAKE) sccache-setup
	@$(MAKE) upx-install
	@./scripts/advanced-build.sh --pgo --strip --upx --benchmark

# Cross-compilation helpers
build-linux-arm64:
	@echo "🐧 Building for Linux ARM64..."
	@./scripts/advanced-build.sh --target aarch64-unknown-linux-gnu

build-macos-arm64:
	@echo "🍎 Building for macOS ARM64..."
	@./scripts/advanced-build.sh --target aarch64-apple-darwin --pgo

build-windows:
	@echo "🪟 Building for Windows..."
	@./scripts/advanced-build.sh --target x86_64-pc-windows-msvc --pgo
