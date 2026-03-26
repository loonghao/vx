#!/usr/bin/env bash
# vx installer script for Linux and macOS
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With specific version:
#   VX_VERSION="0.8.4" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With custom install directory:
#   VX_INSTALL_DIR="$HOME/bin" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With custom release mirrors (comma separated):
#   VX_RELEASE_BASE_URLS="https://mirror.example.com/vx/releases,https://github.com/loonghao/vx/releases" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
#
# With GitHub token (to avoid rate limits when specifying a version):
#   GITHUB_TOKEN="your_token" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash


set -euo pipefail

# Global temp dir so the EXIT trap can always reference it
_VX_TEMP_DIR=""

REPO_OWNER="loonghao"
REPO_NAME="vx"
BASE_URL="https://github.com/$REPO_OWNER/$REPO_NAME/releases"

VX_VERSION="${VX_VERSION:-}"
VX_INSTALL_DIR="${VX_INSTALL_DIR:-$HOME/.local/bin}"
VX_RELEASE_BASE_URLS="${VX_RELEASE_BASE_URLS:-$BASE_URL}"

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

get_release_base_urls() {
    local raw="${VX_RELEASE_BASE_URLS//;/,}"
    IFS=',' read -r -a _vx_urls <<< "$raw"

    for u in "${_vx_urls[@]}"; do
        # trim leading spaces
        u="${u#${u%%[![:space:]]*}}"
        # trim trailing spaces
        u="${u%${u##*[![:space:]]}}"
        [[ -n "$u" ]] && printf '%s\n' "$u"
    done
}

resolve_latest_version() {

    local auth_opts=""
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        auth_opts="-H \"Authorization: Bearer $GITHUB_TOKEN\""
    fi

    local api_url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases?per_page=20"
    local json=""

    if command -v curl >/dev/null 2>&1; then
        json=$(eval curl -fsSL --connect-timeout 15 --max-time 30 $auth_opts "\"$api_url\"" 2>/dev/null || true)
    elif command -v wget >/dev/null 2>&1; then
        json=$(wget -qO- --timeout=30 "$api_url" 2>/dev/null || true)
    fi

    [[ -z "$json" ]] && return 1

    # Find the first non-draft, non-prerelease release that has actual binary
    # assets.  This avoids selecting a release whose build workflow failed and
    # left the release with zero downloadable files.
    #
    # Strategy: iterate through releases in the JSON and for each one check
    # that it is not a draft/prerelease and that it contains at least one
    # "browser_download_url" entry (which indicates an uploaded asset).
    local tag=""
    # Use POSIX-compatible awk (works on macOS and Linux)
    tag=$(printf '%s' "$json" | awk '
        BEGIN { in_release = 0; cur_tag = ""; is_draft = 0; is_pre = 0; has_assets = 0 }
        /"tag_name"/ {
            # Check if the previous release qualifies
            if (in_release && cur_tag != "" && !is_draft && !is_pre && has_assets) {
                print cur_tag
                exit
            }
            # Start tracking a new release
            in_release = 1
            is_draft = 0
            is_pre = 0
            has_assets = 0
            # Extract tag value (POSIX awk compatible)
            s = $0
            gsub(/.*"tag_name"[[:space:]]*:[[:space:]]*"/, "", s)
            gsub(/".*/, "", s)
            cur_tag = s
        }
        /"draft"[[:space:]]*:[[:space:]]*true/ { is_draft = 1 }
        /"prerelease"[[:space:]]*:[[:space:]]*true/ { is_pre = 1 }
        /"browser_download_url"/ { has_assets = 1 }
        END {
            if (in_release && cur_tag != "" && !is_draft && !is_pre && has_assets) {
                print cur_tag
            }
        }
    ')

    [[ -z "$tag" ]] && return 1

    tag="${tag#v}"
    tag="${tag#vx-v}"
    printf '%s\n' "$tag"
}

# ── Main ──────────────────────────────────────────────────────────────────────

main() {

    local platform
    platform=$(detect_platform)

    step "Installing vx for $(uname -s)..."
    step "Detected: $(uname -s) $(uname -m) -> $platform"

    _VX_TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$_VX_TEMP_DIR"' EXIT
    local temp_dir="$_VX_TEMP_DIR"

    # Build list of (url, archive) candidates to try
    local candidates=()
    local release_bases=()
    while IFS= read -r _base; do
        [[ -n "$_base" ]] && release_bases+=("$_base")
    done < <(get_release_base_urls)

    if [[ ${#release_bases[@]} -eq 0 ]]; then
        release_bases=("$BASE_URL")
    fi

    if [[ ${#release_bases[@]} -gt 1 ]]; then
        step "Using release mirrors: ${release_bases[*]}"
    fi

    if [[ -z "$VX_VERSION" || "$VX_VERSION" == "latest" ]]; then
        # No version specified — try latest URL first, then resolved stable version
        for base in "${release_bases[@]}"; do
            candidates+=("$base/latest/download/vx-$platform.tar.gz")
        done

        local latest_ver=""
        latest_ver=$(resolve_latest_version || true)
        if [[ -n "$latest_ver" ]]; then
            step "Resolved latest stable version with assets: $latest_ver"
            for base in "${release_bases[@]}"; do
                for tag in "v$latest_ver" "vx-v$latest_ver"; do
                    candidates+=("$base/download/$tag/vx-$latest_ver-$platform.tar.gz")
                    candidates+=("$base/download/$tag/vx-$platform.tar.gz")
                done

                if [[ "$platform" == *"linux-gnu"* ]]; then
                    local fallback="${platform/linux-gnu/linux-musl}"
                    for tag in "v$latest_ver" "vx-v$latest_ver"; do
                        candidates+=("$base/download/$tag/vx-$latest_ver-$fallback.tar.gz")
                        candidates+=("$base/download/$tag/vx-$fallback.tar.gz")
                    done
                elif [[ "$platform" == *"linux-musl"* ]]; then
                    local fallback="${platform/linux-musl/linux-gnu}"
                    for tag in "v$latest_ver" "vx-v$latest_ver"; do
                        candidates+=("$base/download/$tag/vx-$latest_ver-$fallback.tar.gz")
                        candidates+=("$base/download/$tag/vx-$fallback.tar.gz")
                    done
                fi
            done
        fi
    else

        # Normalize version
        local ver="${VX_VERSION#v}"
        ver="${ver#vx-v}"

        # Try v{ver} tag first (v0.7.0+), then vx-v{ver} (legacy)
        for base in "${release_bases[@]}"; do
            for tag in "v$ver" "vx-v$ver"; do
                candidates+=("$base/download/$tag/vx-$ver-$platform.tar.gz")
                candidates+=("$base/download/$tag/vx-$platform.tar.gz")
            done

            # Also try fallback libc variant for Linux
            if [[ "$platform" == *"linux-gnu"* ]]; then
                local fallback="${platform/linux-gnu/linux-musl}"
                for tag in "v$ver" "vx-v$ver"; do
                    candidates+=("$base/download/$tag/vx-$ver-$fallback.tar.gz")
                    candidates+=("$base/download/$tag/vx-$fallback.tar.gz")
                done
            elif [[ "$platform" == *"linux-musl"* ]]; then
                local fallback="${platform/linux-musl/linux-gnu}"
                for tag in "v$ver" "vx-v$ver"; do
                    candidates+=("$base/download/$tag/vx-$ver-$fallback.tar.gz")
                    candidates+=("$base/download/$tag/vx-$fallback.tar.gz")
                done
            fi
        done
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
        local hint_ver="${latest_ver:-0.8.4}"
        fail "Download failed. Check your internet connection or specify a version:
  VX_VERSION='$hint_ver' curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash"
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
