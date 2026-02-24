# provider.star - curl provider
#
# cURL - command-line tool for transferring data with URLs
# Inheritance pattern: Level 1 (fully custom, system detection only)
#
# curl is pre-installed on most modern systems (Windows 10+, macOS, Linux).
# vx only detects the system installation.

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "curl"
description = "Command-line tool for transferring data with URLs"
homepage    = "https://curl.se"
repository  = "https://github.com/curl/curl"
license     = "MIT"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "curl",
        "executable":  "curl",
        "description": "Transfer data from or to a server",
        "aliases":     [],
        "priority":    100,
        "system_paths": [
            # Windows
            "C:/Windows/System32/curl.exe",
            "C:/Program Files/Git/mingw64/bin/curl.exe",
            "C:/msys64/usr/bin/curl.exe",
            # Unix
            "/usr/bin/curl",
            "/usr/local/bin/curl",
            "/opt/homebrew/bin/curl",
        ],
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "curl \\d+\\.\\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": [],
    "fs":   [
        "C:/Windows/System32",
        "C:/Program Files/Git",
        "/usr/bin",
        "/usr/local/bin",
        "/opt/homebrew/bin",
    ],
    "exec": ["curl"],
}

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    """curl version is detected from system installation."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not managed by vx
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """curl is a system tool — install via system package manager."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(_ctx):
    """Return the vx store root directory for curl."""
    return _ctx.vx_home + "/store/curl"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/curl.exe"
    else:
        return ctx.install_dir + "/curl"

def post_install(_ctx, _version):
    """Post-install hook (no-op for curl)."""
    return None
