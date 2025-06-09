#!/bin/bash
# Installation script for vx
# This script downloads and installs the latest version of vx from GitHub releases

set -e

# Default values
VERSION="latest"
INSTALL_DIR="$HOME/.vx/bin"

# GitHub repository information
OWNER="loonghao"
REPO="vx"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --version VERSION     Install specific version (default: latest)"
            echo "  --install-dir DIR     Installation directory (default: ~/.vx/bin)"
            echo "  -h, --help           Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Detect OS and architecture
detect_platform() {
    local os
    local arch
    
    case "$(uname -s)" in
        Linux*)
            os="Linux"
            ;;
        Darwin*)
            os="Darwin"
            ;;
        FreeBSD*)
            os="FreeBSD"
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${os}_${arch}"
}

# Download and install vx
install_vx() {
    local platform
    platform=$(detect_platform)
    
    log_info "Installing vx for $platform..."
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR"
    
    # Get latest version if not specified
    if [[ "$VERSION" == "latest" ]]; then
        log_info "Fetching latest release information..."
        VERSION=$(curl -s "https://api.github.com/repos/$OWNER/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
        if [[ -z "$VERSION" ]]; then
            log_error "Failed to fetch latest version"
            exit 1
        fi
    fi
    
    # Construct download URL
    local filename="vx_${platform}.tar.gz"
    local download_url="https://github.com/$OWNER/$REPO/releases/download/$VERSION/$filename"
    
    log_info "Downloading vx $VERSION..."
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT
    
    # Download and extract
    if command -v curl >/dev/null 2>&1; then
        curl -L "$download_url" | tar -xz -C "$temp_dir"
    elif command -v wget >/dev/null 2>&1; then
        wget -O- "$download_url" | tar -xz -C "$temp_dir"
    else
        log_error "Neither curl nor wget is available"
        exit 1
    fi
    
    # Move binary to installation directory
    if [[ -f "$temp_dir/vx" ]]; then
        mv "$temp_dir/vx" "$INSTALL_DIR/vx"
        chmod +x "$INSTALL_DIR/vx"
        log_info "vx installed successfully to $INSTALL_DIR/vx"
    else
        log_error "Binary not found in the downloaded archive"
        exit 1
    fi
    
    # Add to PATH if not already there
    local shell_profile
    if [[ -n "$ZSH_VERSION" ]]; then
        shell_profile="$HOME/.zshrc"
    elif [[ -n "$BASH_VERSION" ]]; then
        shell_profile="$HOME/.bashrc"
    else
        shell_profile="$HOME/.profile"
    fi
    
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_profile"
        log_info "Added $INSTALL_DIR to PATH in $shell_profile"
        log_warn "Please restart your terminal or run: source $shell_profile"
    else
        log_info "Directory already in PATH"
    fi
    
    log_info "Installation complete! Run 'vx --version' to verify."
}

# Main execution
main() {
    log_info "vx installer"
    install_vx
}

main "$@"
