#!/bin/bash
# vx Universal Development Tool Manager Installation Script
# This script detects your platform and installs vx using the appropriate package manager

set -e

# Default values
VERSION="latest"
INSTALL_DIR="$HOME/.vx/bin"
USE_PACKAGE_MANAGER="auto"

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
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
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
        --package-manager)
            USE_PACKAGE_MANAGER="$2"
            shift 2
            ;;
        --no-package-manager)
            USE_PACKAGE_MANAGER="false"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --version VERSION         Install specific version (default: latest)"
            echo "  --install-dir DIR         Installation directory (default: ~/.vx/bin)"
            echo "  --package-manager TYPE    Use specific package manager (auto|brew|apt|yum|pacman|false)"
            echo "  --no-package-manager      Skip package manager, use direct download"
            echo "  -h, --help               Show this help message"
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

# Try to install via package manager
try_package_manager() {
    local os_name
    os_name=$(uname -s | tr '[:upper:]' '[:lower:]')

    case $os_name in
        linux)
            if [[ "$USE_PACKAGE_MANAGER" == "auto" || "$USE_PACKAGE_MANAGER" == "apt" ]] && command_exists apt-get; then
                log_info "Attempting to install via apt-get..."
                log_warn "Package not yet available in official repos. Using direct download."
                return 1
            elif [[ "$USE_PACKAGE_MANAGER" == "auto" || "$USE_PACKAGE_MANAGER" == "yum" ]] && command_exists yum; then
                log_info "Attempting to install via yum..."
                log_warn "Package not yet available in official repos. Using direct download."
                return 1
            elif [[ "$USE_PACKAGE_MANAGER" == "auto" || "$USE_PACKAGE_MANAGER" == "pacman" ]] && command_exists pacman; then
                log_info "Attempting to install via pacman (AUR)..."
                if command_exists yay; then
                    log_info "Installing from AUR using yay..."
                    yay -S vx-bin --noconfirm
                    return 0
                elif command_exists paru; then
                    log_info "Installing from AUR using paru..."
                    paru -S vx-bin --noconfirm
                    return 0
                else
                    log_warn "AUR helper not found. Using direct download."
                    return 1
                fi
            else
                return 1
            fi
            ;;
        darwin)
            if [[ "$USE_PACKAGE_MANAGER" == "auto" || "$USE_PACKAGE_MANAGER" == "brew" ]] && command_exists brew; then
                log_info "Installing via Homebrew..."
                brew tap loonghao/tap 2>/dev/null || true
                brew install vx
                return 0
            else
                return 1
            fi
            ;;
        *)
            return 1
            ;;
    esac
}

# Download and install vx directly
install_vx_direct() {
    local platform
    platform=$(detect_platform)

    log_info "Installing vx for $platform via direct download..."
    
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

# Verify installation
verify_installation() {
    if command_exists vx; then
        local version
        version=$(vx --version 2>/dev/null || echo "unknown")
        log_success "vx is installed and working! Version: $version"
        log_info "Run 'vx --help' to get started."
        return 0
    else
        log_error "Installation verification failed. vx command not found."
        log_info "You may need to restart your shell or update your PATH."
        return 1
    fi
}

# Main execution
main() {
    echo "🚀 vx Universal Development Tool Manager Installer"
    echo "=================================================="
    echo

    log_info "Starting vx installation..."

    # Check if already installed
    if command_exists vx; then
        local current_version
        current_version=$(vx --version 2>/dev/null || echo "unknown")
        log_warn "vx is already installed (version: $current_version)"
        read -p "Do you want to reinstall/update? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Installation cancelled."
            exit 0
        fi
    fi

    # Try package manager first (unless disabled)
    if [[ "$USE_PACKAGE_MANAGER" != "false" ]]; then
        if try_package_manager; then
            log_success "Successfully installed via package manager!"
            verify_installation
            echo
            log_success "🎉 Installation completed successfully!"
            echo
            log_info "Next steps:"
            echo "  1. Run 'vx --help' to see available commands"
            echo "  2. Run 'vx list' to see available tools"
            echo "  3. Run 'vx install <tool>' to install a development tool"
            echo
            log_info "Documentation: https://github.com/loonghao/vx"
            return 0
        fi
    fi

    # Fall back to direct download
    log_info "Falling back to direct download..."
    install_vx_direct
    verify_installation

    echo
    log_success "🎉 Installation completed successfully!"
    echo
    log_info "Next steps:"
    echo "  1. Run 'vx --help' to see available commands"
    echo "  2. Run 'vx list' to see available tools"
    echo "  3. Run 'vx install <tool>' to install a development tool"
    echo
    log_info "Documentation: https://github.com/loonghao/vx"
}

main "$@"
