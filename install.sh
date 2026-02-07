#!/usr/bin/env bash
# vx installer script for Linux and macOS
#
# Basic usage:
#   curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With specific version (use tag format like "v0.6.0" or just "0.6.0"):
#   VX_VERSION="0.5.7" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With GitHub token (to avoid rate limits):
#   GITHUB_TOKEN="your_token" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# Build from source:
#   BUILD_FROM_SOURCE=true curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# Use package manager (auto-detect or specify):
#   USE_PACKAGE_MANAGER=auto curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#   USE_PACKAGE_MANAGER=brew curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#   USE_PACKAGE_MANAGER=apt curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# Prefer static binary (musl):
#   PREFER_STATIC=true curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# Alternative package managers (manual):
#   # Homebrew: brew install loonghao/vx/vx
#   # Cargo: cargo install vx
#   # APT (Debian/Ubuntu): See instructions below

set -euo pipefail

# Configuration
REPO_OWNER="loonghao"
REPO_NAME="vx"
BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases"

# Default values
VX_VERSION="${VX_VERSION:-latest}"
VX_INSTALL_DIR="${VX_INSTALL_DIR:-$HOME/.local/bin}"
BUILD_FROM_SOURCE="${BUILD_FROM_SOURCE:-false}"
USE_PACKAGE_MANAGER="${USE_PACKAGE_MANAGER:-}"
PREFER_STATIC="${PREFER_STATIC:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions (all write to stderr to avoid polluting stdout used for return values)
info() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

# Detect available package managers
detect_package_manager() {
    local os_type
    os_type=$(uname -s)
    
    case "$os_type" in
        Darwin)
            # macOS - prefer Homebrew
            if command -v brew >/dev/null 2>&1; then
                echo "brew"
                return
            fi
            ;;
        Linux)
            # Check for various Linux package managers
            if command -v apt-get >/dev/null 2>&1; then
                echo "apt"
                return
            elif command -v dnf >/dev/null 2>&1; then
                echo "dnf"
                return
            elif command -v yum >/dev/null 2>&1; then
                echo "yum"
                return
            elif command -v pacman >/dev/null 2>&1; then
                echo "pacman"
                return
            elif command -v apk >/dev/null 2>&1; then
                echo "apk"
                return
            elif command -v zypper >/dev/null 2>&1; then
                echo "zypper"
                return
            elif command -v brew >/dev/null 2>&1; then
                # Linuxbrew
                echo "brew"
                return
            fi
            ;;
    esac
    
    echo "none"
}

# Install using Homebrew (macOS/Linux)
install_with_brew() {
    info "Installing vx using Homebrew..."
    
    # Check if tap exists, add if not
    if ! brew tap | grep -q "loonghao/vx"; then
        info "Adding loonghao/vx tap..."
        brew tap loonghao/vx
    fi
    
    if [[ "$VX_VERSION" == "latest" ]]; then
        brew install loonghao/vx/vx
    else
        # Install specific version if available
        brew install "loonghao/vx/vx@$VX_VERSION" 2>/dev/null || brew install loonghao/vx/vx
    fi
    
    success "vx installed via Homebrew"
    return 0
}

# Install using APT (Debian/Ubuntu)
install_with_apt() {
    info "Installing vx using APT..."
    
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT
    
    # Download the .deb package from GitHub releases
    local tag_name version_number deb_url deb_file arch
    
    if [[ "$VX_VERSION" == "latest" ]]; then
        tag_name=$(get_latest_version)
    else
        if [[ "$VX_VERSION" =~ ^v ]]; then
            tag_name="$VX_VERSION"
        else
            tag_name="v$VX_VERSION"
        fi
    fi
    
    version_number=$(echo "$tag_name" | sed -E 's/^(vx-)?v//')
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64) arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) error "Unsupported architecture for APT install: $(uname -m)"; return 1 ;;
    esac
    
    deb_file="vx_${version_number}_${arch}.deb"
    deb_url="$BASE_URL/download/$tag_name/$deb_file"
    
    info "Downloading $deb_file..."
    if curl -fsSL "$deb_url" -o "$temp_dir/$deb_file"; then
        info "Installing package..."
        sudo dpkg -i "$temp_dir/$deb_file" || sudo apt-get install -f -y
        success "vx installed via APT"
        return 0
    else
        warn "DEB package not available, falling back to binary install"
        return 1
    fi
}

# Install using DNF/YUM (Fedora/RHEL/CentOS)
install_with_dnf() {
    local pkg_manager="$1"
    info "Installing vx using $pkg_manager..."
    
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT
    
    # Download the .rpm package from GitHub releases
    local tag_name version_number rpm_url rpm_file arch
    
    if [[ "$VX_VERSION" == "latest" ]]; then
        tag_name=$(get_latest_version)
    else
        if [[ "$VX_VERSION" =~ ^v ]]; then
            tag_name="$VX_VERSION"
        else
            tag_name="v$VX_VERSION"
        fi
    fi
    
    version_number=$(echo "$tag_name" | sed -E 's/^(vx-)?v//')
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) error "Unsupported architecture for RPM install: $(uname -m)"; return 1 ;;
    esac
    
    rpm_file="vx-${version_number}-1.${arch}.rpm"
    rpm_url="$BASE_URL/download/$tag_name/$rpm_file"
    
    info "Downloading $rpm_file..."
    if curl -fsSL "$rpm_url" -o "$temp_dir/$rpm_file"; then
        info "Installing package..."
        sudo "$pkg_manager" install -y "$temp_dir/$rpm_file"
        success "vx installed via $pkg_manager"
        return 0
    else
        warn "RPM package not available, falling back to binary install"
        return 1
    fi
}

# Install using Pacman (Arch Linux)
install_with_pacman() {
    info "Installing vx using Pacman..."
    
    # Check if yay or paru is available for AUR
    if command -v yay >/dev/null 2>&1; then
        yay -S --noconfirm vx-bin 2>/dev/null || yay -S --noconfirm vx
        success "vx installed via yay (AUR)"
        return 0
    elif command -v paru >/dev/null 2>&1; then
        paru -S --noconfirm vx-bin 2>/dev/null || paru -S --noconfirm vx
        success "vx installed via paru (AUR)"
        return 0
    else
        warn "AUR helper (yay/paru) not found, falling back to binary install"
        return 1
    fi
}

# Install using APK (Alpine Linux)
install_with_apk() {
    info "Installing vx using APK..."
    
    # Alpine uses musl, so we'll download the static binary
    warn "APK package not available, will install static musl binary"
    PREFER_STATIC=true
    return 1
}

# Try to install using package manager
try_package_manager_install() {
    local pm="$1"
    
    case "$pm" in
        brew)
            install_with_brew && return 0
            ;;
        apt)
            install_with_apt && return 0
            ;;
        dnf)
            install_with_dnf "dnf" && return 0
            ;;
        yum)
            install_with_dnf "yum" && return 0
            ;;
        pacman)
            install_with_pacman && return 0
            ;;
        apk)
            install_with_apk && return 0
            ;;
        zypper)
            warn "Zypper package not yet available, falling back to binary install"
            return 1
            ;;
        *)
            return 1
            ;;
    esac
    
    return 1
}

# Detect platform and architecture - returns Rust target triple
detect_platform() {
    local os arch libc

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

    # For Linux, determine libc type (gnu vs musl)
    if [[ "$os" == "unknown-linux" ]]; then
        # Check if user prefers static binary
        if [[ "$PREFER_STATIC" == "true" ]]; then
            libc="musl"
        # Check if running on Alpine or musl-based system
        elif [[ -f /etc/alpine-release ]] || ldd --version 2>&1 | grep -q musl; then
            libc="musl"
        else
            libc="gnu"
        fi
        echo "$arch-$os-$libc"
    else
        # macOS doesn't have libc suffix
        echo "$arch-$os"
    fi
}

# Get latest version from GitHub API with optional authentication and fallback
# Returns the full tag name (e.g., "v0.5.7") of a release that has assets
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
            response=$(curl -s -H "$auth_header" -H "Accept: application/vnd.github.v3+json" "$api_url" 2>/dev/null || echo "")
        else
            response=$(curl -s -H "Accept: application/vnd.github.v3+json" "$api_url" 2>/dev/null || echo "")
        fi

        # Check for rate limit error
        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429\|API rate limit exceeded"; then
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

        # Rate limit hit - try fallback method via redirect
        if echo "$response" | grep -q "rate limit\|429\|API rate limit exceeded"; then
            warn "GitHub API rate limit exceeded. Trying fallback method..."
            # Try to get version from releases page redirect
            local redirect_url
            redirect_url=$(curl -sI "https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest" 2>/dev/null | grep -i "^location:" | sed 's/location: *//i' | tr -d '\r\n')
            if [[ "$redirect_url" =~ /releases/tag/(.+)$ ]]; then
                local tag_name="${BASH_REMATCH[1]}"
                info "Found version via redirect: $tag_name"
                echo "$tag_name"
                return
            fi
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [[ -n "$auth_header" ]]; then
            response=$(wget -qO- --header="$auth_header" --header="Accept: application/vnd.github.v3+json" "$api_url" 2>/dev/null || echo "")
        else
            response=$(wget -qO- --header="Accept: application/vnd.github.v3+json" "$api_url" 2>/dev/null || echo "")
        fi

        # Check for rate limit error
        if [[ -n "$response" ]] && ! echo "$response" | grep -q "rate limit\|429\|API rate limit exceeded"; then
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
    echo "" >&2
    echo "Solutions:" >&2
    echo "1. Set GITHUB_TOKEN environment variable:" >&2
    echo "   GITHUB_TOKEN='your_token_here' $0" >&2
    echo "" >&2
    echo "2. Specify version explicitly:" >&2
    echo "   VX_VERSION='0.6.7' $0" >&2
    echo "" >&2
    echo "3. Use package managers:" >&2
    echo "   brew install loonghao/vx/vx" >&2
    echo "   cargo install vx" >&2
    echo "" >&2
    echo "4. Download directly from:" >&2
    echo "   https://github.com/loonghao/vx/releases/latest" >&2
    echo "" >&2
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
        # User specified version - normalize to tag format
        # Accept: "v0.6.7", "0.6.7", "vx-v0.6.7"
        if [[ "$VX_VERSION" =~ ^vx-v ]]; then
            tag_name="$VX_VERSION"
        elif [[ "$VX_VERSION" =~ ^v ]]; then
            tag_name="$VX_VERSION"
        else
            tag_name="v$VX_VERSION"
        fi
    fi

    info "Installing vx $tag_name for $platform..."

    # Extract version number from tag (e.g., "v0.5.7" -> "0.5.7", "vx-v0.6.27" -> "0.6.27")
    version_number=$(echo "$tag_name" | sed -E 's/^(vx-)?v//')

    # Determine all possible tag formats for this version
    # v0.7.0+ uses v{ver} (cargo-dist), v0.6.x and earlier use vx-v{ver}
    local major minor
    major=$(echo "$version_number" | cut -d. -f1)
    minor=$(echo "$version_number" | cut -d. -f2)
    local tag_candidates=()
    if [[ "$major" -gt 0 ]] || [[ "$major" -eq 0 && "$minor" -ge 7 ]]; then
        # v0.7.0+: try v{ver} first, then vx-v{ver} as fallback
        tag_candidates=("v${version_number}" "vx-v${version_number}")
    else
        # v0.6.x and earlier: try vx-v{ver} first, then v{ver} as fallback
        tag_candidates=("vx-v${version_number}" "v${version_number}")
    fi
    # Override tag_name with the primary candidate
    tag_name="${tag_candidates[0]}"
    info "Using tag format: $tag_name (fallback: ${tag_candidates[1]:-none})"

    # Construct download URL based on Rust target triple
    # Three naming eras:
    #   v0.5.x: unversioned (vx-{triple}.tar.gz) with tag vx-v{ver}
    #   v0.6.x: versioned (vx-{ver}-{triple}.tar.gz) with tag vx-v{ver}
    #   v0.7.0+: unversioned (vx-{triple}.tar.gz) with tag v{ver} (cargo-dist)
    local fallback_archive=""
    local unversioned_archive=""
    
    # Determine the Rust target triple
    local target_triple=""
    local fallback_triple=""
    case "$platform" in
        x86_64-unknown-linux-gnu)
            target_triple="x86_64-unknown-linux-gnu"
            fallback_triple="x86_64-unknown-linux-musl"
            ;;
        x86_64-unknown-linux-musl)
            target_triple="x86_64-unknown-linux-musl"
            fallback_triple="x86_64-unknown-linux-gnu"
            ;;
        aarch64-unknown-linux-gnu)
            target_triple="aarch64-unknown-linux-gnu"
            fallback_triple="aarch64-unknown-linux-musl"
            ;;
        aarch64-unknown-linux-musl)
            target_triple="aarch64-unknown-linux-musl"
            fallback_triple="aarch64-unknown-linux-gnu"
            ;;
        x86_64-apple-darwin)
            target_triple="x86_64-apple-darwin"
            ;;
        aarch64-apple-darwin)
            target_triple="aarch64-apple-darwin"
            ;;
        *) error "Unsupported platform: $platform"; exit 1 ;;
    esac

    # Primary: try versioned naming (vx-{ver}-{triple}.tar.gz)
    archive_name="vx-${version_number}-${target_triple}.tar.gz"
    # Secondary: try unversioned naming (vx-{triple}.tar.gz) for cargo-dist and legacy
    unversioned_archive="vx-${target_triple}.tar.gz"
    # Tertiary: fallback triple (e.g., musl <-> gnu)
    if [[ -n "$fallback_triple" ]]; then
        fallback_archive="vx-${version_number}-${fallback_triple}.tar.gz"
    fi

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

    # Build comprehensive list of (tag, archive) combinations to try
    # Order: primary tag + versioned, primary tag + unversioned,
    #        fallback tag + versioned, fallback tag + unversioned,
    #        then same with fallback triples
    local try_list=()
    for try_tag in "${tag_candidates[@]}"; do
        try_list+=("$try_tag|$archive_name")
        if [[ "$unversioned_archive" != "$archive_name" ]]; then
            try_list+=("$try_tag|$unversioned_archive")
        fi
    done
    # Also try fallback triple if available
    if [[ -n "$fallback_triple" ]]; then
        local fallback_versioned="vx-${version_number}-${fallback_triple}.tar.gz"
        local fallback_unversioned="vx-${fallback_triple}.tar.gz"
        for try_tag in "${tag_candidates[@]}"; do
            try_list+=("$try_tag|$fallback_versioned")
            try_list+=("$try_tag|$fallback_unversioned")
        done
    fi

    # Try each (tag, archive) combination
    for combo in "${try_list[@]}"; do
        [[ "$download_success" == "true" ]] && break
        local try_tag="${combo%%|*}"
        local try_archive="${combo##*|}"
        local download_url="$BASE_URL/download/$try_tag/$try_archive"

        for retry in $(seq 1 $max_retries); do
            if [[ $retry -gt 1 ]]; then
                sleep $retry_delay
            fi

            info "Trying: $download_url"

            local dl_ok=false
            if command -v curl >/dev/null 2>&1; then
                if eval curl -fsSL --connect-timeout 10 --max-time 120 --retry 3 --retry-delay 2 $curl_auth_opts "\"$download_url\"" -o "\"$temp_dir/$try_archive\"" 2>/dev/null; then
                    dl_ok=true
                fi
            elif command -v wget >/dev/null 2>&1; then
                if eval wget -q --timeout=120 --tries=3 --waitretry=2 $wget_auth_opts "\"$download_url\"" -O "\"$temp_dir/$try_archive\"" 2>/dev/null; then
                    dl_ok=true
                fi
            else
                error "Neither curl nor wget is available"
                exit 1
            fi

            if [[ "$dl_ok" == "true" ]] && [[ -f "$temp_dir/$try_archive" ]]; then
                local file_size
                file_size=$(stat -f%z "$temp_dir/$try_archive" 2>/dev/null || stat -c%s "$temp_dir/$try_archive" 2>/dev/null || echo 0)
                if [[ "$file_size" -gt 1024 ]]; then
                    success "Successfully downloaded ($(echo "scale=2; $file_size/1024/1024" | bc 2>/dev/null || echo "unknown") MB)"
                    archive_name="$try_archive"
                    tag_name="$try_tag"
                    download_success=true
                    break
                fi
            fi

            rm -f "$temp_dir/$try_archive"
        done
    done

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
        echo "   vx uv self version"
    else
        error "Installation verification failed"
        exit 1
    fi
}

# Main execution function
main() {
    info "vx installer"
    echo ""

    # Try package manager install if requested
    if [[ -n "$USE_PACKAGE_MANAGER" ]]; then
        local pm="$USE_PACKAGE_MANAGER"
        
        # Auto-detect package manager if "auto" is specified
        if [[ "$pm" == "auto" ]]; then
            pm=$(detect_package_manager)
            if [[ "$pm" == "none" ]]; then
                info "No supported package manager found, falling back to binary install"
            else
                info "Detected package manager: $pm"
            fi
        fi
        
        if [[ "$pm" != "none" ]]; then
            if try_package_manager_install "$pm"; then
                # Package manager install succeeded
                test_installation "$(command -v vx || echo "/usr/local/bin/vx")"
                return 0
            fi
            # Package manager install failed, continue with binary install
            info "Package manager install failed, falling back to binary install"
        fi
    fi

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
