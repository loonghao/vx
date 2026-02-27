#!/usr/bin/env bash
# vx installer script for Linux and macOS
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With specific version:
#   VX_VERSION="0.7.0" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With custom install directory:
#   VX_INSTALL_DIR="$HOME/bin" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With GitHub token (to avoid rate limits when specifying a version):
#   GITHUB_TOKEN="your_token" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

set -euo pipefail

REPO_OWNER="loonghao"
REPO_NAME="vx"
BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases"

VX_VERSION="${VX_VERSION:-}"
VX_INSTALL_DIR="${VX_INSTALL_DIR:-$HOME/.local/bin}"

# ── Logging ───────────────────────────────────────────────────────────────────

step() { printf "  \033[36m%s\033[0m %s\n" "$REPO_NAME" "$1" >&2; }
ok()   { printf "  \033[32m%s\033[0m %s\n" "$REPO_NAME" "$1" >&2; }
fail() { printf "  \033[31m%s\033[0m %s\n" "$REPO_NAME" "$1" >&2; exit 1; }

# ── Platform detection ────────────────────────────────────────────────────────

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="unknown-linux" ;;
        Darwin*) os="apple-darwin"  ;;
        *)       fail "Unsupported OS: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64"  ;;
        aarch64|arm64)  arch="aarch64" ;;
        *)               fail "Unsupported architecture: $(uname -m)" ;;
    esac

    if [[ "$os" == "unknown-linux" ]]; then
        # Prefer musl on Alpine or when PREFER_STATIC is set
        local libc="gnu"
        if [[ "${PREFER_STATIC:-false}" == "true" ]] || \
           [[ -f /etc/alpine-release ]] || \
           (ldd --version 2>&1 | grep -q musl); then
            libc="musl"
        fi
        echo "$arch-$os-$libc"
    else
        echo "$arch-$os"
    fi
}

# ── Download helper ───────────────────────────────────────────────────────────

download() {
    local url="$1" dest="$2"
    local auth_opts=""
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        auth_opts="-H \"Authorization: Bearer $GITHUB_TOKEN\""
    fi

    local max_retries=3
    for i in $(seq 1 $max_retries); do
        if command -v curl >/dev/null 2>&1; then
            if eval curl -fsSL --connect-timeout 15 --max-time 120 $auth_opts "\"$url\"" -o "\"$dest\"" 2>/dev/null; then
                local size
                size=$(stat -f%z "$dest" 2>/dev/null || stat -c%s "$dest" 2>/dev/null || echo 0)
                [[ "$size" -gt 1024 ]] && return 0
            fi
        elif command -v wget >/dev/null 2>&1; then
            if eval wget -q --timeout=120 $auth_opts "\"$url\"" -O "\"$dest\"" 2>/dev/null; then
                local size
                size=$(stat -f%z "$dest" 2>/dev/null || stat -c%s "$dest" 2>/dev/null || echo 0)
                [[ "$size" -gt 1024 ]] && return 0
            fi
        else
            fail "Neither curl nor wget is available"
        fi
        rm -f "$dest"
        [[ $i -lt $max_retries ]] && sleep 2
    done
    return 1
}

# ── Main ──────────────────────────────────────────────────────────────────────

main() {
    local platform
    platform=$(detect_platform)

    step "Installing vx for $(uname -s)..."
    step "Detected: $(uname -s) $(uname -m) -> $platform"

    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # Build list of (url, archive) candidates to try
    local candidates=()

    if [[ -z "$VX_VERSION" ]]; then
        # No version specified — use latest/download directly (no API call needed)
        candidates+=("$BASE_URL/latest/download/vx-$platform.tar.gz")
    else
        # Normalize version
        local ver="${VX_VERSION#v}"
        ver="${ver#vx-v}"

        # Try v{ver} tag first (v0.7.0+), then vx-v{ver} (legacy)
        for tag in "v$ver" "vx-v$ver"; do
            candidates+=("$BASE_URL/download/$tag/vx-$ver-$platform.tar.gz")
            candidates+=("$BASE_URL/download/$tag/vx-$platform.tar.gz")
        done

        # Also try fallback libc variant for Linux
        if [[ "$platform" == *"linux-gnu"* ]]; then
            local fallback="${platform/linux-gnu/linux-musl}"
            for tag in "v$ver" "vx-v$ver"; do
                candidates+=("$BASE_URL/download/$tag/vx-$ver-$fallback.tar.gz")
                candidates+=("$BASE_URL/download/$tag/vx-$fallback.tar.gz")
            done
        elif [[ "$platform" == *"linux-musl"* ]]; then
            local fallback="${platform/linux-musl/linux-gnu}"
            for tag in "v$ver" "vx-v$ver"; do
                candidates+=("$BASE_URL/download/$tag/vx-$ver-$fallback.tar.gz")
                candidates+=("$BASE_URL/download/$tag/vx-$fallback.tar.gz")
            done
        fi
    fi

    # Try each candidate URL
    local archive_path=""
    local archive_name=""
    for url in "${candidates[@]}"; do
        archive_name="${url##*/}"
        local dest="$temp_dir/$archive_name"
        step "Downloading from: $url"
        if download "$url" "$dest"; then
            archive_path="$dest"
            break
        fi
    done

    if [[ -z "$archive_path" ]]; then
        fail "Download failed. Check your internet connection or specify a version:
  VX_VERSION='0.7.0' curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash"
    fi

    # Extract
    step "Extracting..."
    mkdir -p "$VX_INSTALL_DIR"
    tar -xzf "$archive_path" -C "$temp_dir"

    # Find and install binary
    local binary
    binary=$(find "$temp_dir" -name "vx" -type f | head -n1)
    [[ -z "$binary" ]] && fail "vx binary not found in archive"

    cp "$binary" "$VX_INSTALL_DIR/vx"
    chmod +x "$VX_INSTALL_DIR/vx"

    # Detect installed version
    local installed_version
    installed_version=$("$VX_INSTALL_DIR/vx" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown")

    ok "Installed: vx $installed_version"

    # Update PATH
    local path_export="export PATH=\"$VX_INSTALL_DIR:\$PATH\""
    if [[ ":$PATH:" != *":$VX_INSTALL_DIR:"* ]]; then
        local shell_config
        case "${SHELL:-bash}" in
            */zsh)  shell_config="$HOME/.zshrc"  ;;
            */fish) shell_config="$HOME/.config/fish/config.fish" ;;
            *)      shell_config="$HOME/.bashrc" ;;
        esac

        if [[ -w "$(dirname "$shell_config")" ]]; then
            echo "" >> "$shell_config"
            echo "# Added by vx installer" >> "$shell_config"
            if [[ "$shell_config" == *"fish"* ]]; then
                echo "set -gx PATH \"$VX_INSTALL_DIR\" \$PATH" >> "$shell_config"
            else
                echo "$path_export" >> "$shell_config"
            fi
            ok "Added to PATH in $shell_config"
        fi

        export PATH="$VX_INSTALL_DIR:$PATH"
    fi

    # GitHub Actions support
    if [[ -n "${GITHUB_PATH:-}" ]]; then
        echo "$VX_INSTALL_DIR" >> "$GITHUB_PATH"
    fi

    echo "" >&2
    ok "vx installed successfully!"
    echo "" >&2
    printf "  Run: vx --help\n" >&2
    printf "  Docs: https://github.com/%s/%s\n" "$REPO_OWNER" "$REPO_NAME" >&2
    echo "" >&2
}

main "$@"
