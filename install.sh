#!/usr/bin/env bash
# vx installer script for Linux and macOS with multi-channel distribution support
#
# Basic usage:
#   curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With specific version:
#   VX_VERSION="0.1.0" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With GitHub token (to avoid rate limits):
#   GITHUB_TOKEN="your_token" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# Build from source:
#   BUILD_FROM_SOURCE=true curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# Alternative package managers:
#   # Homebrew: brew install loonghao/vx/vx
#   # Cargo: cargo install vx

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

# Get latest version from GitHub API with optional authentication and fallback
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    local auth_header=""

    # Check for GitHub token
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        auth_header="Authorization: Bearer $GITHUB_TOKEN"
        info "Using authenticated GitHub API request"
    else
        info "Using unauthenticated GitHub API request (rate limited)"
    fi

    # Try GitHub API first
    local response
    if command -v curl >/dev/null 2>&1; then
        if [[ -n "$auth_header" ]]; then
            response=$(curl -s -H "$auth_header" "$api_url" 2>/dev/null || echo "")
        else
            response=$(curl -s "$api_url" 2>/dev/null || echo "")
        fi

        # Check for rate limit error
        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429"; then
            echo "$response" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//' || echo ""
            return
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [[ -n "$auth_header" ]]; then
            response=$(wget -qO- --header="$auth_header" "$api_url" 2>/dev/null || echo "")
        else
            response=$(wget -qO- "$api_url" 2>/dev/null || echo "")
        fi

        # Check for rate limit error
        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429"; then
            echo "$response" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//' || echo ""
            return
        fi
    fi

    # Fallback: try jsDelivr API
    warn "GitHub API unavailable, trying alternative methods..."
    local jsdelivr_url="https://data.jsdelivr.com/v1/package/gh/$REPO_OWNER/$REPO_NAME"

    if command -v curl >/dev/null 2>&1; then
        local jsdelivr_response
        jsdelivr_response=$(curl -s "$jsdelivr_url" 2>/dev/null || echo "")
        if [[ -n "$jsdelivr_response" ]]; then
            echo "$jsdelivr_response" | grep -o '"version":"[^"]*"' | head -1 | sed 's/"version":"//;s/"//' | sed 's/^v//' || echo ""
            return
        fi
    elif command -v wget >/dev/null 2>&1; then
        local jsdelivr_response
        jsdelivr_response=$(wget -qO- "$jsdelivr_url" 2>/dev/null || echo "")
        if [[ -n "$jsdelivr_response" ]]; then
            echo "$jsdelivr_response" | grep -o '"version":"[^"]*"' | head -1 | sed 's/"version":"//;s/"//' | sed 's/^v//' || echo ""
            return
        fi
    fi

    # If all else fails, provide helpful error message
    error "Unable to determine latest version automatically due to rate limiting."
    echo ""
    echo "🔧 Solutions:"
    echo "1. Set GITHUB_TOKEN environment variable:"
    echo "   GITHUB_TOKEN='your_token_here' $0"
    echo ""
    echo "2. Specify version explicitly:"
    echo "   VX_VERSION='0.1.0' $0"
    echo ""
    echo "3. Use package managers:"
    echo "   brew install loonghao/vx/vx"
    echo "   cargo install vx"
    echo ""
    echo "4. Build from source:"
    echo "   BUILD_FROM_SOURCE=true $0"
    echo ""
    exit 1
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

    # Create temporary directory
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # Download with multi-channel fallback support
    download_success=false

    # Define download channels in order of preference
    local channels=(
        "GitHub Releases|$BASE_URL/download/v$version/$archive_name"
        "jsDelivr CDN|https://cdn.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version/$archive_name"
        "Fastly CDN|https://fastly.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version/$archive_name"
    )

    # Try each channel
    for channel_info in "${channels[@]}"; do
        local channel_name="${channel_info%%|*}"
        local download_url="${channel_info##*|}"

        info "Trying $channel_name: $download_url"

        if command -v curl >/dev/null 2>&1; then
            if curl -fsSL --connect-timeout 10 --max-time 30 "$download_url" -o "$temp_dir/$archive_name" 2>/dev/null; then
                # Verify download
                if [[ -f "$temp_dir/$archive_name" ]] && [[ $(stat -f%z "$temp_dir/$archive_name" 2>/dev/null || stat -c%s "$temp_dir/$archive_name" 2>/dev/null || echo 0) -gt 1024 ]]; then
                    local file_size=$(stat -f%z "$temp_dir/$archive_name" 2>/dev/null || stat -c%s "$temp_dir/$archive_name" 2>/dev/null || echo 0)
                    success "Successfully downloaded from $channel_name ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                    download_success=true
                    break
                else
                    warn "Downloaded file too small, trying next channel..."
                    rm -f "$temp_dir/$archive_name"
                fi
            fi
        elif command -v wget >/dev/null 2>&1; then
            if wget -q --timeout=30 "$download_url" -O "$temp_dir/$archive_name" 2>/dev/null; then
                # Verify download
                if [[ -f "$temp_dir/$archive_name" ]] && [[ $(stat -f%z "$temp_dir/$archive_name" 2>/dev/null || stat -c%s "$temp_dir/$archive_name" 2>/dev/null || echo 0) -gt 1024 ]]; then
                    local file_size=$(stat -f%z "$temp_dir/$archive_name" 2>/dev/null || stat -c%s "$temp_dir/$archive_name" 2>/dev/null || echo 0)
                    success "Successfully downloaded from $channel_name ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                    download_success=true
                    break
                else
                    warn "Downloaded file too small, trying next channel..."
                    rm -f "$temp_dir/$archive_name"
                fi
            fi
        else
            error "Neither curl nor wget is available"
            exit 1
        fi

        warn "Failed to download from $channel_name"
    done

    # Try fallback archive if primary failed and fallback exists
    if [[ "$download_success" != "true" ]] && [[ -n "${fallback_archive:-}" ]]; then
        warn "All channels failed for primary archive, trying fallback archive..."

        local fallback_channels=(
            "GitHub Releases|$BASE_URL/download/v$version/$fallback_archive"
            "jsDelivr CDN|https://cdn.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version/$fallback_archive"
            "Fastly CDN|https://fastly.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version/$fallback_archive"
        )

        for channel_info in "${fallback_channels[@]}"; do
            local channel_name="${channel_info%%|*}"
            local download_url="${channel_info##*|}"

            info "Trying $channel_name (fallback): $download_url"

            if command -v curl >/dev/null 2>&1; then
                if curl -fsSL --connect-timeout 10 --max-time 30 "$download_url" -o "$temp_dir/$fallback_archive" 2>/dev/null; then
                    if [[ -f "$temp_dir/$fallback_archive" ]] && [[ $(stat -f%z "$temp_dir/$fallback_archive" 2>/dev/null || stat -c%s "$temp_dir/$fallback_archive" 2>/dev/null || echo 0) -gt 1024 ]]; then
                        archive_name="$fallback_archive"
                        success "Successfully downloaded fallback from $channel_name"
                        download_success=true
                        break
                    fi
                fi
            elif command -v wget >/dev/null 2>&1; then
                if wget -q --timeout=30 "$download_url" -O "$temp_dir/$fallback_archive" 2>/dev/null; then
                    if [[ -f "$temp_dir/$fallback_archive" ]] && [[ $(stat -f%z "$temp_dir/$fallback_archive" 2>/dev/null || stat -c%s "$temp_dir/$fallback_archive" 2>/dev/null || echo 0) -gt 1024 ]]; then
                        archive_name="$fallback_archive"
                        success "Successfully downloaded fallback from $channel_name"
                        download_success=true
                        break
                    fi
                fi
            fi

            warn "Failed to download fallback from $channel_name"
        done
    fi

    if [[ "$download_success" != "true" ]]; then
        error "Failed to download vx binary from all channels"
        error "Available channels: GitHub Releases, jsDelivr CDN, Fastly CDN"
        error "Try building from source with: BUILD_FROM_SOURCE=true $0"
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
