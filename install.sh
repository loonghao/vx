#!/usr/bin/env bash
# vx installer script for Linux and macOS
#
# Basic usage:
#   curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With specific version (use tag format like "vx-v0.5.7" or just "0.5.7"):
#   VX_VERSION="0.5.7" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
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

# Detect platform and architecture - returns Rust target triple
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="unknown-linux" ;;
        Darwin*) os="apple-darwin" ;;
        *)       error "Unsupported operating system: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac

    # Return Rust target triple format
    echo "$arch-$os"
}

# Get latest version from GitHub API with optional authentication and fallback
# Returns the full tag name (e.g., "vx-v0.5.7") of a release that has assets
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases?per_page=10"
    local auth_header=""

    # Check for GitHub token
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        auth_header="Authorization: Bearer $GITHUB_TOKEN"
        info "Using authenticated GitHub API request"
    else
        info "Using unauthenticated GitHub API request (rate limited)"
    fi

    # Try GitHub API - get releases list and find one with assets
    local response
    if command -v curl >/dev/null 2>&1; then
        if [[ -n "$auth_header" ]]; then
            response=$(curl -s -H "$auth_header" "$api_url" 2>/dev/null || echo "")
        else
            response=$(curl -s "$api_url" 2>/dev/null || echo "")
        fi

        # Check for rate limit error
        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429"; then
            # Find the first non-prerelease with assets using jq if available
            if command -v jq >/dev/null 2>&1; then
                local tag_name
                tag_name=$(echo "$response" | jq -r '
                    [.[] | select(.assets | length > 0) | select(.prerelease == false)] |
                    first | .tag_name // empty
                ')
                if [[ -n "$tag_name" && "$tag_name" != "null" ]]; then
                    echo "$tag_name"
                    return
                fi
            else
                # Fallback: use grep/sed to find first release with assets
                # This is less reliable but works without jq
                local tag_name
                tag_name=$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
                if [[ -n "$tag_name" ]]; then
                    echo "$tag_name"
                    return
                fi
            fi
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [[ -n "$auth_header" ]]; then
            response=$(wget -qO- --header="$auth_header" "$api_url" 2>/dev/null || echo "")
        else
            response=$(wget -qO- "$api_url" 2>/dev/null || echo "")
        fi

        # Check for rate limit error
        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429"; then
            if command -v jq >/dev/null 2>&1; then
                local tag_name
                tag_name=$(echo "$response" | jq -r '
                    [.[] | select(.assets | length > 0) | select(.prerelease == false)] |
                    first | .tag_name // empty
                ')
                if [[ -n "$tag_name" && "$tag_name" != "null" ]]; then
                    echo "$tag_name"
                    return
                fi
            else
                local tag_name
                tag_name=$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
                if [[ -n "$tag_name" ]]; then
                    echo "$tag_name"
                    return
                fi
            fi
        fi
    fi

    # If all else fails, provide helpful error message
    error "Unable to determine latest version automatically due to rate limiting."
    echo ""
    echo "Solutions:"
    echo "1. Set GITHUB_TOKEN environment variable:"
    echo "   GITHUB_TOKEN='your_token_here' $0"
    echo ""
    echo "2. Specify version explicitly:"
    echo "   VX_VERSION='vx-v0.5.7' $0"
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

# Download and install vx from GitHub releases with retry
install_from_release() {
    local platform tag_name version_number archive_name archive_name_versioned archive_name_legacy download_url temp_dir
    local max_retries=3
    local retry_delay=2

    platform=$(detect_platform)

    if [[ "$VX_VERSION" == "latest" ]]; then
        info "Fetching latest version..."
        tag_name=$(get_latest_version)
        if [[ -z "$tag_name" ]]; then
            error "Failed to get latest version"
            exit 1
        fi
    else
        # User specified version - could be "vx-v0.5.7" or "0.5.7"
        if [[ "$VX_VERSION" =~ ^vx-v ]]; then
            tag_name="$VX_VERSION"
        elif [[ "$VX_VERSION" =~ ^v ]]; then
            tag_name="vx-$VX_VERSION"
        else
            tag_name="vx-v$VX_VERSION"
        fi
    fi

    info "Installing vx $tag_name for $platform..."

    # Extract version number from tag (e.g., "vx-v0.5.7" -> "0.5.7", "v0.5.7" -> "0.5.7")
    version_number=$(echo "$tag_name" | sed -E 's/^(vx-)?v//')

    # Determine artifact naming format based on version
    # v0.6.0+ uses versioned naming (vx-0.6.1-target.tar.gz)
    # v0.5.x and earlier use legacy naming (vx-target.tar.gz)
    local major minor use_versioned_first
    major=$(echo "$version_number" | cut -d. -f1)
    minor=$(echo "$version_number" | cut -d. -f2)

    if [[ "$major" -gt 0 ]] || { [[ "$major" -eq 0 ]] && [[ "$minor" -ge 6 ]]; }; then
        use_versioned_first=true
    else
        use_versioned_first=false
    fi

    # Construct download URL based on Rust target triple
    # New format: vx-{version}-{target}.tar.gz (e.g., vx-0.6.1-x86_64-unknown-linux-gnu.tar.gz)
    # Legacy format: vx-{target}.tar.gz (e.g., vx-x86_64-unknown-linux-gnu.tar.gz)
    case "$platform" in
        x86_64-unknown-linux)
            # Try musl first (static binary), fallback to gnu
            archive_name_versioned="vx-${version_number}-x86_64-unknown-linux-musl.tar.gz"
            archive_name_legacy="vx-x86_64-unknown-linux-musl.tar.gz"
            fallback_archive_versioned="vx-${version_number}-x86_64-unknown-linux-gnu.tar.gz"
            fallback_archive_legacy="vx-x86_64-unknown-linux-gnu.tar.gz"
            ;;
        aarch64-unknown-linux)
            # Try musl first (static binary), fallback to gnu
            archive_name_versioned="vx-${version_number}-aarch64-unknown-linux-musl.tar.gz"
            archive_name_legacy="vx-aarch64-unknown-linux-musl.tar.gz"
            fallback_archive_versioned="vx-${version_number}-aarch64-unknown-linux-gnu.tar.gz"
            fallback_archive_legacy="vx-aarch64-unknown-linux-gnu.tar.gz"
            ;;
        x86_64-apple-darwin)
            archive_name_versioned="vx-${version_number}-x86_64-apple-darwin.tar.gz"
            archive_name_legacy="vx-x86_64-apple-darwin.tar.gz"
            ;;
        aarch64-apple-darwin)
            archive_name_versioned="vx-${version_number}-aarch64-apple-darwin.tar.gz"
            archive_name_legacy="vx-aarch64-apple-darwin.tar.gz"
            ;;
        *) error "Unsupported platform: $platform"; exit 1 ;;
    esac

    # Create temporary directory
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # Download from GitHub Releases with retry
    download_success=false

    # Prepare auth header for download if token is available
    local curl_auth_opts=""
    local wget_auth_opts=""
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        curl_auth_opts="-H \"Authorization: Bearer $GITHUB_TOKEN\""
        wget_auth_opts="--header=\"Authorization: Bearer $GITHUB_TOKEN\""
    fi

    # Order archives based on version - try the expected format first
    local archives_to_try
    if [[ "$use_versioned_first" == "true" ]]; then
        archives_to_try=("$archive_name_versioned" "$archive_name_legacy")
    else
        archives_to_try=("$archive_name_legacy" "$archive_name_versioned")
    fi
    local archive_name=""

    for try_archive in "${archives_to_try[@]}"; do
        local download_url="$BASE_URL/download/$tag_name/$try_archive"

        # Download with retry
        for retry in $(seq 1 $max_retries); do
            if [[ $retry -gt 1 ]]; then
                info "Retry attempt $retry of $max_retries..."
                sleep $retry_delay
            fi

            info "Downloading from GitHub Releases: $download_url"

            if command -v curl >/dev/null 2>&1; then
                if eval curl -fsSL --connect-timeout 10 --max-time 120 --retry 3 --retry-delay 2 $curl_auth_opts "\"$download_url\"" -o "\"$temp_dir/$try_archive\"" 2>/dev/null; then
                    # Verify download
                    if [[ -f "$temp_dir/$try_archive" ]] && [[ $(stat -f%z "$temp_dir/$try_archive" 2>/dev/null || stat -c%s "$temp_dir/$try_archive" 2>/dev/null || echo 0) -gt 1024 ]]; then
                        local file_size=$(stat -f%z "$temp_dir/$try_archive" 2>/dev/null || stat -c%s "$temp_dir/$try_archive" 2>/dev/null || echo 0)
                        success "Successfully downloaded ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                        download_success=true
                        archive_name="$try_archive"
                        break 2
                    fi
                fi
            elif command -v wget >/dev/null 2>&1; then
                if eval wget -q --timeout=120 --tries=3 --waitretry=2 $wget_auth_opts "\"$download_url\"" -O "\"$temp_dir/$try_archive\"" 2>/dev/null; then
                    # Verify download
                    if [[ -f "$temp_dir/$try_archive" ]] && [[ $(stat -f%z "$temp_dir/$try_archive" 2>/dev/null || stat -c%s "$temp_dir/$try_archive" 2>/dev/null || echo 0) -gt 1024 ]]; then
                        local file_size=$(stat -f%z "$temp_dir/$try_archive" 2>/dev/null || stat -c%s "$temp_dir/$try_archive" 2>/dev/null || echo 0)
                        success "Successfully downloaded ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                        download_success=true
                        archive_name="$try_archive"
                        break 2
                    fi
                fi
            else
                error "Neither curl nor wget is available"
                exit 1
            fi

            warn "Download attempt $retry failed, cleaning up..."
            rm -f "$temp_dir/$try_archive"
        done

        info "Archive $try_archive not found, trying next format..."
    done

    # Try fallback archives if primary failed and fallback exists
    if [[ "$download_success" != "true" ]] && [[ -n "${fallback_archive_versioned:-}" ]]; then
        warn "Primary archive failed, trying fallback archives..."
        local fallback_archives
        if [[ "$use_versioned_first" == "true" ]]; then
            fallback_archives=("$fallback_archive_versioned" "$fallback_archive_legacy")
        else
            fallback_archives=("$fallback_archive_legacy" "$fallback_archive_versioned")
        fi

        for fallback_archive in "${fallback_archives[@]}"; do
            local fallback_url="$BASE_URL/download/$tag_name/$fallback_archive"

            for retry in $(seq 1 $max_retries); do
                if [[ $retry -gt 1 ]]; then
                    info "Retry attempt $retry of $max_retries..."
                    sleep $retry_delay
                fi

                info "Downloading fallback from GitHub Releases: $fallback_url"

                if command -v curl >/dev/null 2>&1; then
                    if eval curl -fsSL --connect-timeout 10 --max-time 120 --retry 3 --retry-delay 2 $curl_auth_opts "\"$fallback_url\"" -o "\"$temp_dir/$fallback_archive\"" 2>/dev/null; then
                        if [[ -f "$temp_dir/$fallback_archive" ]] && [[ $(stat -f%z "$temp_dir/$fallback_archive" 2>/dev/null || stat -c%s "$temp_dir/$fallback_archive" 2>/dev/null || echo 0) -gt 1024 ]]; then
                            archive_name="$fallback_archive"
                            success "Successfully downloaded fallback"
                            download_success=true
                            break 2
                        fi
                    fi
                elif command -v wget >/dev/null 2>&1; then
                    if eval wget -q --timeout=120 --tries=3 --waitretry=2 $wget_auth_opts "\"$fallback_url\"" -O "\"$temp_dir/$fallback_archive\"" 2>/dev/null; then
                        if [[ -f "$temp_dir/$fallback_archive" ]] && [[ $(stat -f%z "$temp_dir/$fallback_archive" 2>/dev/null || stat -c%s "$temp_dir/$fallback_archive" 2>/dev/null || echo 0) -gt 1024 ]]; then
                            archive_name="$fallback_archive"
                            success "Successfully downloaded fallback"
                            download_success=true
                            break 2
                        fi
                    fi
                fi

                warn "Fallback download attempt $retry failed, cleaning up..."
                rm -f "$temp_dir/$fallback_archive"
            done
        done
    fi

    if [[ "$download_success" != "true" ]]; then
        error "Failed to download vx binary after $max_retries retries"
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

    success "vx $tag_name installed to $VX_INSTALL_DIR/vx"
}

# Update PATH environment variable
update_path() {
    local install_path="$1"
    local shell_config
    local path_export="export PATH=\"$install_path:\$PATH\""

    # Detect shell and config file
    case "$SHELL" in
        */bash) shell_config="$HOME/.bashrc" ;;
        */zsh)  shell_config="$HOME/.zshrc" ;;
        */fish) shell_config="$HOME/.config/fish/config.fish" ;;
        *) shell_config="$HOME/.profile" ;;
    esac

    # Check if install directory is already in PATH
    if [[ ":$PATH:" == *":$install_path:"* ]]; then
        info "Install directory already in PATH"
        return
    fi

    # Check if already configured in shell config
    if [[ -f "$shell_config" ]] && grep -q "$install_path" "$shell_config" 2>/dev/null; then
        info "PATH already configured in $shell_config"
        # Update current session
        export PATH="$install_path:$PATH"
        return
    fi

    # Automatically add to shell config
    if [[ -f "$shell_config" ]] || [[ -w "$(dirname "$shell_config")" ]]; then
        echo "" >> "$shell_config"
        echo "# Added by vx installer" >> "$shell_config"
        if [[ "$shell_config" == *"fish"* ]]; then
            echo "set -gx PATH \"$install_path\" \$PATH" >> "$shell_config"
        else
            echo "$path_export" >> "$shell_config"
        fi
        success "Added $install_path to PATH in $shell_config"

        # Update current session
        export PATH="$install_path:$PATH"
        info "Updated current session PATH"
    else
        warn "Could not automatically update PATH"
        echo "  Add this to your shell configuration:"
        echo "  $path_export"
    fi

    # Also check for CI environments (GitHub Actions, GitLab CI, etc.)
    if [[ -n "${GITHUB_PATH:-}" ]]; then
        echo "$install_path" >> "$GITHUB_PATH"
        info "Added to GITHUB_PATH for subsequent steps"
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
