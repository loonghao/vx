# provider.star - brew provider
#
# Homebrew - The Missing Package Manager for macOS (or Linux)
# Inheritance pattern: Level 2 (custom fetch_versions + script install)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via shell script)
#
# Installation: /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "brew"
description = "The Missing Package Manager for macOS (or Linux)"
homepage    = "https://brew.sh"
repository  = "https://github.com/Homebrew/brew"
license     = "BSD-2-Clause"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Platform constraint: macOS and Linux only
# ---------------------------------------------------------------------------

def supported_platforms():
    return [
        {"os": "macos", "arch": "x64"},
        {"os": "macos", "arch": "arm64"},
        {"os": "linux", "arch": "x64"},
        {"os": "linux", "arch": "arm64"},
    ]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "brew",
        "executable":  "brew",
        "description": "Homebrew package manager",
        "aliases":     ["homebrew"],
        "priority":    100,
        "system_paths": [
            # macOS Apple Silicon
            "/opt/homebrew/bin/brew",
            # macOS Intel
            "/usr/local/bin/brew",
            # Linux (Linuxbrew)
            "/home/linuxbrew/.linuxbrew/bin/brew",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "Homebrew \\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "raw.githubusercontent.com"],
    "fs":   [
        "/opt/homebrew",
        "/usr/local",
        "/home/linuxbrew",
    ],
    "exec": ["bash", "curl"],
}

# ---------------------------------------------------------------------------
# fetch_versions — inherited from GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("Homebrew", "brew")

# ---------------------------------------------------------------------------
# download_url — None (installed via shell script)
#
# Homebrew is installed via official shell script, not a binary download.
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """Homebrew is installed via shell script, not direct download."""
    return None

# ---------------------------------------------------------------------------
# script_install — shell script installation
# ---------------------------------------------------------------------------

def script_install(_ctx):
    """Return the shell script install command for Homebrew."""
    return {
        "command": '/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"',
        "post_install": [
            'eval "$(/opt/homebrew/bin/brew shellenv)"',
            'eval "$(/usr/local/bin/brew shellenv)"',
            'eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"',
        ],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    os   = ctx.platform.os
    arch = ctx.platform.arch

    if os == "macos" and arch == "arm64":
        return [env_prepend("PATH", "/opt/homebrew/bin")]
    elif os == "macos":
        return [env_prepend("PATH", "/usr/local/bin")]
    elif os == "linux":
        return [env_prepend("PATH", "/home/linuxbrew/.linuxbrew/bin")]
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for brew."""
    return ctx.vx_home + "/store/brew"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/brew.exe"
    else:
        return ctx.install_dir + "/brew"

def post_install(_ctx, _version):
    """Post-install hook (no-op for brew)."""
    return None
