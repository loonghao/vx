#!/usr/bin/env bash
# vx installer script for Linux and macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
# Usage with version: VX_VERSION="0.1.0" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

set -euo pipefail

# Configuration
REPO_OWNER="loonghao"
REPO_NAME="vx"
BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases"

# Default values
VX_VERSION="${VX_VERSION:-latest}"
VX_INSTALL_DIR="${VX_INSTALL_DIR:-$HOME/.local/bin}"
BUILD_FROM_SOURCE="${BUILD_FROM_SOURCE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Detect platform and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        *)       error "Unsupported operating system: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    echo "$os-$arch"
}

# Get latest version from GitHub API (no authentication required)
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"

    if command -v curl >/dev/null 2>&1; then
        # Use curl without authentication - public API access
        curl -s "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//'
    elif command -v wget >/dev/null 2>&1; then
        # Use wget without authentication - public API access
        wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//'
    else
        error "Neither curl nor wget is available"
        exit 1
    fi
}

# Build from source (fallback method)
build_from_source() {
    info "Building vx from source..."

    # Check if Rust is installed
    if ! command -v cargo >/dev/null 2>&1; then
        error "Rust is not installed. Please install Rust first: https://rustup.rs/"
        exit 1
    fi

    # Check if we're in the vx repository
    if [[ ! -f "Cargo.toml" ]]; then
        error "Not in vx repository. Please clone the repository first:"
        echo "  git clone https://github.com/$REPO_OWNER/$REPO_NAME.git"
        echo "  cd $REPO_NAME"
        echo "  BUILD_FROM_SOURCE=true ./install.sh"
        exit 1
    fi

    # Build the project
    info "Building vx..."
    cargo build --release

    # Create installation directory
    mkdir -p "$VX_INSTALL_DIR"

    # Copy the binary
    cp "target/release/vx" "$VX_INSTALL_DIR/vx"
    chmod +x "$VX_INSTALL_DIR/vx"

    success "vx built and installed from source to: $VX_INSTALL_DIR"
}

# Download and install vx from GitHub releases
install_from_release() {
    local platform version archive_name download_url temp_dir

    platform=$(detect_platform)

    if [[ "$VX_VERSION" == "latest" ]]; then
        info "Fetching latest version..."
        version=$(get_latest_version)
        if [[ -z "$version" ]]; then
            error "Failed to get latest version"
            exit 1
        fi
    else
        version="$VX_VERSION"
    fi

    info "Installing vx v$version for $platform..."

    # Construct download URL based on actual release asset naming
    # Format: vx-{OS}-{variant}-{arch}.tar.gz
    case "$platform" in
        linux-x64)
            # Try musl first (static binary), fallback to gnu
            archive_name="vx-Linux-musl-x86_64.tar.gz"
            fallback_archive="vx-Linux-gnu-x86_64.tar.gz"
            ;;
        linux-arm64)
            # Try musl first (static binary), fallback to gnu
            archive_name="vx-Linux-musl-arm64.tar.gz"
            fallback_archive="vx-Linux-gnu-arm64.tar.gz"
            ;;
        macos-x64)    archive_name="vx-macOS-x86_64.tar.gz" ;;
        macos-arm64)  archive_name="vx-macOS-arm64.tar.gz" ;;
        *) error "Unsupported platform: $platform"; exit 1 ;;
    esac

    download_url="$BASE_URL/download/v$version/$archive_name"

    # Create temporary directory
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # Download with fallback support
    info "Downloading from $download_url..."
    download_success=false

    if command -v curl >/dev/null 2>&1; then
        if curl -fsSL "$download_url" -o "$temp_dir/$archive_name" 2>/dev/null; then
            download_success=true
        elif [[ -n "${fallback_archive:-}" ]]; then
            warn "Primary download failed, trying fallback..."
            fallback_url="$BASE_URL/download/v$version/$fallback_archive"
            info "Downloading from $fallback_url..."
            if curl -fsSL "$fallback_url" -o "$temp_dir/$fallback_archive" 2>/dev/null; then
                archive_name="$fallback_archive"
                download_success=true
            fi
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -q "$download_url" -O "$temp_dir/$archive_name" 2>/dev/null; then
            download_success=true
        elif [[ -n "${fallback_archive:-}" ]]; then
            warn "Primary download failed, trying fallback..."
            fallback_url="$BASE_URL/download/v$version/$fallback_archive"
            info "Downloading from $fallback_url..."
            if wget -q "$fallback_url" -O "$temp_dir/$fallback_archive" 2>/dev/null; then
                archive_name="$fallback_archive"
                download_success=true
            fi
        fi
    else
        error "Neither curl nor wget is available"
        exit 1
    fi

    if [[ "$download_success" != "true" ]]; then
        error "Failed to download vx binary"
        exit 1
    fi

    # Extract
    info "Extracting to $VX_INSTALL_DIR..."
    mkdir -p "$VX_INSTALL_DIR"

    if [[ "$archive_name" == *.tar.gz ]]; then
        tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"
    else
        error "Unsupported archive format: $archive_name"
        exit 1
    fi

    # Find and copy the binary
    local binary_path
    binary_path=$(find "$temp_dir" -name "vx" -type f | head -n1)
    if [[ -z "$binary_path" ]]; then
        error "vx binary not found in archive"
        exit 1
    fi

    cp "$binary_path" "$VX_INSTALL_DIR/vx"
    chmod +x "$VX_INSTALL_DIR/vx"

    success "vx v$version installed to $VX_INSTALL_DIR/vx"
}

# Update PATH environment variable
update_path() {
    local install_path="$1"
    local shell_config

    # Detect shell and config file
    case "$SHELL" in
        */bash) shell_config="$HOME/.bashrc" ;;
        */zsh)  shell_config="$HOME/.zshrc" ;;
        */fish) shell_config="$HOME/.config/fish/config.fish" ;;
        *) shell_config="$HOME/.profile" ;;
    esac

    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$install_path:"* ]]; then
        warn "Add $install_path to your PATH to use vx from anywhere:"
        echo "  echo 'export PATH=\"$install_path:\$PATH\"' >> $shell_config"
        echo "  source $shell_config"
        echo ""
        echo "Or add it manually to your shell configuration file"
    fi
}

# Verify installation
test_installation() {
    local binary_path="$1"

    if "$binary_path" --version >/dev/null 2>&1; then
        success "Installation verified successfully!"
        echo ""
        echo "🎉 vx is ready to use!"
        echo "📖 Try these commands:"
        echo "   vx --help"
        echo "   vx list"
        echo "   vx npm --version"
        echo "   vx uv --version"
    else
        error "Installation verification failed"
        exit 1
    fi
}

# Main execution function
main() {
    info "vx installer"
    echo ""

    # Install vx
    if [[ "$BUILD_FROM_SOURCE" == "true" ]]; then
        build_from_source
    else
        if ! install_from_release; then
            warn "Failed to download pre-built binary, falling back to building from source..."
            build_from_source
        fi
    fi

    # Update PATH and verify installation
    local binary_path="$VX_INSTALL_DIR/vx"
    update_path "$VX_INSTALL_DIR"
    test_installation "$binary_path"
}

# Run main function
main "$@"
