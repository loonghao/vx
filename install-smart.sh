#!/bin/bash
# Smart vx installer with intelligent channel selection and fallback
# This installer automatically detects the best distribution channel based on
# geographic location, network conditions, and availability
#
# Usage: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install-smart.sh | bash
# Usage with version: VX_VERSION="0.1.0" bash <(curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install-smart.sh)
# Usage with token: GITHUB_TOKEN="token" bash <(curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install-smart.sh)

set -euo pipefail

# Configuration
REPO_OWNER="loonghao"
REPO_NAME="vx"
VX_VERSION="${VX_VERSION:-latest}"
VX_INSTALL_DIR="${VX_INSTALL_DIR:-$HOME/.local/bin}"
VX_BUILD_FROM_SOURCE="${VX_BUILD_FROM_SOURCE:-false}"
VX_FORCE_CHANNEL="${VX_FORCE_CHANNEL:-}"
PREFER_STATIC="${PREFER_STATIC:-false}"
USE_PACKAGE_MANAGER="${USE_PACKAGE_MANAGER:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

debug() {
    if [[ "${VX_DEBUG:-}" == "true" ]]; then
        echo -e "${CYAN}[DEBUG]${NC} $1" >&2
    fi
}

# Detect platform
detect_platform() {
    local os arch libc
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch"; exit 1 ;;
    esac

    case "$os" in
        linux)
            # Determine libc type for Linux
            if [[ "$PREFER_STATIC" == "true" ]]; then
                libc="musl"
            elif [[ -f /etc/alpine-release ]] || ldd --version 2>&1 | grep -q musl; then
                libc="musl"
            else
                libc="gnu"
            fi
            echo "linux-$libc-$arch"
            ;;
        darwin)
            echo "darwin-$arch"
            ;;
        *)
            error "Unsupported OS: $os"
            exit 1
            ;;
    esac
}

# Detect geographic region for optimal CDN selection
detect_region() {
    local region="global"

    # Try to detect region from various sources
    if command -v curl >/dev/null 2>&1; then
        # Try ipinfo.io for region detection
        local country
        country=$(curl -s --connect-timeout 3 --max-time 5 "https://ipinfo.io/country" 2>/dev/null || echo "")

        case "$country" in
            CN|HK|TW|SG|JP|KR|MY|TH|VN|ID|PH) region="asia" ;;
            US|CA|MX|BR|AR|CL|PE|CO|VE) region="americas" ;;
            GB|DE|FR|IT|ES|NL|SE|NO|DK|FI|PL|RU) region="europe" ;;
            AU|NZ) region="oceania" ;;
            *) region="global" ;;
        esac
    fi

    debug "Detected region: $region"
    echo "$region"
}

# Test channel speed and availability
test_channel_speed() {
    local url="$1"
    local timeout="${2:-5}"

    if command -v curl >/dev/null 2>&1; then
        # Test with a small HEAD request
        local start_time end_time duration
        start_time=$(date +%s%N 2>/dev/null || date +%s)

        if curl -s --head --connect-timeout "$timeout" --max-time "$timeout" "$url" >/dev/null 2>&1; then
            end_time=$(date +%s%N 2>/dev/null || date +%s)
            if [[ "$start_time" =~ N ]]; then
                duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
            else
                duration=$(( (end_time - start_time) * 1000 ))     # Convert to milliseconds
            fi
            echo "$duration"
            return 0
        fi
    fi

    echo "999999"  # Return high value for failed tests
    return 1
}

# Get optimal channel order based on region and speed tests
get_optimal_channels() {
    local region="$1"
    local version="$2"
    local platform="$3"

    # Define all available channels
    local -A channels=(
        ["github"]="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/v$version"
        ["jsdelivr"]="https://cdn.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version"
        ["fastly"]="https://fastly.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version"
    )

    # Region-specific channel preferences
    local channel_order
    case "$region" in
        "asia")
            channel_order=("jsdelivr" "fastly" "github")
            ;;
        "europe")
            channel_order=("fastly" "jsdelivr" "github")
            ;;
        "americas")
            channel_order=("github" "fastly" "jsdelivr")
            ;;
        *)
            channel_order=("github" "jsdelivr" "fastly")
            ;;
    esac

    # If user forced a specific channel, use it first
    if [[ -n "$VX_FORCE_CHANNEL" ]]; then
        debug "Using forced channel: $VX_FORCE_CHANNEL"
        echo "$VX_FORCE_CHANNEL ${channel_order[@]}" | tr ' ' '\n' | awk '!seen[$0]++'
        return
    fi

    # Test channel speeds (optional, can be disabled for faster installs)
    if [[ "${VX_SPEED_TEST:-true}" == "true" ]]; then
        info "Testing channel speeds..."
        local -A speeds

        for channel in "${channel_order[@]}"; do
            local test_url="${channels[$channel]}"
            local speed
            speed=$(test_channel_speed "$test_url" 3)
            speeds[$channel]=$speed
            debug "Channel $channel speed: ${speed}ms"
        done

        # Sort channels by speed
        for channel in $(printf '%s\n' "${!speeds[@]}" | sort -k1,1 -t' ' | while read -r ch; do echo "$ch ${speeds[$ch]}"; done | sort -k2,2n | cut -d' ' -f1); do
            echo "$channel"
        done
    else
        printf '%s\n' "${channel_order[@]}"
    fi
}

# Get latest version with intelligent fallback
get_latest_version() {
    local region="$1"

    # Try GitHub API first (with auth if available)
    local api_url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    local auth_header=""

    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        auth_header="Authorization: Bearer $GITHUB_TOKEN"
        info "Using authenticated GitHub API request"
    fi

    if command -v curl >/dev/null 2>&1; then
        local response
        if [[ -n "$auth_header" ]]; then
            response=$(curl -s -H "$auth_header" "$api_url" 2>/dev/null || echo "")
        else
            response=$(curl -s "$api_url" 2>/dev/null || echo "")
        fi

        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429"; then
            local version
            version=$(echo "$response" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//' || echo "")
            if [[ -n "$version" ]]; then
                debug "Got version from GitHub API: $version"
                echo "$version"
                return
            fi
        fi
    fi

    # Fallback to jsDelivr API
    warn "GitHub API unavailable, trying jsDelivr API..."
    local jsdelivr_url="https://data.jsdelivr.com/v1/package/gh/$REPO_OWNER/$REPO_NAME"

    if command -v curl >/dev/null 2>&1; then
        local jsdelivr_response
        jsdelivr_response=$(curl -s "$jsdelivr_url" 2>/dev/null || echo "")
        if [[ -n "$jsdelivr_response" ]]; then
            local version
            version=$(echo "$jsdelivr_response" | grep -o '"version":"[^"]*"' | head -1 | sed 's/"version":"//;s/"//' | sed 's/^v//' || echo "")
            if [[ -n "$version" ]]; then
                debug "Got version from jsDelivr API: $version"
                echo "$version"
                return
            fi
        fi
    fi

    # Final fallback: provide helpful error message
    error "Unable to determine latest version automatically"
    echo ""
    echo "🔧 Solutions:"
    echo "1. Set GITHUB_TOKEN: GITHUB_TOKEN='token' $0"
    echo "2. Specify version: VX_VERSION='0.1.0' $0"
    echo "3. Use package managers: brew install loonghao/vx/vx"
    echo "4. Build from source: VX_BUILD_FROM_SOURCE=true $0"
    exit 1
}

# Download with intelligent channel selection
download_with_smart_fallback() {
    local version="$1"
    local platform="$2"
    local archive_name="$3"
    local temp_dir="$4"
    local region="$5"

    local archive_path="$temp_dir/$archive_name"
    local channels
    readarray -t channels < <(get_optimal_channels "$region" "$version" "$platform")

    info "Trying channels in optimal order for region: $region"

    for channel in "${channels[@]}"; do
        local download_url
        case "$channel" in
            "github")
                download_url="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/v$version/$archive_name"
                ;;
            "jsdelivr")
                download_url="https://cdn.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version/$archive_name"
                ;;
            "fastly")
                download_url="https://fastly.jsdelivr.net/gh/$REPO_OWNER/$REPO_NAME@v$version/$archive_name"
                ;;
            *)
                warn "Unknown channel: $channel"
                continue
                ;;
        esac

        info "Trying $channel: $download_url"

        if command -v curl >/dev/null 2>&1; then
            if curl -fsSL --connect-timeout 10 --max-time 30 "$download_url" -o "$archive_path"; then
                # Verify download
                if [[ -f "$archive_path" ]] && [[ $(stat -f%z "$archive_path" 2>/dev/null || stat -c%s "$archive_path" 2>/dev/null || echo 0) -gt 1024 ]]; then
                    local file_size
                    file_size=$(stat -f%z "$archive_path" 2>/dev/null || stat -c%s "$archive_path" 2>/dev/null || echo 0)
                    success "Downloaded from $channel ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                    return 0
                else
                    warn "Downloaded file too small, trying next channel..."
                    rm -f "$archive_path"
                fi
            fi
        elif command -v wget >/dev/null 2>&1; then
            if wget -q --timeout=30 "$download_url" -O "$archive_path"; then
                # Verify download
                if [[ -f "$archive_path" ]] && [[ $(stat -f%z "$archive_path" 2>/dev/null || stat -c%s "$archive_path" 2>/dev/null || echo 0) -gt 1024 ]]; then
                    local file_size
                    file_size=$(stat -f%z "$archive_path" 2>/dev/null || stat -c%s "$archive_path" 2>/dev/null || echo 0)
                    success "Downloaded from $channel ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                    return 0
                else
                    warn "Downloaded file too small, trying next channel..."
                    rm -f "$archive_path"
                fi
            fi
        fi

        warn "Failed to download from $channel"
    done

    return 1
}

# Install from release with smart channel selection
install_from_release() {
    local platform version archive_name temp_dir region

    platform=$(detect_platform)
    region=$(detect_region)

    if [[ "$VX_VERSION" == "latest" ]]; then
        info "Fetching latest version..."
        version=$(get_latest_version "$region")
        if [[ -z "$version" ]]; then
            error "Failed to get latest version"
            exit 1
        fi
    else
        version="$VX_VERSION"
    fi

    info "Installing vx v$version for $platform (region: $region)"

    # Determine archive name based on platform
    # New format: vx-{version}-{target}.tar.gz (e.g., vx-0.6.0-x86_64-unknown-linux-musl.tar.gz)
    # Legacy format: vx-{target}.tar.gz (e.g., vx-x86_64-unknown-linux-musl.tar.gz)
    local archive_name_versioned archive_name_legacy fallback_archive_versioned fallback_archive_legacy
    case "$platform" in
        linux-gnu-x86_64)
            archive_name_versioned="vx-$version-x86_64-unknown-linux-gnu.tar.gz"
            archive_name_legacy="vx-x86_64-unknown-linux-gnu.tar.gz"
            fallback_archive_versioned="vx-$version-x86_64-unknown-linux-musl.tar.gz"
            fallback_archive_legacy="vx-x86_64-unknown-linux-musl.tar.gz"
            ;;
        linux-musl-x86_64)
            archive_name_versioned="vx-$version-x86_64-unknown-linux-musl.tar.gz"
            archive_name_legacy="vx-x86_64-unknown-linux-musl.tar.gz"
            fallback_archive_versioned="vx-$version-x86_64-unknown-linux-gnu.tar.gz"
            fallback_archive_legacy="vx-x86_64-unknown-linux-gnu.tar.gz"
            ;;
        linux-gnu-aarch64)
            archive_name_versioned="vx-$version-aarch64-unknown-linux-gnu.tar.gz"
            archive_name_legacy="vx-aarch64-unknown-linux-gnu.tar.gz"
            fallback_archive_versioned="vx-$version-aarch64-unknown-linux-musl.tar.gz"
            fallback_archive_legacy="vx-aarch64-unknown-linux-musl.tar.gz"
            ;;
        linux-musl-aarch64)
            archive_name_versioned="vx-$version-aarch64-unknown-linux-musl.tar.gz"
            archive_name_legacy="vx-aarch64-unknown-linux-musl.tar.gz"
            fallback_archive_versioned="vx-$version-aarch64-unknown-linux-gnu.tar.gz"
            fallback_archive_legacy="vx-aarch64-unknown-linux-gnu.tar.gz"
            ;;
        darwin-x86_64)
            archive_name_versioned="vx-$version-x86_64-apple-darwin.tar.gz"
            archive_name_legacy="vx-x86_64-apple-darwin.tar.gz"
            ;;
        darwin-aarch64)
            archive_name_versioned="vx-$version-aarch64-apple-darwin.tar.gz"
            archive_name_legacy="vx-aarch64-apple-darwin.tar.gz"
            ;;
        *) error "Unsupported platform: $platform"; exit 1 ;;
    esac

    # Try versioned archive first, then fallback to legacy
    archive_name="$archive_name_versioned"

    # Create temporary directory
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # Download with smart fallback - try versioned first, then legacy
    local download_success=false

    # Try versioned archive first
    if download_with_smart_fallback "$version" "$platform" "$archive_name_versioned" "$temp_dir" "$region"; then
        archive_name="$archive_name_versioned"
        download_success=true
    # Try legacy archive format
    elif download_with_smart_fallback "$version" "$platform" "$archive_name_legacy" "$temp_dir" "$region"; then
        archive_name="$archive_name_legacy"
        download_success=true
    fi

    # Try fallback archives for Linux (musl -> gnu) if primary failed
    if [[ "$download_success" != "true" ]] && [[ "$platform" == linux-musl-* ]]; then
        warn "Musl binary failed, trying GNU libc version..."

        # Try versioned fallback first
        if [[ -n "${fallback_archive_versioned:-}" ]] && download_with_smart_fallback "$version" "$platform" "$fallback_archive_versioned" "$temp_dir" "$region"; then
            archive_name="$fallback_archive_versioned"
            download_success=true
        # Try legacy fallback
        elif [[ -n "${fallback_archive_legacy:-}" ]] && download_with_smart_fallback "$version" "$platform" "$fallback_archive_legacy" "$temp_dir" "$region"; then
            archive_name="$fallback_archive_legacy"
            download_success=true
        fi
    fi

    if [[ "$download_success" != "true" ]]; then
        error "Failed to download from all channels and fallbacks"
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

# Build from source (fallback method)
build_from_source() {
    info "Building vx from source..."

    # Check for required tools
    if ! command -v git >/dev/null 2>&1; then
        error "git is required to build from source"
        exit 1
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        error "Rust/Cargo is required to build from source"
        error "Install Rust: https://rustup.rs/"
        exit 1
    fi

    # Clone and build
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT
    cd "$temp_dir"

    info "Cloning repository..."
    git clone "https://github.com/$REPO_OWNER/$REPO_NAME.git"
    cd "$REPO_NAME"

    if [[ "$VX_VERSION" != "latest" ]]; then
        info "Checking out version v$VX_VERSION..."
        git checkout "v$VX_VERSION"
    fi

    info "Building vx..."
    cargo build --release

    # Install binary
    mkdir -p "$VX_INSTALL_DIR"
    cp "target/release/vx" "$VX_INSTALL_DIR/vx"
    chmod +x "$VX_INSTALL_DIR/vx"

    success "vx built and installed from source to $VX_INSTALL_DIR/vx"
}

# Update PATH
update_path() {
    local install_path="$1"
    local shell_profile

    # Detect shell and profile file
    case "$SHELL" in
        */bash) shell_profile="$HOME/.bashrc" ;;
        */zsh)  shell_profile="$HOME/.zshrc" ;;
        */fish) shell_profile="$HOME/.config/fish/config.fish" ;;
        *) shell_profile="$HOME/.profile" ;;
    esac

    # Check if directory is already in PATH
    if [[ ":$PATH:" == *":$install_path:"* ]]; then
        info "Install directory already in PATH"
        return
    fi

    # Add to PATH in profile
    if [[ -f "$shell_profile" ]]; then
        echo "export PATH=\"$install_path:\$PATH\"" >> "$shell_profile"
        info "Added $install_path to PATH in $shell_profile"
        info "Run 'source $shell_profile' or restart your shell to use vx"
    else
        warn "Could not update PATH automatically"
        info "Add this to your shell profile: export PATH=\"$install_path:\$PATH\""
    fi
}

# Test installation
test_installation() {
    local binary_path="$1"

    if [[ -x "$binary_path" ]]; then
        local version_output
        version_output=$("$binary_path" --version 2>/dev/null || echo "")
        if [[ -n "$version_output" ]]; then
            success "Installation verified: $version_output"
        else
            warn "Binary installed but version check failed"
        fi
    else
        error "Installation failed: binary not found or not executable"
        exit 1
    fi
}

# Main execution
main() {
    info "vx smart installer"
    echo ""

    # Show configuration
    debug "Configuration:"
    debug "  Version: $VX_VERSION"
    debug "  Install Dir: $VX_INSTALL_DIR"
    debug "  Build from Source: $VX_BUILD_FROM_SOURCE"
    debug "  Force Channel: ${VX_FORCE_CHANNEL:-auto}"
    debug "  Speed Test: ${VX_SPEED_TEST:-true}"

    # Check if we should build from source
    if [[ "$VX_BUILD_FROM_SOURCE" == "true" ]]; then
        build_from_source
    else
        install_from_release
    fi

    # Update PATH and test
    update_path "$VX_INSTALL_DIR"
    test_installation "$VX_INSTALL_DIR/vx"

    echo ""
    success "vx installation completed!"
    info "Run 'vx --help' to get started"

    # Show some helpful commands
    echo ""
    echo "📖 Quick start:"
    echo "   vx --help          # Show help"
    echo "   vx list            # List available tools"
    echo "   vx npm --version   # Use npm through vx"
    echo "   vx uv self version    # Use uv through vx"
}

# Run main function
main "$@"
