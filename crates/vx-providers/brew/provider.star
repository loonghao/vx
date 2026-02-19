# provider.star - brew provider
#
# Homebrew - The Missing Package Manager for macOS (or Linux)
# Inheritance pattern: Level 2 (custom fetch_versions + script install)
#   - fetch_versions: inherited from github.star
#   - download_url:   None (installed via shell script)
#
# Installation: /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

load("@vx//stdlib:github.star", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "brew"

def description():
    return "The Missing Package Manager for macOS (or Linux)"

def homepage():
    return "https://brew.sh"

def repository():
    return "https://github.com/Homebrew/brew"

def license():
    return "BSD-2-Clause"

def ecosystem():
    return "system"

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

def download_url(ctx, version):
    """Homebrew is installed via shell script, not direct download."""
    return None

# ---------------------------------------------------------------------------
# script_install — shell script installation
# ---------------------------------------------------------------------------

def script_install(ctx):
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

def environment(ctx, version, install_dir):
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "macos" and arch == "arm64":
        return {"PATH": "/opt/homebrew/bin"}
    elif os == "macos":
        return {"PATH": "/usr/local/bin"}
    elif os == "linux":
        return {"PATH": "/home/linuxbrew/.linuxbrew/bin"}
    return {}
