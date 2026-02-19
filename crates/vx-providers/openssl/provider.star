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

def name():
    return "openssl"

def description():
    return "Cryptography and SSL/TLS toolkit"

def homepage():
    return "https://www.openssl.org"

def repository():
    return "https://github.com/openssl/openssl"

def license():
    return "Apache-2.0"

def ecosystem():
    return "system"

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

def fetch_versions(ctx):
    """openssl version is detected from system installation."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not managed by vx
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """openssl is a system tool — install via system package manager."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
