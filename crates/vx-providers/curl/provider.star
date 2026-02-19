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

def name():
    return "curl"

def description():
    return "Command-line tool for transferring data with URLs"

def homepage():
    return "https://curl.se"

def repository():
    return "https://github.com/curl/curl"

def license():
    return "MIT"

def ecosystem():
    return "system"

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

def fetch_versions(ctx):
    """curl version is detected from system installation."""
    return [{"version": "system", "lts": True, "prerelease": False}]

# ---------------------------------------------------------------------------
# download_url — not managed by vx
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """curl is a system tool — install via system package manager."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {}
