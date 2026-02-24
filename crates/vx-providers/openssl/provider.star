# provider.star - openssl provider
#
# OpenSSL - Cryptography and SSL/TLS toolkit
# Inheritance pattern: Level 1 (fully custom, system detection only)
#
# OpenSSL is pre-installed on most systems.
# vx only detects the system installation.

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "openssl"
description = "Cryptography and SSL/TLS toolkit"
homepage    = "https://www.openssl.org"
repository  = "https://github.com/openssl/openssl"
license     = "Apache-2.0"
ecosystem   = "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "openssl",
        "executable":  "openssl",
        "description": "OpenSSL command line tool",
        "aliases":     [],
        "priority":    100,
        "system_paths": [
            # Windows
            "C:/Program Files/Git/mingw64/bin/openssl.exe",
            "C:/Program Files/Git/usr/bin/openssl.exe",
            "C:/ProgramData/chocolatey/lib/openssl/tools/openssl/bin/openssl.exe",
            "C:/OpenSSL-Win64/bin/openssl.exe",
            "C:/OpenSSL-Win32/bin/openssl.exe",
            "C:/msys64/usr/bin/openssl.exe",
            # macOS
            "/opt/homebrew/bin/openssl",
            "/usr/local/bin/openssl",
            "/usr/bin/openssl",
            # Linux
            "/usr/bin/openssl",
            "/usr/local/bin/openssl",
            "/bin/openssl",
        ],
        "test_commands": [
            {"command": "{executable} version", "name": "version_check", "expected_output": "OpenSSL \\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": [],
    "fs":   [
        "C:/Program Files/Git",
        "C:/OpenSSL-Win64",
        "C:/OpenSSL-Win32",
        "/usr/bin",
        "/usr/local/bin",
        "/opt/homebrew/bin",
    ],
    "exec": ["openssl"],
}

# ---------------------------------------------------------------------------
# fetch_versions — system detection only
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    """openssl version is detected from system installation."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not managed by vx
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    """openssl is a system tool — install via system package manager."""
    return None

# ---------------------------------------------------------------------------
# store_root — not managed by vx (system tool)
# ---------------------------------------------------------------------------

def store_root(_ctx, _version):
    """OpenSSL is a system tool — no vx store root."""
    return None

# ---------------------------------------------------------------------------
# get_execute_path — system detection only
# ---------------------------------------------------------------------------

def get_execute_path(_ctx, _version, install_dir):
    """OpenSSL is located via system_paths; no vx-managed install_dir."""
    return None

# ---------------------------------------------------------------------------
# post_install — nothing to do
# ---------------------------------------------------------------------------

def post_install(_ctx, _version):
    """No post-install steps required for openssl."""
    return []

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    return []
